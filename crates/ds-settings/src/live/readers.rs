//! Who is sent `Changed`: the peers `permit` let read, by unique name.
//!
//! A signal with no destination goes to every peer that added a match rule for it, and anyone on
//! the bus may add one, so a broadcast `Changed` would hand every value to an app that `permit`
//! refused. The skeleton remembers the peers it let read (`Describe`, `Get`, `Snapshot`, or
//! [`LiveClient::changes`](super::LiveClient::changes), which reads once) and sends `Changed` to
//! each of them alone.

use super::SenderName;
use ds_peer::{Deadline, within};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;
use zbus::object_server::SignalEmitter;

/// The most readers remembered at once: the Settings app, a second window of it, a tool. A peer
/// beyond this (after the departed are dropped) pushes out the oldest.
const MAX_READERS: usize = 32;

/// The peers that may read, oldest first. Cheap to clone; the skeleton and [`Served`](super::Served)
/// share one.
#[derive(Debug, Clone, Default)]
pub(super) struct Readers(Arc<Mutex<Vec<SenderName>>>);

impl Readers {
    fn names(&self) -> MutexGuard<'_, Vec<SenderName>> {
        // A poisoned lock still holds a whole list: the only writers push and retain.
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Remember `reader`, which `permit` let read.
    pub(super) async fn add(&self, connection: &Connection, reader: &SenderName) {
        let full = {
            let mut names = self.names();
            if names.contains(reader) {
                return;
            }
            names.push(reader.clone());
            names.len() > MAX_READERS
        };
        if full {
            self.forget_departed(connection).await;
            let mut names = self.names();
            let excess = names.len().saturating_sub(MAX_READERS);
            drop(names.drain(..excess));
        }
    }

    /// The readers still on the bus.
    pub(super) async fn present(&self, connection: &Connection) -> Vec<SenderName> {
        self.forget_departed(connection).await;
        self.names().clone()
    }

    async fn forget_departed(&self, connection: &Connection) {
        if let Some(on_bus) = names_on_bus(connection).await {
            self.names().retain(|reader| on_bus.contains(&reader.0));
        }
    }
}

/// Every name the bus knows now, or `None` when it did not say.
async fn names_on_bus(connection: &Connection) -> Option<Vec<String>> {
    let listed = within(Deadline::READ, "The message bus", async {
        DBusProxy::new(connection)
            .await
            .ok()?
            .list_names()
            .await
            .ok()
    })
    .await
    .ok()??;
    Some(listed.iter().map(ToString::to_string).collect())
}

/// `emitter` addressed to `reader` alone, so the signal it emits reaches that peer and no match
/// rule. `None` when `reader` is not a bus name.
pub(super) fn addressed<'e>(
    emitter: &SignalEmitter<'e>,
    reader: SenderName,
) -> Option<SignalEmitter<'e>> {
    let name = BusName::try_from(reader.0).ok()?;
    Some(emitter.clone().set_destination(name))
}
