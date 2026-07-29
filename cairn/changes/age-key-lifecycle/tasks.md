---
cairn: tasks
change: age-key-lifecycle
---

## Ready now (backup-independent)
- [ ] `serve` fails closed: refuse to start if the age key is missing AND the store holds credentials (never silently regenerate)
- [ ] `carillon-server keygen <path>` subcommand: create an age key `0600`
- [ ] Decide auto-generate policy: fresh-store-only vs always require keygen; update the module/dev path accordingly
- [ ] Fold the fail-closed requirement into [[hardening]] / [[production]]; log entry

## Deferred → sequence with [[backup-and-restore]]
- [ ] `carillon-server rotate-key --old <p> --new <p>`: offline, transactional re-encrypt of all `enc_*` fields, pre-copy the DB, verify a credential decrypts under the new key before finishing
- [ ] Write the sops-coordination rotation runbook (re-encrypt DB + swap sops secret + redeploy + verify + keep old key for rollback) into [[hardening]] / [[production]]
