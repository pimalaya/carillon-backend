---
cairn: log
change: server-core-reconcile
landed: 2026-07-28
---

# Drive carillon-server on the I/O-free carillon-core; adopt the content-free event

The server ran its own IMAP watch loop — the second watcher the
`carillon-core-split` exists to delete. It now drives `carillon-core`'s
I/O-free `CarillonImapWatch`, the same coroutine the CLI drives, and the
delivered event became the generic content-free doorbell.

## What landed

**Watcher swap.** `imap/session.rs` gained `connect_tls` (a raw post-TLS
stream, no greeting); `imap/pump.rs` kept its one-shot `run` (for probe /
listing / `/test`) and replaced `run_watch` / `run_watch_idle` (and
`idle_once` / `drain_idle` / `examine_uid_next`) with a single async
`run_watch_core` that pumps `CarillonImapWatch` — core greets, authenticates,
EXAMINEs and holds IDLE; the server owns only the socket, the IDLE-refresh
timeout, and the reconnect. The supervisor dropped the QRESYNC-vs-IDLE-only
branch and `io_imap::watch::ImapMailboxWatch`, and maps its `ImapAccount` /
`ImapAuth` to `CarillonImapBackend` + `CarillonCredential`. The now-dead
`Session` / `connect` (old greet+auth watch path) were removed.

**Content-free generic event.** `ChangeEvent` is now `(id, ts, account,
source, target, state)` — dropping `uid`, the `ChangeKind`, and the CardDAV
`resource`. The names are generic (`source`/`target`/`state`, no
mailbox/uid/message vocabulary) so a new account type is a source addition,
not a rename. `delivery.rs` sends `x-carillon-source` (was `x-carillon-event`)
and logs source/target; the `delivery` store table was **overridden** (no
migration — no live users) to `source`/`target`/`state`; `live.rs`, the
`/deliveries` API view, `openapi.yaml`, and the `carillon-admin` dashboard
(schemas, mocks, `SourceBadge`, deliveries log) followed.

**CardDAV kept, folded content-free.** Not removed (it is woven through
onboarding / discovery / service-model / storage / OAuth). `carddav/pump.rs`
now emits one content-free ring per poll that saw a change (`source=carddav`,
`target`=collection, `state`=sync-token) instead of per-member `ChangeKind` +
`resource`. Its native poller, config, API and store columns are untouched; a
proper CardDAV-onto-core pass comes later.

Capabilities moved: **webhooks** — the payload and headers requirements now
describe the six-field generic content-free body and `X-Carillon-Source`.

## Verification

`carillon-server` is green: `cargo build`, `clippy --all-targets`, and `fmt`
all clean. `carillon-admin` passes `tsc -b` and its vitest suite (12/12).

Live-server smoke test against a local Stalwart (io-imap's `tests/stalwart.sh`,
plain IMAP on `127.0.0.1:143`): the shared `CarillonImapWatch` coroutine — the
substance of the server's `run_watch_core` — connects, LOGINs, EXAMINEs, holds
IDLE, and rings a content-free `Changed` (`UIDVALIDITY:UIDNEXT` token, no uid /
kind) the instant a message is APPENDed. Kept as `core/tests/stalwart_smoke.rs`
(`#[ignore]`; needs the local server).

Still open: the full server → signed-webhook path is not separately exercised
against Stalwart (the server dials implicit TLS; the Stalwart test listener is
plain), but the delivery/signing code is unchanged by this reconcile.
