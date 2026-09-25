//! A notification card's swipe to dismiss (sill Q122): whether the card takes one, and the
//! glue between its pointer and wheel events and the pure machine (`motion::swipe`).
//!
//! When a swipe ends past a threshold the card flies out to the right from where it is. Alone,
//! it plays `banner-out` itself and reports `on_dismiss` at `settle(BannerOut)`, so a caller that
//! unmounts it there never cuts the flight short. Inside a `BannerStack` the stack's row carries
//! the flight: the card holds its offset, reports at once, and the caller's removal of it makes
//! the row slide out from that offset (the two transforms compose), then the rows below heal.

use crate::components::press::{PointerButton, button_of};
use crate::geometry::Px;
use crate::motion::anim::Anim;
use crate::motion::swipe::{Click, SwipeInput, SwipeLook, SwipeMetrics};
use crate::motion::timer::use_motion_timer;
use crate::motion::use_swipe::{Held, Swiper, use_swipe};
use dioxus::html::geometry::WheelDelta;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;

/// Whether a card can be swiped away, and who hears it.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Swipe {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Flight {
    /// Back past the edge it entered by (the caller dropped it: a timeout, a close).
    Edge,
    /// To the right, the way its card was swiped.
    Swipe,
}

impl Flight {
    /// `data-flight`, written only for a swipe: an edge flight reads the stack's own vector.
    pub(crate) fn slug(self) -> Option<&'static str> {
        match self {
            Flight::Edge => None,
            Flight::Swipe => Some("swipe"),
        }
    }
}

/// Pixels per line of a line-based wheel delta: what Blitz scrolls a line by.
const LINE_PX: f64 = 20.0;

/// The card's swipe for one render: the machine and whether it is on.
#[derive(Clone, Copy)]
pub(crate) struct CardSwipe {
    swiper: Swiper,
    on: SwipeOn,
}

/// Whether the card listens for a swipe at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SwipeOn {
    Yes,
    No,
}

/// The card's swipe. Hooks run whether or not the swipe is on, so the card's hook order never
/// changes with its props.
pub(crate) fn use_card_swipe(swipe: &Swipe, metrics: SwipeMetrics) -> CardSwipe {
    let flight = use_motion_timer(Anim::BannerOut);
    let carried = try_use_context::<Carried>();
    let heard = match swipe {
        Swipe::Dismiss(handler) => Some(*handler),
        Swipe::Off => None,
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
    }
}

impl CardSwipe {
    /// `data-swipe`, only on a card that can be swiped.
    pub(crate) fn look(&self) -> Option<&'static str> {
        self.live().map(|swiper| swiper.state().look().slug())
    }

    /// The card's inline offset, `--swipe-dx`, while it is off its place.
    pub(crate) fn style(&self) -> Option<String> {
        let state = self.live()?.state();
        let off = state.offset().0 != 0.0 || state.look() == SwipeLook::Gone;
        off.then(|| format!("--swipe-dx:{}px", state.offset().0))
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
                Held::Primary
            } else {
                Held::Nothing
            };
            swiper.pointer_moved(Px(event.client_coordinates().x as f32), held);
        }
    }

    /// The pointer was released over the card, or left it.
    pub(crate) fn up(&self) {
        if let Some(swiper) = self.live() {
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
