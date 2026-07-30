---
cairn: log
date: 2026-07-31
change: dependency-audit-gate
---

# cargo-deny advisories gate enforced (and the license gate un-broke)

[[security-model]] Layer 6/8 hardening. `deny.toml` curated licenses/sources but had
no advisories gate, and `cargo deny check` was actually **failing** on an unlisted
`Zlib` license — so nothing was enforced.

## What landed (`deny.toml`)
- `[advisories]` (version 2, `yanked = "deny"`) — a known-vulnerable or yanked crate
  now fails the check. (`cargo deny check advisories` → clean today.)
- `[bans]` (`multiple-versions = "warn"`, `wildcards = "warn"` + `allow-wildcard-paths`)
  — surfaces duplication and wildcard reqs without breaking the local path-dep dev
  workflow (carillon-server has one wildcard path dep cargo-deny won't exempt for a
  publishable crate).
- Allowed `Zlib` (permissive, transitive) so licenses pass.

Result: `cargo deny check` → **advisories ok, bans ok, licenses ok, sources ok**.

## Capabilities moved
- [[security-model]] — Layer 6 dependency-review is now an enforced gate; Layer 4
  notes the new carillon-core adversarial parser harness.

## Follow-up
- Run `cargo deny check` in CI once a pipeline exists.
