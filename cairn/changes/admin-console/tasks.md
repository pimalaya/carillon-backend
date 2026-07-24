---
cairn: tasks
change: admin-console
---

## Config
- [x] Add `[admin]` section: `listen` (default `127.0.0.1:3001`) and `emails` (whitelist), wired through `Config` in `src/config.rs`
- [x] Thread `[admin]` into `AppState` (whitelist set; reuse existing `admin_token`)

## Transport (loopback listener)
- [x] In `serve()` (`src/main.rs`), bind a second `TcpListener` on `admin.listen` and `axum::serve` it alongside the public listener (add to the existing shutdown `select!`)
- [x] Build an admin router (`api::admin_router`) sharing `AppState`; mount admin routes ONLY there, never on the public router
- [x] Serve the same UI dir on the admin listener so the SPA's `/admin` route loads over the tunnel

## Identity
- [x] `AdminCaller` extractor: accept a whitelisted-email capability session OR `api.admin_token`; reject otherwise
- [x] Confirm the extractor can only ever be reached via the admin router (no public mount)

## Store
- [x] Add `created_at` column to `account` (migration; backfill existing rows) for the signups view
- [x] Add `blocked` column to `account` (migration, default 0)
- [x] `list_accounts_admin` (id, email, credits, blocked, created), `set_blocked`, signup-count-over-window query
- [x] Enforce `blocked` at auth / session-mint / watch-create so a blacklisted account is inert

## Verbs (admin router)
- [x] `GET` list users + signup counts
- [x] `GET` credits (per-account + fleet aggregate)
- [x] `POST` adjust credits (reuse `add_credits` / `debit_credits`)
- [x] `POST` block / unblock an account

## Frontend (companion change, frontend repo)
- [x] Add a bare `/admin` route + views (users, credits, adjust, blacklist); no nav/access button

## Docs & spec
- [x] Update `openapi.yaml` — admin routes documented as private/loopback
- [x] Deployment note: reverse proxy serves only the public listener; admin listener reached only via `ssh -L` / SOCKS
- [x] Fold delta into `auth` (and `serving` if touched); write `log/2026-..-admin-console.md`; set status `landed`
