---
cairn: log
date: 2026-07-31
change: magic-link-get-no-consume
---

# Magic-link GET no longer consumes the token (prefetch/scanner safe)

First concrete slice of the [[security-model]] Layer-4 auth review. The emailed
sign-in link's API fallback (`GET /auth/magic/verify?token=…`) consumed the single-use
token on GET, so email security scanners (Outlook SafeLinks, corporate proxies/AV) and
browser link-prefetchers burned it before the human clicked — a silent "expired link"
sign-in failure concentrated on exactly the managed inboxes paying users have.

## What landed
- **`GET /auth/magic/verify` no longer consumes** (`src/api.rs`): the handler dropped
  its store param — it now *structurally cannot* spend the token — and renders a
  click-to-confirm page. The token is validated as bounded lowercase hex before being
  reflected (else an empty slot), closing a reflected-markup vector.
- **`POST /auth/magic/verify/confirm`** (new, form-encoded): the single-use verify,
  returning the same `postMessage` popup as before. A plain HTML button submits it
  with no JavaScript, so a scanner that fetches the page but never submits cannot burn
  the token.
- **`Referrer-Policy: no-referrer`** on the confirm page (meta + header) and on the
  shared `oauth_popup`, so the query-string token does not leak via `Referer`.
- The programmatic JSON `POST /auth/magic/verify` (dashboard SPA) is unchanged.

## Capabilities moved
- [[auth]] — new "Magic-link verification is prefetch-safe and single-use"
  requirement + scanner/prefetch scenario.
- [[security-model]] — Layer 4 note: the prefetch/scanner token-burn availability gap
  is closed; interception and `localStorage`-XSS takeover stay 🟡, tracked separately.

## Verification
Server build + 50 unit tests (2 new: GET renders confirm without a store; a non-hex /
markup token is never reflected) + 1 qresync integration + clippy (`--all-targets`) +
`cargo fmt --check` all green.

## Still open (Layer-4 auth follow-ups, out of scope here)
- Session token in `localStorage` → XSS-readable: httpOnly+Secure+SameSite cookie, or
  strict CSP + HSTS.
- Untrusted mail-server parser: fuzz harness + `cargo deny` in CI (the other Layer-4
  🔴).
