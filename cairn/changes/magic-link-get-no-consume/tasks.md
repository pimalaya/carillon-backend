---
cairn: tasks
change: magic-link-get-no-consume
---

## Code (`src/api.rs`) — DONE
- [x] `magic_verify_get`: drop the store param, validate the token as bounded hex, render a click-to-confirm page (no consume)
- [x] `magic_confirm_page(Option<&str>)`: POST-to-confirm form, `no-referrer` meta + header, empty slot for a malformed token
- [x] `magic_verify_confirm` (`Form<MagicVerifyRequest>`): the consume path, returns the `postMessage` popup
- [x] Route `POST /auth/magic/verify/confirm`; import `Form`
- [x] `oauth_popup`: add `Referrer-Policy: no-referrer`

## Verify — DONE
- [x] Tests: GET renders the confirm form without a store (cannot consume); a non-hex/markup token is never reflected
- [x] `nix develop --command cargo test` (50 unit + 1 qresync) green
- [x] `cargo clippy --all-targets` + `cargo fmt --check` green

## Spec & log (forcing rule) — DONE
- [x] Add the magic-link verification requirement to [[auth]]; note in [[security-model]] Layer 4
- [x] Log entry; `status: landed`
