---
cairn: tasks
change: dependency-audit-gate
---

## Config (`deny.toml`) — DONE
- [x] Add `[advisories]` (version 2, `yanked = "deny"`)
- [x] Add `[bans]` (`multiple-versions = "warn"`, `wildcards = "warn"`, `allow-wildcard-paths`)
- [x] Allow `Zlib` in `[licenses]` (fix the pre-existing license-gate failure)

## Verify — DONE
- [x] `cargo deny check` → advisories ok, bans ok, licenses ok, sources ok

## Spec & log — DONE
- [x] Note the enforced gate in [[security-model]] Layer 6/8
- [x] Log entry; `status: landed`

## Follow-up (not here)
- [ ] Run `cargo deny check` in CI once a CI pipeline exists
