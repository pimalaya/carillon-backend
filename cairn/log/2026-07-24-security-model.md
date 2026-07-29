---
cairn: log
change: security-model
landed: 2026-07-24
---

# Seed the security-model capability (sensitive-zones map)

Added `cairn/spec/security-model.md`, a living map of the project's sensitive zones
and trust boundaries, seeded from current truth (no behaviour change). It exists so
the security posture is visualisable in one place and the per-layer hardening work
can be cut from it, rather than living only in chat.

Structure: eight layers from crown jewels → key chain → host perimeter → the app's
attack surface (notably **parsing untrusted mail-server data in-process**) →
outbound/SSRF → third-party & supply-chain trust → data beyond the box (backups,
snapshots) → build/release, each tagged handled / partial / open / inherent. Plus
four normative requirements: on-box access is total compromise; reduce blast radius
over chasing impossible secrecy; untrusted-server parsing is a first-class surface;
keep the map current.

Cross-links [[hardening]] (controls), [[auth]] (identity), [[architecture]],
[[nixos]] / [[production]] (host), and the in-flight [[credential-custody-boundary]]
and [[backup-and-restore]] changes.

## Capabilities moved
- **security-model** (new) — seeded as current truth.
