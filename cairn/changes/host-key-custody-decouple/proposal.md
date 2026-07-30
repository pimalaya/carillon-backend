---
cairn: change
id: host-key-custody-decouple
status: landed
created: 2026-07-31
---

# Decouple the sops decryption key from the SSH host key

## Why
[[security-model]] Layer 2 flagged the **host SSH key** 🔴: `deploy` had
`sops.age.sshKeyPaths = ["/etc/ssh/ssh_host_ed25519_key"]`, so the box's
network-facing SSH identity **doubled as the decrypt-everything sops key**. That key
is generated on the workstation, stored in `secrets/`, staged in `extra/`, and lives
on the box — a lot of copies of a decrypt-everything capability, and any leak of the
host key (rotation, stale copy, scan) would decrypt every secret.

Separately, the spec was stale: Layer 7 backups are in fact **done** in `deploy`
(nightly SQLite `.backup` → read-only SFTP-chrooted pull), but the security-model
still read 🔴 deferred.

## What
- **Deploy decouple** (`deploy/configuration.nix` + `README.md`): sops-nix now
  decrypts via a **dedicated age key** at `/var/lib/sops-nix/key.txt`
  (`sops.age.keyFile`, `generateKey = false`), staged at install. The SSH host key is
  **generated on the box** and is an identity only — never a secrets key. README step
  3/5, the key-custody table, and a non-breaking **"Migrating an existing box"**
  runbook (add recipient → `updatekeys` → stage → verify → drop host recipient) are
  updated.
- **Spec sync** (`security-model.md`): Layer 2 host SSH key → 🟢 identity-only; the
  box sops key is the dedicated decrypt-everything key; the "total compromise"
  requirement no longer lists the host key; **Layer 7 backups → 🟢** (catching up to
  the already-landed deploy work).

## Non-goals / notes
- The **on-box-root** residual is unchanged and inherent (the daemon holds decrypted
  creds; the sops key is on disk to boot). Decoupling removes only the *key-in-hand*
  path.
- Deploy is not a Cairn repo and cannot be built/verified here (NixOS + a live box);
  the config parses (`nix-instantiate --parse`) and the migration is operator-run.
