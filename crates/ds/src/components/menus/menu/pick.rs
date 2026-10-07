//! What a pick does to a menu (design/30 section 1.3, emphasis): the picked item blinks twice,
//! then the menu yields the value and closes. Split from `menu`, with the menu's closing and
//! gesture states.

use crate::components::menus::item::item::AfterPick;
use crate::components::menus::menu::blink::{Blink, half, phases, picked};
use crate::components::overlays::flow::Flow;
use dioxus::prelude::*;
use ds_core::time::clock::sleep;
use ds_style::task::spawn_in;

/// Where a pointer gesture over an open menu is: a press-drag-release onto an item picks it
/// (design/13 section 13.3.2), which is a release arriving after the pointer came in with no
/// press of its own inside the menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Gesture {
    /// The pointer has not been over the menu.
    Outside,
    /// The pointer came in; no press started inside the menu.
    Entered,
    /// A press started inside the menu: its release is a click, not a drag's end.
    Pressed,
}

/// Whether the menu is closing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Closing {
    /// Open.
    No,
    /// Playing `menu-out`; `onclose` runs when it settles.
    Fading,
}

/// What a panel reports when a choice is picked.
pub(crate) struct Picked<T> {
    /// What the item yields.
    pub value: T,
    /// The depth of the panel it was picked in: the root is 0.
    pub depth: u8,
    /// Whether the menu closes after it.
    pub after: AfterPick,
}

/// The handler a panel calls with a pick: the first pick of an item that closes the menu gives the
/// keyboard back, blinks the item, yields the value and closes the menu, and every later one is ignored. A floating
/// menu fades out on closing; an inline one, part of its caller's card, is closed at once. An
/// item that keeps the menu open yields its value at once, with no blink and no closing, as
/// often as it is picked.
pub(crate) fn picker<T: 'static>(
    blink: Signal<Blink>,
    closing: Signal<Closing>,
    flow: Flow,
    onpick: EventHandler<T>,
    close: Closer,
) -> EventHandler<Picked<T>> {
    let scope = dioxus::core::current_scope_id();
    let mut blink = blink;
    EventHandler::new(
        move |Picked {
                  value,
                  depth,
                  after,
              }: Picked<T>| {
            if picked(*blink.peek()) || *closing.peek() != Closing::No {
                return;
            }
            if after == AfterPick::KeepOpen {
                onpick.call(value);
                return;
            }
            close.give_back.call(());
            blink.set(Blink::Lit { depth });
            spawn_in(scope, async move {
                for phase in phases(depth) {
                    blink.set(phase);
                    if phase != Blink::Done {
                        sleep(half()).await;
                    }
                }
                onpick.call(value);
                match flow {
                    Flow::Floating => close.fade.call(()),
                    Flow::Inline => close.now.call(()),
                }
            });
        },
    )
}

/// How a menu closes after a pick: fading out first, or at once.
#[derive(Clone, Copy)]
pub(crate) struct Closer {
    /// Fade out, then close.
    pub fade: EventHandler<()>,
    /// Close now.
    pub now: EventHandler<()>,
    /// Give the keyboard back to the opener: called as a pick starts, before the item's handler
    /// runs, so a handler that moves the keyboard on has the last word.
    pub give_back: EventHandler<()>,
}
