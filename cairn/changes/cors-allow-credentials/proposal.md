---
cairn: change
id: cors-allow-credentials
status: landed
created: 2026-07-31
---

# CORS: allow credentials for the cookie front

## Why
Bug from [[httponly-session-cookie]]: the frontend now sends every request with
`credentials: "include"` (to carry the httpOnly cookie), but `cors_layer` never set
`Access-Control-Allow-Credentials: true`. The browser blocked every cross-origin call
(e.g. dev `localhost:5173` → `127.0.0.1:3000`): *"expected 'true' in CORS header
Access-Control-Allow-Credentials"*.

## What
- `cors_layer` now sets `.allow_credentials(true)`.
- Because `ACAC: true` is incompatible with `ACAO: *`, the `*` config mirrors the
  request origin (`AllowOrigin::mirror_request`) instead of a literal star; a specific
  origin is echoed exactly. Safe because the cookie is `SameSite=Strict` (not attached
  cross-*site*).
- Preflight regression tests (`tower::ServiceExt::oneshot`) assert
  `Access-Control-Allow-Credentials: true` + the right origin for both the specific and
  `*` configs; `tower` added as a dev-dependency.

## Non-goals
- Truly cross-site fronts (still Bearer; `SameSite=Strict` withholds the cookie there).
