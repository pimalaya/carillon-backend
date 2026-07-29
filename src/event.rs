//! The canonical, content-free change event: the "doorbell".
//!
//! Every watcher folds its native change into this one generic shape. It
//! carries that *something changed* at an addressable place, and the
//! opaque resync token to catch up from — never the sender, subject,
//! body, a UID, or a change kind. Enriching the signal is the consumer's
//! job (it holds the credentials); a consumer re-derives what changed by
//! looking for itself, so a dropped or duplicated ring is harmless.
//!
//! The field names are deliberately **generic** — `source` (the account
//! kind), `target` (the watched thing), `state` (the resync token) —
//! carrying no IMAP/mailbox/message vocabulary, so a new account type is
//! a source addition, not a rename.
//!
//! [`id`](ChangeEvent::id) lets receivers dedupe retries;
//! [`ts`](ChangeEvent::ts) is folded into the signed preimage for replay
//! protection. Both are stamped once, at fold time, so every retry of the
//! same event carries the same id, timestamp and signature.

use rand::RngExt;
use serde::Serialize;

use crate::util::now_secs;

/// The signed, content-free payload POSTed to a watch's notify URL.
#[derive(Clone, Debug, Serialize)]
pub struct ChangeEvent {
    /// Unique event id (128-bit random, hex), stable across retries so
    /// receivers can dedupe.
    pub id: String,
    /// Unix timestamp (seconds) the event was folded, stable across
    /// retries and signed for replay protection.
    pub ts: i64,
    /// The watch (billing account's PIM account) this change belongs to.
    pub account: String,
    /// The account kind that rang: `imap` today (generic on purpose, more
    /// sources later).
    pub source: &'static str,
    /// The watched thing that changed: a mailbox name for IMAP, a
    /// collection reference for CardDAV. Opaque to the consumer.
    pub target: String,
    /// The opaque per-source resync token (IMAP `UIDVALIDITY:HIGHESTMODSEQ`
    /// or `UIDVALIDITY:UIDNEXT`, a CardDAV sync-token), used to catch up.
    /// Absent when the source exposes none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

impl ChangeEvent {
    /// Folds a change into the canonical shape, stamping a fresh id and
    /// timestamp. `source` is a static kind (`"imap"`, `"carddav"`),
    /// `target` the watched thing, `state` the resync token if any.
    pub fn new(
        account: impl Into<String>,
        source: &'static str,
        target: impl Into<String>,
        state: Option<String>,
    ) -> Self {
        Self {
            id: new_id(),
            ts: now_secs(),
            account: account.into(),
            source,
            target: target.into(),
            state,
        }
    }
}

/// A 128-bit random, hex-encoded event id.
fn new_id() -> String {
    format!("{:032x}", rand::rng().random::<u128>())
}
