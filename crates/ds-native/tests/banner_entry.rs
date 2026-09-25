//! A `BannerStack`'s entry edge on a real Blitz document (`notifications.banner_entry_direction`,
//! design/05 section 12 item 7): a banner entering from the right starts past its right edge and
//! one entering from below starts under its place, each coming to rest where the other would;
//! a card swiped away in a from-below stack still marks its row to leave along the swipe.
//!
//! Blitz's layout rects leave transforms out (`Harness::rect`), so where the card is drawn is
//! read with the hit test, which applies them: a point just past the resting card's edge on the
//! entry side lands on the card while it is still sliding in, and never on the other side.

use dioxus::prelude::*;
use ds::{
    AppMark, Appearance, Banner, BannerEntry, BannerKey, BannerStack, Ds, Icon, IconSource,
    Material, NotificationCard, Point, Px, Rect, Swipe,
};
use ds_native::harness::settle_until;
use ds_native::{Harness, Viewport};
use std::cell::Cell;
use std::time::Duration;

/// Wider than the stack, so a card displaced past its right edge is still on the page.
const VIEW: Viewport = Viewport {
    width: 840,
    height: 400,
    scale_percent: 100,
};

static SHOWN: GlobalSignal<Vec<u32>> = Signal::global(Vec::new);
static ENTRY: GlobalSignal<BannerEntry> = Signal::global(BannerEntry::default);

#[allow(non_snake_case)]
fn Stack() -> Element {
    let banners: Vec<Banner> = SHOWN()
        .into_iter()
        .map(|n| Banner {
            key: BannerKey(n),
            card: rsx! {
                NotificationCard {
                    app: AppMark { icon: IconSource::Glyph(Icon::Mail), name: "Mail".into() },
                    age: "now",
                    summary: "Banner {n}",
                    on_close: |_| {},
                    on_open: |_| {},
                    swipe: Swipe::Dismiss(EventHandler::new(move |()| SHOWN.write().retain(|k| *k != n))),
                }
            },
        })
        .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Toast,
            div { style: "width:420px",
                BannerStack { banners, entry: ENTRY() }
            }
        }
    }
}

const CARD: &str = ".ds-notification";

/// Probe points around the resting card: its centre, and just past its right and bottom edges.
struct Probes {
    rest: Point,
    right: Point,
    below: Point,
}

impl Probes {
    fn around(card: Rect) -> Self {
        let centre = Point {
            x: Px(card.origin.x.0 + card.size.width.0 / 2.0),
            y: Px(card.origin.y.0 + card.size.height.0 / 2.0),
        };
        Probes {
            rest: centre,
            right: Point {
                x: Px(card.origin.x.0 + card.size.width.0 + 8.0),
                y: centre.y,
            },
            below: Point {
                x: centre.x,
                y: Px(card.origin.y.0 + card.size.height.0 + 8.0),
            },
        }
    }
}

/// Where the card was drawn over its entrance: past the right edge, past the bottom edge.
#[derive(Debug, PartialEq, Eq)]
struct Seen {
    right: Visit,
    below: Visit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Visit {
    Never,
    Once,
}

fn visit(hit: bool) -> Visit {
    if hit { Visit::Once } else { Visit::Never }
}

/// Mounts one banner entering from `entry` and polls until it rests on its place, noting every
/// side of it the card was drawn past on the way.
fn enter(entry: BannerEntry) -> (Harness, Probes, Seen) {
    let mut harness = Harness::new(Stack, VIEW);
    harness.within(|| {
        *ENTRY.write() = entry;
        *SHOWN.write() = vec![1];
    });
    harness.advance(Duration::from_millis(1));
    // The layout rect is the resting place: transforms never move it.
    let probes = Probes::around(harness.rect(CARD).expect("the card"));
    let right = Cell::new(false);
    let below = Cell::new(false);
    settle_until(&mut harness, |h| {
        right.set(right.get() || h.hits(probes.right, CARD));
        below.set(below.get() || h.hits(probes.below, CARD));
        h.attr(".ds-banner", "data-presence").as_deref() == Some("present")
            && h.hits(probes.rest, CARD)
            && !h.hits(probes.right, CARD)
            && !h.hits(probes.below, CARD)
    });
    let seen = Seen {
        right: visit(right.get()),
        below: visit(below.get()),
    };
    (harness, probes, seen)
}

#[test]
fn from_the_right_a_banner_starts_past_its_right_edge() {
    let (harness, _, seen) = enter(BannerEntry::FromRight);
    assert_eq!(
        harness.attr(".ds-banner-stack", "data-entry").as_deref(),
        Some("right")
    );
    assert_eq!(
        seen,
        Seen {
            right: Visit::Once,
            below: Visit::Never,
        }
    );
}

#[test]
fn from_below_a_banner_rises_into_its_place() {
    let (harness, _, seen) = enter(BannerEntry::FromBelow);
    assert_eq!(
        harness.attr(".ds-banner-stack", "data-entry").as_deref(),
        Some("below")
    );
    assert_eq!(
        seen,
        Seen {
            right: Visit::Never,
            below: Visit::Once,
        }
    );
}

#[test]
fn a_card_swiped_from_a_from_below_stack_leaves_along_the_swipe() {
    let (mut harness, probes, _) = enter(BannerEntry::FromBelow);
    assert_eq!(harness.attr(".ds-banner", "data-flight"), None);
    let at = probes.rest;
    harness.pointer_down(at);
    for step in 1..=4u8 {
        harness.advance(Duration::from_millis(100));
        harness.pointer_move(Point {
            x: Px(at.x.0 + 25.0 * f32::from(step)),
            y: at.y,
        });
    }
    harness.pointer_up(Point {
        x: Px(at.x.0 + 100.0),
        y: at.y,
    });
    assert_eq!(
        harness.attr(".ds-banner", "data-presence").as_deref(),
        Some("leaving")
    );
    assert_eq!(
        harness.attr(".ds-banner", "data-flight").as_deref(),
        Some("swipe")
    );
    settle_until(&mut harness, |h| h.count(".ds-banner") == 0);
}
