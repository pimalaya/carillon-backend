---
cairn: delta
id: server-core-reconcile
---

## MODIFIED Requirements

### Requirement: The server watches by driving carillon-core
The server SHALL watch a mailbox by driving `carillon-core`'s I/O-free
`CarillonImapWatch` coroutine with an async pump, not a bespoke watch loop. The
server SHALL own only the transport (TCP, TLS, SSRF guard, keepalive), the
reconnect/backoff, the credential resolution, and the event fan-out; the greet /
authenticate / EXAMINE / IDLE / re-EXAMINE protocol SHALL live in core, shared
with the CLI. `probe`, mailbox listing, and `/test` MAY keep their own one-shot
io-imap usage, since core does not cover them.

#### Scenario: A watcher fix lands in both frontends
- **GIVEN** a defect in the IDLE handling fixed in carillon-core
- **WHEN** the server is rebuilt
- **THEN** it inherits the fix with no server-side watch loop to patch, exactly
  as the CLI does

### Requirement: The delivered event is the content-free doorbell
The event the server signs and delivers SHALL be `(id, ts, account, source,
target, state)` and nothing more. It SHALL NOT carry a message UID, a change
kind, or a CardDAV resource href. The webhook SHALL carry the source in an
`x-carillon-source` header (replacing `x-carillon-event`); the signing scheme
(`HMAC-SHA256` over `"{ts}.{body}"`) is unchanged. The delivery log SHALL record
the source, target, and state rather than the kind and UID.

#### Scenario: A new mail delivers a content-free webhook
- **GIVEN** a watched mailbox on the server
- **WHEN** a message arrives
- **THEN** the signed webhook body is `(id, ts, account, source=imap, target,
  state)`, with no uid / kind / sender / subject, and a consumer re-derives what
  changed by looking for itself
