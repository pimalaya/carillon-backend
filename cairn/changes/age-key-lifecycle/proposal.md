---
cairn: change
id: age-key-lifecycle
status: active
created: 2026-07-25
---

# Age-key lifecycle: fail closed, generate deliberately, rotate safely

## Why
Two gaps in the crown-jewel key's lifecycle (see [[security-model]] Layer 1/2):
- **Silent-regenerate footgun.** `Crypto::load_or_create` mints a *fresh* age key if
  the configured path is empty. If sops ever fails to deliver the real key, the
  daemon generates a new one and **every existing credential becomes undecryptable**
  — watches brick with no obvious error.
- **No rotation tooling.** `production.md` requires a rotation runbook (decrypt-all /
  re-encrypt-all), but nothing implements it and there is no CLI for it.

## What
- **Fail closed (ready now, backup-independent).** `serve` SHALL refuse to start when
  the age key is missing AND the store already holds credentials, rather than
  silently generating a new key. Auto-generate is allowed only for a genuinely-fresh
  store (or dropped entirely in favour of explicit keygen).
- **`keygen` subcommand (ready now).** Deliberately create an age key (`0600`) so the
  operator can generate one offline instead of relying on first-run auto-create.
- **`rotate-key` subcommand — DEFERRED to the backup layer.** Offline, transactional
  re-encrypt of every `enc_*` field old→new, with a pre-copy of the DB and a
  post-verify, plus the sops-coordination runbook. Rotation rewrites the crown
  jewels, so it leans on a safe copy — it is sequenced **with** [[backup-and-restore]]
  and not built before it.

## Timing
Fail-closed + keygen can land anytime (small, no backup dependency). rotate-key is
parked until the backup work, per the operator's call. Until then the operator's
Layer-1/2 duty is simply: generate the age key offline and keep the **personal sops
key** (which unlocks `secrets.yaml`, hence the age key) safely backed up.

## Non-goals
Rotating the other secrets (admin_token, Stripe, Resend — their own runbooks) and the
per-watch HMAC secrets (already `/rotate-secret`).
