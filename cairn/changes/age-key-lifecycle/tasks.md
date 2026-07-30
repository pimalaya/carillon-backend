---
cairn: tasks
change: age-key-lifecycle
---

## Ready now (backup-independent) — LANDED 2026-07-30
- [x] `serve` fails closed: refuse to start if the age key is missing AND the store holds credentials (never silently regenerate) — `main.rs::open_crypto` + `store.rs::has_credentials`
- [x] `carillon-server keygen [config]` subcommand: create an age key `0600` — `main.rs::keygen`, uses the configured `age_key_path` (refuses to overwrite)
- [x] Decide auto-generate policy: **fresh-store-only** (auto-create allowed only when `has_credentials()` is false; `Crypto` split into `load` / `generate`, `load_or_create` dropped)
- [x] Fold the fail-closed requirement into [[hardening]]; log entry

## Deferred → sequence with [[backup-and-restore]]
- [ ] `carillon-server rotate-key --old <p> --new <p>`: offline, transactional re-encrypt of all `enc_*` fields, pre-copy the DB, verify a credential decrypts under the new key before finishing
- [ ] Write the sops-coordination rotation runbook (re-encrypt DB + swap sops secret + redeploy + verify + keep old key for rollback) into [[hardening]] / [[production]]
