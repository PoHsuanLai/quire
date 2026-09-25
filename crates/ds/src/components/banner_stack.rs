//! BannerStack: the notification banners on screen, newest first (sill Q121; design/20 section
//! 1.6, design/13 section 13.3.6). The caller lists the banners it shows; the stack plays each
//! arrival in from its `entry` edge (`Anim::BannerIn`, `--t-move --e-spring`), and each banner
//! the caller stops listing back out past it (`Anim::BannerOut`, `--t-move --e-exit`), keeping
//! it in the tree until its exit settles; the banners after it then heal into its place, by
//! the height it measured. `on_hidden` hears each banner's key once its exit has settled, so a
//! host whose list is empty then can unmap the surface. The hold timer, the stack's cap and
//! grouping are the caller's.
//!
//! `entry` is `notifications.banner_entry_direction` (design/05 section 12 item 7): from the
//! right by default, or rising from below. Both play the one `banner-in`/`banner-out` pair,
//! which translates by `--banner-dx`/`--banner-dy`; the stack's `data-entry` sets them, so the
//! edge is a setting and not a second set of keyframes (and of `Anim` variants to settle).
//!
//! A card swiped away inside the stack holds where the finger left it and reports at once; the
//! caller's removal makes its row slide out from there, to the right whatever the entry edge,
//! since it leaves along the swipe (`NotificationCard`'s swipe, the row's `data-flight`).

use crate::components::banner_row::BannerRow;
use crate::geometry::Px;
use crate::motion::presence::Exit;
use crate::motion::roster::RowPitch;
use crate::motion::roster_exits::{Leaving, use_leaving_roster, use_pitches};
use crate::tokens::notifications::STACK_GAP;
use dioxus::prelude::*;

/// A banner's identity: the notification's id, which the server gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BannerKey(pub u32);

/// One banner the caller shows: its key and its card (a `NotificationCard`, usually).
#[derive(Debug, Clone, PartialEq)]
pub struct Banner {
    /// Which notification.
    pub key: BannerKey,
    /// Its card.
    pub card: Element,
}

/// Where the stack stands (`notifications.banner_position`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BannerPosition {
    /// Under the bar at the right, newest on top; a banner leaving lets the ones below it rise.
    #[default]
    TopRight,
    /// Above the dock at the right, newest at the bottom; a banner leaving lets the ones above
    /// it drop.
    BottomRight,
}

impl BannerPosition {
    /// The `data-position` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            BannerPosition::TopRight => "top-right",
            BannerPosition::BottomRight => "bottom-right",
        }
    }

    /// Which way the rows after a leaving one heal: up into its place at the top right (they
    /// start below it), down at the bottom right (they start above it).
    pub(crate) fn heal_sign(self) -> f32 {
        match self {
            BannerPosition::TopRight => 1.0,
            BannerPosition::BottomRight => -1.0,
        }
    }
}

/// Which edge a banner enters from and leaves by (`notifications.banner_entry_direction`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BannerEntry {
    /// From past the surface's right edge, the edge the stack stands at.
    #[default]
    FromRight,
    /// Rising from below its place, and sinking back down as it fades out.
    FromBelow,
}

impl BannerEntry {
    /// The `data-entry` word, which picks the `--banner-dx`/`--banner-dy` the keyframes read.
    pub fn slug(self) -> &'static str {
        match self {
            BannerEntry::FromRight => "right",
            BannerEntry::FromBelow => "below",
        }
    }
}

/// The pitch a banner that measured nothing heals by: the least banner and the gap.
const FALLBACK_PITCH: RowPitch = RowPitch(Px(64.0 + 8.0));

/// The banners, newest first. `gap` overrides `--notifications-stack-gap` (the
/// `notifications.stack_gap_px` a `NotificationMetrics` around the stack writes); `entry` is the
/// edge banners come in from.
#[component]
pub fn BannerStack(
    banners: Vec<Banner>,
    #[props(default)] position: BannerPosition,
    #[props(default)] entry: BannerEntry,
    #[props(default)] gap: Option<Px>,
    #[props(default)] on_hidden: Option<EventHandler<BannerKey>>,
) -> Element {
    let pitches = use_pitches();
    let keys: Vec<BannerKey> = banners.iter().map(|banner| banner.key).collect();
    let cards = use_cards(&banners);
    let roster = use_leaving_roster(
        keys,
        Leaving {
            exit: Exit::BannerOut,
            pitch: FALLBACK_PITCH,
            pitches,
            on_settled: EventHandler::new(move |key| {
                cards.forget(key);
                if let Some(on_hidden) = on_hidden {
                    on_hidden.call(key);
                }
            }),
        },
    );
    let style = gap.map(|gap| STACK_GAP.write(&format!("{}px", gap.0)));
    rsx! {
        div {
            class: "ds-banner-stack",
            "data-position": position.slug(),
            "data-entry": entry.slug(),
            role: "log",
            "aria-label": "Notifications",
            style,
            for entry in roster.entries() {
                BannerRow {
                    key: "{entry.key.0}",
                    banner: entry.key,
                    presence: entry.presence,
                    position,
                    pitches,
                    card: cards.of(entry.key),
                }
            }
        }
    }
}

/// The last card each key was listed with, so a banner the caller has dropped still draws
/// while it leaves.
#[derive(Clone, Copy)]
struct Cards(CopyValue<Vec<(BannerKey, Element)>>);

impl Cards {
    fn of(&self, key: BannerKey) -> Element {
        self.0
            .peek()
            .iter()
            .find(|(held, _)| *held == key)
            .map_or_else(|| rsx! {}, |(_, card)| card.clone())
    }

    fn forget(&self, key: BannerKey) {
        let mut book = self.0;
        let _ = book
            .try_write()
            .map(|mut book| book.retain(|(held, _)| *held != key));
    }
}

/// The book of cards, refreshed from this render's banners (a listed card is always the
/// newest one; a dropped one keeps its last).
fn use_cards(banners: &[Banner]) -> Cards {
    let mut book = use_hook(|| CopyValue::new(Vec::<(BannerKey, Element)>::new()));
    let mut next: Vec<(BannerKey, Element)> = banners
        .iter()
        .map(|banner| (banner.key, banner.card.clone()))
        .collect();
    let kept = book
        .peek()
        .iter()
        .filter(|(key, _)| banners.iter().all(|banner| banner.key != *key))
        .cloned()
        .collect::<Vec<_>>();
    next.extend(kept);
    book.set(next);
    Cards(book)
}
