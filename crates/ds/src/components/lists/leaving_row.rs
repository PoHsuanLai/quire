//! One row of a `LeavingList`: the consumer's content in a `div.ds-leaving-row`
//! that plays the row's entrance, exit and heal, and measures its height, which is how far the
//! rows below it heal when it leaves.

use crate::style::task::spawn_in;
use crate::core::word::Word;
use crate::host::measure::client_rect;
use crate::motion::presence::Presence;
use crate::motion::roster::{Heal, RosterEntry, RowPitch, presence_slug};
use crate::motion::use_roster::Pitches;
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use std::rc::Rc;

/// One row in the list.
#[component]
pub(crate) fn LeavingRow<K: Clone + PartialEq + 'static>(
    entry: RosterEntry<K>,
    pitches: Pitches<K>,
    row: Element,
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
            class: "ds-leaving-row",
            role: "listitem",
            "data-presence": presence_slug(entry.presence, entry.heal),
            "data-exit": exit_slug(entry.presence),
            style: motion_style(&entry),
            onmounted: move |event| {
                element.set(Some(event.data()));
                measure();
            },
            {row}
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
    use super::motion_style;
    use crate::core::geometry::units::Px;
    use crate::motion::presence::{Exit, Presence};
    use crate::motion::roster::{Heal, RosterEntry};

    #[test]
    fn only_a_healing_row_writes_its_distance() {
        let entry = |presence, heal| RosterEntry {
            key: 1u8,
            presence,
            heal,
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
