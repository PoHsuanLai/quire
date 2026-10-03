//! `xdg_activation_v1` on a window that exists: `activate(token, surface)` with the token a
//! launcher, a notification click or a D-Bus activation handed to the running app. winit offers
//! the token only at window creation, so the request is made on the app's own connection
//! (`crate::wayland_surface`) and the compositor decides, from the token, whether the window
//! takes the keyboard.

use crate::wayland_surface::Adopted;
use wayland_client::globals::{BindError, GlobalError, GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::{Connection, Dispatch, QueueHandle};
use wayland_protocols::xdg::activation::v1::client::xdg_activation_v1::XdgActivationV1;

/// Why an activation was not sent.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ActivationError {
    /// The registry could not be read.
    #[error("reading the registry: {0}")]
    Registry(#[from] GlobalError),
    /// The compositor offers no `xdg_activation_v1`.
    #[error("the compositor offers no xdg_activation_v1: {0}")]
    Unsupported(#[from] BindError),
    /// The connection failed while the request went out.
    #[error("sending the request: {0}")]
    Send(#[from] wayland_client::backend::WaylandError),
}

/// Ask the compositor to activate the window `adopted` with `token`.
pub(crate) fn activate(adopted: &Adopted<'_>, token: &str) -> Result<(), ActivationError> {
    let connection = adopted.connection();
    let (globals, queue) = registry_queue_init::<Quiet>(connection)?;
    let activation: XdgActivationV1 = globals.bind(&queue.handle(), 1..=1, ())?;
    activation.activate(token.to_owned(), adopted.surface());
    activation.destroy();
    connection.flush()?;
    Ok(())
}

/// The state of the one-shot queue: it asks, and nothing it creates is read.
struct Quiet;

impl Dispatch<WlRegistry, GlobalListContents> for Quiet {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: <WlRegistry as wayland_client::Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<XdgActivationV1, ()> for Quiet {
    fn event(
        _: &mut Self,
        _: &XdgActivationV1,
        _: <XdgActivationV1 as wayland_client::Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

#[cfg(test)]
mod fake_compositor;
#[cfg(test)]
mod tests;
