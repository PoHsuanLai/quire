//! The event loop `launch` runs: dioxus-native's application for the first window, and one more
//! for every window `open_window` adds, all on the same winit loop.
//!
//! dioxus-native at this rev has no public way for a running app to open a window: its
//! application sets up only the window it was built with (`DioxusNativeApplication::add_window`
//! skips the dioxus contexts and waits for a resume that never comes on a desktop), and
//! `launch_cfg_with_props` runs the loop itself. So ds-blitz builds the loop and each window's
//! document itself (`crate::window_build`, as `launch_cfg_with_props` does) and runs a
//! `DioxusNativeApplication` per window under this handler, which routes each winit event to the
//! application whose window it is and forwards everything else to all of them.
//!
//! Every window is the same kind of thing here: the first window `launch` opens is a request like
//! any `open_window` makes, and the loop starts with none. Closing a window drops only its
//! application; whether the loop then ends is the app's choice (`crate::app_life`: the
//! `LastWindowClosed` policy and `AppHandle::quit`), applied after each close and when a linger
//! runs out.
//!
//! Each application's shell events come through a relay: its documents post to a proxy whose
//! queue only this handler reads, and it hands them on unless they close a window. That is how a
//! closing window is kept from ending the loop (blitz-shell exits when an application's last
//! window closes), and how every window is dropped before the loop ends (winit wants windows
//! dropped before the loop exits).
//!
//! **A closed window's renderer is kept, and parked.** Each vello-hybrid renderer owns a wgpu
//! instance of its own, and dropping one while another window is still drawing crashed the next
//! frame of the other inside the NVIDIA Vulkan driver (`vkAcquireNextImageKHR` through a null
//! pointer, on Wayland, 2026-09-27). So a closing window's renderer is set aside, and the next
//! window opened draws with it (which also keeps its device: a warm open skips the adapter probe,
//! 33 ms against 100 to 170 ms): the app holds at most as many instances as it ever had windows
//! open at once. But the renderer caches the window it drew past `suspend`, so the closed window
//! stayed on screen, frozen, until the next window replaced it, and for good once the app stays
//! warm with none. So a renderer is set aside only after it has been resumed on the parking
//! window, a window of the loop's own that is never shown (Wayland maps a window only when it
//! gets a buffer, and the parking surface is never drawn), which takes the closed window out of
//! the renderer and so lets it go.
//!
use crate::app_handle::Remote;
use crate::app_life::{Lifecycle, Verdict};
use crate::open_window::WindowHandle;
use crate::startup_token::take_startup_token;
use crate::window_activate::raise;
use crate::window_build::{Base, Shape, WindowSlot, window_config};
use crate::window_requests::{Request, WindowKey, WindowLife};
use anyrender::WindowRenderer;
use blitz_shell::{BlitzShellEvent, BlitzShellProxy, WindowConfig};
use dioxus_native::winit::application::ApplicationHandler;
use dioxus_native::winit::application::macos::ApplicationHandlerExtMacOS;
use dioxus_native::winit::event::{StartCause, WindowEvent};
use dioxus_native::winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoopProxy};
use dioxus_native::winit::window::WindowId;
use dioxus_native::winit::window::{Window, WindowAttributes};
use dioxus_native::{DioxusNativeApplication, DioxusNativeWindowRenderer};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Instant;

/// One window's dioxus-native application and its relay.
struct Sub {
    app: DioxusNativeApplication,
    slot: WindowSlot,
    /// The window's renderer, kept past the window's close (see the module documentation).
    renderer: DioxusNativeWindowRenderer,
    /// What the window's documents and shell post.
    relay: Receiver<BlitzShellEvent>,
    /// The application's own queue.
    forward: Sender<BlitzShellEvent>,
}

impl Sub {
    fn new(
        proxy: EventLoopProxy,
        config: WindowConfig<DioxusNativeWindowRenderer>,
        slot: WindowSlot,
        renderer: DioxusNativeWindowRenderer,
    ) -> Sub {
        let (posted, relay) = BlitzShellProxy::new(proxy);
        let (forward, queue) = channel();
        Sub {
            app: DioxusNativeApplication::new(posted, queue, config),
            slot,
            renderer,
            relay,
            forward,
        }
    }

    fn window_id(&self) -> Option<WindowId> {
        self.slot.window().map(|window| window.id())
    }
}

/// Every window of the app, as winit's application.
pub(crate) struct Windows {
    subs: Vec<(WindowKey, Sub)>,
    /// Renderers of closed windows, parked, for the next windows to open.
    spare: Vec<DioxusNativeWindowRenderer>,
    /// The window a closed window's renderer is parked on: created at the first close, never
    /// shown or drawn.
    parking: Option<Arc<dyn Window>>,
    base: Base,
    life: Lifecycle,
    /// Whether the loop can create windows yet (winit's `can_create_surfaces` has come).
    ready: bool,
}

impl Windows {
    /// An app with no window yet; the first one is a request in `base.requests`.
    pub(crate) fn new(base: Base, life: Lifecycle) -> Self {
        Windows {
            subs: Vec::new(),
            spare: Vec::new(),
            parking: None,
            base,
            life,
            ready: false,
        }
    }

    fn key_of(&self, window: WindowId) -> Option<WindowKey> {
        self.subs
            .iter()
            .find(|(_, sub)| sub.window_id() == Some(window))
            .map(|(key, _)| *key)
    }

    fn sub_mut(&mut self, key: WindowKey) -> Option<&mut Sub> {
        self.subs
            .iter_mut()
            .find(|(held, _)| *held == key)
            .map(|(_, sub)| sub)
    }

    fn all(&mut self) -> impl Iterator<Item = &mut Sub> {
        self.subs.iter_mut().map(|(_, sub)| sub)
    }

    /// Drop the window `key`: it leaves the screen, and its document and VirtualDom go with it.
    fn drop_window(&mut self, event_loop: &dyn ActiveEventLoop, key: WindowKey) {
        if let Some(at) = self.subs.iter().position(|(held, _)| *held == key) {
            let (_, sub) = self.subs.remove(at);
            let renderer = sub.renderer.clone();
            drop(sub);
            if self.park(event_loop, &renderer) {
                self.spare.push(renderer);
            }
            self.life.closed(Instant::now());
        }
        self.base.requests.set_life(key, WindowLife::Closed);
    }

    fn drop_all(&mut self, event_loop: &dyn ActiveEventLoop) {
        let keys: Vec<WindowKey> = self.subs.iter().map(|(key, _)| *key).collect();
        keys.into_iter()
            .for_each(|key| self.drop_window(event_loop, key));
    }

    /// Take the closed window out of `renderer` by resuming it on the parking window and
    /// suspending it again, which keeps its device for the next window. `false` when that could
    /// not be done: the renderer is then dropped with the window it caches, and the instance
    /// goes with it.
    fn park(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        renderer: &DioxusNativeWindowRenderer,
    ) -> bool {
        if self.parking.is_none() {
            let attributes = WindowAttributes::default()
                .with_title("")
                .with_visible(false)
                .with_decorations(false);
            self.parking = event_loop.create_window(attributes).ok().map(Arc::from);
        }
        let Some(window) = self.parking.clone() else {
            return false;
        };
        let mut renderer = renderer.clone();
        renderer.resume(Arc::new(window), 1, 1, || {});
        let parked = renderer.complete_resume();
        renderer.suspend();
        parked
    }

    /// Answer what the app's components and threads asked since the last wake.
    fn serve(&mut self, event_loop: &dyn ActiveEventLoop) {
        if !self.ready {
            // Windows cannot be created yet; `can_create_surfaces` serves what waits.
            return;
        }
        for remote in self.base.handle.take() {
            match remote {
                Remote::Open { spec, make } => {
                    self.base.requests.open(spec, make());
                }
                Remote::Redraw => self.all().for_each(|sub| {
                    if let Some(window) = sub.slot.window() {
                        window.request_redraw();
                    }
                }),
                Remote::Quit => self.life.quit(),
                Remote::Hold => self.life.hold(),
                Remote::Release => self.life.release(Instant::now()),
            }
        }
        for request in self.base.requests.take() {
            match request {
                Request::Open { key, spec, root } => self.open(event_loop, key, &spec, root),
                Request::Close(key) => self.drop_window(event_loop, key),
                Request::Focus { key, token } => {
                    if let Some(window) = self.sub_mut(key).and_then(|sub| sub.slot.window()) {
                        raise(&*window, token);
                    }
                }
            }
        }
    }

    fn open(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        key: WindowKey,
        spec: &crate::open_window::WindowSpec,
        root: crate::window_requests::Root,
    ) {
        if self.base.requests.life(key) != WindowLife::Opening {
            // Closed by its handle before the loop got to it.
            return;
        }
        let shape = Shape {
            title: spec.title().to_owned(),
            size: spec.size(),
            app_id: spec.app_id_or(self.base.app_id.as_ref()),
            decorations: spec.decorations_or(self.base.decorations),
            token: take_startup_token(event_loop),
        };
        let slot = WindowSlot::default();
        let handle = WindowHandle::new(key, self.base.requests.clone());
        let renderer = self.spare.pop().unwrap_or_default();
        let config = window_config(root, shape, &self.base, &slot, handle, renderer.clone());
        let mut sub = Sub::new(event_loop.create_proxy(), config, slot, renderer);
        sub.app.can_create_surfaces(event_loop);
        self.subs.push((key, sub));
        self.life.opened();
        self.base.requests.set_life(key, WindowLife::Open);
    }

    /// Hand each application what its documents posted, keeping back the closes this handler
    /// decides.
    fn relay(&mut self, event_loop: &dyn ActiveEventLoop) {
        let mut closes = Vec::new();
        for sub in self.all() {
            let posted: Vec<BlitzShellEvent> = sub.relay.try_iter().collect();
            if posted.is_empty() {
                continue;
            }
            for event in posted {
                match event {
                    BlitzShellEvent::CloseWindow { window_id } => closes.push(window_id),
                    event => {
                        let _ = sub.forward.send(event);
                    }
                }
            }
            sub.app.proxy_wake_up(event_loop);
        }
        for window_id in closes {
            self.close(event_loop, window_id);
        }
    }

    /// A window was asked to close, by the compositor or by its own frame: only it closes.
    fn close(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId) {
        if let Some(key) = self.key_of(window_id) {
            self.drop_window(event_loop, key);
        }
    }

    /// Apply the app's policy: end the loop, or tell it when to look again.
    fn settle(&mut self, event_loop: &dyn ActiveEventLoop) {
        if !self.ready {
            return;
        }
        match self.life.verdict(Instant::now()) {
            Verdict::Exit => {
                self.drop_all(event_loop);
                self.spare.clear();
                self.parking = None;
                self.base.handle.end();
                event_loop.exit();
            }
            Verdict::Run => event_loop.set_control_flow(
                self.life
                    .wake_at()
                    .map_or(ControlFlow::Wait, ControlFlow::WaitUntil),
            ),
        }
    }
}

impl ApplicationHandler for Windows {
    fn macos_handler(&mut self) -> Option<&mut dyn ApplicationHandlerExtMacOS> {
        self.subs
            .first_mut()
            .and_then(|(_, sub)| sub.app.macos_handler())
    }

    fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.all().for_each(|sub| sub.app.resumed(event_loop));
    }

    fn suspended(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.all().for_each(|sub| sub.app.suspended(event_loop));
    }

    fn destroy_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.all()
            .for_each(|sub| sub.app.destroy_surfaces(event_loop));
    }

    fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.all().for_each(|sub| sub.app.about_to_wait(event_loop));
        self.settle(event_loop);
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.all()
            .for_each(|sub| sub.app.can_create_surfaces(event_loop));
        self.ready = true;
        self.serve(event_loop);
        self.settle(event_loop);
    }

    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.all()
            .for_each(|sub| sub.app.new_events(event_loop, cause));
        self.settle(event_loop);
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            self.close(event_loop, window_id);
            self.settle(event_loop);
            return;
        }
        if let Some(sub) = self.key_of(window_id).and_then(|key| self.sub_mut(key)) {
            sub.app.window_event(event_loop, window_id, event);
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.serve(event_loop);
        self.relay(event_loop);
        self.settle(event_loop);
    }
}
