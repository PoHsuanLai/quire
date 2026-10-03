//! A compositor in a thread, over a socket pair: `wl_compositor` for a client to make a surface
//! with, and, unless left out, `xdg_activation_v1`, which records every `activate` it is sent.

use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use wayland_protocols::xdg::activation::v1::server::{
    xdg_activation_token_v1::{self, XdgActivationTokenV1},
    xdg_activation_v1::{self, XdgActivationV1},
};
use wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use wayland_server::protocol::{
    wl_compositor::{self, WlCompositor},
    wl_surface::{self, WlSurface},
};
use wayland_server::{
    Client, DataInit, Dispatch, Display, DisplayHandle, GlobalDispatch, New, Resource,
};

/// One `activate` the compositor was sent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Activated {
    pub(super) token: String,
    /// The protocol id of the surface it named.
    pub(super) surface: u32,
}

/// What the compositor offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Offer {
    /// `wl_compositor` and `xdg_activation_v1`.
    Activation,
    /// `wl_compositor` alone.
    NoActivation,
}

/// The running compositor: stopped and joined when dropped.
pub(super) struct Fake {
    seen: Arc<Mutex<Vec<Activated>>>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Fake {
    /// Start a compositor offering `offer`, and the client's end of its socket.
    pub(super) fn start(offer: Offer) -> (Fake, UnixStream) {
        let (client, server) = UnixStream::pair().expect("a socket pair");
        let seen = Arc::new(Mutex::new(Vec::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let (seen, stop) = (Arc::clone(&seen), Arc::clone(&stop));
            std::thread::spawn(move || serve(offer, server, seen, stop))
        };
        let fake = Fake {
            seen,
            stop,
            thread: Some(thread),
        };
        (fake, client)
    }

    /// Every activation so far, oldest first.
    pub(super) fn activations(&self) -> Vec<Activated> {
        self.seen.lock().expect("the compositor's log").clone()
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn serve(
    offer: Offer,
    stream: UnixStream,
    seen: Arc<Mutex<Vec<Activated>>>,
    stop: Arc<AtomicBool>,
) {
    let mut display = Display::<State>::new().expect("a display");
    let mut handle = display.handle();
    handle.create_global::<State, WlCompositor, ()>(1, ());
    if offer == Offer::Activation {
        handle.create_global::<State, XdgActivationV1, ()>(1, ());
    }
    handle
        .insert_client(stream, Arc::new(Gone))
        .expect("the client's socket");
    let mut state = State { seen };
    while !stop.load(Ordering::Acquire) {
        display.dispatch_clients(&mut state).expect("dispatching");
        display.flush_clients().expect("flushing");
        std::thread::sleep(Duration::from_millis(1));
    }
}

struct State {
    seen: Arc<Mutex<Vec<Activated>>>,
}

struct Gone;

impl ClientData for Gone {
    fn initialized(&self, _: ClientId) {}
    fn disconnected(&self, _: ClientId, _: DisconnectReason) {}
}

impl GlobalDispatch<WlCompositor, ()> for State {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<WlCompositor>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}

impl Dispatch<WlCompositor, ()> for State {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &WlCompositor,
        request: wl_compositor::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        if let wl_compositor::Request::CreateSurface { id } = request {
            init.init(id, ());
        }
    }
}

impl Dispatch<WlSurface, ()> for State {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &WlSurface,
        _: wl_surface::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
    }
}

impl GlobalDispatch<XdgActivationV1, ()> for State {
    fn bind(
        _: &mut Self,
        _: &DisplayHandle,
        _: &Client,
        resource: New<XdgActivationV1>,
        _: &(),
        init: &mut DataInit<'_, Self>,
    ) {
        init.init(resource, ());
    }
}

impl Dispatch<XdgActivationV1, ()> for State {
    fn request(
        state: &mut Self,
        _: &Client,
        _: &XdgActivationV1,
        request: xdg_activation_v1::Request,
        _: &(),
        _: &DisplayHandle,
        init: &mut DataInit<'_, Self>,
    ) {
        match request {
            xdg_activation_v1::Request::Activate { token, surface } => {
                state
                    .seen
                    .lock()
                    .expect("the compositor's log")
                    .push(Activated {
                        token,
                        surface: surface.id().protocol_id(),
                    });
            }
            xdg_activation_v1::Request::GetActivationToken { id } => {
                init.init(id, ());
            }
            _ => {}
        }
    }
}

impl Dispatch<XdgActivationTokenV1, ()> for State {
    fn request(
        _: &mut Self,
        _: &Client,
        _: &XdgActivationTokenV1,
        _: xdg_activation_token_v1::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
    }
}
