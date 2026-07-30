---
cairn: log
date: 2026-07-31
change: ci-workflows
---

# CI: cargo-deny audit + test workflows

Delivered the CI follow-up from [[dependency-audit-gate]], modelled on
`pimalaya/vcard-rs`.

## What landed
- `.github/workflows/audit.yml` — reuses `pimalaya/nix/.github/workflows/audit.yml`
  (`cargo-deny check` via `nix develop`), `secrets: inherit`.
- `.github/workflows/tests.yml` — reuses `pimalaya/nix/.github/workflows/tests.yml`
  (`cargo test --all-features` via `nix develop`).

## Capabilities moved
- [[security-model]] — Layer 6/8: `cargo deny` (here) and `cargo-fuzz` (carillon-core)
  are now CI-enforced, not follow-ups.

## Note
Workflow files are not exercisable in this environment; they mirror the working
`pimalaya/vcard-rs` setup and reference the shared reusable workflows.
