---
cairn: tasks
id: server-core-reconcile
---

# Tasks

## Increment 1 — reconcile IMAP onto core + generic content-free event
- [x] Add `carillon-core` dep (path `../core`, `imap` feature)
- [x] `imap/session.rs`: split `connect_tls` (TCP + TLS + SSRF + keepalive, no greeting) out of `open`
- [x] `imap/pump.rs`: add async `run_watch_core` over `CarillonImapWatch`; keep `run`; delete `run_watch`/`run_watch_idle`/`idle_once`/`drain_idle`/`examine_uid_next`
- [x] map `ImapAccount`/`ImapAuth` → `CarillonImapBackend` + `CarillonCredential`
- [x] `event.rs`: `ChangeEvent` → generic `(id, ts, account, source, target, state)`; drop `ChangeKind`/`uid`/`resource`
- [x] `supervisor.rs`: `connect_tls` + `run_watch_core`; drop the QRESYNC branch + `ImapMailboxWatch`; remove dead `Session`/`connect`
- [x] `carddav/pump.rs`: fold into the generic event (`source=carddav`, `target`=collection, `state`=sync-token)
- [x] `delivery.rs`: `x-carillon-event` → `x-carillon-source`; drop `uid`; shrunk body
- [x] `store.rs`: override `delivery` table to `source`/`target`/`state`; `DeliveryRow` + insert + `recent_deliveries*`
- [x] `live.rs`: SSE serialises the shrunk event

## Increment 2 — contract docs & UI
- [x] `openapi.yaml`: `DeliveryView` schema (source/target)
- [x] `admin/`: `schemas.ts`, mocks, `SourceBadge`, deliveries log (⚠ not typechecked — no Node here)

## Close-out
- [x] `cargo build` / `clippy` / `fmt` green (server)
- [x] `carillon-admin` typecheck (`tsc -b`) + tests green (12/12)
- [x] live-server smoke test vs local Stalwart: the `CarillonImapWatch` coroutine (the server's `run_watch_core` substance) connects → LOGIN → EXAMINE → IDLE → rings a content-free `Changed` on APPEND (`core/tests/stalwart_smoke.rs`, `#[ignore]`)
- [ ] end-to-end server → signed webhook against Stalwart (delivery/signing path unchanged; not separately exercised — Stalwart is plain, the server dials TLS)
- [x] fold delta into `spec/webhooks.md`; write log; set status landed
- [ ] tick the matching `carillon-core-split` server tasks (that umbrella change stays open)
