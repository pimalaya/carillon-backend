---
cairn: log
date: 2026-07-30
change: encrypt-hmac-secret
---

# Per-watch HMAC signing secret is now encrypted at rest

Closed the last Layer-1 🟡: the per-watch HMAC signing secret — a webhook-forgery
key — was the only *secret* still stored in cleartext (mailbox credentials were
already age-encrypted). A leaked cold DB copy (the operator's `VACUUM INTO` + off-box
scp backup) is now **capability-inert**: every stored secret is ciphertext; only PII
and structural metadata remain, which is the accepted residual.

## What landed
- **Storage** (`src/store.rs`): columns `hmac_secret` → `enc_hmac_secret`,
  `hmac_secret_prev` → `enc_hmac_secret_prev` (the expiry timestamp stays cleartext);
  `Watch` fields renamed and documented as ciphertext. `signing_secrets` now takes
  `&Crypto` and returns `Vec<SecretString>`, decrypting just-in-time via
  `decrypt_secret` (zeroize-on-drop). `upsert_watch` / `rotate_secret` use the new
  columns; `rotate_secret` takes the already-encrypted new secret.
- **Migration**: idempotent `ALTER TABLE … RENAME COLUMN` in `migrate()` (guarded by
  `column_exists`), plus a new crypto-aware `Store::encrypt_legacy_hmac_secrets` that
  encrypts any legacy plaintext value in place (try-decrypt; encrypt on failure).
  Runs at startup after the age key is loaded, before the delivery loop spawns
  (`serve` + `import`). One test: `encrypt_legacy_hmac_secrets_backfills_and_is_idempotent`.
- **Signing paths** (`src/delivery.rs`, `src/metering.rs`): `Crypto` threaded through
  `delivery::run` → `deliver`, `deliver_notice`, and `metering::run` → `sweep` →
  `emit_watch_notice`; decrypt-then-`expose_secret()` into `sign`, skip-with-warn on
  decrypt failure. `deliver_test` unchanged (signs with a caller-supplied plaintext,
  never stored).
- **Write boundaries** (`src/api.rs`, `src/main.rs`): watch creation, self-host
  import, and `/rotate-secret` encrypt before persisting; `/rotate-secret` still
  returns the plaintext to the caller.

## Decisions
- **Whole-file encryption (SQLCipher) rejected.** Equivalent to per-field against the
  on-box live adversary (key resident either way), and it would forfeit the minimal
  plaintext-residency of JIT decryption. Recorded as a requirement in
  [[security-model]].
- **PII stays cleartext by decision.** Emails/logins/hosts/mailbox names/notify URLs
  are not secrets; encrypting them is a separate, deferred privacy call.

## Capabilities moved
- [[security-model]] — Layer 1 HMAC secret now 🟢; Layer-1 cleartext zone narrowed to
  PII-only; new "per-field age encryption, not whole-file" requirement; Layer-2 age
  key noted as also decrypting the HMAC secrets.
- [[hardening]] — threat model + minimal-residency requirement extended to the HMAC
  signing secret.

## Verification
Server build + 48 unit tests + 1 qresync integration test + clippy
(`--all-targets`) + `cargo fmt --check` all green.
