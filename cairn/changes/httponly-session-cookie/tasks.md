---
cairn: tasks
change: httponly-session-cookie
---

## Server (`src/api.rs`) — DONE
- [x] `session_token(headers)` = `Authorization: Bearer` OR `carillon_session` cookie (bearer wins); `cookie_value` parser
- [x] `session_cookie` / `expire_session_cookie` (HttpOnly, SameSite=Strict, Path=/, Max-Age=TTL, Secure on https) + `attach_cookie`
- [x] Route the token read through `session_token` everywhere (Caller/AdminCaller extractors, `/signout`, `/auth` join, all authenticated handlers)
- [x] Set the cookie on every browser mint: `POST /auth`, OAuth popup callback, magic verify (JSON) + confirm (popup)
- [x] `POST /signout` expires the cookie (in addition to revoking the link)
- [x] `Secure` derived from `public_url` scheme; CORS comment corrected (cookie is same-origin, Bearer for cross-origin CDN)

## Verify — DONE
- [x] Tests: cookie read, bearer-wins, none-without-creds, cookie attribute shape
- [x] `cargo test` (54 unit + 1 qresync) + clippy `--all-targets` + `cargo fmt --check` green

## Spec & log (forcing rule) — DONE
- [x] Fold into [[auth]] (session transport) and [[security-model]] Layer 4 (server supports httpOnly cookie; frontend pending)
- [x] Log entry; `status: landed`

## Follow-ups (not here)
- [ ] Frontend migration (single-session cookie, drop `localStorage` token) — `admin` repo change
- [ ] Once the frontend no longer reads it, stop returning the link in browser responses
