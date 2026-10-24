//! A window's `quire_window_v1`, made through `adopt` and `attach` against a fake compositor: what
//! it receives is `set_frame(compositor_shadow, the window radius token)`, and a compositor
//! without the global is told nothing and nothing fails.

use super::fake_compositor::{Fake, Frame, Mode, Offer};
use super::{FrameError, attach, window_radius};
use crate::wayland_surface::{Adopted, Foreign, adopt};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ptr::NonNull;
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::wl_compositor::WlCompositor;
use wayland_client::protocol::wl_registry::WlRegistry;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle};
use wayland_protocols::xdg::shell::client::xdg_surface::XdgSurface;
use wayland_protocols::xdg::shell::client::xdg_toplevel::XdgToplevel;
use wayland_protocols::xdg::shell::client::xdg_wm_base::XdgWmBase;

/// The client the app is: a connection, and the surface and toplevel of its one window.
struct App {
    connection: Connection,
    queue: EventQueue<Windowless>,
    surface: WlSurface,
    toplevel: XdgToplevel,
}

struct Windowless;

macro_rules! ignore_events {
    ($($interface:ty => $data:ty),*) => {$(
        impl Dispatch<$interface, $data> for Windowless {
            fn event(
                _: &mut Self,
                _: &$interface,
                _: <$interface as Proxy>::Event,
                _: &$data,
                _: &Connection,
                _: &QueueHandle<Self>,
            ) {
            }
        }
    )*};
}

ignore_events!(
    WlRegistry => GlobalListContents,
    WlCompositor => (),
    WlSurface => (),
    XdgWmBase => (),
    XdgSurface => (),
    XdgToplevel => ()
);

impl App {
    fn connect(stream: std::os::unix::net::UnixStream) -> App {
        let connection = Connection::from_socket(stream).expect("the socket connects");
        let (globals, mut queue) = registry_queue_init::<Windowless>(&connection).expect("globals");
        let handle = queue.handle();
        let compositor: WlCompositor = globals.bind(&handle, 1..=1, ()).expect("a compositor");
        let base: XdgWmBase = globals.bind(&handle, 1..=1, ()).expect("a wm base");
        let surface = compositor.create_surface(&handle, ());
        let toplevel = base
            .get_xdg_surface(&surface, &handle, ())
            .get_toplevel(&handle, ());
        queue
            .roundtrip(&mut Windowless)
            .expect("the toplevel exists");
        App {
            connection,
            queue,
            surface,
            toplevel,
        }
    }

    /// Wait until the compositor has handled everything this app sent.
    fn settle(&mut self) {
        self.queue.roundtrip(&mut Windowless).expect("a roundtrip");
    }

    fn window(&self) -> Foreign {
        Foreign::of(&self.connection, &self.surface).expect("the system libwayland")
    }

    /// The pointer winit reports as the window's `xdg_toplevel`.
    fn toplevel_ptr(&self) -> NonNull<std::ffi::c_void> {
        NonNull::new(self.toplevel.id().as_ptr().cast()).expect("a live toplevel")
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
fn the_compositor_is_told_to_draw_the_shadow_at_the_window_radius() {
    let (compositor, stream) = Fake::start(Offer::Quire);
    let mut app = App::connect(stream);
    let window = app.window();

    let mut told = attach(&adopted(&window), app.toplevel_ptr()).expect("the object is made");
    told.set_radius(window_radius()).expect("sent");
    app.settle();

    let log = compositor.log();
    assert_eq!(log.windows, 1);
    assert_eq!(
        log.frames,
        [Frame {
            mode: Mode::CompositorShadow,
            corner_radius: 10,
        }]
    );
    assert_eq!(
        window_radius(),
        10,
        "the token the stylesheet's --r-window reads"
    );
    drop(told);
}

#[test]
fn the_object_lives_until_the_window_lets_go_of_it() {
    let (compositor, stream) = Fake::start(Offer::Quire);
    let mut app = App::connect(stream);
    let window = app.window();

    let mut told = attach(&adopted(&window), app.toplevel_ptr()).expect("the object is made");
    told.set_radius(window_radius()).expect("sent");
    app.settle();
    assert_eq!(compositor.log().destroyed, 0);

    drop(told);
    app.settle();
    assert_eq!(compositor.log().destroyed, 1);
}

#[test]
fn a_radius_is_sent_again_only_when_it_changes() {
    let (compositor, stream) = Fake::start(Offer::Quire);
    let mut app = App::connect(stream);
    let window = app.window();

    let mut told = attach(&adopted(&window), app.toplevel_ptr()).expect("the object is made");
    told.set_radius(10).expect("sent");
    told.set_radius(10).expect("skipped");
    told.set_radius(26).expect("sent");
    app.settle();

    let radii: Vec<u32> = compositor
        .log()
        .frames
        .into_iter()
        .map(|frame| frame.corner_radius)
        .collect();
    assert_eq!(radii, [10, 26]);
}

#[test]
fn a_compositor_without_the_global_is_told_nothing_and_nothing_fails() {
    let (compositor, stream) = Fake::start(Offer::NoQuire);
    let mut app = App::connect(stream);
    let window = app.window();

    let error = attach(&adopted(&window), app.toplevel_ptr()).expect_err("nothing to bind");
    app.settle();

    assert!(matches!(error, FrameError::Unsupported(_)), "{error}");
    assert_eq!(compositor.log(), Default::default());
}
