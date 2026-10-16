//! The `com.canonical.dbusmenu` object: the methods a client calls, the signals it listens for.

use super::pace::{EVENTS_PER_CALL_MAX, Pace, Pass};
use super::permit::{self, Caller, MenuPermit, SenderName, Verdict};
use super::wire::{self, Layout, Props, WireError};
use ds::base::command::CommandId;
use ds::components::menus::export::{
    Change, Depth, Dispatch, MenuState, MenuTree, NodeId, PropNames, Revision,
};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;
use zbus::fdo;
use zbus::message::Header;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

type Activate = Box<dyn Fn(CommandId) + Send + Sync>;

/// What the object and its owner share: the state being served, and where a click goes.
pub(super) struct Shared {
    state: Mutex<MenuState>,
    activate: Activate,
}

impl Shared {
    pub(super) fn new(state: MenuState, activate: Activate) -> Shared {
        Shared {
            state: Mutex::new(state),
            activate,
        }
    }

    fn state(&self) -> MutexGuard<'_, MenuState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn revision(&self) -> Revision {
        self.state().revision
    }

    /// Swap in `tree`: what changed, the revision now, and the tree.
    pub(super) fn replace(&self, tree: MenuTree) -> (Change, Revision, MenuTree) {
        let mut state = self.state();
        let (next, change) = state.clone().replaced(tree);
        *state = next;
        (change, state.revision, state.tree.clone())
    }
}

/// A [`MenuPermit`] with its future boxed, so the served object names no type parameter.
trait AskPermit: Send + Sync {
    fn ask<'a>(&'a self, caller: &'a Caller) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'a>>;
}

impl<P: MenuPermit> AskPermit for P {
    fn ask<'a>(&'a self, caller: &'a Caller) -> Pin<Box<dyn Future<Output = Verdict> + Send + 'a>> {
        Box::pin(self.permit(caller))
    }
}

/// The served object. `permit` decides who may press an item; reads (`GetLayout` and the rest)
/// answer anyone, because the menu is what the window already shows.
pub(super) struct DbusMenu {
    shared: Arc<Shared>,
    permit: Box<dyn AskPermit>,
    pace: Mutex<Pace>,
}

impl DbusMenu {
    pub(super) fn new(shared: Arc<Shared>, permit: impl MenuPermit) -> DbusMenu {
        DbusMenu {
            shared,
            permit: Box::new(permit),
            pace: Mutex::new(Pace::default()),
        }
    }

    /// The sender of an item press, once the permit lets it: a message that names no sender, or
    /// one the permit refuses, is `AccessDenied`.
    async fn admit(
        &self,
        header: &Header<'_>,
        connection: &zbus::Connection,
    ) -> fdo::Result<SenderName> {
        let sender = permit::named(header)?;
        let caller = Caller {
            sender: sender.clone(),
            connection: connection.clone(),
        };
        match self.permit.ask(&caller).await {
            Verdict::Allow => Ok(sender),
            Verdict::Refuse => Err(permit::denied()),
        }
    }

    fn pace(&self) -> MutexGuard<'_, Pace> {
        // A poisoned lock still holds a whole table: the only writer inserts and prunes.
        self.pace.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn click(&self, id: i32, event_id: &str) -> fdo::Result<()> {
        let dispatch = self.shared.state().tree.event(NodeId(id), event_id);
        match dispatch {
            Ok(Dispatch::Run(command)) => {
                (self.shared.activate)(command);
                Ok(())
            }
            Ok(Dispatch::Nothing) => Ok(()),
            Err(error) => Err(fdo::Error::InvalidArgs(error.to_string())),
        }
    }
}

fn failed(error: WireError) -> fdo::Error {
    fdo::Error::Failed(error.0)
}

#[zbus::interface(name = "com.canonical.dbusmenu")]
impl DbusMenu {
    fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: Vec<String>,
    ) -> fdo::Result<(u32, Layout)> {
        let state = self.shared.state();
        let layout = state
            .tree
            .layout(
                NodeId(parent_id),
                Depth::from_wire(recursion_depth),
                &PropNames(property_names),
            )
            .ok_or_else(|| fdo::Error::InvalidArgs(format!("no menu item has id {parent_id}")))?;
        Ok((state.revision.0, wire::layout(&layout).map_err(failed)?))
    }

    fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> fdo::Result<Vec<(i32, Props)>> {
        let ids: Vec<NodeId> = ids.into_iter().map(NodeId).collect();
        let items = self
            .shared
            .state()
            .tree
            .group_properties(&ids, &PropNames(property_names));
        items
            .iter()
            .map(wire::item)
            .collect::<Result<_, _>>()
            .map_err(failed)
    }

    fn get_property(&self, id: i32, name: String) -> fdo::Result<zbus::zvariant::OwnedValue> {
        let prop = self.shared.state().tree.property(NodeId(id), &name);
        prop.ok_or_else(|| fdo::Error::InvalidArgs(format!("item {id} has no property {name}")))
            .and_then(|prop| {
                wire::props(&[prop])
                    .map_err(failed)?
                    .into_values()
                    .next()
                    .ok_or_else(|| fdo::Error::Failed("no value".to_owned()))
            })
    }

    async fn event(
        &self,
        id: i32,
        event_id: String,
        _data: Value<'_>,
        _timestamp: u32,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> fdo::Result<()> {
        self.admit(&header, connection).await?;
        self.click(id, &event_id)
    }

    /// The ids whose event was refused. At most [`EVENTS_PER_CALL_MAX`] events a call and
    /// `CALLS_PER_SECOND_MAX` calls a second from one sender.
    async fn event_group(
        &self,
        events: Vec<(i32, String, Value<'_>, u32)>,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &zbus::Connection,
    ) -> fdo::Result<Vec<i32>> {
        let sender = self.admit(&header, connection).await?;
        let pass = self.pace().admit(&sender, Instant::now());
        if pass == Pass::Wait {
            return Err(fdo::Error::LimitsExceeded(
                "too many event groups a second".to_owned(),
            ));
        }
        if events.len() > EVENTS_PER_CALL_MAX {
            return Err(fdo::Error::InvalidArgs(format!(
                "an event group carries at most {EVENTS_PER_CALL_MAX} events"
            )));
        }
        let refused: Vec<i32> = events
            .iter()
            .filter(|(id, event_id, _, _)| self.click(*id, event_id).is_err())
            .map(|(id, ..)| *id)
            .collect();
        match (refused.len(), events.len()) {
            (refused, total) if refused == total && total > 0 => Err(fdo::Error::InvalidArgs(
                "no event was for a menu item".to_owned(),
            )),
            _ => Ok(refused),
        }
    }

    /// The menu is always current: nothing to refetch.
    fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    fn about_to_show_group(&self, _ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        (Vec::new(), Vec::new())
    }

    #[zbus(signal)]
    async fn items_properties_updated(
        emitter: &SignalEmitter<'_>,
        updated: Vec<(i32, Props)>,
        removed: Vec<(i32, Vec<String>)>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn layout_updated(
        emitter: &SignalEmitter<'_>,
        revision: u32,
        parent: i32,
    ) -> zbus::Result<()>;

    #[zbus(property)]
    fn version(&self) -> u32 {
        3
    }

    #[zbus(property)]
    fn text_direction(&self) -> String {
        "ltr".to_owned()
    }

    #[zbus(property)]
    fn status(&self) -> String {
        "normal".to_owned()
    }

    #[zbus(property)]
    fn icon_theme_path(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Tell the clients of `change`, which brought `tree` to `revision`.
pub(super) fn announce(
    connection: &zbus::blocking::Connection,
    change: &Change,
    revision: Revision,
    tree: &MenuTree,
) -> Result<(), super::MenuExportError> {
    let emitter = SignalEmitter::new(connection.inner(), super::AppMenuAddress::PATH)?;
    let updated = match change {
        Change::Same => return Ok(()),
        Change::Layout => Vec::new(),
        Change::Properties(ids) => tree
            .group_properties(ids, &PropNames::default())
            .iter()
            .map(wire::item)
            .collect::<Result<_, _>>()
            .map_err(|error| super::MenuExportError::Wire(error.0))?,
    };
    zbus::block_on(async {
        if !updated.is_empty() {
            DbusMenu::items_properties_updated(&emitter, updated, Vec::new()).await?;
        }
        DbusMenu::layout_updated(&emitter, revision.0, NodeId::ROOT.0).await
    })?;
    Ok(())
}
