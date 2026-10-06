//! ThreadRow: a mail row's content in a `Row` (design/30 section 2.11, mail content inside `Row`):
//! a dot, the sender's name and via, subject, snippet, time, tags, star, a trailing `more`
//! action and a hover-strip slot.

use crate::components::app::thread_row_hooks::{PartHooks, use_back};
use crate::components::app::thread_row_star::star_button;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::lists::row::row::{Row, relay};
use crate::components::lists::row::size::RowSize;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::text::clip::clip_chars;
use ds_core::vocab::{Check, RowState};

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
            NameFit::Fitting => "ds-thread-name",
            NameFit::Overflowing => "ds-thread-name ds-truncate",
        }
    }
}

/// One mail row: a [`Row`] whose `content` is the thread's own layout.
///
/// `state` is the row's [`RowState`]: whether it is selected, unread, working or disabled, and
/// its part in a drag (`drop`: `Source` while it is the thread being dragged (dimmed), `Target`
/// while something dragged over it would land on it). Its entrance, exit and heal are the
/// enclosing `List`'s. `onclick` receives the press, so the caller can read Shift to peek.
///
/// `more` is one trailing action (a [`RowMore`](crate::components::app::row_more::RowMore)) laid
/// out in flow beside the tags in the tail, under the time: it can never cover the time or the
/// text. `strip` is the older raised pill, absolutely placed over the row's right edge; the two
/// are independent. The row's height is [`thread_card_height`](crate::components::app::thread_height::thread_card_height), whichever slots it carries.
///
/// `subject` and `snippet` are [`TextLine`]: a string, or the runs a search hit marked.
/// `on_sender` and `on_time` hear the pointer entering and leaving the name and the time (their
/// own hover cards); `onpointerenter`, `onpointerleave` and `onpointerdown` hear the row itself
/// (the thread card, a drag's start). `onpointerback` hears the pointer leave the name or the
/// time and rest on the row for the card's close grace (`CardClose`): where a row reopens its own
/// card, which the parts' own leave cannot tell, since the pointer may have left for a card
/// floating over the rows (FINDINGS "A part's leave is not the row's enter").
#[component]
pub fn ThreadRow(
    #[props(default)] state: RowState,
    name: String,
    via: Option<Element>,
    #[props(into)] subject: TextLine,
    snippet: Option<TextLine>,
    time: String,
    tags: Element,
    star: Option<(Check, EventHandler<Check>)>,
    strip: Option<Element>,
    more: Option<Element>,
    onclick: EventHandler<Press>,
    #[props(default)] on_sender: Option<PartHooks>,
    #[props(default)] on_time: Option<PartHooks>,
    #[props(default)] onpointerenter: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerleave: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerdown: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerback: Option<EventHandler<PointerEvent>>,
    #[props(default)] common: Common,
) -> Element {
    let back = use_back(onpointerback);
    let content = rsx! {
        div { class: "ds-thread",
            div { class: "ds-thread-dot",
                span { class: "ds-dot" }
            }
            div { class: "ds-thread-main",
                div { class: "ds-thread-from",
                    span {
                        class: NameFit::of(&name, NAME_BUDGET).class(),
                        onpointerenter: back.part_enter(on_sender),
                        onpointerleave: back.part_leave(on_sender),
                        "{name}"
                    }
                    if let Some(via) = via {
                        span { class: "ds-thread-via", {via} }
                    }
                }
                div { class: "ds-thread-sub ds-truncate", {text(&subject)} }
                if let Some(snippet) = snippet {
                    div { class: "ds-thread-snip ds-truncate", {text(&snippet)} }
                }
            }
            div { class: "ds-thread-tail",
                span {
                    class: "ds-thread-time",
                    onpointerenter: back.part_enter(on_time),
                    onpointerleave: back.part_leave(on_time),
                    "{time}"
                }
                span { class: "ds-thread-tags",
                    {tags}
                    if let Some(more) = more {
                        {more}
                    }
                }
            }
            if let Some((state, onchange)) = star {
                {star_button(state, onchange)}
            }
            if let Some(strip) = strip {
                {strip}
            }
        }
    };
    rsx! {
        Row {
            title: TextLine::default(),
            content,
            state,
            size: RowSize::Card,
            onclick,
            onpointerenter: back.enter(onpointerenter),
            onpointerleave: back.leave(onpointerleave),
            onpointerdown: relay(onpointerdown),
            common,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NAME_BUDGET, NameFit};

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
        assert_eq!(NameFit::Fitting.class(), "ds-thread-name");
        assert_eq!(NameFit::Overflowing.class(), "ds-thread-name ds-truncate");
    }
}
