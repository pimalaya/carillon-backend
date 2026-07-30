---
cairn: change
id: encrypt-hmac-secret
status: landed
created: 2026-07-30
---

# Encrypt the per-watch HMAC signing secret at rest

## Why
[[security-model]] Layer 1 currently holds the per-watch **HMAC signing secret**
(`watch.hmac_secret` / `hmac_secret_prev`) in the clear on disk. Mailbox credentials
are already age-encrypted (`enc_password`, `enc_refresh_token`, `enc_client_secret`),
so the HMAC secret is the **last remaining *secret*** — not PII — sitting in
plaintext. It grants an active capability: whoever reads it can **forge valid signed
webhooks** impersonating Carillon to a customer's `notify_url`.

The operator's chosen backup posture is a periodic manual copy of the DB (`VACUUM
INTO` + scp), with the age key held out-of-band. Under that posture a leaked DB copy
should be **capability-inert** — it should grant no power. Today it does not: the
copy carries live webhook-forgery keys. Encrypting the HMAC secret closes that gap
and makes the cold copy inert (credentials + HMAC secret all ciphertext; only PII and
skeletal metadata remain, which the operator has explicitly accepted).

This is deliberately scoped to the *secret*. The surrounding PII (emails, logins,
hosts, mailbox names, notify URLs) stays cleartext by decision — see the discussion
folded into [[security-model]]. Whole-file encryption (SQLCipher) was considered and
**rejected**: against the on-box live adversary it is equivalent (the key is resident
either way), and it would kill the just-in-time-decrypt + zeroize property that keeps
crown-jewel plaintext residency minimal.

## What
- **Encrypt at rest.** Store the HMAC secret and its rotation-overlap predecessor as
  age ciphertext, reusing the existing `Crypto` encryptor and the `enc_*` column /
  field convention. Rename `hmac_secret` → `enc_hmac_secret` and `hmac_secret_prev`
  → `enc_hmac_secret_prev` (the expiry timestamp stays cleartext).
- **Decrypt just-in-time on the signing path.** `Watch::signing_secrets` SHALL take
  the `Crypto` and return zeroize-on-drop `SecretString`s (via `decrypt_secret`), so
  a decrypted signing secret never lingers past the delivery it signs — the same hot
  path discipline the mailbox password already follows.
- **Encrypt at every write.** Watch creation (`/watches`), self-host import, and
  `rotate-secret` SHALL encrypt before persisting; `rotate-secret` still returns the
  plaintext to the caller (they need it to configure their receiver).
- **Migrate existing rows.** A schema rename of the columns plus a one-time,
  idempotent, crypto-aware backfill that encrypts any legacy plaintext value in
  place (try-decrypt; on failure treat as plaintext and encrypt). Runs at startup
  after the age key is loaded, before any delivery is signed.

## Non-goals
- Encrypting the Layer-1 PII columns (accepted cleartext by decision).
- Whole-file/SQLCipher encryption (rejected — see above and [[security-model]]).
- Changing the wire signature scheme, the rotation overlap semantics, or
  `/rotate-secret`'s response shape.
