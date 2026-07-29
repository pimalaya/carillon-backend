---
cairn: tasks
change: carillon-core-split
---

## `carillon-core` — layer 1: scaffold (landed in the core repo)
- [x] Scaffold the crate with the minimal `CarillonEvent` (`id, ts, account, source, target, state`), `CarillonSource`, `CarillonBackend` (+ `CarillonImapBackend` / `CarillonCardDavBackend`), `CarillonCredential`, and a stubbed one-session `watch`
- [x] Give each source a `transport_class` (`standing-connection` | `poll` | `public-callback`); a frontend advertises the classes it hosts and refuses to arm an unadvertised class
- [x] Apply the Pimalaya guidelines from commit one (domain prefix, per-type files, no root re-exports, architecture header, docs) and start Cairn (spec + log)

## `carillon-core` — layer 2a: relocate the IMAP watch (landed)
- [x] Add the `imap` cargo feature pulling `io-imap` + `anyhow` (gates real deps, satisfying crate-003)
- [x] Relocate the pump + auth into core under `imap`; `imap::watch` greets, authenticates (LOGIN / OAUTHBEARER), holds IDLE and rings on each EXAMINE state advance, over a caller-owned stream
- [x] Core owns no transport: generic over the async stream, no TLS/rustls/socket2 dep; the frontend opens the connection and owns reconnect
- [x] `imap::watch` runs one session and returns when the stream drops or shutdown is set
- [x] Keep core dependency-clean: no store, keyring, OAuth exchange, notification library, process spawning, delivery, or TLS
- [x] `CarillonEvent.state` is `UIDVALIDITY:HIGHESTMODSEQ` via `EXAMINE (CONDSTORE)` when advertised (rings on flag/delete too), falling back to `UIDVALIDITY:UIDNEXT` (new mail only) otherwise

## `carillon-core` — layer 2b: relocate the CardDAV poll (landed via [[carddav-poll]])
- [x] Add the `carddav` cargo feature pulling `io-webdav` (+ io-http + url), off by default
- [x] Relocate the poll into core as `CarillonCardDavPoll` (`sync-collection`, `getetag` only); content-free `CarillonCardDavChange` (changed/state/invalid_token/truncated); the server driver rings the content-free `carddav` event and owns interval/reconnect/checkpoint
- [x] Encode the CardDAV sync-token into the event `state`

## `carillon` CLI frontend (landed)
- [x] Depend on `carillon-core`; TOML config loader building a `CarillonBackend` + `CarillonCredential` per watch
- [x] Own the reconnect/backoff loop and the consumer fan-out over core's event channel, resolving a fresh credential per attempt
- [x] Built-in `notify` consumer (content-free desktop system notification)
- [x] Built-in `exec` consumer (spawn the command directly via tokio, not the deprecated io-process; ring fields as `CARILLON_*` env vars)
- [x] Run one IMAP watch end to end from a TOML file with zero backend apparatus
- [x] Repo skeleton + Cairn (spec `daemon`, log, README, config.sample.toml, licenses)
- [ ] OAuth credential resolution (only password / password_command wired; OAuth minting deferred)
- [x] Align onto the pimalaya-cli toolkit (done: standard-structure rebuild on pimalaya-cli/config/stream)

## `carillon-server` backend frontend (landed via [[server-core-reconcile]])
- [x] Depend on `carillon-core`; delete the now-moved per-session watch loop (the bespoke `event.rs` became the generic content-free `ChangeEvent`)
- [x] Keep the supervisor (reconcile against store, entitlement gate, reconnect/backoff, handshake semaphore, status bus) as backend apparatus wrapping core's watch
- [x] Re-express webhook delivery and the SSE stream as the backend's consumers draining core's content-free event channel
- [x] Build a `CarillonImapBackend` + `CarillonCredential` from the ingested watch (ingestion stays here; OAuth minting and keyring stay upstream of core)
- [x] Keep auth, custody, metering, billing, SQLite as the backend's apparatus around core
- [ ] Advertise the `public-callback` transport class (public HTTPS endpoint); Gmail/Graph/WebDAV-Push sources are backend-only (deferred — no source wired here beyond IMAP IDLE)

## Naming & repo renames
- [x] Rename `carillon-backend` → `carillon-server` (crate + local dir `backend` → `server`; remote already `carillon-server`)
- [x] Rename `carillon-frontend` → `carillon-admin` (package + local dir `frontend` → `admin`; remote set-url to `carillon-admin`)
- [x] Keep `carillon-website` as-is (not renamed to `carillon-site`)
- [ ] Publish `carillon-core` as its own crate/repo; `carillon` (CLI) depends on it as an equal

## Docs & spec (the forcing rule)
- [x] Fold into `spec/architecture.md` + `spec/webhooks.md` (watcher = carillon-core; content-free `source`/`target`/`state`) — via [[server-core-reconcile]]
- [x] Note the rename in the admin/website READMEs (swept in the rename passes)
- [ ] Write `log/…-carillon-core-split.md`; set this change `status: landed` (stays active: CardDAV layer 2b + `carillon-core` publishing remain)
