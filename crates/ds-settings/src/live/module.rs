//! What a daemon implements to serve a live module. The daemon owns the state; this crate only
//! carries the calls (the stateless-library rule of the quire refactor).

use std::future::Future;

use super::{LiveError, LiveSchema, Snapshot};
use crate::schema::KeyPath;

/// The unique bus name of the peer that made a call (`:1.42`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderName(pub String);

/// Who is calling: the sender's bus name and the connection that received the call, so the
/// implementer can resolve the peer's role its own way (accountd's `Peer` registry, or the bus's
/// connection credentials).
#[derive(Debug, Clone)]
pub struct Caller {
    pub sender: SenderName,
    pub connection: zbus::Connection,
}

/// What a call does to the module's state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Access {
    /// `Get` and `Snapshot`.
    Read,
    /// `Set`, including running an action key.
    Write,
}

/// The answer to [`LiveModule::permit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    /// The caller's role may not do this. The call fails with `NotPermitted`.
    Refuse,
}

/// One live module: its schema, its values, and who may change them.
///
/// accountd serves one at `/org/quire/Accounts1/settings`, inferd one at
/// `/org/quire/Inference1/settings`; both `Refuse` every `Write` unless the caller holds the
/// Settings role.
pub trait LiveModule: Send + Sync + 'static {
    /// The caller-role hook, asked before every `Describe`, `Get` and `Snapshot` (`Access::Read`) and
    /// every `Set` (`Access::Write`). A peer it lets read is also the peer `Changed` is sent to, so
    /// it is the one place that decides who hears a value. No default: a module states its policy.
    fn permit(&self, caller: &Caller, access: Access) -> impl Future<Output = Verdict> + Send;

    /// The keys right now (accounts come and go, so this is asked on every `Describe`).
    fn describe(&self) -> impl Future<Output = LiveSchema> + Send;

    /// A key's current value, or [`LiveError::UnknownKey`].
    fn get(&self, key: &KeyPath) -> impl Future<Output = Result<toml::Value, LiveError>> + Send;

    /// The schema and every value, consistent with each other (`Snapshot` on the bus; one call
    /// where `describe` then a `get` per key could see the module change in between). The default
    /// reads `describe` then each key and drops a key that answers `UnknownKey` from schema and
    /// values alike; a module that can read its state once overrides it, and asks no permission
    /// here (the skeleton asked `Access::Read` already).
    fn snapshot(&self) -> impl Future<Output = Result<Snapshot, LiveError>> + Send {
        async move {
            let schema = self.describe().await;
            let mut reads = Vec::with_capacity(schema.key.len());
            for spec in &schema.key {
                reads.push((spec.path.clone(), self.get(&spec.path).await));
            }
            Snapshot::from_reads(schema, reads)
        }
    }

    /// Sets a key, or runs it when it is an action key (then `value` is `true` by convention and
    /// carries nothing). The skeleton sends `Changed` with the value a following `get`
    /// reports to the peers that may read, so an implementation does not signal its own `Set`.
    fn set(
        &self,
        key: &KeyPath,
        value: toml::Value,
    ) -> impl Future<Output = Result<(), LiveError>> + Send;
}
