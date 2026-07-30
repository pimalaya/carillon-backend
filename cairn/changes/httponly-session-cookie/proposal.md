---
cairn: change
id: httponly-session-cookie
status: landed
created: 2026-07-31
---

# httpOnly session cookie for the browser (server side)

## Why
The dashboard session token (the capability link) is held in `localStorage` and sent
as a `Bearer` header, so any dashboard XSS can read and **exfiltrate a long-lived
token** for offline reuse ([[security-model]] Layer 4, the 🟡 `localStorage` item).
Moving the browser's session into an **httpOnly + Secure + SameSite=Strict cookie**
means JS can no longer read it: an XSS can still act within the live session (the
browser attaches the cookie), but it cannot steal a reusable token.

The operator chose a **single-session** model: one Carillon account per browser, held
in the cookie; switching accounts is a fresh magic-link sign-in (the client-side
multi-account switcher is retired on the frontend).

## What (server, backward-compatible)
This is the server half; it is purely **additive** so the current Bearer frontend keeps
working until the frontend migrates.
- **Read either.** A new `session_token(headers)` resolves the capability link from
  the `carillon_session` cookie **or** the existing `Authorization: Bearer` header.
  The `Caller` / `AdminCaller` extractors, `POST /signout`, the `/auth` join path, and
  every authenticated handler use it. Programmatic/CLI Bearer callers are unaffected.
- **Set on mint.** Every browser-facing session mint — `POST /auth`, the OAuth popup
  callback, and magic-link verify/confirm — additionally emits `Set-Cookie:
  carillon_session=<link>; HttpOnly; SameSite=Strict; Path=/; Max-Age=<CAPABILITY_TTL>`,
  with `Secure` when `public_url` is https. The link stays in the response body /
  `postMessage` for now (transitional), so the pre-migration frontend still works.
- **Clear on signout.** `POST /signout` expires the cookie (`Max-Age=0`) in addition
  to revoking the link server-side.

CSRF: `SameSite=Strict` is the defense; the dashboard is served same-origin with the
API (`ui_dir`) by default, so state-changing requests carry the cookie and cross-site
requests cannot.

## Non-goals / follow-ups
- Removing the link from browser responses entirely (once the frontend no longer reads
  it) — a later hardening.
- The frontend migration (single-session, cookie, no `localStorage` token) — its own
  change in the `admin` repo.
- Cross-origin CDN dashboards (`cors_allow_origin`/distinct `dashboard_origin`) keep
  the Bearer path; cookies target the same-origin deployment.
