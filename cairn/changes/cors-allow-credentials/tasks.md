---
cairn: tasks
change: cors-allow-credentials
---

- [x] `cors_layer`: `.allow_credentials(true)`; `*` → `AllowOrigin::mirror_request`
- [x] Preflight tests (specific origin + `*`); `tower` dev-dep
- [x] `cargo test` (56) + clippy + fmt + `cargo deny` green
- [x] [[auth]] spec: same-site cross-origin works via CORS credentials; log; `status: landed`
