//! One row of a `BannerStack` (sill Q121): the card in a `div.ds-banner` that plays the row's
//! exit and heal (`banner-out`, `heal`), with the entrance (`banner-in`) on the `div.ds-banner-card`
//! inside it, and the height it measures, which is how far the rows after it heal when it
//! leaves. The gap to the next banner is the row's own padding, so the measured height is the
//! whole pitch.
//!
//! The entrance is the inner element's, played once as it mounts, rather than a rule on the
//! row's `data-presence=entering`: Blitz keeps the last animated value when an animation is
//! taken off an element, so a presence that turned `present` before a frame had been resolved
//! past the entrance's end (a loaded machine, a snapshot's clock) froze the row mid-slide.

use crate::components::banner_stack::{BannerKey, BannerPosition};
use crate::geometry::Px;
use crate::geometry::measure::client_rect;
use crate::motion::presence::Presence;
use crate::motion::roster::RowPitch;
use crate::motion::roster_exits::Pitches;
use crate::task::spawn_in;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use std::rc::Rc;

/// One banner in the stack.
#[component]
pub(crate) fn BannerRow(
    banner: BannerKey,
    presence: Presence,
    position: BannerPosition,
    pitches: Pitches<BannerKey>,
    card: Element,
) -> Element {
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    let mut leaving_seen = use_hook(|| CopyValue::new(Seen::No));
    let scope = use_hook(current_scope_id);
    // A task of the row's own scope: the read also runs from an effect, which has none.
    let measure = move || {
        if let Some(mounted) = element.peek().clone() {
            spawn_in(scope, async move {
                if let Some(rect) = client_rect(&mounted).await {
                    pitches.set(banner, RowPitch(rect.size.height));
                }
            });
        }
    };
    // Measured again as it starts to leave: hover may have opened it since it arrived.
    if matches!(presence, Presence::Leaving(_)) && *leaving_seen.peek() == Seen::No {
        leaving_seen.set(Seen::Yes);
        dioxus::core::queue_effect(measure);
    }
    rsx! {
        div {
            class: "ds-banner",
            "data-banner": "{banner.0}",
            "data-presence": presence.slug(),
            "data-exit": exit_slug(presence),
            style: heal_style(presence, position),
            onmounted: move |event| {
                element.set(Some(event.data()));
                measure();
            },
            div { class: "ds-banner-card", {card} }
        }
    }
}

/// Whether the row has been measured for its exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Seen {
    No,
    Yes,
}

/// `data-exit` while leaving.
fn exit_slug(presence: Presence) -> Option<&'static str> {
    match presence {
        Presence::Leaving(exit) => Some(exit.slug()),
        Presence::Entering | Presence::Present | Presence::Healing { .. } => None,
    }
}

/// A healing row's distance and delay: `--dy` signed for the stack's direction, `--d` heal
/// steps.
fn heal_style(presence: Presence, position: BannerPosition) -> Option<String> {
    match presence {
        Presence::Healing { dy, d } => Some(format!(
            "--dy:{}px;--d:{}",
            Px(dy.0 * position.heal_sign()).0,
            d.get()
        )),
        Presence::Entering | Presence::Present | Presence::Leaving(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::heal_style;
    use crate::components::banner_stack::BannerPosition;
    use crate::components::vocab::StaggerIndex;
    use crate::geometry::Px;
    use crate::motion::presence::Presence;

    #[test]
    fn a_healing_row_moves_towards_the_gap() {
        let healing = Presence::Healing {
            dy: Px(96.0),
            d: StaggerIndex::new(1),
        };
        let cases = [
            (healing, BannerPosition::TopRight, Some("--dy:96px;--d:1")),
            (
                healing,
                BannerPosition::BottomRight,
                Some("--dy:-96px;--d:1"),
            ),
            (Presence::Present, BannerPosition::TopRight, None),
        ];
        for (presence, position, want) in cases {
            assert_eq!(
                heal_style(presence, position).as_deref(),
                want,
                "{presence:?} {position:?}"
            );
        }
    }
}
