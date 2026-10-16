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
//! Who may press an item is the app's [`MenuPermit`], a required argument of [`MenuExport::start`]:
//! there is no default, so a menu anyone on the bus may press cannot be exported by leaving it out.
//! Our desktop's is `ShellOnly` (feature `peephole`, Linux): only the shell. A press from a
//! message that names no sender, or from a caller the permit refuses, fails with `AccessDenied`;
//! reading the menu stays open, because it is what the window already shows. An `EventGroup`
//! carries at most 64 events, and one sender may make at most 8 such calls a second.
//!
//! The well-known name is requested with `DoNotQueue`: a second process of the same `app_id`
//! cannot take the menu over, and its export fails with [`MenuExportError::NameTaken`].
//!
//! Off by default, so an app that exports nothing builds no D-Bus client for it.

mod pace;
mod permit;
mod service;
#[cfg(feature = "peephole")]
mod shell_only;
mod wire;

#[cfg(feature = "testing")]
pub use permit::AllowAnyCaller;
pub use permit::{Caller, MenuPermit, SenderName, Verdict};
#[cfg(feature = "peephole")]
pub use shell_only::ShellOnly;

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
    /// Another process already owns the name this app's menu is served under (a second copy of
    /// the same app, or a program squatting on it). The export is not started.
    #[error("the menu name {0} is already in use")]
    NameTaken(String),
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
}

impl std::fmt::Debug for MenuExport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuExport")
            .field("address", &self.address)
            .finish_non_exhaustive()
    }
}

impl MenuExport {
    /// Serve `tree` for `config.app_id`. `on_activate` runs, on a bus thread, with the command
    /// of every enabled item a client, which `permit` let press items, clicks. Fails with
    /// [`MenuExportError::NameTaken`] when another process owns the menu's name.
    pub fn start(
        config: &MenuExportConfig,
        tree: MenuTree,
        on_activate: impl Fn(CommandId) + Send + Sync + 'static,
        permit: impl MenuPermit,
    ) -> Result<MenuExport, MenuExportError> {
        let address = AppMenuAddress::for_app_id(&config.app_id.0)?;
        let shared = Arc::new(Shared::new(MenuState::new(tree), Box::new(on_activate)));
        // zbus may run on the process-wide runtime (its `tokio` feature, when a shell turns it
        // on), which must be entered while the connection is built.
        let _runtime = crate::enter_runtime()?;
        let connection = connect(&config.bus, &address, &shared, permit)?;
        request_name(&connection, &address)?;
        Ok(MenuExport {
            connection,
            shared,
            address,
        })
    }

    /// Where the menu is served: the bus name (when owned) and the object path.
    pub fn address(&self) -> &AppMenuAddress {
        &self.address
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
    permit: impl MenuPermit,
) -> Result<zbus::blocking::Connection, zbus::Error> {
    let builder = match bus {
        Bus::Session => Builder::session()?,
        Bus::Address(text) => Builder::address(text.as_str())?,
    };
    builder
        .serve_at(
            address.path.as_str(),
            DbusMenu::new(Arc::clone(shared), permit),
        )?
        .build()
}

/// Ask for the well-known name once the object is served (so no call can arrive before it is).
/// `DoNotQueue`, and neither replacing nor replaceable: a name that is taken stays with its owner,
/// and a process that wanted it is told, not queued behind it.
fn request_name(
    connection: &zbus::blocking::Connection,
    address: &AppMenuAddress,
) -> Result<(), MenuExportError> {
    let reply = zbus::block_on(connection.inner().request_name_with_flags(
        address.service.as_str(),
        RequestNameFlags::DoNotQueue.into(),
    ))?;
    match reply {
        RequestNameReply::PrimaryOwner | RequestNameReply::AlreadyOwner => Ok(()),
        RequestNameReply::InQueue | RequestNameReply::Exists => {
            Err(MenuExportError::NameTaken(address.service.clone()))
        }
    }
}
