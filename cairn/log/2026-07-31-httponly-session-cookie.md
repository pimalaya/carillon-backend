---
cairn: log
date: 2026-07-31
change: httponly-session-cookie
---

# Server accepts an httpOnly session cookie (browser XSS token-theft defense)

The server half of the [[security-model]] Layer-4 `localStorage`-XSS item. The browser
session can now live in an httpOnly cookie JS cannot read, so a dashboard XSS can no
longer exfiltrate a long-lived capability link. Additive and backward-compatible: the
existing Bearer frontend keeps working until it migrates.

## What landed (`src/api.rs`)
- **`session_token(headers)`** resolves the capability link from the `carillon_session`
  cookie OR the `Authorization: Bearer` header (Bearer wins); `cookie_value` parses the
  `Cookie` header. Every authenticated read — `Caller` / `AdminCaller` extractors,
  `/signout`, the `/auth` join, all handlers — routes through it. Programmatic/CLI
  Bearer callers are unaffected.
- **Set on mint**: `POST /auth`, the OAuth popup callback, and magic-link verify
  (JSON) + confirm (popup) attach `Set-Cookie: carillon_session=<link>; HttpOnly;
  SameSite=Strict; Path=/; Max-Age=<CAPABILITY_TTL>`, `Secure` when `public_url` is
  https (`session_cookie` + `attach_cookie`).
- **Clear on signout**: `expire_session_cookie` (`Max-Age=0`) alongside the server-side
  revoke.
- CORS comment corrected: the cookie is same-origin (`SameSite=Strict`); a cross-origin
  CDN dashboard keeps the Bearer path.
- The link is still returned in the body / `postMessage` transitionally, so the
  pre-migration frontend is unbroken.

## Design
- **Single-account per browser** (operator's call): one cookie per origin; switching
  accounts is a fresh magic-link sign-in. The client-side multi-account switcher is
  retired on the frontend (separate `admin`-repo change).
- CSRF: `SameSite=Strict` + same-origin dashboard (`ui_dir`).

## Capabilities moved
- [[auth]] — new "session may travel as an httpOnly cookie or a Bearer" requirement
  (single-account browser session).
- [[security-model]] — Layer 4 note: server supports the cookie; the `localStorage`-XSS
  item flips to 🟢 once the dashboard migrates.

## Verification
Server build + 54 unit tests (4 new: cookie read, bearer-wins, none-without-creds,
cookie attribute shape) + 1 qresync integration + clippy (`--all-targets`) +
`cargo fmt --check` all green.

## Still open
- Frontend migration (single-session cookie, drop the `localStorage` token,
  `credentials: 'include'`, retire the account switcher) — `admin` repo.
- Once the frontend no longer reads it, stop returning the link in browser responses.
