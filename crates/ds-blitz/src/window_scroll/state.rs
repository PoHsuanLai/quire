//! A window's scroll state and what its document and listeners are told.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use blitz_dom::BaseDocument;
use blitz_kit::scroll::accel::Accel;
use blitz_kit::scroll::cmd::ScrollCmd;
use blitz_kit::scroll::doc::{self, WheelRoute};
use blitz_kit::scroll::driver::{KeyRepeat, KeyUse, Moved, Moves, ScrollDriver};
use blitz_kit::scroll::engine::Motion;
use blitz_kit::scroll::geom::ViewPoint;
use blitz_kit::scroll::keys::ScrollKey;
use blitz_kit::scroll::time::Elapsed;
use blitz_kit::scroll::tuning::Tuning;
use chordkit::Platform;
use ds::host::gesture::{Gesture, GestureBus, GesturePhase, ScrollSource, WheelDelivery};
use ds::prelude::*;
use keyboard_types::Modifiers;

use super::coast::{Coast, Heard};
use super::eased::Eased;
use super::wheel::{WheelInput, pointer_scrolls};
use crate::node_ref::{DocRef, Written};
use crate::phase::Phase;

/// Whether the window needs another frame for scrolling: an offset moved, or an animation runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frames {
    Wanted,
    Idle,
}

/// Whether a wheel went to the engine or stays with the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelUse {
    /// The engine scrolls for it; the document must not see the wheel.
    Taken,
    /// The document handles it: a `data-wheel="capture"` element, or no document or pointer yet.
    Passed,
}

/// One window's scroll state, shared by the loop that routes its events and the host component
/// that hands out its handle.
#[derive(Clone)]
pub struct WindowScroll {
    shared: Rc<Shared>,
}

impl std::fmt::Debug for WindowScroll {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowScroll").finish_non_exhaustive()
    }
}

struct Shared {
    /// The document, and the runtime listeners are called in.
    phase: Phase,
    bus: GestureBus,
    /// Where the engine's timeline starts.
    origin: Instant,
    state: RefCell<State>,
}

struct State {
    driver: ScrollDriver,
    tuning: Tuning,
    /// Where the pointer last was, in the window's logical pixels.
    pointer: Option<ViewPoint>,
    held: Modifiers,
    /// The platform whose primary modifier makes a wheel a zoom.
    platform: Platform,
    eased: Eased,
    coast: Coast,
    /// How fast the wheel is being spun, for its acceleration.
    accel: Accel,
    commands: Vec<ScrollCmd>,
}

impl State {
    /// Whether the wheel is a zoom: the platform's primary modifier is down (Command on a Mac
    /// and our desktop, Ctrl elsewhere).
    fn zooming(&self) -> bool {
        ds::base::command::holds_primary(self.platform, self.held)
    }

    /// The pointer's place as the gestures report it (the corner until it has moved).
    fn point(&self) -> Point {
        let at = self.pointer.unwrap_or_default();
        Point {
            x: Px(at.x as f32),
            y: Px(at.y as f32),
        }
    }
}

/// The gesture an eased listener hears for `heard` from `source`, with the pointer and the
/// modifiers as they are.
fn scroll_gesture(state: &State, source: ScrollSource, heard: Heard) -> Gesture {
    Gesture::Scroll {
        source,
        phase: heard.phase,
        by: Point {
            x: Px(heard.by.0 as f32),
            y: Px(heard.by.1 as f32),
        },
        at: state.point(),
        held: state.held,
    }
}

/// What the fingers' run says to the eased listeners, as gestures.
fn heard_as_gestures(state: &State, heard: Vec<Heard>) -> Vec<Gesture> {
    heard
        .into_iter()
        .map(|heard| scroll_gesture(state, ScrollSource::Finger, heard))
        .collect()
}

impl WindowScroll {
    /// Scrolling the document `phase` is attached to, publishing gestures to `bus`, on a
    /// timeline that starts at `origin`, on `platform` (whose primary modifier zooms the wheel).
    pub fn new(phase: Phase, bus: GestureBus, origin: Instant, platform: Platform) -> WindowScroll {
        WindowScroll {
            shared: Rc::new(Shared {
                phase,
                bus,
                origin,
                state: RefCell::new(State {
                    driver: ScrollDriver::default(),
                    tuning: Tuning::default(),
                    pointer: None,
                    held: Modifiers::empty(),
                    platform,
                    eased: Eased::default(),
                    coast: Coast::default(),
                    accel: Accel::default(),
                    commands: Vec::new(),
                }),
            }),
        }
    }

    /// The frame phase of the document this scrolls.
    pub(crate) fn phase(&self) -> Phase {
        self.shared.phase.clone()
    }

    /// The window's gesture listeners.
    pub fn bus(&self) -> GestureBus {
        self.shared.bus.clone()
    }

    /// The pointer's place and the modifier keys down, as the gestures report them.
    pub fn pointer(&self) -> (Point, Modifiers) {
        let state = self.shared.state.borrow();
        (state.point(), state.held)
    }

    /// The pointer moved to `at` (the window's logical pixels).
    pub fn track_pointer(&self, at: ViewPoint) {
        self.shared.state.borrow_mut().pointer = Some(at);
    }

    /// The modifier keys down are now `held`.
    pub fn track_modifiers(&self, held: Modifiers) {
        self.shared.state.borrow_mut().held = held;
    }

    /// Queue a programmatic scroll for the next frame.
    pub fn queue(&self, command: ScrollCmd) {
        self.shared.state.borrow_mut().commands.push(command);
    }

    fn elapsed(&self, now: Instant) -> Elapsed {
        Elapsed(now.saturating_duration_since(self.shared.origin))
    }

    /// Run `act` on the document and the state, if the document is free.
    fn with_doc<T>(&self, act: impl FnOnce(&mut BaseDocument, &mut State) -> T) -> Option<T> {
        let doc: DocRef = self.shared.phase.document()?;
        let mut out = None;
        let written = doc.write(|doc| {
            out = Some(act(doc, &mut self.shared.state.borrow_mut()));
        });
        match written {
            Written::Done => out,
            Written::Busy => None,
        }
    }

    /// Run `call` where the window's listeners can be called; nobody hears it before the
    /// document is attached.
    fn publish(&self, call: impl FnOnce(&GestureBus)) {
        let _ = self.shared.phase.in_runtime(|| call(&self.shared.bus));
    }

    /// A wheel or touchpad event: the listeners hear it, and the engine scrolls for it unless
    /// the element under the pointer takes the wheel itself.
    pub fn wheel(&self, input: WheelInput, now: Instant) -> (WheelUse, Frames) {
        let el = self.elapsed(now);
        let input = self.accelerated(input, el);
        let (gesture, pointer, held, detent_px, zoom) = {
            let state = self.shared.state.borrow();
            let detent_px = state.tuning.settings.wheel_detent_px.get();
            let gesture = input.gesture(state.point(), state.held, detent_px);
            (
                gesture,
                state.pointer,
                state.held,
                detent_px,
                state.zooming(),
            )
        };
        match (input.source(), zoom) {
            // A zoom, not a scroll: every listener hears each detent whole.
            (ScrollSource::Wheel, true) => self.publish(|bus| bus.publish(gesture)),
            (ScrollSource::Wheel, false) => {
                self.ease(input, detent_px, el);
                self.publish(|bus| bus.publish_to(WheelDelivery::AsReceived, gesture));
            }
            (ScrollSource::Finger, _) => {
                self.publish(|bus| bus.publish_to(WheelDelivery::AsReceived, gesture));
                self.coast(input, detent_px, el);
            }
        }
        let Some(pointer) = pointer else {
            return (WheelUse::Passed, Frames::Idle);
        };
        let routed = self
            .shared
            .phase
            .document()
            .and_then(|doc| doc.read(|doc| doc::wheel_route(doc, pointer)));
        match routed {
            Some(WheelRoute::Engine) => {
                let moved = self.with_doc(|doc, state| {
                    let env = state.tuning.at(el);
                    pointer_scrolls(input, held)
                        .into_iter()
                        .map(|scroll| state.driver.pointer(doc, scroll, pointer, env))
                        .fold(Moves::none(), Moves::then)
                });
                (WheelUse::Taken, self.frames_after(moved))
            }
            Some(WheelRoute::Capture | WheelRoute::Nothing) | None => {
                (WheelUse::Passed, Frames::Idle)
            }
        }
    }

    /// `input` with a spun wheel's clicks carried further (`blitz_kit::scroll::accel`). A
    /// primary-modifier wheel is a zoom and fingers are tracked 1:1 (a fast lift glides further
    /// instead, `Physics::fling`), so neither is touched.
    fn accelerated(&self, input: WheelInput, el: Elapsed) -> WheelInput {
        let mut state = self.shared.state.borrow_mut();
        let zoom = state.zooming();
        match (input.source(), zoom) {
            (ScrollSource::Wheel, false) => {
                let max = state.tuning.settings.wheel_accel_max.get();
                let (accel, gain) = std::mem::take(&mut state.accel).feed(input.turned(), el, max);
                state.accel = accel;
                input.scaled(gain)
            }
            _ => input,
        }
    }

    /// Add a wheel's detents to what the eased listeners will be handed, if any listens.
    fn ease(&self, input: WheelInput, detent_px: f64, el: Elapsed) {
        if !self.shared.bus.has_listener(WheelDelivery::Eased) {
            return;
        }
        let (x, y) = input.motion(detent_px);
        let mut state = self.shared.state.borrow_mut();
        state.eased = state.eased.push(x, y, el);
    }

    /// The fingers' motion, for the listeners that asked for it eased: it passes through as it
    /// came, and their lift starts the engine's glide (`coast`), if any listens.
    fn coast(&self, input: WheelInput, detent_px: f64, el: Elapsed) {
        if !self.shared.bus.has_listener(WheelDelivery::Eased) {
            return;
        }
        let gestures = {
            let mut state = self.shared.state.borrow_mut();
            let (next, heard) = std::mem::take(&mut state.coast).touch(
                input.phase,
                input.motion(detent_px),
                el,
                &state.tuning.physics,
            );
            state.coast = next;
            heard_as_gestures(&state, heard)
        };
        self.publish_eased(gestures);
    }

    /// Hand `gestures` to the eased listeners.
    fn publish_eased(&self, gestures: Vec<Gesture>) {
        for gesture in gestures {
            self.publish(|bus| bus.publish_to(WheelDelivery::Eased, gesture));
        }
    }

    /// A scroll key went down or repeated. The key still goes on to the document: the engine
    /// takes it only when nothing on the focus path does (`blitz_kit::scroll::doc::key_focus`).
    pub fn key(&self, key: ScrollKey, repeat: KeyRepeat, now: Instant) -> Frames {
        let el = self.elapsed(now);
        let moved = self.with_doc(|doc, state| {
            let env = state.tuning.at(el);
            match state.driver.key(doc, key, repeat, state.pointer, env) {
                KeyUse::Scrolled(moves) => moves,
                KeyUse::Passed => Moves::none(),
            }
        });
        self.frames_after(moved)
    }

    /// A held scroll key came up.
    pub fn key_up(&self, now: Instant) -> Frames {
        let el = self.elapsed(now);
        let moved = self.with_doc(|doc, state| {
            let env = state.tuning.at(el);
            state.driver.key_up(doc, env)
        });
        self.frames_after(moved)
    }

    /// A frame is about to be drawn at `now`: run the queued commands, advance the engine, and
    /// hand the listeners that asked for eased detents this frame's share. Called before the
    /// document resolves, so the layout sees this frame's offsets.
    pub fn frame(&self, now: Instant) -> Frames {
        let el = self.elapsed(now);
        let moved = self.with_doc(|doc, state| {
            let env = state.tuning.at(el);
            let commands = std::mem::take(&mut state.commands);
            let ran = commands
                .iter()
                .map(|command| state.driver.command(doc, command, env))
                .fold(Moves::none(), Moves::then);
            ran.then(state.driver.frame(doc, env))
        });
        self.eased_frame(el);
        self.frames_after(moved)
    }

    /// Hand the eased listeners what the detents' ease and the fingers' glide move this frame.
    fn eased_frame(&self, el: Elapsed) {
        let gestures = {
            let mut state = self.shared.state.borrow_mut();
            let (next, (dx, dy)) = state.eased.advance(el);
            state.eased = next;
            let detents = (dx != 0.0 || dy != 0.0).then_some(Heard {
                phase: GesturePhase::Changed,
                by: (dx, dy),
            });
            let (next, glide) = std::mem::take(&mut state.coast).advance(el, &state.tuning.physics);
            state.coast = next;
            let mut out = Vec::new();
            out.extend(detents.map(|h| scroll_gesture(&state, ScrollSource::Wheel, h)));
            out.extend(heard_as_gestures(&state, glide));
            out
        };
        self.publish_eased(gestures);
    }

    /// Whether a frame is wanted after `moved`: an offset moved, or something still animates.
    fn frames_after(&self, moved: Option<Moves>) -> Frames {
        let state = self.shared.state.borrow();
        let moved = moved.map_or(Moved::Still, |moves| moves.moved());
        let animating = state.driver.motion() == Motion::Animating
            || state.eased.motion() == Motion::Animating
            || state.coast.motion() == Motion::Animating
            || !state.commands.is_empty();
        match (moved, animating) {
            (Moved::Still, false) => Frames::Idle,
            _ => Frames::Wanted,
        }
    }
}
