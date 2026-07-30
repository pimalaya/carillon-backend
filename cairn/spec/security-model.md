---
cairn: spec
capability: security-model
status: current
---

# Security model — sensitive zones & trust boundaries

This is the living map of **what is sensitive, where the trust boundaries are, and
what an attacker ultimately wants** — a companion to [[hardening]] (the controls),
[[auth]] (identity & access), [[architecture]] (runtime shape), and [[nixos]] /
[[production]] (host binding & runbook). Controls live in those specs; this page is
the *inventory and the boundaries*, kept current as the project grows. It is
organised in layers, from "what the attacker wants" outward to "how they'd reach
it." Each zone carries a status: 🟢 handled · 🟡 partial · 🔴 open/deferred ·
⚪ inherent trust (cannot be removed, only bounded).

## Layer 1 — The crown jewels (the target)
- **Credential store** (`carillon.db`): mailbox **passwords & OAuth tokens** plus the
  per-watch **HMAC signing secret** (webhook-forgery key), all held **age-encrypted**
  (ciphertext at rest). The assets worth stealing. 🟢 encrypted at rest; plaintext
  residency minimised (decrypt just-in-time, zeroize — see [[hardening]]).
- **Cleartext PII alongside them in the same DB**: account emails, logins,
  mail-server hosts, mailbox names, webhook URLs. Not secrets — no capability — but
  real PII (a deanonymisation / phishing-target list). 🟡 in the clear on disk by
  **deliberate decision**; it matters for backups and provider snapshots (Layer 7),
  but a leaked cold copy is now **capability-inert** (every *secret* is ciphertext).

## Layer 2 — The key chain that unlocks it
Three distinct keys, two of them on-box "decrypt-everything" keys:
- **Operator sops key** (age/PGP) — edits `secrets.yaml`. Lives **off-box**. ⚪/🟢
- **Host SSH key** (`/etc/ssh/ssh_host_ed25519_key`) — what sops uses to decrypt
  **all** secrets **at runtime on the box**; also the SSH host identity. **On-box**,
  doubly load-bearing. 🔴 whoever holds it decrypts every secret.
- **Carillon age key** — decrypts the **mailbox credentials and the per-watch HMAC
  signing secrets**. On-box (tmpfs); **must be backed up out-of-band, never in the DB
  backup bucket**. 🟡 losing it bricks every watch; leaking it + a DB dump is full
  credential compromise.

## Layer 3 — Host access (the perimeter)
- **SSH**: 🟢 root-only, **key-only, no password** (`PasswordAuthentication=false`,
  `PermitRootLogin=prohibit-password`).
- **Firewall**: 🟢 only 22/80/443 open; the control API and admin console bind
  loopback and are never exposed.

## Layer 4 — The running app's attack surface
- **Public onboarding endpoints** (`/discover`, `/test`, `/auth`, `/oauth`):
  public, unauthenticated **credential oracles**. 🟢 rate-limited.
- **⚠️ Parsing untrusted mail-server data in-process** — the daemon connects OUT to
  whatever server a user names and parses its responses (`imap/pump.rs`,
  `carddav/pump.rs`). A user can point a watch at a **malicious server** feeding
  malformed data; a parsing bug here is code execution **in the process holding
  decrypted credentials**. 🔴 first-class surface — treat with memory-safety
  discipline, dependency hygiene, and fuzzing of the parsers.
- **Webhook delivery** to a user-supplied `notify_url`: 🟢 SSRF-guarded (Layer 5).
- **Admin console**: 🟢 loopback-only + email-whitelist / break-glass token, code
  compiled to exclude the dev bypass in release (see [[auth]]).
- **Auth flows** (magic-link, capability-link session token in `localStorage`):
  🟡 account-takeover surface via magic-link email interception or dashboard XSS.
  The magic-link **prefetch/scanner token-burn** availability gap is closed — the
  emailed `GET` no longer consumes the token (click-to-confirm `POST`; see [[auth]]).
  Interception and `localStorage`-XSS takeover remain 🟡, tracked separately.

## Layer 5 — Outbound / SSRF
- The server originates connections to caller-supplied hosts/URLs. 🟢 guarded by
  `guard.rs` (blocks loopback / RFC1918 / cloud-metadata, rebinding-safe) **plus** a
  kernel `IPAddressDeny` backstop (see [[hardening]]).

## Layer 6 — Third-party & supply-chain trust
- **Resend** (magic-link email): ⚪/🔴 if compromised, **account takeover** via link
  interception. High trust.
- **Stripe**: 🟢 no card data stored (offloaded); the secret + webhook keys are
  sensitive. Webhook signature-verified.
- **OAuth providers** + built-in public client IDs: ⚪ provider trust.
- **Cloud provider (OVH)**: ⚪ can snapshot disk/RAM — the operator cannot hide from
  the host. Bounds the honest claim (see [[credential-custody-boundary]]).
- **Dependencies** (Rust crates, npm, flake inputs): 🔴 run **in-process** with the
  credentials; a compromised dependency is a full compromise. Reproducible builds +
  dependency review are the answers.

## Layer 7 — Data beyond the box
- **Backups**: 🔴 deferred (see [[backup-and-restore]]). When built: a second
  location holding PII + encrypted creds + HMAC secrets → must be **confidential**
  and **failure-independent** of the primary (different provider), with the age key
  never in it.
- **Provider snapshots**: 🟡 any auto-snapshot is an unmanaged copy of everything —
  know whether the host takes them.

## Layer 8 — Build & release
- Frontend CI-built, backend nix-built, deployed via flake inputs. 🔴 a compromised
  build pipeline ships a malicious binary; reproducible builds are the trust anchor.

## Trust boundaries, stated plainly

### Requirement: On-box access is treated as total compromise
Because the daemon must decrypt credentials unattended, both on-box keys (host SSH
key, carillon age key) are "decrypt-everything" keys, and a live process holds
decrypted secrets. Carillon SHALL therefore treat **host/root access as total
compromise** and invest the perimeter accordingly — strong key-only SSH, firewall,
loopback binds, sane defaults — rather than relying on at-rest encryption to defend
a live box. At-rest encryption defends **cold theft and backups**, not an on-box
adversary. See the trilemma in [[overview]] / [[credential-custody-boundary]].

### Requirement: Reduce blast radius over chasing impossible secrecy
Since on-box compromise cannot be crypto-prevented, the load-bearing mitigations
SHALL be blast-radius reducers: scoped read-only OAuth / app-passwords (never a
primary password — [[credential-custody-boundary]]), content-free payloads,
read-only posture, and per-service credential isolation, so a breach leaks
**signals, not mail**, and cannot write/send/delete.

### Requirement: secrets at rest use per-field age encryption, not whole-file
Confidentiality of secrets at rest SHALL be per-field age encryption with
just-in-time decryption into zeroize-on-drop values, and every stored *secret*
(mailbox credentials + per-watch HMAC signing secret) SHALL be so encrypted; only
PII and structural/temporal/operational columns remain cleartext. Whole-database
encryption (SQLCipher and equivalents) SHALL NOT be adopted: against the on-box live
adversary it is equivalent (the key is resident either way — Layer 2), while it would
forfeit the minimal-plaintext-residency property that per-field JIT decryption gives
the crown jewels. The operator's cold-copy backup (`VACUUM INTO` + off-box copy, age
key held out-of-band) therefore carries **no usable secret** — only the accepted PII
residual.

### Requirement: Untrusted-server parsing is a first-class attack surface
The IMAP/CardDAV parsers consume data from arbitrary, user-chosen (thus
attacker-influenceable) servers inside the process that holds decrypted
credentials. Carillon SHALL treat this path as a primary attack surface: keep the
parsing dependencies current and reviewed, prefer memory-safe parsing, and fuzz the
parsers; a bug here is more dangerous than a public-endpoint bug.

### Requirement: This map is kept current
This security model SHALL be updated whenever a zone changes (a new external
dependency, a new endpoint, a new data-at-rest location), so the trust boundaries
are never stale. It is the index the per-layer hardening changes are cut from.
