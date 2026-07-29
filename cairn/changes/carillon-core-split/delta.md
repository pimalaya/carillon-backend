---
cairn: delta
change: carillon-core-split
---

## ADDED Requirements

### Requirement: The watcher lives in a shared `carillon-core` crate
The watch loop SHALL live in a standalone `carillon-core` crate that is an
**async watch client** (not sans-io): it drives the protocol clients (`io-imap`
IDLE now; `io-jmap` EventSource / `io-maildir` notify / CardDAV poll later) over
a **caller-owned async stream**, and owns the ideal content-free self-addressed
`Event` type, the `Backend` and resolved `Credential` types, and the one-session
`watch` entry point that watches that stream and rings into a channel. Core SHALL
be generic over the stream and SHALL NOT own the transport: opening the
connection (TCP, TLS, keepalive, any address or SSRF policy) is the frontend's.
Core SHALL know only an auth *mechanism* and its secret; it SHALL NOT depend on a
TLS stack, a datastore, a keyring, an OAuth token exchange, a notification
library, process spawning, or the delivery/consumer fan-out — those effects SHALL
be supplied upstream by a frontend, and credentials SHALL arrive already
resolved. Reconnect, backoff, and
liveness supervision SHALL live in the hosting frontend, not core: core runs one
session and returns when it drops. Both the CLI and the backend SHALL depend on
`carillon-core`; neither SHALL embed the other's binary.

#### Scenario: A watcher bug is fixed once
- **GIVEN** a defect in the per-session watch (a mishandled QRESYNC delta, a missed dead socket)
- **WHEN** it is fixed in `carillon-core`
- **THEN** both the CLI daemon and the backend receive the fix, with no duplicated per-session watcher to fix twice

### Requirement: One ideal `Event`, emitted once into a channel
A watch SHALL, on a detected change, construct exactly one content-free,
self-addressed `Event` carrying `(id, ts, account, source, target, state)` and
nothing more, and emit it once into the event channel core was given. The
hosting frontend SHALL fan that event out to its own consumers; core itself
holds no `Consumer` trait and no per-watch consumer registry. The shipped
`ChangeEvent`'s `uid`, CardDAV `resource`, and `ChangeKind` fields SHALL be
REMOVED: a ring says only that *something* changed at an address, never what or
how. `id` and `ts` SHALL be stamped once at fold and stable across retries;
`state` SHALL be the opaque per-source resync token. Every consumer, local or
network, SHALL receive the identical `Event` shape. A consumer SHALL be able to
route and dedup from the `Event` alone, and the system SHALL tolerate a dropped
or duplicated `Event` because consumers re-derive truth rather than trusting a
payload.

#### Scenario: The same event feeds a local and a network consumer
- **GIVEN** a watch configured with both a local `notify` consumer and a `webhook` consumer
- **WHEN** the mailbox changes
- **THEN** both consumers receive the same content-free `Event`, differing only in what each does with it (toast vs signed POST)

### Requirement: The CLI is a self-hostable daemon with built-in local consumers
The `carillon` CLI SHALL be a frontend that hosts `carillon-core`, reads its
watches from a TOML config file (consistent with other Pimalaya tools), and
ships two built-in local consumers: `notify` (a desktop system notification) and
`exec` (run a configured command). It SHALL own its own reconnect loop and its
consumer fan-out over core's event channel. It SHALL require none of the
backend's apparatus (no HTTP listener, datastore, auth, custody, metering, or
billing) to run a watch end to end on one machine.

#### Scenario: Local watch with no server
- **GIVEN** a `carillon` TOML config describing one IMAP watch with a `notify` consumer
- **WHEN** the daemon runs and the mailbox changes
- **THEN** a desktop notification fires, with no network delivery and no Carillon account involved

### Requirement: Sources carry a transport class; frontends host only what they advertise
Every source in `carillon-core` SHALL declare a **transport class**:
`standing-connection` (the watcher dials out and holds an outbound connection —
`io-imap` IDLE, `io-jmap` EventSource, `io-maildir` notify), `poll` (the watcher
dials out but re-checks on an interval — CardDAV `sync-collection`), or
`public-callback` (the source is delivered by an inbound POST mediated by
external infrastructure — Gmail push via Cloud Pub/Sub, Microsoft Graph
subscriptions, WebDAV-Push). Each frontend SHALL advertise which transport
classes it can host: the CLI advertises `standing-connection` and `poll`; the
backend advertises all three. `carillon-core` SHALL
refuse to arm a watch whose source transport class is not advertised by the
hosting frontend. The `Event` produced SHALL be identical regardless of class,
so no consumer depends on how the source was acquired.

#### Scenario: Gmail watch requested on the CLI
- **GIVEN** the `carillon` CLI, which advertises `standing-connection` and `poll` but not `public-callback`
- **WHEN** a watch is configured for a `public-callback` source such as Gmail push
- **THEN** `carillon-core` refuses to arm it, and the CLI never offers Gmail as a watchable source

#### Scenario: Gmail watch on the backend
- **GIVEN** the `carillon-server` backend, which advertises `public-callback` and hosts a public HTTPS endpoint
- **WHEN** a Gmail source delivers a Pub/Sub `historyId` notification
- **THEN** the backend accepts the inbound callback and emits the same content-free, self-addressed `Event` any other source would

### Requirement: Config types in core, ingestion per frontend
The watch config *types* SHALL be core's own: a source is a `Backend` and its
secret is a resolved `Credential`. There SHALL be no separate `WatchConfig`
struct in core. A frontend SHALL wrap those in its own watch registration
(adding the account id and its enabled consumers) and SHALL be the sole owner of
*ingestion*: the CLI SHALL build them from a TOML file; the backend SHALL build
them from an `Add service` API call. The two frontends SHALL differ only in
where watch registrations come from and which consumers they enable.

## MODIFIED Requirements

### Requirement: Axum control API
The backend `carillon-server` frontend SHALL host `carillon-core`'s watch loop
in the same process as an axum HTTP server exposing REST+JSON for control and
SSE for the live delivery-log / connection-status stream. The axum layer,
credential store, metering, and billing are the backend frontend's apparatus
around the shared watcher — not part of `carillon-core`. It SHALL remain
splittable into a separate service later purely for fault isolation and
independent deploy.

### Requirement: Decoupled delivery worker
Webhook delivery and the SSE change-stream SHALL be consumers in the
`carillon-server` frontend that drain core's event channel, decoupled from the
watcher hot path: a `webhook` consumer POSTs the content-free `Event`,
HMAC-signed and retried with backoff, and an `sse` consumer fans the `Event` to
authenticated long-lived subscribers. Blocking DB/crypto work SHALL run off the runtime (`spawn_blocking`)
so it never stalls the watcher. These are the network peers of the CLI's local
`notify`/`exec` consumers, over the same `Event`.

### Requirement: Tech stack shape
The MVP stack SHALL be organised as: `carillon-core` (Rust + tokio; `io-imap`
for async IMAP/IDLE; the `Event` model, the `Backend`/`Credential` types, and
the one-session `watch`) depended on by two frontends — `carillon` (CLI daemon; TOML
config; `notify` + `exec` consumers) and `carillon-server` (backend; axum for
API and console control; SQLite via `sqlx` in WAL; `reqwest` HMAC-signed
retrying POSTs and SSE as delivery consumers; the `age` crate for credentials at
rest; Stripe for billing per § [[billing]]). On the host: Debian with systemd
and Caddy for auto TLS.
