---
cairn: delta
change: magic-link-get-no-consume
---

## ADDED Requirements

### Requirement: Magic-link verification is prefetch-safe and single-use
The emailed magic-link token SHALL be consumed only by an explicit, human-driven
action, never by a bare `GET` of the emailed URL. `GET /auth/magic/verify` SHALL NOT
verify or consume the token; it SHALL render a click-to-confirm page whose button
`POST`s to a dedicated confirm route that performs the single-use verify. The GET
handler SHALL NOT hold the store, so it cannot spend the token, and the confirm page
SHALL be a plain form (no auto-submit) so a scanner that only fetches the page cannot
burn the token. This defends the single-use token against email security scanners
(SafeLinks, corporate proxies/AV) and browser link-prefetchers that issue the GET
before the human clicks. The token SHALL be reflected into the page only after
validation as bounded lowercase hex, and the verification pages SHALL set
`Referrer-Policy: no-referrer` so the query-string token does not leak via `Referer`.
The programmatic JSON `POST /auth/magic/verify` (dashboard/SPA) is unaffected.

## MODIFIED Requirements

### Layer 4 auth-flows note (security-model)
The magic-link **prefetch/scanner token-burn** availability gap is closed (GET no
longer consumes). The remaining Layer-4 auth items — account takeover via magic-link
email interception and dashboard XSS reading the `localStorage` session token — stay
🟡 and are tracked separately.
