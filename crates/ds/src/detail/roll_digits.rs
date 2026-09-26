//! RollDigits: a readout's digits roll to a new value instead of snapping (design/26-DETAILS.md
//! A7, section 8 item 3).

use super::level::use_level;
use crate::appearance::MotionLevel;
use crate::motion::{Anim, TimerPhase, use_motion_timer};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The value shown and, while it rolls, the one before.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Rolling {
    value: String,
    before: Option<String>,
    round: u32,
}

/// One place of the readout, from the right.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Place {
    /// The same character before and after.
    Kept(char),
    /// A character that changed: the old one (if the place existed) rolls out as the new one (if
    /// it still exists) rolls in.
    Rolled(Option<char>, Option<char>),
}

/// `value`, each character that changed since the last value rolling up into place over
/// `--t-quick`; unchanged characters stand still. The first frame is still, the same text rolls
/// nothing (R2), and under Reduced a change snaps (R7). The text is the element's accessible name.
#[component]
pub fn RollDigits(value: String) -> Element {
    let timer = use_motion_timer(Anim::RollIn);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let env = use_level();
    let mut seen = use_hook(|| {
        CopyValue::new(Rolling {
            value: value.clone(),
            before: None,
            round: 0,
        })
    });
    let was = seen.peek().clone();
    if was.value != value {
        let reduced = env.now() == MotionLevel::Reduced;
        seen.set(Rolling {
            value: value.clone(),
            before: (!reduced).then(|| was.value.clone()),
            round: was.round.wrapping_add(1),
        });
        if !reduced {
            queue_effect(move || timer.start(settled));
        }
    }
    let shown = seen.peek().clone();
    let rolling = match (timer.phase(), &shown.before) {
        (TimerPhase::Running, Some(before)) => Some(places(before, &shown.value)),
        _ => None,
    };
    let alias = if shown.round.is_multiple_of(2) {
        "b"
    } else {
        "a"
    };
    rsx! {
        span { class: "ds-roll-digits", role: "img", "aria-label": "{value}",
            match rolling {
                None => rsx! { span { class: "ds-roll-digit", "aria-hidden": "true", "{value}" } },
                Some(places) => rsx! {
                    for (at, place) in places.into_iter().enumerate() {
                        {place_view(at, place, shown.round, alias)}
                    }
                },
            }
        }
    }
}

/// One place's markup while the readout rolls.
fn place_view(at: usize, place: Place, round: u32, alias: &'static str) -> Element {
    match place {
        Place::Kept(kept) => rsx! {
            span { key: "{round}-{at}", class: "ds-roll-digit", "aria-hidden": "true", "{kept}" }
        },
        Place::Rolled(old, new) => rsx! {
            span { key: "{round}-{at}", class: "ds-roll-col", "aria-hidden": "true",
                if let Some(old) = old {
                    span { class: "ds-roll-digit ds-roll-old {Anim::RollOut.class()}", "data-pulse": alias, "{old}" }
                }
                span {
                    class: "ds-roll-digit {Anim::RollIn.class()}",
                    "data-pulse": alias,
                    {new.map(String::from).unwrap_or_default()}
                }
            }
        },
    }
}

/// `before` and `after` aligned by their right ends, place by place from the left.
fn places(before: &str, after: &str) -> Vec<Place> {
    let old: Vec<char> = before.chars().collect();
    let new: Vec<char> = after.chars().collect();
    let width = old.len().max(new.len());
    let at = |chars: &[char], place: usize| {
        (place + chars.len())
            .checked_sub(width)
            .and_then(|index| chars.get(index).copied())
    };
    (0..width)
        .map(|place| match (at(&old, place), at(&new, place)) {
            (Some(a), Some(b)) if a == b => Place::Kept(b),
            (a, b) => Place::Rolled(a, b),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Place, places};

    #[test]
    fn places_align_by_their_right_ends() {
        assert_eq!(
            places("79%", "80%"),
            vec![
                Place::Rolled(Some('7'), Some('8')),
                Place::Rolled(Some('9'), Some('0')),
                Place::Kept('%'),
            ]
        );
        assert_eq!(
            places("9", "10"),
            vec![
                Place::Rolled(None, Some('1')),
                Place::Rolled(Some('9'), Some('0'))
            ]
        );
        assert_eq!(
            places("100", "99"),
            vec![
                Place::Rolled(Some('1'), None),
                Place::Rolled(Some('0'), Some('9')),
                Place::Rolled(Some('0'), Some('9')),
            ]
        );
    }
}
