//! One row of a `LeavingList` (sill Q510): the consumer's content in a `div.ds-leaving-row`
//! that plays the row's entrance, exit and heal, and measures its height, which is how far the
//! rows below it heal when it leaves.

use crate::geometry::measure::client_rect;
use crate::motion::presence::Presence;
use crate::motion::roster::{RosterEntry, RowPitch};
use crate::motion::roster_exits::Pitches;
use crate::task::spawn_in;
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
            "data-presence": entry.presence.slug(),
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
        Presence::Entering | Presence::Present | Presence::Healing { .. } => None,
    }
}

/// The row's motion variables: its stagger while it enters or leaves, its heal distance and
/// delay while it heals.
fn motion_style<K>(entry: &RosterEntry<K>) -> Option<String> {
    match entry.presence {
        Presence::Entering | Presence::Leaving(_) => Some(format!("--i:{}", entry.index.get())),
        Presence::Healing { dy, d } => Some(format!("--dy:{}px;--d:{}", dy.0, d.get())),
        Presence::Present => None,
    }
}

#[cfg(test)]
mod tests {
    use super::motion_style;
    use crate::components::vocab::StaggerIndex;
    use crate::geometry::Px;
    use crate::motion::presence::{Exit, Presence};
    use crate::motion::roster::RosterEntry;

    #[test]
    fn each_state_writes_its_own_variables() {
        let entry = |presence| RosterEntry {
            key: 1u8,
            presence,
            index: StaggerIndex::new(3),
        };
        let cases = [
            (Presence::Entering, Some("--i:3")),
            (Presence::Leaving(Exit::Fold), Some("--i:3")),
            (
                Presence::Healing {
                    dy: Px(96.5),
                    d: StaggerIndex::new(2),
                },
                Some("--dy:96.5px;--d:2"),
            ),
            (Presence::Present, None),
        ];
        for (presence, want) in cases {
            assert_eq!(
                motion_style(&entry(presence)).as_deref(),
                want,
                "{presence:?}"
            );
        }
    }
}
