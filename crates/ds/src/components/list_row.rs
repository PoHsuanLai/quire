//! ListRow: one item in a list, the thread row (design/04-COMPONENTS.md section 16).

use crate::components::row_click::snapshot;
use crate::components::row_hooks::{PartHooks, enter, leave, relay};
use crate::components::row_star::star_button;
use crate::components::text_runs::{Text, text};
use crate::components::vocab::{DropState, Emphasis, PulseKey, Selection, StaggerIndex, Switch};
use crate::motion::presence::Presence;
use crate::text::clip_chars;
use dioxus::prelude::*;

/// How many characters of a name the name column holds before it must fade: the column at
/// the narrowest window S draws (980 px, design/01-LAYOUT.md section 1), which leaves the list
/// column about 352 px and, after the list's padding, the row's padding and border, the dot
/// column, the grid gaps, the tail (a data-10 time and a chip) and the via mark, about 196 px for
/// the name; at ui 13.5 / 700 (design/02-TYPE.md) a Karla character averages about 7.4 px.
/// A name within it is drawn whole; only a longer one gets `.ds-truncate`'s end fade, because
/// CSS cannot ask Blitz whether text overflows (spike S13) and a fade on text that fits eats its
/// last letters (gallery fix A).
const NAME_BUDGET: usize = 26;

/// Whether a name fits its column or runs past it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NameFit {
    /// Drawn whole, no fade.
    Fitting,
    /// Longer than the column: its end fades (`.ds-truncate`).
    Overflowing,
}

impl NameFit {
    /// Whether `name` fits in `budget` characters: what [`clip_chars`] would leave untouched.
    fn of(name: &str, budget: usize) -> Self {
        if clip_chars(name, budget) == name {
            NameFit::Fitting
        } else {
            NameFit::Overflowing
        }
    }

    /// The name span's class list.
    fn class(self) -> &'static str {
        match self {
            NameFit::Fitting => "ds-row-name",
            NameFit::Overflowing => "ds-row-name ds-truncate",
        }
    }
}

/// The `data-emphasis` word.
fn emphasis_slug(emphasis: Emphasis) -> &'static str {
    match emphasis {
        Emphasis::Strong => "strong",
        Emphasis::Plain => "plain",
    }
}

/// The row's inline custom properties: the stagger `--i` always, and while healing the
/// distance `--dy` and the heal index `--d`.
fn row_style(index: StaggerIndex, presence: Presence) -> String {
    let i = index.get();
    match presence {
        Presence::Healing { dy, d } => format!("--i:{i};--dy:{}px;--d:{}", dy.0, d.get()),
        Presence::Entering | Presence::Present | Presence::Leaving(_) => format!("--i:{i}"),
    }
}

/// `data-exit`, on a leaving row only.
fn exit(presence: Presence) -> Option<&'static str> {
    match presence {
        Presence::Leaving(exit) => Some(exit.slug()),
        Presence::Entering | Presence::Present | Presence::Healing { .. } => None,
    }
}

/// One row: dot, name and via, subject, snippet, tail, star, and a hover-strip slot.
///
/// `presence` comes from `use_roster`: an entering row rises staggered by `index` when its
/// list is first shown and plays `row-in` when it arrives later; a leaving row plays its exit
/// (an unread fold is the heavy one, as the roster settles it); a healing row slides up from
/// `dy`, delayed by `d` heal steps. `star_pulse` is a `use_pulse(Anim::StarPop)` key, fired on
/// every toggle. `onclick` receives the pointer's data, so the consumer can read Shift to peek.
/// `drop` is the row's part in a drag: `Source` while it is the thread being dragged (dimmed),
/// `Target` while something dragged over it would land on it.
///
/// `subject` and `snippet` are [`Text`]: a string as before, or the runs a search hit marked.
/// `on_sender` and `on_time` hear the pointer entering and leaving the name and the time (their
/// own hover cards); `onpointerenter`, `onpointerleave` and `onpointerdown` hear the row itself
/// (the thread card, a drag's start). `aria_label` names the row for a screen reader ("Open
/// Re: UIDL stability"); absent, the row is named by its contents as before.
#[component]
pub fn ListRow(
    selection: Selection,
    emphasis: Emphasis,
    index: StaggerIndex,
    presence: Presence,
    name: String,
    via: Option<Element>,
    #[props(into)] subject: Text,
    snippet: Option<Text>,
    time: String,
    tags: Element,
    star: Option<(Switch, EventHandler<Switch>)>,
    star_pulse: PulseKey,
    strip: Option<Element>,
    onclick: EventHandler<MouseData>,
    #[props(default)] drop: DropState,
    #[props(default)] on_sender: Option<PartHooks>,
    #[props(default)] on_time: Option<PartHooks>,
    #[props(default)] onpointerenter: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerleave: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerdown: Option<EventHandler<PointerEvent>>,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    rsx! {
        li {
            class: "ds-row",
            role: "option",
            "aria-selected": selection.aria(),
            "aria-label": aria_label,
            "data-emphasis": emphasis_slug(emphasis),
            "data-presence": presence.slug(),
            "data-exit": exit(presence),
            "data-drop": drop.drop_attr(),
            "data-drag": drop.drag_attr(),
            style: row_style(index, presence),
            onclick: move |event| onclick.call(snapshot(&event.data())),
            onpointerenter: relay(onpointerenter),
            onpointerleave: relay(onpointerleave),
            onpointerdown: relay(onpointerdown),
            div { class: "ds-row-dot",
                span { class: "ds-dot" }
            }
            div { class: "ds-row-main",
                div { class: "ds-row-from",
                    span {
                        class: NameFit::of(&name, NAME_BUDGET).class(),
                        onpointerenter: enter(on_sender),
                        onpointerleave: leave(on_sender),
                        "{name}"
                    }
                    if let Some(via) = via {
                        span { class: "ds-row-via", {via} }
                    }
                }
                div { class: "ds-row-sub ds-truncate", {text(&subject)} }
                if let Some(snippet) = snippet {
                    div { class: "ds-row-snip ds-truncate", {text(&snippet)} }
                }
            }
            div { class: "ds-row-tail",
                span {
                    class: "ds-row-time",
                    onpointerenter: enter(on_time),
                    onpointerleave: leave(on_time),
                    "{time}"
                }
                span { class: "ds-row-tags", {tags} }
            }
            if let Some((state, onchange)) = star {
                {star_button(state, onchange, star_pulse)}
            }
            if let Some(strip) = strip {
                {strip}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NAME_BUDGET, NameFit, exit, row_style};
    use crate::components::vocab::StaggerIndex;
    use crate::geometry::Px;
    use crate::motion::presence::{Exit, Presence};

    #[test]
    fn a_healing_row_carries_its_distance_and_delay() {
        let healing = Presence::Healing {
            dy: Px(79.0),
            d: StaggerIndex::new(2),
        };
        assert_eq!(
            row_style(StaggerIndex::new(4), healing),
            "--i:4;--dy:79px;--d:2"
        );
        assert_eq!(
            row_style(StaggerIndex::new(40), Presence::Present),
            "--i:12"
        );
    }

    #[test]
    fn only_a_name_longer_than_its_column_fades() {
        const CASES: &[(&str, NameFit)] = &[
            ("", NameFit::Fitting),
            ("Dana Okafor", NameFit::Fitting),
            // Exactly the budget: 26 characters.
            ("Abcdefghij Klmnopqrs Tuvwx", NameFit::Fitting),
            ("Abcdefghij Klmnopqrs Tuvwxy", NameFit::Overflowing),
            (
                "Maximilian Alexander von Hohenberg-Wittelsbach",
                NameFit::Overflowing,
            ),
            // Characters, not bytes: 21 characters in 24 bytes.
            ("Léa Martin-Ørsted-Åsa", NameFit::Fitting),
        ];
        for (name, want) in CASES {
            assert_eq!(NameFit::of(name, NAME_BUDGET), *want, "{name:?}");
        }
        assert_eq!(NameFit::Fitting.class(), "ds-row-name");
        assert_eq!(NameFit::Overflowing.class(), "ds-row-name ds-truncate");
    }

    #[test]
    fn only_a_leaving_row_names_its_exit() {
        const CASES: &[(Presence, Option<&str>)] = &[
            (Presence::Leaving(Exit::Fold), Some("fold")),
            (Presence::Leaving(Exit::Curl), Some("curl")),
            (Presence::Leaving(Exit::Crumple), Some("crumple")),
            (Presence::Leaving(Exit::TabOut), Some("tab-out")),
            (Presence::Entering, None),
            (Presence::Present, None),
        ];
        for (presence, want) in CASES {
            assert_eq!(exit(*presence), *want, "{presence:?}");
        }
    }
}
