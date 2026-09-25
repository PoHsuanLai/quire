//! A document's frame must have a height (sill F172, F225, Q104; CONSUMING section 2 "A
//! document's frame"). Blitz lays out `#main` at `height:auto`, so a frame placed with
//! `position:absolute; inset:0` whose content is out of flow (a card hung from its anchor) is
//! 0 px tall, and so is the `Ds` root in it. The floor (`display:grid; min-width:100vw;
//! min-height:100vh` on the frame: the viewport's size, and the root stretched to fill its one
//! cell) gives the root the viewport's height even inside an absolutely placed parent; a frame in
//! flow at the viewport's size with the same grid does too. Measured on a real Blitz document.
//!
//! Whether a press lands in a 0 px box is Blitz's hit test's business: in sill's bar popup
//! (F225) none did; in this harness the card's button still takes one. So the tests hold the
//! heights, which are what the rule is about, and check the press only where the frame is sound.

use dioxus::prelude::*;
use ds::{Appearance, Button, ButtonVariant, Ds, Material, Point, Rect};
use ds_native::{Harness, Viewport};
use std::cell::Cell;
use std::time::Duration;

/// How the page places the frame around its `Ds` root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Frame {
    /// Out of flow with insets: the trap.
    Absolute,
    /// Out of flow with insets, and the floor.
    AbsoluteWithFloor,
    /// In flow at the viewport's size, a one-cell grid (sill's popup frame after F225).
    InFlow,
}

impl Frame {
    fn css(self) -> &'static str {
        match self {
            Frame::Absolute => ".frame{position:absolute;top:0;right:0;bottom:0;left:0}",
            Frame::AbsoluteWithFloor => {
                ".frame{position:absolute;top:0;right:0;bottom:0;left:0;display:grid;min-width:100vw;min-height:100vh}"
            }
            Frame::InFlow => ".frame{display:grid;width:100vw;height:100vh}",
        }
    }
}

thread_local! {
    static FRAME: Cell<Frame> = const { Cell::new(Frame::Absolute) };
}

static PRESSES: GlobalSignal<u32> = Signal::global(|| 0);

const VIEW: Viewport = Viewport {
    width: 240,
    height: 160,
    scale_percent: 100,
};

/// The card is placed in the frame, as a popup's card is hung from its anchor: out of flow, so
/// the frame's height is whatever the frame gives itself.
const CARD: &str = ".card{position:absolute;top:40px;left:40px}";

/// A popup-like page: a frame holding a transparent Popover root with a card and a button in it.
#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        style { {FRAME.get().css()} {CARD} }
        div { class: "frame",
            Ds { appearance: Appearance::default(), material: Material::Popover,
                div { class: "card",
                    Button { variant: ButtonVariant::Mini, label: "Press", onclick: move |_| *PRESSES.write() += 1 }
                }
            }
        }
    }
}

fn laid_out(frame: Frame) -> Harness {
    FRAME.set(frame);
    let mut harness = Harness::new(Page, VIEW);
    harness.within(|| *PRESSES.write() = 0);
    harness.advance(Duration::from_millis(50));
    harness
}

fn height(harness: &Harness, selector: &str) -> f32 {
    harness
        .rect(selector)
        .map_or(0.0, |rect: Rect| rect.size.height.0)
}

/// Press the button where it is drawn; how many presses the page heard.
fn press(harness: &mut Harness) -> u32 {
    let button = harness.rect(".ds-button").expect("the button is laid out");
    harness.click(Point {
        x: ds::Px(button.origin.x.0 + button.size.width.0 / 2.0),
        y: ds::Px(button.origin.y.0 + button.size.height.0 / 2.0),
    });
    harness.advance(Duration::from_millis(50));
    harness.within(|| *PRESSES.peek())
}

#[test]
fn an_absolutely_placed_frame_with_no_floor_is_0_px() {
    let harness = laid_out(Frame::Absolute);
    assert_eq!(height(&harness, ".frame"), 0.0, "the trap is still a trap");
    assert_eq!(height(&harness, ".ds"), 0.0);
    assert_eq!(height(&harness, "html"), 0.0, "every box from html down");
}

#[test]
fn the_floor_gives_the_root_a_height_inside_an_absolutely_placed_parent() {
    let mut harness = laid_out(Frame::AbsoluteWithFloor);
    assert_eq!(height(&harness, ".frame"), 160.0);
    assert_eq!(height(&harness, ".ds"), 160.0, "the root fills the frame");
    assert_eq!(press(&mut harness), 1);
}

#[test]
fn a_frame_in_flow_at_the_viewports_size_takes_the_press() {
    let mut harness = laid_out(Frame::InFlow);
    assert_eq!(height(&harness, ".frame"), 160.0);
    assert_eq!(height(&harness, ".ds"), 160.0);
    assert_eq!(press(&mut harness), 1);
}
