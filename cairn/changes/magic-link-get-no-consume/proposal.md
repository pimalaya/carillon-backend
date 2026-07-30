---
cairn: change
id: magic-link-get-no-consume
status: landed
created: 2026-07-31
---

# Magic-link verification must not consume the token on GET

## Why
The emailed sign-in link's API fallback opened `GET /auth/magic/verify?token=…`,
which **consumed** the single-use token immediately (minting the session and
`postMessage`-ing it back). That is an availability bug: email security scanners
(Outlook SafeLinks, corporate proxies / AV) and browser link-prefetchers issue a GET
on links in mail **before the human clicks**, spending the one-shot token. The
legitimate user then lands on an "invalid or expired sign-in link" — a silent,
hard-to-diagnose sign-in failure that scales with exactly the corporate/managed
inboxes a paying user is likely to have.

Secondary issue: the token rides in the query string, so it can leak via the
`Referer` header on any sub-resource the landing page loads.

This is the concrete, self-contained slice of the [[security-model]] Layer-4 auth
review (the account-takeover-via-interception / dashboard-XSS items remain separate,
larger follow-ups).

## What
- **GET SHALL NOT consume.** `GET /auth/magic/verify` renders a **click-to-confirm**
  page instead of verifying. The handler takes no store, so it *structurally* cannot
  spend the token. A plain HTML form (no auto-submit) means a scanner that fetches
  the page but never submits cannot burn the token.
- **Consume on an explicit POST.** A new `POST /auth/magic/verify/confirm`
  (form-encoded, so a bare button submits it with no JavaScript) does the single-use
  verify and returns the same `postMessage` popup as before.
- **No token reflection / no referrer leak.** The confirm page reflects the token
  only after validating it as bounded lowercase hex (else an empty slot), and sends
  `Referrer-Policy: no-referrer` (also added to the shared OAuth/magic popup).

## Non-goals
- The programmatic `POST /auth/magic/verify` (JSON, dashboard SPA) is unchanged.
- Session-token-in-`localStorage` (XSS) and email-interception takeover — the other
  Layer-4 auth items — are deliberately out of scope here.
