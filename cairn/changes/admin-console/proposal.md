---
cairn: change
id: admin-console
status: landed
created: 2026-07-24
---

# A localhost-only admin console for user, credit, and blacklist management

## Why
There is today no way to run the fleet: see how many accounts exist and how
fast they are signing up, inspect and manually adjust an account's credit
balance, or block an abusive account. The only administrative primitive is the
shared `api.admin_token` (§ [[auth]]), which grants *unscoped, fleet-wide read
+ create-anywhere* on the **public** listener — but it exposes no management
verbs (no credit-adjust, no blacklist, no signups view) and, being reachable
from the internet, is a single-factor break-glass secret. Growing it into the
key for destructive account administration on the public interface would be a
real regression: a leaked token (config, cron env, `journalctl`, shell history)
would become remote account takeover for the whole fleet.

We want an admin surface that is safe *by construction*, not merely by correct
application-layer auth:

1. **The destructive verbs must not be reachable from the public internet at
   all** — an application-layer auth bug (magic-link, session resolution, the
   whitelist check) must not be sufficient to reach them. Network position is
   the strong, non-app-layer factor.
2. **Admin identity should be per-person**, so admins can be added/removed
   without rotating a shared secret, and so consequential actions
   (credit-adjust, blacklist) are attributable.
3. **Break-glass must survive the session infra being broken** — if magic-link
   / session resolution is itself the thing being fixed, there must still be a
   way in that does not depend on it.

## What

### Transport: a second, loopback-only listener
Bind a **second `axum` listener on `127.0.0.1` (`[admin] listen`)** in `serve()`,
alongside the existing public listener, sharing the same `AppState` and handler
module — *not* a separate service. The admin routes are mounted **only** on this
router; the public router never learns them (an off-tunnel probe gets `404`, not
`403` — nothing to leak). The sole way to reach the admin listener is host-level
access: an `ssh -L` tunnel or SOCKS proxy. The reverse proxy in front of the
public deployment proxies only the public listener; the admin listener is never
proxied. This transport boundary is the primary control.

### Identity: whitelisted email OR the admin token
An `AdminCaller` extractor gates every admin route, accepting **either**:
- a capability-link session whose account email ∈ `[admin] emails` (the everyday
  human path, attributable), **or**
- the existing `api.admin_token` (break-glass — does not depend on the
  magic-link / session chain, so it still works when that chain is broken).

Either alone is useless without also being on the loopback listener. This is
defense in depth: two independent factors (network position AND identity), and
an app-auth bug alone is not enough.

### Verbs (first cut)
On the admin router only:
- **View users & signups** — list accounts (id, email, credits, blocked,
  created), plus new-signup counts over a window.
- **View credits** — per-account balance and fleet aggregate.
- **Adjust credits** — add/remove credits on an account (`store::add_credits` /
  `debit_credits` already exist).
- **Blacklist** — block/unblock an account; a blocked account is refused at auth
  and cannot mint/refresh a session or create watches.

### Store additions
The `account` table needs two columns it lacks today: `created_at` (for the
signups view; currently absent) and `blocked` (for blacklist enforcement), plus
`list_accounts_admin`, `set_blocked`, and a signup-count query. Credit-adjust
reuses existing methods.

### Frontend
The admin views are **routes under `/admin` in the existing SPA** — no separate
build, no nav/access button anywhere (a bare route is enough). Because the
self-host embed resolves the API base as `window.location.origin`
(`frontend/src/lib/config.ts`), the same build is inert off-tunnel for free:
loaded over the public URL its `/admin/*` fetches hit the public origin and
`404`; loaded over the tunnel they hit the loopback origin and work. This is
exactly the intended "the front just won't work because the API isn't found"
behaviour, at zero extra frontend infrastructure. (Companion change in the
frontend repo adds the `/admin` route + views.)

### Non-goals
No change to the public listener's behaviour or to `api.admin_token`'s existing
public-listener powers (unchanged blast radius on the public side). No admin
nav/discovery in the UI. No role hierarchy beyond the flat email whitelist.
