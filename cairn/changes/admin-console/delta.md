---
cairn: delta
change: admin-console
---

## ADDED Requirements

### Requirement: Admin console is served on a loopback-only listener
Carillon SHALL expose administrative routes only on a second listener bound to a
loopback address (`[admin] listen`), sharing the process `AppState` with the
public listener but mounting the admin routes exclusively on the admin router.
The public router SHALL NOT mount any admin route, so an off-tunnel request to an
admin path receives `404 Not Found` (the route does not exist there), not `403`.
The sole intended access path is host-level: an SSH tunnel or SOCKS proxy to the
loopback listener; the public reverse proxy SHALL front only the public listener.

#### Scenario: Admin route reached on the public listener
- **GIVEN** a request to an admin path arriving on the public listener
- **WHEN** the router resolves it
- **THEN** it is `404`, because the admin routes are not mounted on the public router

#### Scenario: Admin route reached over the tunnel
- **GIVEN** an operator with an `ssh -L` tunnel (or SOCKS proxy) to the loopback listener
- **WHEN** they request an admin route with a valid admin identity
- **THEN** the admin router serves it

### Requirement: AdminCaller requires network position AND identity
Every admin route SHALL be gated by an `AdminCaller` extractor that authorizes a
request only when it (a) arrived on the loopback admin listener AND (b) carries
either a capability-link session whose account email is in `[admin] emails`, or
the configured `api.admin_token`. Neither identity alone, presented on the public
listener, SHALL reach any admin verb. This is defense in depth: an
application-layer auth defect (magic-link, session resolution, or the whitelist
check) is insufficient to reach admin functionality without also host access.

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
- **THEN** the request is rejected

### Requirement: Admin console manages users, credits, and blacklist
The admin console SHALL provide, on the admin router only: listing accounts (id,
email, credit balance, blocked flag, creation time) and new-signup counts over a
window; viewing per-account and fleet-aggregate credit balances; manually
adjusting an account's credit balance up or down; and blocking or unblocking an
account.

### Requirement: A blocked account is inert
Carillon SHALL persist a per-account `blocked` flag. A blocked account SHALL be
refused at authentication and SHALL NOT mint or refresh a capability session or
create watches, so blacklisting an abusive account takes effect immediately
across every front.

#### Scenario: Blocked account attempts to authenticate
- **GIVEN** an account whose `blocked` flag is set
- **WHEN** it attempts `POST /auth` or to mint/refresh a session
- **THEN** the request is refused and no session is issued

## MODIFIED Requirements

### Requirement: The Caller extractor resolves the bearer
A `Caller` extractor SHALL resolve the presented bearer token to exactly one of
two identities: a capability-link account, scoped to its own watches,
deliveries, events, and pool; or the optional unscoped admin token
`api.admin_token`, which grants fleet-wide access for ops and headless use. When
`api.admin_token` is unset (the default), no unscoped access exists at all. The
`api.admin_token` additionally authorizes the loopback admin console (see
`AdminCaller`) as a break-glass identity; its powers on the public listener are
unchanged by the admin console.
