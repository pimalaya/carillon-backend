---
cairn: change
id: dependency-audit-gate
status: landed
created: 2026-07-31
---

# Enforce a cargo-deny advisories gate

## Why
[[security-model]] Layer 6/8: the whole dependency tree runs in-process with decrypted
secrets, so "a compromised dependency is a full compromise" and dependency review is
the stated answer. `deny.toml` already curated licenses and sources, but had **no
`[advisories]` gate** (known-vuln / yanked enforcement) and no `[bans]` section, and
`cargo deny check` was in fact **failing** on an unlisted `Zlib` license — so the gate
was not green and not enforcing.

## What
- Add `[advisories]` (`version = 2`, `yanked = "deny"`) so a known-vulnerable or yanked
  dependency fails the check.
- Add `[bans]` (`multiple-versions = "warn"`, `wildcards = "warn"` — the local
  path-dep dev workflow uses some) to surface duplication and wildcards without
  breaking development.
- Allow the `Zlib` license (permissive, OSI-approved; pulled in transitively) so the
  license gate passes.

## Non-goals
- Wiring a CI job (there is no CI yet); the gate is run with `cargo deny check`.
- Removing the wildcard path dep (a dev-workflow artifact).
