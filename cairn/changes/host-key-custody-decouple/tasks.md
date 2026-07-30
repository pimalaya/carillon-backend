---
cairn: tasks
change: host-key-custody-decouple
---

## Deploy (`deploy/`) — DONE
- [x] `configuration.nix`: `sops.age.keyFile = "/var/lib/sops-nix/key.txt"` + `generateKey = false`; `sshKeyPaths` commented as a transition fallback
- [x] `README.md` step 3: generate a dedicated sops key (not ssh-to-age of the host key)
- [x] `README.md` step 5: stage `extra/var/lib/sops-nix/key.txt`; box self-generates the SSH host key; shred the workstation copy after a verified boot
- [x] Key-custody table + intro updated (box sops key vs identity-only host key); honest on-box-root residual retained
- [x] "Migrating an existing box" non-breaking runbook added
- [x] `nix-instantiate --parse configuration.nix` OK

## Spec & log (forcing rule) — DONE
- [x] `security-model.md` Layer 2: host SSH key 🟢 identity-only; box sops key is the dedicated decrypt-everything key; total-compromise wording fixed
- [x] `security-model.md` Layer 7: backups 🟢 (spec sync to the landed deploy backup)
- [x] Log entry; `status: landed`

## Operator (cannot run here)
- [ ] Run the migration runbook against the live box and verify a boot decrypts before dropping the host recipient
