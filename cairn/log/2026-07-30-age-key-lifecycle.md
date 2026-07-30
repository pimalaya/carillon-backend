---
cairn: log
date: 2026-07-30
change: age-key-lifecycle
---

# Age-key fail-closed + `keygen` landed (rotate-key still deferred)

Landed the backup-independent half of [[age-key-lifecycle]]: the daemon no
longer silently mints a fresh age key when the configured key is missing.

## What landed
- **`Crypto` split** (`src/crypto.rs`): `load_or_create` — the silent-regenerate
  footgun — is gone, replaced by an explicit `load(path)` (errors if missing) and
  `generate(path)` (creates `0600`, refuses to overwrite). `from_identity` shared.
- **`Store::has_credentials`** (`src/store.rs`): true when any `watch.enc_password`,
  `password_credential`, or `oauth_credential` row exists. One test:
  `has_credentials_reflects_stored_secrets`.
- **Fail-closed open** (`src/main.rs::open_crypto`, used by both `serve` and
  `import`): loads an existing key; if none exists it auto-creates **only** for a
  genuinely-fresh store (`!has_credentials()`), otherwise it **refuses to start**
  rather than orphan every stored credential under a new key.
- **`keygen` subcommand** (`src/main.rs::keygen`): deliberately generates the age
  key `0600` at the configured `age_key_path`, refusing to overwrite. Documented in
  the crate-level subcommand list.
- **Auto-generate policy decided:** fresh-store-only (not "always require keygen"),
  so dev / self-host first run stays frictionless while the footgun is closed.

## Capabilities moved
- [[hardening]] — "Custody of the age key" requirement now mandates fail-closed
  start and a deliberate `keygen` path.

## Verification
Server build + 47 unit tests + 1 qresync integration test + clippy (`--all-targets`)
+ `cargo fmt --check` all green.

## Still open (change stays `active`)
- `rotate-key` (offline, transactional re-encrypt of all `enc_*` old→new, pre-copy
  the DB, verify-under-new-key) + the sops rotation runbook — sequenced **with**
  [[backup-and-restore]], per the proposal. Not built here.
