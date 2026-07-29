---
cairn: spec
capability: webhooks
status: current
---

# Webhook Delivery Contract

When a watched source changes, Carillon POSTs a small, content-free JSON body to the user's notify URL. The signature is the authentication: the notify URL is a public endpoint anyone who learns it can POST to, so genuineness, integrity, and freshness are carried entirely by a Stripe-style HMAC-SHA256 signature over a timestamped preimage, backed by event-id idempotency. See [[overview]] for the signal-not-sync principle, and [[carddav]] for the CardDAV variant of the payload.

### Requirement: Content-free payload
The delivery body SHALL be a JSON object of six generic fields and nothing more: `id` (a per-event id) and `ts` (an observation timestamp); the watch identity (`account`); the account kind (`source`, e.g. `imap`); the watched thing that changed (`target`, a mailbox for IMAP, a collection for CardDAV); and the opaque per-source resync token (`state`, absent when the source exposes none). It SHALL NOT carry a UID, a change kind, a resource href, or any sender, subject, body, envelope, or other message content. The names are generic so a new account type is a source addition, not a rename. A consumer wanting richer detail fetches it itself with its own credentials after the ping, re-deriving what changed from `state`; Carillon never sees it.

### Requirement: Delivery headers
Each delivery SHALL carry `X-Carillon-Signature` (the `t=…,v1=…` signature described below), `X-Carillon-Id` (the event `id`, for idempotency), `X-Carillon-Source` (the source kind), `X-Carillon-Account` (the watch id), and `Content-Type: application/json`.

### Requirement: Stripe-style HMAC-SHA256 signature
Carillon SHALL sign each delivery with HMAC-SHA256 over the preimage `"{ts}.{raw_body}"` — the timestamp, a literal `.`, then the exact raw request body — and SHALL present the result in `X-Carillon-Signature` as `t=<ts>,v1=<hex>`, carrying one or more `v1` values. Including the timestamp inside the signed content is what enables replay protection; the signed body is what makes tampering detectable.

### Requirement: Replay protection
The signed `t` SHALL let a receiver reject stale or replayed deliveries by bounding `abs(now - t)` against a tolerance (~5 minutes). Because `t` is inside the signed preimage, it cannot be altered without invalidating the signature.

### Requirement: Idempotency by event id
Each event SHALL carry an `id` that is unique per event and stable across retries, so a receiver can dedupe. Because Carillon retries failed deliveries, the same `id` can arrive more than once and a repeated `id` SHALL be treated as already handled.

### Requirement: HTTPS-only notify URLs
Carillon SHALL deliver only over `https://`, with the sole exception of a loopback host (for local sinks and self-host). A non-loopback `http://` notify URL SHALL be refused at watch-creation time.

### Requirement: Secret rotation with dual-signed overlap
Rotating a watch's HMAC secret SHALL support an overlap window during which each delivery is signed with both the previous and the new secret, presented as multiple `v1` values in the header. A receiver holding either secret SHALL keep validating through the cutover with no dropped events; once the overlap expires, only the new secret is used.

### Requirement: Receiver verification
A receiver SHALL, on every call: read `t` and the `v1` list from the header; recompute `hex(HMAC_SHA256(secret, t + "." + raw_body))` using the exact raw bytes received without re-serializing the JSON; constant-time compare it against each `v1` and accept on any match; enforce the timestamp tolerance; dedupe by event `id`; and respond `2xx` quickly, processing asynchronously.

#### Scenario: A secret is mid-rotation
- **GIVEN** a watch whose secret is rotating within its overlap window
- **WHEN** a delivery arrives signed with both the old and new secrets (two `v1` values)
- **THEN** a receiver still holding the old secret matches on the old `v1` and accepts
- **AND** a receiver already updated to the new secret matches on the new `v1` and accepts
