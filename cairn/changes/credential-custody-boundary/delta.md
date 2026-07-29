---
cairn: delta
change: credential-custody-boundary
---

## ADDED Requirements

### Requirement: Custody trilemma governs the posture
Carillon SHALL treat autonomous watching, password custody, and operator-zero-knowledge as mutually constrained — any two, not all three — because a daemon that re-authenticates unattended must be able to obtain the credential unattended. The credential posture SHALL therefore be layered — reduce what is held (read-only OAuth / app-passwords), minimise how long plaintext exists in memory, keep backups confidential with the key kept out of them, and make the residual trust verifiable — rather than claiming an unreachable "cannot decrypt." Moving the wrap-key off the box (external KMS/HSM/TPM) is recorded as an OPTIONAL future, deliberately DECLINED for the single-operator VPS: it defends cold disk theft (not a real threat for a rented box) and not a live/on-box compromise, and the backup-leak case it would help is covered by backup hygiene ([[backup-and-restore]]). Operator-zero-knowledge (user-supplied unlock, plaintext RAM-only) is likewise an opt-in future that trades away unattended restart, not a default. ([[overview]])

## MODIFIED Requirements

### Requirement: Credential custody as the adoption ground
Because Carillon holds mailbox credentials in order to watch, credential custody SHALL be the trust-sensitive core and the adoption gate. Carillon SHALL make scoped **read-only OAuth the default** wherever the provider allows it (Gmail, Microsoft, Fastmail), so even a full breach cannot write, send, or delete and the grant is user-revocable without a password change. Where OAuth is unavailable, onboarding SHALL steer the user to a **dedicated, revocable app-password** (never the account's primary password; read-only scope where offered), storing no more authority than watching needs. This steering is onboarding guidance (a front-end concern), not backend enforcement. Credentials SHALL be encrypted at rest with a per-box key (external wrapping declined — see the trilemma requirement), and self-hosting SHALL remain a real option. For password mailboxes read-only is code discipline only, so the content-free payload and read-only posture remain the breach backstops. ([[overview]])
