---
cairn: delta
change: encrypt-hmac-secret
---

## MODIFIED Requirements

### Layer 1 — the per-watch HMAC signing secret is encrypted at rest
Previously 🟡: the per-watch HMAC signing secret was cleartext on disk. Now the
secret and its rotation-overlap predecessor SHALL be stored age-encrypted
(`enc_hmac_secret` / `enc_hmac_secret_prev`) and decrypted just-in-time on the
signing path into zeroize-on-drop secrets, matching the mailbox-credential
discipline. A cold DB copy (the operator's `VACUUM INTO` + scp backup) is therefore
**capability-inert**: every stored *secret* (mailbox credentials + HMAC secret) is
ciphertext, and only PII plus skeletal metadata remain — the accepted residual leak.

### Cleartext Layer-1 zone narrows to PII only
The Layer-1 "cleartext alongside" zone SHALL be understood as **PII only** (account
emails, logins, mail hosts, mailbox names, notify URLs) — a deliberate,
capability-free residual. No *secret* remains in the clear at rest.

### Whole-file (SQLCipher) encryption is rejected
Whole-database encryption SHALL NOT be adopted for the credential store: against the
on-box live adversary it is equivalent to per-field (the key is resident either way),
and it would forfeit the just-in-time-decrypt + zeroize property that minimises
crown-jewel plaintext residency. Confidentiality of secrets at rest SHALL remain
per-field age encryption with JIT decryption.
