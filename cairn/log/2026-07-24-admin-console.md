---
cairn: log
change: admin-console
landed: 2026-07-24
---

# A localhost-only admin console for user, credit, and blacklist management

Landed the admin console as a **second listener** bound to a loopback address
(`[admin] listen`, default `127.0.0.1:3001`), spawned in `serve()` alongside the
public control API and sharing the same `AppState`. The admin routes are mounted
**only** on this listener's router (`api::admin_router`); the public router never
learns them, so an admin path on the public listener `404`s. The two servers
share one shutdown flag — ctrl_c flips the `watch` channel that both graceful
shutdowns await. This is the primary security control: the destructive verbs are
unreachable from the public internet, so an application-layer auth defect alone
cannot reach them. The only intended access path is an SSH tunnel / SOCKS proxy.

Authorization is a new `AdminCaller` extractor accepting **either** a
capability-link session whose account email is in `[admin] emails` (the everyday,
attributable path) **or** the existing `api.admin_token` (break-glass, independent
of the magic-link/session chain). A missing bearer is `401`, an unauthorized one
`403`. The `admin_token`'s existing public-listener powers are unchanged — the
console only lets it *also* work on the private listener.

Verbs (admin router only): `GET /admin/overview` (account count, signups over a
30-day window, aggregate credits), `GET /admin/accounts` (id, email, credits,
created_at, blocked), `POST /admin/accounts/{id}/credits` (signed delta;
downward-beyond-balance is `409`), `POST /admin/accounts/{id}/block` (`404` on
unknown account).

Store: the `account` table gained `created_at` (backfilled on migrate from the
earliest membership `added_at`; new accounts stamp it at insert) and `blocked`
(default 0). Added `list_accounts_admin`, `signup_counts`, `total_credits`,
`set_blocked`, `is_blocked`, `email_is_blocked`. A **blocked account is inert**:
the `Caller` and `CapabilityAccount` extractors reject a blocked account `403`
(a live link stops working the instant you blacklist), and both mint paths
(`/auth`, magic-link verify) refuse to issue a session to a blocked account.

Verified end-to-end against a running daemon: admin path `404`s on the public
listener; on the admin listener it is `401` without a bearer, `403` with a wrong
one, `200` with the admin token; credit add/remove and the `409`/`404` edges
behave; the account list reflects `blocked`/`created_at`. Documented the `[admin]`
section in `carillon.sample.toml` and the routes under a new `admin` tag in
`openapi.yaml` (marked loopback-only).

## Follow-up: dev-only open console (same day)

Added `[admin] dev_allow_insecure` for local development: with `cargo run`
(backend) + `npm run dev` (frontend) there is no signed-in session on the admin
origin, so the console had no way to authenticate. When this flag is set **in a
debug build**, `AdminCaller` authorizes with no credential and the admin routes
are also mounted on the public listener, so the dev frontend reaches them at its
normal API base (reusing the existing `cors_allow_origin`). The bypass is gated on
`cfg!(debug_assertions)`: a release build (`rustPlatform.buildRustPackage`
defaults to release) compiles it out and ignores the flag with a warning, so the
production binary can never expose it — verified by building `--release` with the
flag set and confirming the public listener `404`s and the admin listener still
returns `401`/`403`. A prominent startup warning fires whenever the bypass is
effective. This closes the footgun (behind a reverse proxy every request looks
local) at compile time rather than by config discipline.

## Follow-up: per-user watch visibility (same day)

Added a watch count per account to `GET /admin/accounts` (`store::account_watch_counts`,
one grouped `COUNT(*)`) and a new `GET /admin/accounts/{id}/watches` returning the
account's watches as the existing `WatchView`, gated by `AdminCaller`. The frontend
console gained a "Watches" column and an expandable row per user (chevron) whose
collapsible sub-row lazy-loads that user's watches on first expand
(`useAccountWatches`, `enabled` on expand). Reused `watches_by_account` +
`WatchView` — no new serialization.

## Follow-up: full app on the loopback listener (same day)

To make the console usable in prod over an SSH tunnel with the whitelisted-email
path (no shared token in the browser), the loopback listener now serves the FULL
router (`api::router(state, ui_dir, None, mount_admin = true)`) instead of a
minimal admin-only router — so an operator can sign in with their email ON the
`:3001` origin (minting a capability link there) and then reach `/admin`. The
`admin_router` helper was removed; `router()` gained a `mount_admin` flag (always
`true` for the loopback listener; `true` for the public listener only in a dev
build with `dev_allow_insecure`). The public release listener still never mounts
admin (verified: `404`). Also added `watching_until` to the shared `WatchView`
(additive, unit-tested: serialized when set, omitted when null) so the admin
services dialog can show each watch's paid-through time.

Caveat recorded for the operator: the magic-link email points at the configured
`dashboard_url`/`public_url` (by design, to avoid host-header injection), so to
complete sign-in ON the tunnel origin the operator swaps the verify URL's host to
the tunnel (`127.0.0.1:3001`) — the token is origin-independent. A dedicated admin
verify URL could automate this later.

## Capabilities moved

- **auth** — MODIFIED "The Caller extractor resolves the bearer" (admin_token also
  authorizes the loopback console). ADDED "Admin console is served on a
  loopback-only listener", "AdminCaller requires network position AND identity",
  "Admin console manages users, credits, and blacklist", "A blocked account is
  inert". ADDED "Dev-only open admin console is compiled out of release".

## Not done here

The frontend `/admin` route is a companion change in the frontend repo
(`admin-route`). No role hierarchy beyond the flat email whitelist; no admin
nav/discovery.
