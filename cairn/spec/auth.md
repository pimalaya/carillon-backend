---
cairn: spec
capability: auth
status: current
---

# Authentication and Access Scoping

Accounts are database entities, not configuration: a login-less account is a set of mailboxes the user has proven control of, grouped under one bearer capability link. Configuration carries only infrastructure (`[server]` store/keys/tuning, `[api]` listen/bind/auth); it never carries accounts or watches. Every route that touches watches, deliveries, accounts, or the live stream requires a bearer token and is scoped to the caller, in every front — there is no unauthenticated data access. See [[serving]] for how each front is deployed and [[billing]] for what an authenticated account is entitled to.

### Requirement: Accounts live in the database, config is infra only
Carillon SHALL persist accounts and watches in the store, never in configuration. Configuration SHALL carry only `[server]` (store path, encryption key, tuning) and `[api]` (listen/bind, auth) infrastructure. This collapses the config-path-vs-API-path duplication into one path so the UI is a reusable layer over the API in every deployment.

### Requirement: Every data route is authenticated and scoped
Every route that touches watches, deliveries, accounts, or the live stream SHALL require a bearer token on `Authorization: Bearer <token>` and SHALL scope the request to the caller. This holds in every front, including self-host; there is no unauthenticated data access.

### Requirement: The Caller extractor resolves the bearer
A `Caller` extractor SHALL resolve the presented bearer token to exactly one of two identities: a capability-link account, scoped to its own watches, deliveries, events, and pool; or the optional unscoped admin token `api.admin_token`, which grants fleet-wide access for ops and headless use. When `api.admin_token` is unset (the default), no unscoped access exists at all. The `api.admin_token` additionally authorizes the loopback admin console (see the `AdminCaller` requirement) as a break-glass identity; its powers on the public listener are unchanged by the admin console.

#### Scenario: Bearer matches a capability link
- **GIVEN** a request carrying a valid, unexpired, un-revoked capability link
- **WHEN** the `Caller` extractor resolves it
- **THEN** the request is scoped to that link's account and may reach only that account's own resources

#### Scenario: Bearer matches the admin token
- **GIVEN** `api.admin_token` is set and the request carries it
- **WHEN** the `Caller` extractor resolves it
- **THEN** the request is granted unscoped, fleet-wide access to every account

#### Scenario: Admin token unset
- **GIVEN** `api.admin_token` is unset (the default)
- **WHEN** a request presents a bearer that is not a valid capability link
- **THEN** no unscoped access is available and the request is rejected

### Requirement: Single-resource routes 404 across account boundaries
When a scoped caller requests a single resource that belongs to another account, Carillon SHALL respond `404 Not Found` rather than `403`, to hide the existence of resources outside the caller's scope.

### Requirement: Watch creation forces scope and requires proven mailbox
`POST /watches` SHALL force the caller's own account as the owner and SHALL require that the target mailbox has already been proven via `POST /auth`. A watch cannot be created for a mailbox the caller has not authenticated to; this is the anti-farming linchpin, since free watching is granted only for a mailbox the caller has successfully authenticated to.

### Requirement: Public routes opt out of authentication
Only the following routes SHALL be public (no bearer required): `GET /health`, `GET /`, `GET /openapi.yaml`, `POST /test`, `POST /discover`, `POST /auth`, `POST /oauth/start`, `GET /oauth/callback`, `GET /billing/packs`, and the billing webhook. The public onboarding routes (`/test`, `/discover`, `/auth`, `/oauth/*`) SHALL be rate-limited, since they are the credential-oracle surface. Every other route SHALL require an authenticated, scoped `Caller`.

### Requirement: Capability link is an unguessable minted bearer
On successful `POST /auth`, Carillon SHALL mint a capability link: a long, unguessable, per-account bearer token. The link SHALL be stored hashed with an expiry, SHALL be validated by the server on every call (never client-only gating), and SHALL be one account per link. First auth creates an account and issues its link; authenticating to another mailbox while holding the link adds that mailbox to the same account.

### Requirement: Capability link supports rotation and expiry
Carillon SHALL support minting, rotating, and expiring a capability link server-side, so a link's lifetime is bounded and a compromised link can be replaced without abandoning the account. Recovery is re-auth to any member mailbox, which re-mints the account's link.

### Requirement: The session may travel as an httpOnly cookie or a Bearer
The capability-link session SHALL be accepted from either the `Authorization: Bearer` header (programmatic / CLI callers) or an httpOnly `carillon_session` cookie (the browser dashboard); the Bearer SHALL win when both are present. Every browser-facing session mint (`POST /auth`, the OAuth popup callback, magic-link verify/confirm) SHALL set `carillon_session` as `HttpOnly; SameSite=Strict; Path=/; Max-Age=<capability TTL>`, adding `Secure` on https, so a dashboard XSS cannot read or exfiltrate the token. `POST /signout` SHALL expire the cookie in addition to revoking the link. `SameSite=Strict` is the CSRF defense. A **same-site** cross-origin front (e.g. `app.` vs `api.` subdomains, or a dev `localhost` port) works via CORS: when `api.cors_allow_origin` is set, the CORS layer SHALL send `Access-Control-Allow-Credentials: true` and echo the exact origin (never `*` alongside credentials — `*` mirrors the request origin), so the browser attaches the cookie. A truly **cross-site** front gets no `SameSite=Strict` cookie and keeps the Bearer path. The browser session is **single-account** (one cookie per origin); switching accounts is a fresh magic-link sign-in.

### Requirement: Magic-link verification is prefetch-safe and single-use
The magic-link email carries a single-use, hashed-at-rest, short-TTL token. It SHALL be consumed only by an explicit human action, never by a bare `GET` of the emailed URL: `GET /auth/magic/verify` SHALL render a click-to-confirm page and SHALL NOT verify or consume the token (its handler holds no store, so it cannot spend it), while a dedicated `POST /auth/magic/verify/confirm` performs the single-use verify. The confirm page SHALL be a plain form with no auto-submit, so email security scanners (SafeLinks, corporate proxies/AV) and browser link-prefetchers — which issue the GET but never submit — cannot burn the token before the human clicks. The token SHALL be reflected into the page only after validation as bounded lowercase hex, and the verification responses SHALL set `Referrer-Policy: no-referrer` so the query-string token does not leak via `Referer`. The programmatic JSON `POST /auth/magic/verify` (dashboard/SPA) is unaffected.

#### Scenario: A scanner or prefetch fetches the emailed link
- **GIVEN** a valid, unspent magic-link token
- **WHEN** the emailed `GET` URL is fetched by a link scanner or prefetcher
- **THEN** a confirm page is returned, the token is not consumed, and the human's later click still signs in

### Requirement: Sign out revokes the capability link
`POST /signout` SHALL invalidate the caller's capability link so it no longer authenticates any subsequent call.

#### Scenario: A signed-out link is reused
- **GIVEN** a capability link that has been signed out
- **WHEN** a later request presents it
- **THEN** the `Caller` extractor rejects it and no account scope is granted

### Requirement: Admin console is served on a loopback-only listener
Carillon SHALL expose administrative routes only on a second listener bound to a loopback address (`[admin] listen`, default `127.0.0.1:3001`), sharing the process `AppState` with the public listener. The loopback listener SHALL serve the FULL application (every normal route) PLUS the admin routes, so an operator who tunnels in can sign in with their whitelisted email ON THAT ORIGIN and then reach `/admin`. The public listener SHALL NOT mount any admin route, so an off-tunnel request to an admin path receives `404 Not Found` (the route does not exist there), not `403`. The sole intended access path is host-level: an SSH tunnel or SOCKS proxy to the loopback listener; the public reverse proxy SHALL front only the public listener. This transport boundary is the primary control — an application-layer auth defect alone cannot reach an admin verb.

#### Scenario: Admin route reached on the public listener
- **GIVEN** a request to an admin path arriving on the public listener
- **WHEN** the router resolves it
- **THEN** it is `404`, because the admin routes are not mounted on the public router

#### Scenario: Admin route reached over the tunnel
- **GIVEN** an operator with an `ssh -L` tunnel (or SOCKS proxy) to the loopback listener
- **WHEN** they request an admin route with a valid admin identity
- **THEN** the admin router serves it

### Requirement: The public API does not disclose the admin console
The publicly-served OpenAPI contract (`GET /openapi.yaml`, delivered raw) SHALL NOT document — or even mention in a comment — the loopback-only admin console, its routes, or the admin listener address. Advertising them on the public surface is needless reconnaissance; the admin API lives only on the loopback listener. Server error responses (HTTP 500) SHALL return a generic body (`{"error":"internal error"}`) with the real cause logged server-side, never the internal error string, so a public failure cannot leak SQL text, host names, or filesystem paths.

### Requirement: AdminCaller requires network position AND identity
Every admin route SHALL be gated by an `AdminCaller` extractor that authorizes a request only when it (a) arrived on the loopback admin listener AND (b) carries either a capability-link session whose account email is in `[admin] emails`, or the configured `api.admin_token`. Neither identity alone, presented on the public listener, SHALL reach any admin verb. A missing bearer is rejected `401`; a present-but-unauthorized bearer is rejected `403`.

#### Scenario: Whitelisted email over the tunnel
- **GIVEN** a capability session whose account email is in `[admin] emails`, presented on the admin listener
- **WHEN** `AdminCaller` resolves it
- **THEN** the request is authorized

#### Scenario: Admin token as break-glass
- **GIVEN** `api.admin_token` is set and presented on the admin listener
- **WHEN** `AdminCaller` resolves it
- **THEN** the request is authorized even if the magic-link / session chain is unavailable

#### Scenario: Non-whitelisted email
- **GIVEN** a valid capability session whose account email is NOT in `[admin] emails`
- **WHEN** `AdminCaller` resolves it
- **THEN** the request is rejected `403`

### Requirement: Admin console manages users, credits, and blacklist
The admin console SHALL provide, on the admin router only: listing accounts (id, email, credit balance, blocked flag, creation time, and watch count) and new-signup counts over a window; listing one account's individual watches (lazy-loaded per account); viewing per-account and fleet-aggregate credit balances; manually adjusting an account's credit balance up or down (a downward adjustment beyond the balance is refused `409`); and blocking or unblocking an account (`404` when the account is unknown).

### Requirement: A blocked account is inert
Carillon SHALL persist a per-account `blocked` flag. A blocked account SHALL be refused at authentication and SHALL NOT mint or refresh a capability session or create watches, so blacklisting an abusive account takes effect immediately across every front. An already-issued capability link SHALL stop authenticating once its account is blocked (the `Caller` and capability extractors reject a blocked account `403`).

#### Scenario: Blocked account attempts to authenticate
- **GIVEN** an account whose `blocked` flag is set
- **WHEN** it attempts `POST /auth` or a magic-link sign-in
- **THEN** the request is refused and no session is issued

#### Scenario: Blocked account reuses a live link
- **GIVEN** an account blocked after its capability link was issued
- **WHEN** a later request presents that link
- **THEN** the extractor rejects it `403` and no account scope is granted

### Requirement: Dev-only open admin console is compiled out of release
Carillon MAY offer a development convenience (`[admin] dev_allow_insecure`) that skips admin authentication and additionally mounts the admin routes on the public listener, so a local `npm run dev` frontend can reach the console at its normal API base without a token or sign-in. This bypass SHALL be effective ONLY in a debug build; a release build (the production binary) SHALL compile it out entirely and SHALL ignore the flag with a warning, so the production console always demands a real admin identity. When the bypass is effective, Carillon SHALL log a prominent startup warning. This exists because a debug-build gate is a compile-time guarantee, not merely a config default, so the footgun (behind a reverse proxy every request looks local) cannot reach production.

#### Scenario: dev_allow_insecure in a debug build
- **GIVEN** a debug build (`cargo run`) with `[admin] dev_allow_insecure = true`
- **WHEN** any request reaches an admin route on either listener
- **THEN** it is authorized with no credential, and a startup warning was logged

#### Scenario: dev_allow_insecure in a release build
- **GIVEN** a release build with `[admin] dev_allow_insecure = true`
- **WHEN** an admin route is requested on the public listener without an admin identity
- **THEN** the public listener has no admin route (`404`) and the admin listener still requires a real admin identity, the flag having been compiled out
