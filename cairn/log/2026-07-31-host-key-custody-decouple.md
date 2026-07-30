---
cairn: log
date: 2026-07-31
change: host-key-custody-decouple
---

# SSH host key decoupled from secrets decryption; Layer 7 spec synced

Closed [[security-model]] Layer 2's host-SSH-key 🔴 and corrected a stale Layer 7.

## What landed
- **Deploy** (`deploy/configuration.nix` + `README.md`): sops-nix decrypts via a
  **dedicated age key** at `/var/lib/sops-nix/key.txt` (`sops.age.keyFile`,
  `generateKey = false`); the SSH host key is generated on the box and is an identity
  only. README step 3 now generates a dedicated `secrets/sops.key` (no ssh-to-age of
  the host key); step 5 stages `extra/var/lib/sops-nix/key.txt` and shreds the
  workstation copy after a verified boot; the key-custody table distinguishes the box
  sops key from the identity-only host key; a non-breaking **"Migrating an existing
  box"** runbook was added (add recipient → `sops updatekeys` → stage → verify → drop
  the host recipient). `nix-instantiate --parse` OK (deploy is not built here).
- **Spec** (`security-model.md`): Layer 2 host SSH key → 🟢 identity-only; the box
  sops key is the dedicated decrypt-everything key; "on-box access is total
  compromise" no longer lists the host key. Layer 7 backups → 🟢, syncing the spec to
  the already-landed deploy backup (nightly `.backup` → read-only SFTP-chroot pull,
  age key excluded).

## Honest residual
On-box **root** still yields every secret (daemon holds decrypted creds; sops key on
disk to boot unattended). Decoupling removes the key-in-hand path — a leaked host key
no longer decrypts secrets — not the on-box-root one. That is why the perimeter
(key-only SSH, loopback binds, firewall) is the real defense.

## Capabilities moved
- [[security-model]] — Layer 2 (host key 🟢 identity-only; dedicated box sops key) +
  Layer 7 (backups 🟢).

## Operator follow-up
- Run the migration runbook on the live box and verify a boot decrypts before dropping
  the host recipient.
