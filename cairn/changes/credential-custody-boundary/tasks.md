---
cairn: tasks
change: credential-custody-boundary
---

## Tier 1 — reduce what we hold (front story: onboarding guidance, not enforcement)
- [ ] Onboarding presents read-only OAuth as the default/recommended choice where the provider supports it (Gmail, Microsoft, Fastmail); OAuth is the primary call to action
- [ ] For password providers, steer to a dedicated, revocable app-password (never the primary password; read-only scope where offered), with per-provider guidance
- [ ] Clear copy: "never enter your main account password"; warn (not hard-refuse) when a primary password is likely
- [ ] Implement in the carillon-admin repo (this is a front-end concern); this backend change owns only the posture spec below

## Posture documentation
- [ ] Write the trilemma + layered posture into [[overview]] (reduce → shrink residency → backup hygiene → verify), with external wrap recorded as an optional/declined future
- [ ] Make the residual trust verifiable: reference the reproducible build + single audited decrypt path + the content-free / read-only / per-service-isolation backstops
- [ ] Record operator-zero-knowledge mode as an opt-in future non-goal; confidential-computing enclave as the live-daemon-gap option, out of scope

## Tier 2 — DISCARDED (recorded, not built)
- [x] ~~Design the external-wrap seam (KMS/HSM/TPM) behind `Crypto`~~ — discarded (single-operator VPS threat model; see the Decision note in the proposal)
- [x] ~~Spike a KMS/TPM backend end-to-end (audit + revoke + rate-limit)~~ — discarded
- [x] ~~Extend the age-key custody runbook for a wrapped-key model~~ — discarded (backup hygiene handled in [[backup-and-restore]])

## Fold
- [ ] Fold the delta into [[overview]]; add the log entry; set status landed
