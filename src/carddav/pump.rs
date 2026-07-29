//! The CardDAV poll pump.
//!
//! One `sync-collection` round driven over a fresh TLS connection. The
//! sync-collection logic itself lives in carillon-core
//! ([`CarillonCardDavPoll`]) — the same content-free poll a second frontend
//! would share; this owns only the connection and folds a change into a
//! [`ChangeEvent`]. The reconnect / backoff / interval orchestration lives
//! in the [`supervisor`](crate::supervisor); this is the CardDAV analogue
//! of [`crate::imap::pump::run_watch_core`].

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use carillon_core::backend::CarillonCardDavBackend;
use carillon_core::carddav::{
    CarillonCardDavChange, CarillonCardDavPoll, CarillonCardDavPollProgress,
};
use carillon_core::credential::CarillonCredential;
use secrecy::{ExposeSecret, SecretString};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::sync::{Semaphore, mpsc};
use tokio_rustls::TlsConnector;

use crate::carddav::session::{self, CardDavAccount, CardDavAuth};
use crate::event::ChangeEvent;

/// Per-read scratch buffer. A `sync-collection` REPORT that only fetches
/// etags stays small; io-http reassembles across reads.
const READ_BUF: usize = 16 * 1024;

/// Runs one poll of a CardDAV collection, driving carillon-core's I/O-free
/// `CarillonCardDavPoll` over a fresh connection.
///
/// `token` is the last checkpoint (`None` means never synced). A `None`
/// token performs a baseline enumeration whose change is not emitted, so
/// activating a service does not fire on every existing contact, and only
/// checkpoints the returned token. Every later poll emits one content-free
/// ring per change.
///
/// Returns the next token to persist (equal to `token` if nothing changed,
/// or `None` if the server rejected the token and a fresh baseline is
/// needed). A transport / protocol failure is returned as `Err`.
pub async fn poll_once(
    connector: &TlsConnector,
    account: &CardDavAccount,
    watch_id: &str,
    token: Option<String>,
    events: &mpsc::Sender<ChangeEvent>,
    handshake_sem: &Arc<Semaphore>,
) -> Result<Option<String>> {
    let baseline = token.is_none();
    let (host, port, base_url, path) = account.parts()?;
    let backend = CarillonCardDavBackend {
        url: account.url.clone(),
        login: account.login.clone(),
        // Core's poll ignores the interval (the driver owns it); a
        // placeholder keeps the shared backend type honest.
        poll: Duration::ZERO,
    };
    let credential = match &account.auth {
        CardDavAuth::Password(secret) => {
            CarillonCredential::Password(SecretString::from(secret.expose_secret().to_owned()))
        }
        CardDavAuth::Bearer(secret) => {
            CarillonCredential::Bearer(SecretString::from(secret.expose_secret().to_owned()))
        }
    };

    let mut cursor = token;
    loop {
        // NOTE: throttle simultaneous handshakes across all watchers,
        // holding the permit only for the network exchange.
        let permit = handshake_sem
            .clone()
            .acquire_owned()
            .await
            .expect("handshake semaphore never closes");
        let exchange = async {
            let mut stream = session::open(connector, &host, port).await?;
            let mut poll = CarillonCardDavPoll::new(
                &base_url,
                &backend,
                &credential,
                &path,
                cursor.as_deref(),
            );
            drive_poll(&mut stream, &mut poll).await
        }
        .await;
        drop(permit);
        let change = exchange?;

        // NOTE: the token is no longer valid; reset to a fresh baseline
        // next poll.
        if change.invalid_token {
            return Ok(None);
        }

        let next = change.state.or_else(|| cursor.clone());

        // NOTE: the content-free doorbell — one ring per poll that saw any
        // change, carrying the new sync-token as the opaque resync state.
        if !baseline && change.changed {
            let event = ChangeEvent::new(watch_id, "carddav", account.url.clone(), next.clone());
            if events.send(event).await.is_err() {
                bail!("delivery channel closed");
            }
        }

        // NOTE: drain a truncated result set (RFC 6578 §3.6) before
        // returning, so a large first sync fully checkpoints in one poll.
        if change.truncated && next.is_some() && next != cursor {
            cursor = next;
            continue;
        }
        return Ok(next);
    }
}

/// Drives one `CarillonCardDavPoll` to completion over an async stream,
/// performing each read and write it requests (an empty slice on EOF, so a
/// close-delimited body finalizes).
async fn drive_poll<S>(
    stream: &mut S,
    poll: &mut CarillonCardDavPoll,
) -> Result<CarillonCardDavChange>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let mut buf = [0u8; READ_BUF];
    let mut pending: Option<usize> = None;
    let mut eof = false;
    loop {
        let input = pending.take().map(|n| &buf[..n]);
        match poll.resume(input) {
            CarillonCardDavPollProgress::WantsWrite(bytes) => {
                stream.write_all(&bytes).await.context("write failed")?;
            }
            CarillonCardDavPollProgress::WantsRead => {
                if eof {
                    bail!("connection closed by peer");
                }
                let n = stream.read(&mut buf).await.context("read failed")?;
                if n == 0 {
                    eof = true;
                }
                pending = Some(n);
            }
            CarillonCardDavPollProgress::Done(result) => return result.map_err(Into::into),
        }
    }
}
