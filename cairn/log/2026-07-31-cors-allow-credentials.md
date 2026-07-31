---
cairn: log
date: 2026-07-31
change: cors-allow-credentials
---

# CORS allow-credentials for the cookie front

Fixed a [[httponly-session-cookie]] regression: `credentials: "include"` requests were
blocked because `cors_layer` omitted `Access-Control-Allow-Credentials: true`.

## What landed (`src/api.rs`)
- `cors_layer`: `.allow_credentials(true)`; `*` mirrors the request origin
  (`AllowOrigin::mirror_request`) since `ACAC: true` can't pair with `ACAO: *`.
- Two preflight regression tests via `tower::ServiceExt::oneshot` (`tower` dev-dep).

## Verification
56 unit tests + 1 integration + clippy + fmt + `cargo deny` all green.

## Capabilities moved
- [[auth]] — session-transport requirement notes the CORS credentials behavior.
