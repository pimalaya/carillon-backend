---
cairn: log
change: credential-residency-hardening
landed: 2026-07-24
---

# Minimise decrypted-credential residency in the process

Made the password watcher symmetric with OAuth: **hold ciphertext, decrypt
just-in-time per connect, zeroize immediately** — instead of decrypting once at
spawn into a bare `String` that lived for the whole watch.

## What landed

- **`Credential::Password` now carries the age-encrypted blob**, not plaintext
  (`supervisor.rs`). `spawn_watcher` no longer decrypts; each connect (IMAP and
  CardDAV loops) decrypts just-in-time via the `Arc<Crypto>` the watcher already
  holds. A decrypt failure is handled like a connect failure (warn + back off +
  retry) — both loops now unify the password-decrypt and OAuth-mint fallible
  paths through one error arm.
- **`Crypto::decrypt_secret`** decrypts straight into a `secrecy::SecretString`
  (zeroize on drop), via a `Zeroizing` byte buffer so no plaintext survives the
  call. Unit-tested for round-trip + parity with `decrypt`.
- **The auth types hold `SecretString` and dropped `Clone`.** `ImapAuth`
  (`Password`/`OauthBearer`) and `CardDavAuth` (`Password`/`Bearer`) — and the
  `ImapAccount`/`CardDavAccount` that contain them — no longer derive `Clone`
  (nothing cloned the values; only the two per-connect `.clone()`s of the
  plaintext, now gone). Their `Debug` auto-redacts (SecretString), closing a
  latent log-leak. Every construction site (onboarding probes in `api.rs`, the
  supervisor loops, the session layers) wraps in `SecretString`; the login/HTTP
  Basic calls `expose_secret()` only for the auth call.
- **OAuth internals hardened**: `resolve_oauth_access` returns `SecretString`
  (access token) and holds the decrypted refresh token in `Zeroizing` for the
  refresh call only.
- **Accidental-capture backstops in the NixOS unit** (`nix/module.nix`):
  `LimitCORE=0` (no coredumps) and `MemorySwapMax=0` (never swap the unit's pages
  out), so a crash or swap can't spill a mid-connect plaintext credential.
- **Log audit**: no `{:?}`/format site touches a decrypted secret; the two
  `debug!`/`anyhow!` `{:?}` sites print OAuth error codes and server capability
  lists only.

## Verification

- `cargo build` + full `cargo test` green (46 tests, +2 for `decrypt_secret`).
- Per-connect decrypt confirmed by construction: `Credential::Password` holds
  ciphertext; `decrypt_secret` is called inside each loop iteration, so a
  reconnect re-decrypts. A live mid-watch core-dump check was not run (needs a
  real IMAP peer + the sandbox won't hold a long-running watcher); the residency
  reduction is structural (plaintext scoped to the connect, zeroized after).

## Not done here (out of scope → [[credential-custody-boundary]])

The age key's own custody and the operator's *ability* to decrypt at rest. This
change only stops plaintext lingering in RAM; it does not move the wrap-key off
the box.

## Capabilities moved

- **hardening** — ADDED "Minimal decrypted-credential residency" and "Suppress
  accidental secret capture".
- **architecture** — MODIFIED "Credentials encrypted at rest" (add the
  minimise-in-memory clause).
- **nixos** — MODIFIED Layer-1 directive list (`LimitCORE=0`, `MemorySwapMax=0`).
