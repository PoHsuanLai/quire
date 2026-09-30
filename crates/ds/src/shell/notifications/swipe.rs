//! A notification card's swipe to dismiss: whether the card takes one, and the
//! glue between its pointer and wheel events and the pure machine (`motion::swipe`).
//!
//! When a swipe ends past a threshold the card flies out to the right from where it is. Alone,
//! it plays `banner-out` itself and reports `on_dismiss` at `settle(BannerOut)`, so a caller that
//! unmounts it there never cuts the flight short. Inside a `BannerStack` the stack's row carries
//! the flight: the card holds its offset, reports at once, and the caller's removal of it makes
//! the row slide out from that offset (the two transforms compose), then the rows below heal.
//!
//! Driven motion (design/05 section 14): a card let go under both thresholds springs
//! back from where it is, carrying the hand's release velocity, instead of easing on a CSS
//! transition; a new drag mid-return picks it up where it is.

use crate::components::controls::press::button_of;
use dioxus::core::queue_effect;
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::press::PointerButton;
use ds_core::vocab::PressPhase;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::detail::touch::{Contact, Touch};
use ds_motion::swipe::{Click, SwipeInput, SwipeLook, SwipeMetrics, SwipeState};
use ds_motion::timer::use_motion_timer;
use ds_motion::use_swipe::{Swiper, use_swipe};
use ds_motion::{
    spring_spec::SpringSpec,
    timeline::spring::PxPerUnit,
    use_spring::{SpringMotion, use_spring_motion},
    velocity::VelocityMeter,
};

/// Whether a card can be swiped away, and who hears it.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum NotificationSwipe {
    /// It cannot (a row of the center that the caller dismisses otherwise).
    #[default]
    Off,
    /// A drag or a horizontal scroll to the right dismisses it; the handler hears when the card
    /// has gone (`notifications.swipe` says whether the caller keeps it in the center).
    Dismiss(EventHandler<()>),
}

/// Marks a card as carried by a `BannerStack` row, whose own exit flies it out; the card marks
/// the row's flight `Swipe` when it goes, so the row leaves along the swipe rather than by the
/// stack's entry edge.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct Carried(pub(crate) Signal<Flight>);

/// Which way a leaving `BannerStack` row flies out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub(crate) enum Flight {
    /// Back past the edge it entered by (the caller dropped it: a timeout, a close).
    Edge,
    /// To the right, the way its card was swiped.
    Swipe,
}

impl Flight {
    /// `data-flight`, written only for a swipe: an edge flight reads the stack's own vector.
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != Flight::Edge).then(|| self.slug())
    }
}

/// Pixels per line of a line-based wheel delta: what Blitz scrolls a line by.
const LINE_PX: f64 = 20.0;

/// The card's swipe for one render: the machine and whether it is on, and the spring that
/// carries it back to its place.
#[derive(Clone, Copy)]
pub(crate) struct CardSwipe {
    swiper: Swiper,
    on: SwipeOn,
    back: ReturnSpring,
}

/// The spring a card returns on, the pointer's speed as it goes, and the last look the render
/// followed.
#[derive(Clone, Copy)]
struct ReturnSpring {
    spring: SpringMotion,
    meter: Signal<VelocityMeter>,
    released: CopyValue<Touch>,
    followed: CopyValue<SwipeLook>,
}

impl ReturnSpring {
    /// The offset to draw for `state`, moving the spring to follow it: 1:1 while a hand or a
    /// scroll holds the card, springing home once it lets go under both thresholds.
    fn drawn(self, state: SwipeState) -> Px {
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
}

/// Whether the card listens for a swipe at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwipeOn {
    Yes,
    No,
}

/// The card's swipe. Hooks run whether or not the swipe is on, so the card's hook order never
/// changes with its props.
pub(crate) fn use_card_swipe(swipe: &NotificationSwipe, metrics: SwipeMetrics) -> CardSwipe {
    let flight = use_motion_timer(Anim::BannerOut);
    let carried = try_use_context::<Carried>();
    let heard = match swipe {
        NotificationSwipe::Dismiss(handler) => Some(*handler),
        NotificationSwipe::Off => None,
    };
    let on_dismiss = EventHandler::new(move |()| {
        let Some(heard) = heard else { return };
        match carried {
            Some(Carried(mut flight)) => {
                flight.set(Flight::Swipe);
                heard.call(());
            }
            None => flight.start(EventHandler::new(move |()| heard.call(()))),
        }
    });
    CardSwipe {
        swiper: use_swipe(metrics, on_dismiss),
        on: heard.map_or(SwipeOn::No, |_| SwipeOn::Yes),
        back: ReturnSpring {
            spring: use_spring_motion(0.0, PxPerUnit(1.0)),
            meter: use_signal(VelocityMeter::default),
            released: use_hook(|| CopyValue::new(Touch::Remote)),
            followed: use_hook(|| CopyValue::new(SwipeLook::Rest)),
        },
    }
}

impl CardSwipe {
    /// `data-swipe`, only on a card that can be swiped.
    pub(crate) fn look(&self) -> Option<&'static str> {
        self.live().map(|swiper| swiper.state().look().slug())
    }

    /// The card's inline offset, `--swipe-dx`, while it is off its place (on its way back
    /// too). Call once per render: it moves the return spring.
    pub(crate) fn style(&self) -> Option<String> {
        let state = self.live()?.state();
        let drawn = self.back.drawn(state);
        let off = drawn.0 != 0.0 || state.look() == SwipeLook::Gone;
        let rounded = (drawn.0 * 100.0).round() / 100.0;
        off.then(|| format!("--swipe-dx:{rounded}px"))
    }

    /// Whether a click on the card is a press (not the end of a drag).
    pub(crate) fn click_passes(&self) -> bool {
        self.live()
            .is_none_or(|swiper| swiper.take_click() == Click::Passes)
    }

    /// A pointer went down on the card: only the primary button starts a drag.
    pub(crate) fn down(&self, event: &PointerEvent) {
        let primary = button_of(event.trigger_button()) == Some(PointerButton::Primary);
        if let (Some(swiper), true) = (self.live(), primary) {
            let x = Px(event.client_coordinates().x as f32);
            let mut meter = self.back.meter;
            meter.set(VelocityMeter::default().moved(x, ds_core::time::clock::now()));
            swiper.feed(SwipeInput::Down {
                x,
                at: swiper.now(),
            });
        }
    }

    /// The pointer moved over the card.
    pub(crate) fn moved(&self, event: &PointerEvent) {
        if let Some(swiper) = self.live() {
            let held = if event.held_buttons().contains(MouseButton::Primary) {
                PressPhase::Pressed
            } else {
                PressPhase::Idle
            };
            let x = Px(event.client_coordinates().x as f32);
            let mut meter = self.back.meter;
            let measured = meter.peek().moved(x, ds_core::time::clock::now());
            meter.set(measured);
            swiper.pointer_moved(x, held);
        }
    }

    /// The pointer left the card: a release it could not hear, so no contact goes with it.
    pub(crate) fn up(&self) {
        if let Some(swiper) = self.live() {
            swiper.feed(SwipeInput::Up { at: swiper.now() });
        }
    }

    /// The pointer was released over the card: the release is the hand's contact, carrying the
    /// pointer's velocity into the return (design/27 section 3.12).
    pub(crate) fn released(&self, event: &PointerEvent) {
        if let Some(swiper) = self.live() {
            let velocity = self.back.meter.peek().released(ds_core::time::clock::now());
            let mut released = self.back.released;
            released.set(Touch::Contact(
                Contact::from_event(event).with_velocity(velocity),
            ));
            swiper.feed(SwipeInput::Up { at: swiper.now() });
        }
    }

    /// A wheel delta over the card.
    pub(crate) fn wheel(&self, event: &WheelEvent) {
        if let Some(swiper) = self.live() {
            let (dx, dy) = pixels(event.delta());
            swiper.feed(SwipeInput::Scroll { dx, dy });
        }
    }

    fn live(&self) -> Option<Swiper> {
        match self.on {
            SwipeOn::Yes => Some(self.swiper),
            SwipeOn::No => None,
        }
    }
}

/// A wheel delta in pixels, as the event gives it: Blitz forwards winit's sign (positive x is
/// content moving right), so a two-finger swipe to the right moves the card right. A line is
/// Blitz's 20 px; a page moves nothing.
fn pixels(delta: WheelDelta) -> (Px, Px) {
    let (x, y) = match delta {
        WheelDelta::Pixels(v) => (v.x, v.y),
        WheelDelta::Lines(v) => (v.x * LINE_PX, v.y * LINE_PX),
        WheelDelta::Pages(_) => (0.0, 0.0),
    };
    (Px(x as f32), Px(y as f32))
}
