---
cairn: change
id: server-core-reconcile
status: landed
created: 2026-07-28
---

# Drive carillon-server on the I/O-free carillon-core; adopt the content-free event

## Why

The `carillon-core-split` change extracted the watcher into `carillon-core` and
made the CLI its first frontend. Since then, core was pivoted to **I/O-free**:
`CarillonImapWatch` is a coroutine whose driver owns the socket (see the CLI's
`driver.rs`). The server still runs its **own** watch loop (`imap/pump.rs`
`run_watch` / `run_watch_idle`, `imap/session.rs`, the rich `event.rs`). This is
the second watcher the split exists to delete: a watcher fix must land in both
frontends at once.

So: make `carillon-server` the second frontend over `carillon-core`, driving the
same coroutine with an **async** pump (the tokio twin of the CLI's blocking
one), and delete the bespoke watch loop. The apparatus that makes the server a
*service* — supervisor, delivery, SSE, store, billing, auth — stays; only the
watcher guts are swapped.

## What

### The watcher swap (IMAP)

- Core's `CarillonImapWatch` does greet + auth + EXAMINE + IDLE itself over a
  raw stream. So the server opens **TCP + TLS only** (its SSRF guard, keepalive)
  and hands core the raw stream; `imap/session.rs` gains a `connect_tls` for
  this, while its greeting/`probe`/`list_mailboxes` stay for `/test` and
  onboarding (core does not cover those).
- `imap/pump.rs` keeps its one-shot `run` (used by probe/list/test) and replaces
  `run_watch` + `run_watch_idle` (+ `idle_once`, `drain_idle`,
  `examine_uid_next`) with a single async `run_watch_core` that drives
  `CarillonImapWatch::resume`, doing each read/write, honouring the 15-minute
  IDLE-refresh timeout (drop → supervisor reconnects) and the shutdown flag.
- The QRESYNC-vs-IDLE-only branch in the supervisor disappears: core rings on the
  CONDSTORE `UIDVALIDITY:HIGHESTMODSEQ` token, else `UIDVALIDITY:UIDNEXT`.
- `io_imap::watch::ImapMailboxWatch` and the QRESYNC delta machinery leave the
  server.

### The content-free, generic event (the wire + DB break)

`ChangeEvent` becomes the ideal doorbell — `(id, ts, account, source, target,
state)` — dropping `uid`, the `ChangeKind`, and the CardDAV `resource`. The
names stay **generic on purpose**: `source` (the account kind, `imap` now),
`target` (the watched thing, a mailbox for IMAP), and `state` (the opaque resync
token) carry no IMAP/mailbox/message vocabulary, so a second account type slots
in without renaming the shared layer. The driver mints `id`/`ts` (as today) and
folds core's `Changed(state)` into it. This ripples on purpose (no live users,
payments in sandbox, so the break is free):

- **`delivery.rs`**: the webhook body is the shrunk event; the `x-carillon-event`
  (kind) header becomes `x-carillon-source`; the signing preimage `"{ts}.{body}"`
  is unchanged in *scheme* but its body shrinks; per-delivery logging drops
  `uid`.
- **`store.rs`**: the `delivery` table is **overridden, not migrated** (no live
  data) — `event` + `uid` become `source` + `target` + `state`. `DeliveryRow` /
  the insert / the `recent_deliveries*` queries follow.
- **`live.rs`** (SSE): serialises the shrunk event — no code change beyond the
  type.
- **`openapi.yaml`** + **`admin/` (`schemas.ts`, mocks)**: the event schema and
  the `x-carillon-*` headers shrink to match.

### CardDAV is kept, folded content-free (reworked later)

CardDAV is **not** removed — it is woven through onboarding, discovery, the
service model, storage and OAuth, and ripping it out is a separate surgery. It
stays working; only its event fold changes: `carddav/pump.rs` folds into the new
generic event (`source = carddav`, `target` = collection, `state` = the
sync-token) instead of a `ChangeKind` + `resource`. Its native poller, config,
API and store columns are untouched. A proper CardDAV-onto-core pass (and any
cleanup) comes later; core has no `carddav` feature yet anyway (split layer 2b).
The generic event names mean that rework is a source addition, not a rename.

### What does not change

The supervisor's reconcile-against-store, entitlement gate, reconnect/backoff,
handshake semaphore, and status bus; credential resolution and OAuth minting
(upstream of core); delivery signing scheme, metering, billing, auth, SSE
transport. Ingestion (the `Add service` API) still builds the watch; it now maps
to core's `CarillonImapBackend` + `CarillonCredential`.

## Scope / risk

- **IMAP** onto core; **CardDAV kept** but folded content-free (proper rework
  later).
- No live users and payments in sandbox, so the store schema is **overridden**
  (no migration) and the webhook contract breaks freely.
- Shared layers keep **generic names** (`source`/`target`/`state`, not
  `mailbox`/`uid`/`message`); IMAP vocabulary stays inside the `imap` module.
- Lands in verified increments; still wants a real-server smoke test (a live
  IMAP watch → signed webhook) before it is called done.
- Umbrella: [[carillon-core-split]].
