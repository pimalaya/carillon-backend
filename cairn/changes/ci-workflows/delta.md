---
cairn: delta
change: ci-workflows
---

## MODIFIED Requirements

### Layer 6 / Layer 8 note (security-model)
Dependency review is enforced in CI: a `cargo deny` audit workflow (reusable
`pimalaya/nix` audit) runs on every push, and a nightly `cargo-fuzz` regression-replay
job runs in carillon-core. These were previously listed as follow-ups.
