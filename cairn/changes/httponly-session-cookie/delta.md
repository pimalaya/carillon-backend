---
cairn: delta
change: httponly-session-cookie
---

## ADDED Requirements

### Requirement: The session may travel as an httpOnly cookie or a Bearer
Carillon SHALL accept the capability-link session from either the `Authorization:
Bearer` header (programmatic / CLI callers) or an httpOnly `carillon_session` cookie
(the browser dashboard); when both are present the Bearer SHALL win. Every
browser-facing session mint (`POST /auth`, the OAuth popup callback, and magic-link
verify/confirm) SHALL additionally emit `Set-Cookie: carillon_session=<link>;
HttpOnly; SameSite=Strict; Path=/; Max-Age=<capability TTL>`, with `Secure` set when
`public_url` is https. `POST /signout` SHALL expire that cookie in addition to
revoking the link. `SameSite=Strict` is the CSRF defense; the cookie is same-origin
(a cross-origin CDN dashboard keeps the Bearer path). The link remains in the response
body / `postMessage` transitionally so a pre-migration Bearer frontend keeps working.

## MODIFIED Requirements

### Layer 4 auth-flows note (security-model)
The **server** now supports an httpOnly cookie session so the browser need not hold a
JS-readable token. Until the dashboard migrates off the `localStorage` Bearer, the
`localStorage`-XSS token-theft item stays 🟡; it flips once the frontend
(single-session cookie) lands. An XSS can still act within a live cookie session (the
browser attaches it); the win is that a long-lived token can no longer be exfiltrated.
