---
cairn: delta
change: host-key-custody-decouple
---

## MODIFIED Requirements

### Layer 2 — the SSH host key is decoupled from secrets decryption
sops-nix decrypts `secrets.yaml` via a **dedicated age key**
(`sops.age.keyFile = /var/lib/sops-nix/key.txt`), not the SSH host key. The host SSH
key SHALL be an **identity only**, generated on the box, and SHALL NOT decrypt
secrets. The on-box decrypt-everything keys are the box sops key and the carillon age
key; obtaining the host SSH key grants no secret access. Host SSH key 🔴 → 🟢. The
on-box-root total-compromise residual is unchanged (inherent).

### Layer 7 — backups are implemented (spec sync)
Backups are no longer deferred: a nightly systemd timer writes one consistent SQLite
`.backup` snapshot, pulled off-box by a read-only, SFTP-chrooted `carillon-backup`
account (a pull model, so the box holds no backup-destination credential), with the
age key excluded from the snapshot. 🔴 → 🟢.
