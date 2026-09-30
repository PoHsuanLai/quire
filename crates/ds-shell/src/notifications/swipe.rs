//! A notification card's swipe to dismiss: whether the card takes one, and how its flight ends,
//! over the swipe glue a toast shares (`ds::SwipeGlue`).
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

use dioxus::prelude::*;
use ds::{SwipeGlue, SwipeOn, use_swipe_glue};
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::swipe::SwipeMetrics;
use ds_motion::timer::use_motion_timer;

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

/// The card's swipe: the shared glue, on when the caller listens. Hooks run whether or not the
/// swipe is on, so the card's hook order never changes with its props.
pub(crate) fn use_card_swipe(swipe: &NotificationSwipe, metrics: SwipeMetrics) -> SwipeGlue {
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
    let on = heard.map_or(SwipeOn::No, |_| SwipeOn::Yes);
    use_swipe_glue(on, metrics, on_dismiss)
}
