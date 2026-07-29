---
cairn: tasks
change: backup-and-restore
---

## Decision needed first (deferred until there are clients)
- [ ] DECIDE the replica location: non-OVH object storage (R2 / B2) vs own second box (SFTP/MinIO) vs OVH different-region — driver is failure independence, not cost
- [ ] DECIDE compromise resistance: scoped write-only key + object versioning / object-lock; optional offline 3-2-1 copy
- [ ] DECIDE confidentiality approach (private bucket + TLS; self-host / SSE-C if the provider must not see cleartext PII)
- [ ] Interim until then: a documented periodic MANUAL backup (DB copy + age key held out-of-band) — "fine enough" pre-launch

## Replication (deploy repo)
- [ ] Add a Litestream systemd service in `deploy/configuration.nix` replicating the WAL'd DB to a private S3-compatible bucket
- [ ] Add a sops secret slot for the object-storage credentials (own key/secret, distinct from the age-key bucket + creds)
- [ ] Confirm WAL is active on the live DB (app already sets it) and Litestream tracks the `-wal` file

## Confidentiality & key custody
- [ ] Decide backup-at-rest encryption: Litestream age encryption (backup key ≠ credential age key) vs provider SSE-C; document the choice
- [ ] Verify the age key is absent from the backup bucket and every snapshot; confirm it lives in two out-of-band places
- [ ] Confirm the backup bucket is private (no public/list access) and transfer is TLS

## Retention
- [ ] Nightly systemd timer: prune `delivery` rows older than N days + `wal_checkpoint(TRUNCATE)`; no full `VACUUM` while replication is live
- [ ] Dry-run the prune and confirm it does not break replication

## Restore rehearsal (the gate)
- [ ] Script a restore into a scratch dir; a fresh daemon opens it and resumes watches with no data loss
- [ ] Rehearse it end to end before go-live; keep it as a runbook step
- [ ] Verify the `migrate()` path runs cleanly on the restored copy

## Spec & log
- [ ] Fold the delta into [[production]] / [[hardening]] (backup confidentiality; rehearsed-restore gate); write the log entry
- [ ] Remove the `TODO` in `deploy/configuration.nix` once the above land
