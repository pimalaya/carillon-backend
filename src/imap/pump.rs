//! The async coroutine pump.
//!
//! `io-imap` coroutines are I/O-free: they yield `WantsRead` / `WantsWrite`
//! and the caller owns the socket. `pimalaya-stream` ships a blocking,
//! thread-per-connection pump; holding tens of thousands of IDLE
//! connections needs an async one, inlined here over any `tokio` stream.
//!
//! The one-shot [`run`] drives a request/response coroutine (greeting,
//! login, LIST, ...) — used by [`crate::imap::session`] for probe /
//! listing / `/test`. The standing watch is driven by [`run_watch_core`],
//! which pumps carillon-core's I/O-free `CarillonImapWatch`: core owns the
//! greet / authenticate / EXAMINE / IDLE protocol, shared with the CLI,
//! and the server owns only the socket and the reconnect.

use anyhow::{Context, Result, bail};
use carillon_core::imap::{CarillonImapWatch, CarillonImapWatchProgress};
use io_imap::codec::fragmentizer::Fragmentizer;
use io_imap::coroutine::{ImapCoroutine, ImapCoroutineState, ImapYield};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

use crate::event::ChangeEvent;

/// Per-read scratch buffer. IDLE never fetches bodies, so a small
/// buffer is plenty; the fragmentizer reassembles across reads.
const READ_BUF: usize = 8 * 1024;

/// Proactively drop and reconnect an idle connection after this long
/// with no server traffic. Bounds the silent-dead-socket window and
/// refreshes server-side IDLE state well under the RFC 2177 §3 cap.
const IDLE_REFRESH: Duration = Duration::from_secs(15 * 60);

/// Drives a request/response coroutine (greeting, login, capability, ...)
/// to completion over an async stream, returning its terminal value.
pub async fn run<S, C>(
    stream: &mut S,
    fragmentizer: &mut Fragmentizer,
    mut coroutine: C,
) -> Result<C::Return>
where
    S: AsyncRead + AsyncWrite + Unpin,
    C: ImapCoroutine<Yield = ImapYield>,
{
    let mut buf = [0u8; READ_BUF];
    let mut arg: Option<&[u8]> = None;

    loop {
        match coroutine.resume(fragmentizer, arg.take()) {
            ImapCoroutineState::Yielded(ImapYield::WantsWrite(bytes)) => {
                stream.write_all(&bytes).await.context("write failed")?;
            }
            ImapCoroutineState::Yielded(ImapYield::WantsRead) => {
                let n = stream.read(&mut buf).await.context("read failed")?;
                if n == 0 {
                    bail!("connection closed by peer");
                }
                arg = Some(&buf[..n]);
            }
            ImapCoroutineState::Complete(value) => return Ok(value),
        }
    }
}

/// Drives carillon-core's `CarillonImapWatch` over a raw (post-TLS)
/// stream, forwarding each content-free change (tagged with the account
/// and `target`) to `events`. Core greets, authenticates, EXAMINEs and
/// holds IDLE itself; this only performs the reads and writes it asks for.
///
/// Returns `Ok(())` on a clean wind-down (shutdown) or the periodic
/// IDLE-refresh timeout, `Err` on a connection failure; the caller
/// reconnects in both live cases.
pub async fn run_watch_core<S>(
    account: &str,
    target: &str,
    stream: &mut S,
    mut watch: CarillonImapWatch,
    events: &mpsc::Sender<ChangeEvent>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut buf = [0u8; READ_BUF];
    let mut pending: Option<usize> = None;

    loop {
        let input = pending.take().map(|n| &buf[..n]);
        match watch.resume(input) {
            CarillonImapWatchProgress::WantsRead => {
                match timeout(IDLE_REFRESH, stream.read(&mut buf)).await {
                    Ok(Ok(0)) => bail!("connection closed by peer"),
                    Ok(Ok(n)) => pending = Some(n),
                    Ok(Err(err)) => return Err(err).context("read failed"),
                    // NOTE: no traffic for a while; drop and reconnect to
                    // refresh the session.
                    Err(_elapsed) => return Ok(()),
                }
            }
            CarillonImapWatchProgress::WantsWrite(bytes) => {
                stream.write_all(&bytes).await.context("write failed")?;
            }
            CarillonImapWatchProgress::Changed(state) => {
                let event = ChangeEvent::new(account, "imap", target, Some(state));
                if events.send(event).await.is_err() {
                    bail!("delivery channel closed");
                }
            }
            CarillonImapWatchProgress::Done(result) => return result.map_err(Into::into),
        }
    }
}
