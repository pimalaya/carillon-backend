---
cairn: delta
change: backup-and-restore
---

## ADDED Requirements

### Requirement: Backup confidentiality
The database backup SHALL be treated as confidential PII, not merely as
encrypted-credential ciphertext: the store holds user emails and watch metadata
(mailbox, host, provider) in the clear alongside the age-encrypted credentials. The
replica SHALL be kept in a private bucket AND encrypted at rest — via Litestream
age encryption with a backup key distinct from the credential age key, or
provider-side encryption with a customer-managed key — so a leaked backup exposes
neither credentials nor PII. The backup-encryption key SHALL NOT live in the backup
bucket. ([[production]])

## MODIFIED Requirements

### Requirement: Continuous database backup in WAL
The database SHALL be continuously replicated (Litestream) to S3-compatible object
storage for a seconds-scale RPO, with the provider snapshot serving only as coarse
DR. SQLite SHALL run in WAL mode (already enabled by the app). The replicator's
object-storage credentials SHALL be delivered as their own credential and SHALL NOT
reuse the age key's bucket or credentials. The replica SHALL be encrypted at rest
(see Backup confidentiality). A restore SHALL be rehearsed into a scratch directory
before go-live, and a fresh daemon SHALL open the restored store and resume watches
with no data loss — this rehearsal is a hard go-live gate, not optional. ([[production]])
