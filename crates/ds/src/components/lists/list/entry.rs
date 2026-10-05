//! One item of a `List`: the caller's content in a `div.ds-list-item` that plays the item's
//! entrance, exit and heal, and measures its height, which is how far the items below it heal
//! when it leaves.

use crate::host::measure::client_rect;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_motion::presence::Presence;
use ds_motion::roster::{Heal, RosterEntry, RowPitch, presence_slug};
use ds_motion::use_roster::Pitches;
use ds_style::task::spawn_in;
use std::rc::Rc;

/// Where an item stands among the `len` items of its set, for a screen reader and for a selector
/// that addresses the nth item (`[aria-posinset="2"]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SetPlace {
    /// The item's index in the set, from 0.
    pub(crate) index: usize,
    /// How many items the set has.
    pub(crate) len: usize,
}

impl SetPlace {
    /// `aria-posinset`: one-based.
    pub(crate) fn posinset(self) -> String {
        (self.index + 1).to_string()
    }

    /// `aria-setsize`.
    pub(crate) fn setsize(self) -> String {
        self.len.to_string()
    }
}

/// One item in the list.
#[component]
pub(crate) fn ListEntry<K: Clone + PartialEq + 'static>(
    entry: RosterEntry<K>,
    pitches: Pitches<K>,
    place: SetPlace,
    content: Element,
) -> Element {
    let mut element = use_signal(|| None::<Rc<MountedData>>);
    let mut measured_leaving = use_hook(|| CopyValue::new(Measured::AtMount));
    let scope = use_hook(current_scope_id);
    let key = entry.key.clone();
    // A task of the row's own scope: the read also runs from an effect, which has none.
    let measure = move || {
        if let Some(mounted) = element.peek().clone() {
            let key = key.clone();
            spawn_in(scope, async move {
                if let Some(rect) = client_rect(&mounted).await {
                    pitches.set(key, RowPitch(rect.size.height));
                }
            });
        }
    };
    let leaving = matches!(entry.presence, Presence::Leaving(_));
    let last = *measured_leaving.peek();
    match (leaving, last) {
        (true, Measured::AtMount) => {
            measured_leaving.set(Measured::Leaving);
            dioxus::core::queue_effect(measure.clone());
        }
        // Taken back: measured again if it leaves again.
        (false, Measured::Leaving) => measured_leaving.set(Measured::AtMount),
        (true, Measured::Leaving) | (false, Measured::AtMount) => {}
    }
    rsx! {
        div {
            class: "ds-list-item",
            role: "none",
            "aria-posinset": place.posinset(),
            "aria-setsize": place.setsize(),
            "data-presence": presence_slug(entry.presence, entry.heal),
            "data-exit": exit_slug(entry.presence),
            style: motion_style(&entry),
            onmounted: move |event| {
                element.set(Some(event.data()));
                measure();
            },
            {content}
        }
    }
}

/// When the row was last measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Measured {
    /// As it mounted (and, if it has been, as it arrived).
    AtMount,
    /// As it started to leave.
    Leaving,
}

/// `data-exit` while leaving.
fn exit_slug(presence: Presence) -> Option<&'static str> {
    match presence {
        Presence::Leaving(exit) => Some(exit.slug()),
        Presence::Hidden | Presence::Entering | Presence::Present => None,
    }
}

/// The row's motion variable while it heals: the distance `--dy`.
fn motion_style<K>(entry: &RosterEntry<K>) -> Option<String> {
    entry.heal.map(|Heal { dy }| format!("--dy:{}px", dy.0))
}

#[cfg(test)]
mod tests {
    use super::{SetPlace, motion_style};
    use ds_core::geometry::units::Px;
    use ds_motion::presence::{Exit, Presence};
    use ds_motion::roster::{Heal, RosterEntry};

    #[test]
    fn a_place_reads_one_based() {
        let place = SetPlace { index: 0, len: 12 };
        assert_eq!(
            (place.posinset().as_str(), place.setsize().as_str()),
            ("1", "12")
        );
        let place = SetPlace {
            index: 4999,
            len: 10_000,
        };
        assert_eq!(place.posinset(), "5000");
    }

    #[test]
    fn only_a_healing_row_writes_its_distance() {
        let entry = |presence, heal| RosterEntry {
            key: 1u8,
            presence,
            heal,
            until: None,
        };
        let healing = Heal { dy: Px(96.5) };
        let cases = [
            (Presence::Entering, None, None),
            (Presence::Leaving(Exit::Row), None, None),
            (Presence::Present, Some(healing), Some("--dy:96.5px")),
            (Presence::Present, None, None),
        ];
        for (presence, heal, want) in cases {
            assert_eq!(
                motion_style(&entry(presence, heal)).as_deref(),
                want,
                "{presence:?} {heal:?}"
            );
        }
    }
}
