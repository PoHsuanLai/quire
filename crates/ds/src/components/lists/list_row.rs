//! ListRow: one item in a list, the thread row (design/04-COMPONENTS.md section 16).

use crate::components::content::text_runs::{TextLine, text};
use crate::components::lists::row_click::snapshot;
use crate::components::lists::row_hooks::{PartHooks, relay, use_back};
use crate::components::lists::row_star::star_button;
use crate::core::text::clip::clip_chars;
use crate::core::vocab::{Availability, Check, Emphasis, RowState};
use crate::core::word::Word;
use crate::motion::presence::Presence;
use crate::motion::roster::{Heal, presence_slug};
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

/// The row's inline custom property while healing: the distance `--dy`.
fn row_style(heal: Option<Heal>) -> Option<String> {
    heal.map(|Heal { dy }| format!("--dy:{}px", dy.0))
}

/// `data-exit`, on a leaving row only.
fn exit(presence: Presence) -> Option<&'static str> {
    match presence {
        Presence::Leaving(exit) => Some(exit.slug()),
        Presence::Hidden | Presence::Entering | Presence::Present => None,
    }
}

/// One row: dot, name and via, subject, snippet, tail, star, and a hover-strip slot.
///
/// `presence` comes from `use_roster`: an entering row plays `row-in`; a leaving row plays
/// `row-out`; a healing row (`heal`, present meanwhile) slides up from `dy`. `onclick` receives the pointer's data, so the consumer can read Shift to peek.
/// `state` is the row's [`RowState`]: whether it is selected, unread, working or disabled, and
/// its part in a drag (`drop`: `Source` while it is the thread being dragged (dimmed), `Target`
/// while something dragged over it would land on it).
///
/// `subject` and `snippet` are [`Text`]: a string as before, or the runs a search hit marked.
/// `on_sender` and `on_time` hear the pointer entering and leaving the name and the time (their
/// own hover cards); `onpointerenter`, `onpointerleave` and `onpointerdown` hear the row itself
/// (the thread card, a drag's start). `onpointerback` hears the pointer leave the name or the
/// time and rest on the row for the card's close grace (`HoverClose`): where a row reopens its own card, which the parts' own leave
/// cannot tell, since the pointer may have left for a card floating over the rows (FINDINGS
/// "A part's leave is not the row's enter"). `aria_label` names the row for a screen reader ("Open
/// Re: UIDL stability"); absent, the row is named by its contents as before.
#[component]
pub fn ListRow(
    #[props(default)] state: RowState,
    presence: Presence,
    #[props(default)] heal: Option<Heal>,
    name: String,
    via: Option<Element>,
    #[props(into)] subject: TextLine,
    snippet: Option<TextLine>,
    time: String,
    tags: Element,
    star: Option<(Check, EventHandler<Check>)>,
    strip: Option<Element>,
    onclick: EventHandler<MouseData>,
    #[props(default)] on_sender: Option<PartHooks>,
    #[props(default)] on_time: Option<PartHooks>,
    #[props(default)] onpointerenter: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerleave: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerdown: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerback: Option<EventHandler<PointerEvent>>,
    #[props(default)] aria_label: Option<String>,
) -> Element {
    let back = use_back(onpointerback);
    let RowState {
        selection,
        emphasis,
        availability,
        drop,
    } = state;
    let live = availability == Availability::Enabled;
    rsx! {
        li {
            class: "ds-row",
            role: "option",
            "aria-selected": selection.aria(),
            "aria-label": aria_label,
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            "data-emphasis": emphasis_slug(emphasis),
            "data-presence": presence_slug(presence, heal),
            "data-exit": exit(presence),
            "data-drop": drop.drop_attr(),
            "data-drag": drop.drag_attr(),
            style: row_style(heal),
            onclick: move |event| {
                if live {
                    onclick.call(snapshot(&event.data()));
                }
            },
            onpointerenter: back.enter(onpointerenter),
            onpointerleave: back.leave(onpointerleave),
            onpointerdown: relay(onpointerdown),
            div { class: "ds-row-dot",
                span { class: "ds-dot" }
            }
            div { class: "ds-row-main",
                div { class: "ds-row-from",
                    span {
                        class: NameFit::of(&name, NAME_BUDGET).class(),
                        onpointerenter: back.part_enter(on_sender),
                        onpointerleave: back.part_leave(on_sender),
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
                    onpointerenter: back.part_enter(on_time),
                    onpointerleave: back.part_leave(on_time),
                    "{time}"
                }
                span { class: "ds-row-tags", {tags} }
            }
            if let Some((state, onchange)) = star {
                {star_button(state, onchange)}
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
    use crate::core::geometry::units::Px;
    use crate::motion::presence::{Exit, Presence};
    use crate::motion::roster::Heal;

    #[test]
    fn a_healing_row_carries_its_distance() {
        let healing = Heal { dy: Px(79.0) };
        assert_eq!(row_style(Some(healing)).as_deref(), Some("--dy:79px"));
        assert_eq!(row_style(None), None);
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
            (Presence::Leaving(Exit::Row), Some("row")),
            (Presence::Present, None),
            (Presence::Entering, None),
        ];
        for &(presence, want) in CASES {
            assert_eq!(exit(presence), want, "{presence:?}");
        }
    }
}
