---
cairn: delta
change: cors-allow-credentials
---

## MODIFIED Requirements

### Session transport ([[auth]])
When `api.cors_allow_origin` is set, the CORS layer SHALL send
`Access-Control-Allow-Credentials: true` and echo the exact origin (never `*` with
credentials — `*` mirrors the request origin), so a same-site cross-origin front can
attach the httpOnly cookie. A truly cross-site front keeps the Bearer path.
