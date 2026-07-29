---
cairn: delta
change: age-key-lifecycle
---

## MODIFIED Requirements

### Requirement: Custody of the age key
The age key (`server.age_key_file`) decrypts every stored credential and is the crown jewel: losing it bricks every watch, and leaking it plus a DB dump is full credential compromise. It SHALL be generated offline, stored `0600` owned by the service user, and backed up out-of-band — never in the same bucket as the DB. It SHALL be delivered via systemd `LoadCredential=` / `systemd-creds` or a secrets manager in preference to a plaintext file on disk. The daemon SHALL **fail closed**: when the key is missing and the store already holds credentials it SHALL refuse to start rather than silently generate a fresh key (which would orphan every existing credential). A deliberate `keygen` path SHALL exist so operators create the key on purpose rather than by first-run accident. ([[hardening]])

## ADDED Requirements

### Requirement: Age key rotation is a safe, transactional migration
Carillon SHALL provide an offline `rotate-key` operation that re-encrypts every stored `enc_*` field from an old key to a new key in a single transaction, takes a pre-copy of the store first, and verifies a credential decrypts under the new key before completing — with a documented sops-coordination runbook (re-encrypt DB, swap the sops secret, redeploy, verify, keep the old key until verified for rollback). Because it rewrites the crown jewels it depends on a safe copy and SHALL be sequenced with [[backup-and-restore]]. ([[hardening]])
