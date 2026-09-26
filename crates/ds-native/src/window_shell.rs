//! The event loop `launch` runs: dioxus-native's application for the first window, and one more
//! for every window `open_window` adds, all on the same winit loop.
//!
//! dioxus-native at this rev has no public way for a running app to open a window: its
//! application sets up only the window it was built with (`DioxusNativeApplication::add_window`
//! skips the dioxus contexts and waits for a resume that never comes on a desktop), and
//! `launch_cfg_with_props` runs the loop itself. So ds-native builds the loop and each window's
//! document itself (`crate::window_build`, as `launch_cfg_with_props` does) and runs a
//! `DioxusNativeApplication` per window under this handler, which routes each winit event to the
//! application whose window it is and forwards everything else to all of them.
//!
//! Each application's shell events come through a relay: its documents post to a proxy whose
//! queue only this handler reads, and it hands them on unless they close a window. That is how a
//! closing second window is kept from ending the loop (blitz-shell exits when an application's
//! last window closes), and how the first window's close takes the others down before the loop
//! ends (winit wants windows dropped before the loop exits).
//!
//! **A closed window's renderer is kept.** Each vello-hybrid renderer owns a wgpu instance of its
//! own, and dropping one while another window is still drawing crashed the next frame of the
//! other inside the NVIDIA Vulkan driver (`vkAcquireNextImageKHR` through a null pointer, on
//! Wayland, 2026-09-27). So a closing window's renderer is only suspended (its surface released,
//! as blitz-shell does on close) and set aside, and the next window opened draws with it: the app
//! holds at most as many instances as it ever had windows open at once.

use crate::window_build::{Base, Shape, WindowSlot, window_config};
use crate::window_requests::{Request, WindowKey, WindowLife};
use blitz_shell::{BlitzShellEvent, BlitzShellProxy, WindowConfig};
use dioxus_native::winit::application::ApplicationHandler;
use dioxus_native::winit::application::macos::ApplicationHandlerExtMacOS;
use dioxus_native::winit::event::{StartCause, WindowEvent};
use dioxus_native::winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use dioxus_native::winit::window::WindowId;
use dioxus_native::{DioxusNativeApplication, DioxusNativeWindowRenderer};
use std::sync::mpsc::{Receiver, Sender, channel};

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

/// Whose a window is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Owner {
    /// The window `launch` opened (and any window not yet known: dioxus-native ignores it).
    Main,
    /// A window `open_window` opened.
    Extra(WindowKey),
}

/// What a close of a window does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloseStep {
    /// Drop that window's application (and its VirtualDom); the loop runs on.
    DropExtra(WindowKey),
    /// Drop every other window, then let blitz-shell close this one and end the loop, as a
    /// single-window `launch` always did.
    DropExtrasThenEnd,
}

fn close_step(owner: Owner) -> CloseStep {
    match owner {
        Owner::Main => CloseStep::DropExtrasThenEnd,
        Owner::Extra(key) => CloseStep::DropExtra(key),
    }
}

/// Every window of the app, as winit's application.
pub(crate) struct Windows {
    main: Sub,
    extras: Vec<(WindowKey, Sub)>,
    /// Renderers of closed windows, for the next windows to open.
    spare: Vec<DioxusNativeWindowRenderer>,
    base: Base,
}

impl Windows {
    /// The first window, `root` shaped `shape`, on `proxy`'s loop.
    pub(crate) fn new(
        proxy: EventLoopProxy,
        root: crate::window_requests::Root,
        shape: Shape,
        base: Base,
    ) -> Self {
        let slot = WindowSlot::default();
        let renderer = DioxusNativeWindowRenderer::new();
        let config = window_config(root, shape, &base, &slot, renderer.clone());
        Windows {
            main: Sub::new(proxy, config, slot, renderer),
            extras: Vec::new(),
            spare: Vec::new(),
            base,
        }
    }

    fn owner(&self, window: WindowId) -> Owner {
        self.extras
            .iter()
            .find(|(_, sub)| sub.window_id() == Some(window))
            .map_or(Owner::Main, |(key, _)| Owner::Extra(*key))
    }

    fn sub_mut(&mut self, owner: Owner) -> Option<&mut Sub> {
        match owner {
            Owner::Main => Some(&mut self.main),
            Owner::Extra(key) => self
                .extras
                .iter_mut()
                .find(|(held, _)| *held == key)
                .map(|(_, sub)| sub),
        }
    }

    fn all(&mut self) -> impl Iterator<Item = &mut Sub> {
        std::iter::once(&mut self.main).chain(self.extras.iter_mut().map(|(_, sub)| sub))
    }

    /// Drop the window `key`: it leaves the screen, and its document and VirtualDom go with it.
    fn drop_extra(&mut self, key: WindowKey) {
        if let Some(at) = self.extras.iter().position(|(held, _)| *held == key) {
            let (_, sub) = self.extras.remove(at);
            let renderer = sub.renderer.clone();
            drop(sub);
            self.spare.push(renderer);
        }
        self.base.requests.set_life(key, WindowLife::Closed);
    }

    fn drop_extras(&mut self) {
        let keys: Vec<WindowKey> = self.extras.iter().map(|(key, _)| *key).collect();
        keys.into_iter().for_each(|key| self.drop_extra(key));
    }

    /// Answer what the app's components asked since the last wake.
    fn serve(&mut self, event_loop: &dyn ActiveEventLoop) {
        for request in self.base.requests.take() {
            match request {
                Request::Open { key, spec, root } => self.open(event_loop, key, &spec, root),
                Request::Close(key) => self.drop_extra(key),
                Request::Focus(key) => {
                    if let Some(window) = self
                        .sub_mut(Owner::Extra(key))
                        .and_then(|sub| sub.slot.window())
                    {
                        window.focus_window();
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
        };
        let slot = WindowSlot::default();
        let renderer = self.spare.pop().unwrap_or_default();
        let config = window_config(root, shape, &self.base, &slot, renderer.clone());
        let mut sub = Sub::new(event_loop.create_proxy(), config, slot, renderer);
        sub.app.can_create_surfaces(event_loop);
        self.extras.push((key, sub));
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
            self.close(event_loop, window_id, CloseBy::Shell);
        }
    }

    /// A window was asked to close, by the compositor or by its own frame.
    fn close(&mut self, event_loop: &dyn ActiveEventLoop, window_id: WindowId, by: CloseBy) {
        match close_step(self.owner(window_id)) {
            CloseStep::DropExtra(key) => self.drop_extra(key),
            CloseStep::DropExtrasThenEnd => {
                self.drop_extras();
                match by {
                    CloseBy::Compositor => {
                        self.main.app.window_event(
                            event_loop,
                            window_id,
                            WindowEvent::CloseRequested,
                        );
                    }
                    CloseBy::Shell => {
                        let _ = self
                            .main
                            .forward
                            .send(BlitzShellEvent::CloseWindow { window_id });
                        self.main.app.proxy_wake_up(event_loop);
                    }
                }
            }
        }
    }
}

/// Who asked a window to close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloseBy {
    /// The compositor (`WindowEvent::CloseRequested`).
    Compositor,
    /// The window's own frame, through blitz-shell (`BlitzShellEvent::CloseWindow`).
    Shell,
}

impl ApplicationHandler for Windows {
    fn macos_handler(&mut self) -> Option<&mut dyn ApplicationHandlerExtMacOS> {
        self.main.app.macos_handler()
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
    }

    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.all()
            .for_each(|sub| sub.app.can_create_surfaces(event_loop));
    }

    fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
        self.all()
            .for_each(|sub| sub.app.new_events(event_loop, cause));
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if matches!(event, WindowEvent::CloseRequested) {
            self.close(event_loop, window_id, CloseBy::Compositor);
            return;
        }
        let owner = self.owner(window_id);
        if let Some(sub) = self.sub_mut(owner) {
            sub.app.window_event(event_loop, window_id, event);
        }
    }

    fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
        self.serve(event_loop);
        self.relay(event_loop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closing_a_second_window_drops_only_it_and_closing_the_first_ends_the_app() {
        let key = crate::window_requests::tests_key(3);
        assert_eq!(close_step(Owner::Extra(key)), CloseStep::DropExtra(key));
        assert_eq!(close_step(Owner::Main), CloseStep::DropExtrasThenEnd);
    }
}
