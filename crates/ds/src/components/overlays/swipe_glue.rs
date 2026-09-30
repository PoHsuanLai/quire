//! The swipe to dismiss a toast and a notification card share: the glue between an element's
//! pointer and wheel events and the pure machine (`ds_motion::swipe`), and the spring that
//! carries it home when a drag ends under both thresholds (design/30 section 1.4, "Swipe";
//! design/27 section 3.12).
//!
//! Released past 80 px or 600 px/s to the right it is dismissed from where it is; the element's
//! exit starts at the offset the drag left (`--swipe-dx`), so it flies on rather than starting
//! over. Under both it springs back carrying the hand's release velocity. A new drag mid-return
//! picks it up where it is. A horizontal scroll to the right (a two-finger swipe) counts as a
//! drag.

use crate::components::controls::press::button_of;
use dioxus::core::queue_effect;
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::press::PointerButton;
use ds_core::vocab::PressPhase;
use ds_motion::detail::touch::{Contact, Touch};
use ds_motion::swipe::{Click, SwipeInput, SwipeLook, SwipeMetrics, SwipeState};
use ds_motion::use_swipe::{Swiper, use_swipe};
use ds_motion::{
    spring_spec::SpringSpec,
    timeline::spring::PxPerUnit,
    use_spring::{SpringMotion, use_spring_motion},
    velocity::VelocityMeter,
};

/// Pixels per line of a line-based wheel delta: what Blitz scrolls a line by.
const LINE_PX: f64 = 20.0;

/// Whether an element listens for a swipe at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeOn {
    /// A drag or a horizontal scroll to the right dismisses it.
    Yes,
    /// It cannot be swiped (a row the caller dismisses otherwise).
    No,
}

/// One element's swipe for one render.
#[derive(Clone, Copy)]
pub struct SwipeGlue {
    swiper: Swiper,
    on: SwipeOn,
    spring: SpringMotion,
    meter: Signal<VelocityMeter>,
    released: CopyValue<Touch>,
    followed: CopyValue<SwipeLook>,
}

/// An element's swipe: `on_dismiss` hears a release past a threshold. Hooks run whether or not
/// the swipe is `on`, so the element's hook order never changes with its props.
pub fn use_swipe_glue(
    on: SwipeOn,
    metrics: SwipeMetrics,
    on_dismiss: EventHandler<()>,
) -> SwipeGlue {
    SwipeGlue {
        swiper: use_swipe(metrics, on_dismiss),
        on,
        spring: use_spring_motion(0.0, PxPerUnit(1.0)),
        meter: use_signal(VelocityMeter::default),
        released: use_hook(|| CopyValue::new(Touch::Remote)),
        followed: use_hook(|| CopyValue::new(SwipeLook::Rest)),
    }
}

impl std::fmt::Debug for SwipeGlue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwipeGlue").field("on", &self.on).finish()
    }
}

impl SwipeGlue {
    /// The swiper, while the swipe is on.
    fn live(&self) -> Option<Swiper> {
        match self.on {
            SwipeOn::Yes => Some(self.swiper),
            SwipeOn::No => None,
        }
    }

    /// `data-swipe`: `live` while held, `rest`, or `gone`; nothing when the swipe is off.
    pub fn look(&self) -> Option<&'static str> {
        use ds_core::word::Word;
        self.live().map(|swiper| swiper.state().look().slug())
    }

    /// The element's inline offset, `--swipe-dx`, while it is off its place (on its way back
    /// too). Call once per render: it moves the return spring.
    pub fn style(&self) -> Option<String> {
        let state = self.live()?.state();
        let drawn = self.drawn(state);
        let off = drawn.0 != 0.0 || state.look() == SwipeLook::Gone;
        let rounded = (drawn.0 * 100.0).round() / 100.0;
        off.then(|| format!("--swipe-dx:{rounded}px"))
    }

    /// The offset to draw for `state`, moving the spring to follow it: 1:1 while a hand or a
    /// scroll holds the element, springing home once it lets go under both thresholds.
    fn drawn(&self, state: SwipeState) -> Px {
        let (spring, mut followed) = (self.spring, self.followed);
        let look = state.look();
        let before = *followed.peek();
        followed.set(look);
        match look {
            SwipeLook::Live => {
                let at = state.offset().0;
                queue_effect(move || spring.track(at));
                state.offset()
            }
            SwipeLook::Gone => state.offset(),
            SwipeLook::Rest => {
                if before != SwipeLook::Rest {
                    let mut released = self.released;
                    let touch = *released.peek();
                    released.set(Touch::Remote);
                    queue_effect(move || spring.go(0.0, SpringSpec::for_touch(touch)));
                }
                Px(spring.frame().position())
            }
        }
    }

    /// Whether a click on the element is a press (not the end of a drag).
    pub fn click_passes(&self) -> bool {
        self.live()
            .is_none_or(|swiper| swiper.take_click() == Click::Passes)
    }

    /// A pointer went down on the element: only the primary button starts a drag.
    pub fn down(&self, event: &PointerEvent) {
        let primary = button_of(event.trigger_button()) == Some(PointerButton::Primary);
        if let (Some(swiper), true) = (self.live(), primary) {
            let x = Px(event.client_coordinates().x as f32);
            let mut meter = self.meter;
            meter.set(VelocityMeter::default().moved(x, ds_core::time::clock::now()));
            swiper.feed(SwipeInput::Down {
                x,
                at: swiper.now(),
            });
        }
    }

    /// The pointer moved over the element.
    pub fn moved(&self, event: &PointerEvent) {
        if let Some(swiper) = self.live() {
            let held = if event.held_buttons().contains(MouseButton::Primary) {
                PressPhase::Pressed
            } else {
                PressPhase::Idle
            };
            let x = Px(event.client_coordinates().x as f32);
            let mut meter = self.meter;
            let measured = meter.peek().moved(x, ds_core::time::clock::now());
            meter.set(measured);
            swiper.pointer_moved(x, held);
        }
    }

    /// The pointer left the element: a release it could not hear, so no contact goes with it.
    pub fn left(&self) {
        if let Some(swiper) = self.live() {
            swiper.feed(SwipeInput::Up { at: swiper.now() });
        }
    }

    /// The pointer was released over the element: the release is the hand's contact, carrying
    /// the pointer's velocity into the return.
    pub fn released(&self, event: &PointerEvent) {
        if let Some(swiper) = self.live() {
            let velocity = self.meter.peek().released(ds_core::time::clock::now());
            let mut released = self.released;
            released.set(Touch::Contact(
                Contact::from_event(event).with_velocity(velocity),
            ));
            swiper.feed(SwipeInput::Up { at: swiper.now() });
        }
    }

    /// A wheel delta over the element.
    pub fn wheel(&self, event: &WheelEvent) {
        if let Some(swiper) = self.live() {
            let (dx, dy) = pixels(event.delta());
            swiper.feed(SwipeInput::Scroll { dx, dy });
        }
    }
}

/// A wheel delta in pixels, as the event gives it: Blitz forwards winit's sign (positive x is
/// content moving right), so a two-finger swipe to the right moves the element right. A line is
/// Blitz's 20 px; a page moves nothing.
fn pixels(delta: WheelDelta) -> (Px, Px) {
    let (x, y) = match delta {
        WheelDelta::Pixels(v) => (v.x, v.y),
        WheelDelta::Lines(v) => (v.x * LINE_PX, v.y * LINE_PX),
        WheelDelta::Pages(_) => (0.0, 0.0),
    };
    (Px(x as f32), Px(y as f32))
}
