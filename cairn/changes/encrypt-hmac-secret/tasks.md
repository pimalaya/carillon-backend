---
cairn: tasks
change: encrypt-hmac-secret
---

## Store (`src/store.rs`) — DONE
- [x] Rename columns in the `watch` CREATE TABLE: `hmac_secret` → `enc_hmac_secret`, `hmac_secret_prev` → `enc_hmac_secret_prev` (keep `hmac_secret_prev_expires`)
- [x] Rename the matching `Watch` struct fields + doc them as age ciphertext
- [x] `Watch::from_row`, `upsert_watch`, `rotate_secret`: use the new column names
- [x] `signing_secrets(&self, crypto: &Crypto, now) -> Result<Vec<SecretString>>` — decrypt via `Crypto::decrypt_secret` (zeroize-on-drop)
- [x] `migrate`: idempotent `ALTER TABLE watch RENAME COLUMN …` (guarded by `column_exists`) before the ADD COLUMN loop; update the entry to `enc_hmac_secret_prev`
- [x] `encrypt_legacy_hmac_secrets(&Crypto) -> Result<usize>`: idempotent backfill (try-decrypt; encrypt on failure)
- [x] Update the `watch()` test fixture field names

## Delivery (`src/delivery.rs`) — DONE
- [x] Thread `Arc<Crypto>` into `run` and `deliver`; decrypt signing secrets, expose into `sign`, skip-with-warn on decrypt error
- [x] `deliver_notice`: take `&Crypto`, same treatment
- [x] `deliver_test` left unchanged (signs with a caller-supplied plaintext secret, never stored)

## Metering (`src/metering.rs`) — DONE
- [x] Thread `&Crypto` through `run` → `sweep` → `emit_watch_notice` → `deliver_notice`

## Wiring (`src/main.rs`, `src/api.rs`) — DONE
- [x] `main.rs`: pass `crypto.clone()` into `delivery::run` and `metering::run`
- [x] `main.rs` import path: encrypt `account.hmac_secret` into `enc_hmac_secret`
- [x] `main.rs`: call `encrypt_legacy_hmac_secrets` after `open_crypto`, before the delivery loop spawns (serve + import)
- [x] `api.rs` `create_watch`: encrypt `request.hmac_secret`
- [x] `api.rs` `rotate_secret`: encrypt the new secret before `store.rotate_secret`; keep returning plaintext

## Verify — DONE
- [x] `nix develop --command cargo test` (server) green — 48 unit + 1 qresync integration, incl. backfill idempotency test
- [x] `cargo clippy --all-targets` + `cargo fmt --check` green
- [x] Backfill idempotency (legacy plaintext → encrypt in place → still signs → second pass no-op) covered by `encrypt_legacy_hmac_secrets_backfills_and_is_idempotent`

## Spec & log (forcing rule) — DONE
- [x] Folded the delta into [[security-model]] (Layer 1 HMAC secret 🟢; PII-only cleartext zone; SQLCipher rejected; Layer-2 age key note) and [[hardening]]
- [x] Log entry `log/2026-07-30-encrypt-hmac-secret.md`; `status: landed`
