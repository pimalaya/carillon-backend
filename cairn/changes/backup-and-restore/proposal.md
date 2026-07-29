---
cairn: change
id: backup-and-restore
status: active
created: 2026-07-24
---

# Make database backup and restore real, safe, and rehearsed

## Why
Backup/restore is **entirely unbuilt**. `production.md` and `hardening.md` fully
specify it — continuous Litestream replication of the WAL'd SQLite store, separate
object-storage credentials, the age key kept out of the backup bucket, a nightly
delivery-log prune, and a **rehearsed restore** as a go-live gate — but the deploy
repo has only a bare `TODO` (`configuration.nix:107`). Concretely today:

- SQLite **WAL mode is on** (`store.rs:498`), so the app side is replicator-ready.
- There is **no replication, no prune timer, and no tested restore**. A box loss
  means data loss back to the last provider snapshot (if any), with no rehearsed
  path back — and watches cannot be proven to resume.

This also carries the safety weight from the credential-custody discussion: a
leaked backup is only *inert* if the age key is never in it (the stored
credentials are age-encrypted), **and** the DB additionally holds user emails and
watch metadata (mailbox, host, provider) in the clear — so the backup itself is
PII and must be treated as confidential, not just "encrypted credentials".

## Timing & scope (2026-07-24): deferred, decision still open
This is **paused, not scheduled**. The project is pre-launch (Stripe still in test,
zero clients), so seconds-scale continuous replication is premature. **Interim
posture: a periodic manual backup** (dump/copy the DB + confirm the age key is held
out-of-band) is "fine enough" until there is data worth a tight RPO. Revisit and
build the continuous path **once there are paying clients / real data at risk.**

Open decision to define before building (do NOT default it):
- **Where the replica lives.** Cost is negligible either way (~€0.01/mo of object
  storage vs €3–5/mo for a second VPS — a second VPS is the wrong tool for backup).
  The real driver is **failure independence**: the primary is an OVH VPS, so the
  backup SHOULD be on a *different provider* (e.g. Cloudflare R2 / Backblaze B2),
  or at minimum a different OVH region — a same-provider backup dies with the
  primary in a region outage / account lockout / datacenter fire (cf. OVH SBG2,
  2021). Decide: non-OVH object storage vs own second box (SFTP/MinIO).
- **Compromise resistance:** scoped write-only key + object versioning / object-lock
  so a compromised VPS cannot wipe the backup; optionally one truly-offline copy
  (3-2-1).
- **Confidentiality:** stable Litestream does NOT encrypt the replica; rely on a
  private bucket + TLS, and self-host or customer-managed SSE if the storage
  provider must not see the cleartext PII (emails, logins, hosts, mailbox names,
  webhook URLs, HMAC secrets — passwords/tokens are already age-encrypted).

## What
Implement the backup runbook in the deploy repo (`deploy/configuration.nix`), and
fold the resulting safety requirements into the spec:

- **Continuous replication.** A Litestream systemd service replicating the WAL'd
  DB to a private S3-compatible bucket (seconds-scale RPO). Provider snapshots
  remain only coarse DR.
- **Separate credentials.** A new sops secret slot for the object-storage
  credentials — never reusing the age key's bucket or credentials.
- **Age-key exclusion, verified.** Confirm the age key is not in the backup bucket
  or any snapshot, and is backed up in two out-of-band places (existing
  requirement, now checked as part of go-live).
- **Backup confidentiality.** Because the DB holds PII in the clear, keep the
  bucket private AND encrypt the replica at rest (Litestream age encryption with a
  backup key distinct from the credential age key, or provider-side SSE with a
  customer-managed key) — decide and document the option. A leaked backup must
  expose neither credentials (already age-encrypted in-DB) nor PII.
- **Delivery-log prune timer.** An interim nightly systemd timer pruning rows
  older than N days and `wal_checkpoint(TRUNCATE)`, avoiding full `VACUUM` while
  replication is live. (In-app retention is a separate fast-follow.)
- **Rehearsed restore.** A documented, scripted restore into a scratch directory
  that a fresh daemon opens and resumes watches from — run before go-live and
  kept as a runbook step.

## Non-goals
In-app transactional retention (replaces the interim prune timer later); HA /
multi-region; the external-wrap credential custody (discarded — see
[[credential-custody-boundary]]). This change is the durability + safe-restore
layer, not a credential-custody change.
