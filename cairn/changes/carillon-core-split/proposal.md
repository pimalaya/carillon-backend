---
cairn: change
id: carillon-core-split
status: active
created: 2026-07-27
---

# Extract a shared `carillon-core` daemon; make the CLI and the backend two frontends over it

## Why

The watch loop — connect, authenticate, IDLE, detect a change, dispatch — lives
today only inside the backend as tokio tasks (`supervisor.rs`, `imap/`,
`event.rs`, `delivery.rs`). The CLI (`cli/`, crate `carillon`) is an empty stub:
one `main.rs`, no dependencies. As soon as the CLI grows a real watch loop it
will either duplicate the backend's, or the two will quietly diverge. They must
not: a watcher bug fixed in one should be fixed in both, and a new source
protocol added once should light up everywhere.

The deeper realisation is that **the backend is not a bigger thing than the CLI;
it is the same watcher plus a network-and-trust apparatus.** The single-VPS
`serve()` process (§ [[architecture]]) already interleaves "run the watchers"
with "expose an axum control API". Split those two responsibilities cleanly and
the watcher half is exactly what a self-hostable CLI daemon wants — and a
free, self-hostable `carillon` daemon is the top of the funnel that makes "let
us host the watch for you" the natural next sentence.

So: factor the watcher into a shared **`carillon-core`** crate, and let both the
CLI and the backend be thin frontends over it. Neither wraps the other.

## What

### The watch is a doorbell, not a mailman

The event Carillon emits is already **content-free and self-addressed** (§
[[architecture]], § [[webhooks]]): it carries the event type and the mailbox /
collection identity, and *nothing about what changed*. This is deliberate and it
is what makes the whole split possible. Because the payload carries no content:

- The watch has no idea what a "sync" or a "notification" even is — it just
  rings a doorbell that says *"something changed at this address."*
- What "go look" **means** is entirely the consumer's business: a sync consumer
  diffs and reconciles; a notifier shows a toast; a shell hook runs a command; a
  webhook consumer POSTs the signal onward.
- A dropped or duplicated ring is **harmless**: the consumer re-derives truth
  (diffs, re-reads, re-notifies), so a missed signal is recovered by the next
  ring or a periodic re-check. A dumb pinger plus a reconciling consumer is
  self-healing in a way a structured-body watch never is.

"Content-free" does not mean *anonymous*: the event stays **self-identifying**
— `(id, ts, account, source, target, state)` — so a consumer can route
("notify *which* mailbox") and dedup ("already handled this state"). The sweet
spot is JMAP's `StateChange`: enough to address and dedup, zero content.

The shipped `ChangeEvent` is richer than this — it carries a `uid`, a CardDAV
`resource` href, and a `ChangeKind` (`new` / `changed` / `flags_added` / …).
All three are **dropped**: a doorbell does not say *what* changed or *how*, only
that something did, at this address. `id` and `ts` are kept (stamped once at
fold, stable across retries, signed downstream for replay); `state` is the
opaque per-source token (`UIDNEXT`/`MODSEQ`, CardDAV sync-token) a consumer uses
to resync. The product is in beta, so this break is free to take now.

### `carillon-core`: the shared async watch client

`carillon-core` is **not** sans-io. A sans-io "watch aggregator" would repeat
the io-email / io-addressbook / io-calendar mistake: an interface layer whose
maintenance cost outweighs what it saves, which is why every Pimalaya consumer
now rebuilds its own client directly over `io-imap` / `io-webdav` / `io-http`.
Core does the same — it owns the **async network I/O of watching** — with one
difference: it owns it *once*, so both frontends share a single loop instead of
maintaining two. That is the whole, bounded value; core does not try to unify
every operation the way the aggregators did.

A new crate owns:

- the **`Event`** type — the ideal content-free, self-addressed ring, one shape
  for every consumer (see below);
- the **async watch client** — `io-imap` IDLE now, `io-jmap` EventSource /
  `io-maildir` notify / CardDAV poll later; each source declares its **transport
  class** (see below), and only IMAP IDLE is wired at MVP;
- the **`Backend`** to watch and a resolved **`Credential`** (a password or a
  pre-minted bearer token) — core knows only the auth *mechanism* and its
  secret;
- the **one-session `watch` entry point** — watch a caller-opened stream, ring
  into a channel until the session ends.

What core deliberately does **not** own: **credential resolution** (keyring
lookup and OAuth minting/refresh happen upstream — OAuth is just a token to
core), storage, billing, delivery, the consumer fan-out, and
**reconnect/backoff supervision** (core runs one session and returns when it
drops; the frontend decides whether and when to reconnect, resolving a fresh
credential per attempt), and the **transport** itself (TCP, TLS, keepalive,
address/SSRF policy): core drives the protocol clients over a stream the frontend
opened, staying generic over it. Those stay at the frontend edge. So core *does*
pull in an async runtime and the protocol clients (that is the point), but never
a TLS stack, a datastore, a keyring, an OAuth exchange, a notification library,
or process spawning.

### Two kinds of "consumer" — do not conflate them

The word "consumer" has been doing double duty. Pin it down:

1. **Event consumers** — the reactions a frontend runs on each ring: `notify`,
   `exec`, `webhook`, `sse`, and later `sync` (→ neverest). They read the events
   core emits. There is no `Consumer` trait in core: core emits one
   `CarillonEvent` into a channel, and each frontend owns the fan-out to its own
   reactions.
2. **Frontends** — the CLI and the backend. These *host* core and drain its
   event channel.

The CLI and the backend are **not** event consumers. They are two frontends,
and each one *bundles a set of* event consumers plus *a way to ingest watch
config*.

### Two frontends, neither wraps the other

Both depend on `carillon-core` as equals. The backend does not embed the CLI
binary (that would drag in clap, the TTY printer, the spinner, TOML parsing —
none of which a service wants); the CLI does not embed the backend. The
containment intuition ("the server is a superset") is real, but it is expressed
by **sharing a core**, not by one binary wrapping the other.

| | **`carillon`** (CLI daemon) | **`carillon-server`** (backend) |
|---|---|---|
| Hosts `carillon-core` | ✓ | ✓ |
| Config *ingestion* | a TOML file, like every other Pimalaya tool | HTTP API (`Add service`) + a server TOML for ports/keys/db |
| Bundled event consumers | `notify`, `exec` (`sync` later) | `webhook`, `sse` |
| Transport classes hosted | `standing-connection` + `poll` | `standing-connection` + `poll` + `public-callback` |
| Extra apparatus | none | auth, credential custody, metering, billing, SQLite |

### Sources have a transport class; a frontend hosts only what it can satisfy

Not every source can be acquired the same way, and the difference maps exactly
onto the frontend split. A source declares its **transport class**:

- **`standing-connection`** — the watcher dials *out* and holds an outbound
  connection: `io-imap` IDLE, `io-jmap` EventSource, `io-maildir` notify. No
  public endpoint or external infrastructure needed, so *any* frontend — the
  CLI included — can host it.
- **`poll`** — the watcher dials *out* but re-checks on an interval rather than
  holding a connection: CardDAV `sync-collection`. Also NAT-friendly and needs
  no public endpoint, so any frontend can host it; it differs from
  `standing-connection` only in liveness cost, not in what can reach it.
- **`public-callback`** — the source is mediated by cloud infrastructure that
  delivers by POSTing *in*: Gmail push (Google Cloud Pub/Sub → `historyId`
  ping), Microsoft Graph subscriptions (`notificationUrl`), WebDAV-Push. These
  need Carillon itself to host a public HTTPS endpoint (and, for Gmail, a GCP
  project + Pub/Sub topic), so only the **backend** can host them.

The `Event` is identical either way — Gmail's `{emailAddress, historyId}` is
just another content-free, self-addressed ring — so nothing downstream of the
source cares which class produced it. Only *acquisition* differs.

Each frontend advertises which transport classes it can host. `carillon-core`
SHALL refuse to arm a watch whose source class the hosting frontend does not
advertise, so "the CLI can't watch Gmail" falls out as a property rather than a
special case: the CLI hosts only `standing-connection` and `poll`, so
`public-callback` sources like Gmail/Graph are never offered there and its
wizard never lists them. This is the mirror
image, on the *source* side, of the delivery-side rule — the public endpoint is
the backend's defining apparatus, and it gates both what Carillon can *receive*
(callback sources) and what it can *offer* (webhook delivery).

(A Pub/Sub *pull* subscription is technically outbound and could in principle
let a client behind NAT receive Gmail pings — but it still demands a
user-owned GCP project, topic, subscription, and service-account credential,
which defeats the point of a lightweight local daemon. So Gmail stays
`public-callback` / backend-only; the CLI's only honest Gmail option is a poll
fallback, which is polling, not watching, and out of scope here.)

### Config: types in core, ingestion in the frontend

The config *types* are already core's own: a source is a `CarillonBackend` and
its secret is a resolved `CarillonCredential`. There is no separate
`WatchConfig` struct; a frontend wraps those in its own watch registration
(adding an account id and its enabled consumers) and owns *ingestion*. The CLI
builds them from a TOML file; the backend builds them from an `Add service` API
call (§ [[service-model]]). This is the precise sense in which the CLI and the
backend are "two frontends of the daemon": they differ only in **where watch
registrations come from** and **which event consumers ship enabled.**

### Webhook / SSE are just the backend's network consumers

The delivery split already in flight (§ [[webhooks]], the `sse-change-stream`
change) drops out of this cleanly: a **webhook is a network event consumer**
(push to a public endpoint the consumer hosts) and **SSE is another** (an
authenticated long-lived pull, for consumers that cannot be reached inbound —
dashboards, desktop clients). They are the same kind of event consumer as the
CLI's `notify`/`exec`, draining the same core event channel; only the transport
differs. (Note SSE is not the phone story —
holding the socket open drains mobile batteries; phones stay
webhook/SSE → relay → UnifiedPush/FCM.)

### Naming & repo layout

Rename to a scheme where every repo is one word after the prefix, with no
"backend/frontend" relativity now that "frontend" means *core-driver*:

- **`carillon-core`** — the shared daemon library (new crate).
- **`carillon`** — the CLI daemon (already the crate name in `cli/`).
- **`carillon-server`** — the backend service (revert from `backend`; the git
  remote is already `carillon-server`, so the rename is free). "Server" = "the
  daemon wrapped as a service," not "the thing paired with a frontend."
- **`carillon-admin`** — the management UI (rename from `carillon-frontend`).
  A user-scoped self-serve dashboard (with the hidden `/admin` route), named
  for what it is and free of the overloaded word "frontend".
- **`carillon-website`** — the public landing/marketing site (kept as-is; not
  renamed to `carillon-site`).

`carillon-core` becomes its own crate that both frontends depend on as equals,
reinforcing "neither wraps the other."

## Non-goals

- No new source protocols in this change — JMAP EventSource and Maildir notify
  are named as core's shape but only IMAP IDLE is wired, as today.
- No `sync` consumer yet — it is named as the archetypal future consumer to
  prove the model, not built here.
- No change to the delivery contract, the account/service model, billing, or
  auth. This is a **structural refactor plus a rename**: the same behaviour,
  reorganised so the watcher is shared and the two frontends are peers.
- No change to how the console is served or embedded (§ [[serving]]).
