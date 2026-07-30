---
cairn: delta
change: dependency-audit-gate
---

## MODIFIED Requirements

### Layer 6 / Layer 8 note (security-model)
Dependency review is now an enforced gate: `deny.toml` carries an `[advisories]`
section (fails on known-vulnerable or yanked crates) and `[bans]` alongside the
existing curated licenses/sources, and `cargo deny check` passes green (it was
previously failing on an unlisted `Zlib` license). Advisories are the load-bearing
check; run `cargo deny check` (CI wiring is a follow-up).
