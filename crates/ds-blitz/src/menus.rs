//! Exporting an app's menu bar over the session bus (the `menus` feature): the app's
//! [`MenuTree`] served as `com.canonical.dbusmenu` on its own connection, under a well-known
//! name derived from its Wayland `app_id` ([`AppMenuAddress`]), so the shell's bar can show the
//! focused app's menus and an agent can list and run them (design/27 section 5.2 rule a).
//!
//! The app starts the export once, with the tree of its bar and a callback, and calls
//! [`MenuExport::update`] whenever the bar changes (an item enables, a check flips, an item is
//! added). A pick from outside arrives at the callback as the [`CommandId`] the item runs, on a
//! bus thread: hand it to the UI thread (a channel and an
//! [`AppHandle`](crate::AppHandle) redraw), look the value up with
//! `MenuBarModel::command` and run the handler the app's own menu runs.
//!
//! Off by default, so an app that exports nothing builds no D-Bus client for it.

mod service;
mod wire;

use crate::app_id::AppId;
use ds::base::command::{AppCommand, CommandId};
use ds::components::menus::export::{
    AddressError, AppMenuAddress, Change, MenuState, MenuTree, Revision, TreeError,
};
use ds::components::menus::menu_bar::MenuBarModel;
use service::{DbusMenu, Shared};
use std::sync::Arc;
use zbus::blocking::connection::Builder;
use zbus::fdo::{RequestNameFlags, RequestNameReply};

/// Which bus the export is served on.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Bus {
    /// The user's session bus, as `$DBUS_SESSION_BUS_ADDRESS` names it.
    #[default]
    Session,
    /// A bus at this address: a test's private daemon, never the real session.
    Address(String),
}

/// Whether the app has the well-known name for its menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameOutcome {
    /// The app owns [`AppMenuAddress::service`]: the shell finds its menu from the `app_id`.
    Owned,
    /// Another process of the same `app_id` owns it. This one is queued for it and takes it
    /// over, with no action, when that process exits; until then its menu is reachable on this
    /// connection's unique name only, which nothing derives from an `app_id`.
    Queued,
}

/// What the export needs: whose menu it is and where to serve it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuExportConfig {
    /// The app's Wayland `app_id`: the same one its window carries.
    pub app_id: AppId,
    /// The bus to serve on.
    pub bus: Bus,
}

/// Why a menu could not be exported or updated.
#[derive(Debug, thiserror::Error)]
pub enum MenuExportError {
    /// The `app_id` makes no bus name.
    #[error(transparent)]
    Address(#[from] AddressError),
    /// The bar cannot be exported (an item with no command id).
    #[error(transparent)]
    Tree(#[from] TreeError),
    /// The bus refused the export.
    #[error("the bus refused the menu export: {0}")]
    Bus(#[from] zbus::Error),
    /// A value could not be written for the bus.
    #[error("a menu value could not be written for the bus: {0}")]
    Wire(String),
    /// No runtime for zbus to run on.
    #[error(transparent)]
    Runtime(#[from] crate::error::RuntimeError),
}

/// A menu being served. Dropping it closes the connection, which takes the name with it.
pub struct MenuExport {
    connection: zbus::blocking::Connection,
    shared: Arc<Shared>,
    address: AppMenuAddress,
    name: NameOutcome,
}

impl std::fmt::Debug for MenuExport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuExport")
            .field("address", &self.address)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl MenuExport {
    /// Serve `tree` for `config.app_id`. `on_activate` runs, on a bus thread, with the command
    /// of every enabled item a client clicks.
    pub fn start(
        config: &MenuExportConfig,
        tree: MenuTree,
        on_activate: impl Fn(CommandId) + Send + Sync + 'static,
    ) -> Result<MenuExport, MenuExportError> {
        let address = AppMenuAddress::for_app_id(&config.app_id.0)?;
        let shared = Arc::new(Shared::new(MenuState::new(tree), Box::new(on_activate)));
        // zbus may run on the process-wide runtime (its `tokio` feature, when a shell turns it
        // on), which must be entered while the connection is built.
        let _runtime = crate::enter_runtime()?;
        let connection = connect(&config.bus, &address, &shared)?;
        let name = request_name(&connection, &address)?;
        Ok(MenuExport {
            connection,
            shared,
            address,
            name,
        })
    }

    /// Where the menu is served: the bus name (when owned) and the object path.
    pub fn address(&self) -> &AppMenuAddress {
        &self.address
    }

    /// Whether the well-known name is the app's yet.
    pub fn name(&self) -> NameOutcome {
        self.name
    }

    /// The connection's unique name, which always reaches the menu.
    pub fn unique_name(&self) -> String {
        self.connection
            .unique_name()
            .map(|name| name.to_string())
            .unwrap_or_default()
    }

    /// The revision being served.
    pub fn revision(&self) -> Revision {
        self.shared.revision()
    }

    /// Serve `tree` from now on, and tell the clients what changed: `LayoutUpdated` always (the
    /// revision rose), and `ItemsPropertiesUpdated` too when only properties changed.
    pub fn set_tree(&self, tree: MenuTree) -> Result<Change, MenuExportError> {
        let (change, revision, tree) = self.shared.replace(tree);
        service::announce(&self.connection, &change, revision, &tree)?;
        Ok(change)
    }

    /// [`set_tree`](Self::set_tree) of `bar`'s tree.
    pub fn update<T: AppCommand + Clone>(
        &self,
        bar: &MenuBarModel<T>,
    ) -> Result<Change, MenuExportError> {
        self.set_tree(MenuTree::from_bar(bar)?)
    }
}

/// A connection serving the menu, with no name yet.
fn connect(
    bus: &Bus,
    address: &AppMenuAddress,
    shared: &Arc<Shared>,
) -> Result<zbus::blocking::Connection, zbus::Error> {
    let builder = match bus {
        Bus::Session => Builder::session()?,
        Bus::Address(text) => Builder::address(text.as_str())?,
    };
    builder
        .serve_at(address.path.as_str(), DbusMenu::new(Arc::clone(shared)))?
        .build()
}

/// Ask for the well-known name once the object is served (so no call can arrive before it is).
fn request_name(
    connection: &zbus::blocking::Connection,
    address: &AppMenuAddress,
) -> Result<NameOutcome, zbus::Error> {
    // No flags at all: zbus's default would let a second process replace the first, and
    // `DoNotQueue` would refuse it; neither is wanted (a second process waits its turn).
    let no_flags = std::iter::empty::<RequestNameFlags>().collect();
    let reply = zbus::block_on(
        connection
            .inner()
            .request_name_with_flags(address.service.as_str(), no_flags),
    )?;
    Ok(match reply {
        RequestNameReply::InQueue => NameOutcome::Queued,
        _ => NameOutcome::Owned,
    })
}
