---
cairn: change
id: ci-workflows
status: landed
created: 2026-07-31
---

# CI: cargo-deny audit + tests

## Why
[[dependency-audit-gate]] left "run `cargo deny check` in CI" as a follow-up (no
pipeline existed). This adds GitHub Actions for the server, modelled on
`pimalaya/vcard-rs`, so the advisories/licenses/sources gate and the test suite run on
every push — enforced, not just runnable by hand.

## What
- `.github/workflows/audit.yml` — reuses `pimalaya/nix/.github/workflows/audit.yml`
  (`nix develop -c cargo-deny check`), `secrets: inherit`.
- `.github/workflows/tests.yml` — reuses `pimalaya/nix/.github/workflows/tests.yml`
  (`nix develop -c cargo test --all-features`).

The carillon-core fuzz-regression CI is its own change ([[parser-fuzz-harness]] in the
core repo).

## Non-goals
- Coverage upload, release automation (separate reusable workflows exist if wanted).
