//! The activation request, sent through `adopt` and `activate` to a fake compositor: what it
//! receives is the token and the surface the window's handles name.

use super::fake_compositor::{Activated, Fake, Offer};
use super::{ActivationError, activate};
use crate::wayland_surface::{AdoptError, Adopted, Foreign, adopt};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};

/// The client the app is: a connection, and the surface of its one window.
struct App {
    connection: Connection,
    queue: EventQueue<Windowless>,
    surface: WlSurface,
}

struct Windowless;

impl Dispatch<WlRegistry, GlobalListContents> for Windowless {
    fn event(
        _: &mut Self,
        _: &WlRegistry,
        _: <WlRegistry as Proxy>::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlCompositor, ()> for Windowless {
    fn event(
        _: &mut Self,
        _: &WlCompositor,
        _: <WlCompositor as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSurface, ()> for Windowless {
    fn event(
        _: &mut Self,
        _: &WlSurface,
        _: <WlSurface as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl App {
    fn connect(stream: std::os::unix::net::UnixStream) -> App {
        let connection = Connection::from_socket(stream).expect("the socket connects");
        let (globals, mut queue) = registry_queue_init::<Windowless>(&connection).expect("globals");
        let compositor: WlCompositor = globals
            .bind(&queue.handle(), 1..=1, ())
            .expect("a compositor");
        let surface = compositor.create_surface(&queue.handle(), ());
        queue
            .roundtrip(&mut Windowless)
            .expect("the surface exists");
        App {
            connection,
            queue,
            surface,
        }
    }

    /// Wait until the compositor has handled everything this app sent.
    fn settle(&mut self) {
        self.queue.roundtrip(&mut Windowless).expect("a roundtrip");
    }

    fn window(&self) -> Foreign {
        Foreign::of(&self.connection, &self.surface).expect("the system libwayland")
    }
}

fn adopted(window: &Foreign) -> Adopted<'_> {
    adopt(
        window.display_handle().expect("a display handle"),
        window.window_handle().expect("a window handle"),
    )
    .expect("a Wayland window")
}

#[test]
fn the_compositor_is_sent_the_token_and_the_windows_own_surface() {
    let (compositor, stream) = Fake::start(Offer::Activation);
    let mut app = App::connect(stream);
    let window = app.window();

    activate(&adopted(&window), "token-1").expect("the request is sent");
    app.settle();

    assert_eq!(
        compositor.activations(),
        [Activated {
            token: "token-1".to_owned(),
            surface: app.surface.id().protocol_id(),
        }]
    );
}

#[test]
fn each_activation_carries_its_own_token() {
    let (compositor, stream) = Fake::start(Offer::Activation);
    let mut app = App::connect(stream);
    let window = app.window();

    activate(&adopted(&window), "first").expect("sent");
    activate(&adopted(&window), "second").expect("sent");
    app.settle();

    let tokens: Vec<String> = compositor
        .activations()
        .into_iter()
        .map(|seen| seen.token)
        .collect();
    assert_eq!(tokens, ["first", "second"]);
}

#[test]
fn a_compositor_without_xdg_activation_is_an_error_not_a_request() {
    let (compositor, stream) = Fake::start(Offer::NoActivation);
    let mut app = App::connect(stream);
    let window = app.window();

    let error = activate(&adopted(&window), "token-1").expect_err("nothing to bind");
    app.settle();

    assert!(matches!(error, ActivationError::Unsupported(_)), "{error}");
    assert!(compositor.activations().is_empty());
}

#[test]
fn an_object_that_is_not_a_surface_is_not_adopted() {
    let (_compositor, stream) = Fake::start(Offer::Activation);
    let app = App::connect(stream);
    let (globals, queue) = registry_queue_init::<Windowless>(&app.connection).expect("globals");
    let not_a_surface: WlCompositor = globals.bind(&queue.handle(), 1..=1, ()).expect("bound");
    let window = Foreign::of(&app.connection, &not_a_surface).expect("the system libwayland");

    let adopted = adopt(
        window.display_handle().expect("a display handle"),
        window.window_handle().expect("a window handle"),
    );

    assert_eq!(adopted.err(), Some(AdoptError::NotASurface));
}
