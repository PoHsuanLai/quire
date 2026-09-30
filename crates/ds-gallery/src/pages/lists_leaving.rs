//! Lists: the notification center's column as a `LeavingList`, in a Popover root.
//! Each group is its `GroupHeader` over its cards. A card's close dismisses it (a batch of one);
//! a header's Clear drops the whole group in one render, so its rows fold one after another and
//! the groups below heal by the group's summed height; folding a group hides all but its newest
//! card, a batch the rows below heal over the same way; Clear all folds every row, staggered;
//! Post adds a card that enters with `row-in`.

use super::Section;
use crate::axes::{Axes, Showcase};
use dioxus::prelude::*;
use ds::Bezel;
use ds::{
    Appearance, Button, Ds, Icon, IconSource, Inject, LeavingItem, LeavingList, Material, Shown,
};
use ds_shell::{AppMark, GroupHeader, NotificationCard};

/// An app's group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum App {
    Mail,
    Calendar,
    Messages,
}

impl App {
    fn mark(self) -> AppMark {
        let (icon, name) = match self {
            App::Mail => (Icon::Mail, "Mail"),
            App::Calendar => (Icon::Clock, "Calendar"),
            App::Messages => (Icon::Phone, "Messages"),
        };
        AppMark {
            icon: IconSource::Glyph(icon),
            name: name.into(),
        }
    }
}

/// One notification: its id, app, sender and text.
#[derive(Debug, Clone, PartialEq)]
struct Note {
    id: u32,
    app: App,
    summary: &'static str,
    body: &'static str,
}

/// A row of the column: a group's header or one of its cards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Row {
    Head(App),
    Card(u32),
}

const START: [(App, &str, &str); 6] = [
    (
        App::Mail,
        "Grace Hopper",
        "Are we still on for Thursday? I booked the room by the harbour.",
    ),
    (
        App::Mail,
        "Ada Lovelace",
        "The notes are longer than the memoir.",
    ),
    (
        App::Mail,
        "Alan Kay",
        "The best way to predict the future is to ship it.",
    ),
    (App::Calendar, "Standup in 10 minutes", "Room 4 and online."),
    (
        App::Messages,
        "Dana",
        "Running five late, order me the usual.",
    ),
    (App::Messages, "Sam", "Photos from Saturday are up."),
];

const POSTED: [(App, &str, &str); 3] = [
    (App::Messages, "Dana", "Here now, by the window."),
    (App::Mail, "Linus", "Patch looks good, one nit inline."),
    (
        App::Calendar,
        "Lunch with Sam",
        "12:30 at the corner place.",
    ),
];

/// The live column's section.
#[component]
pub fn LeavingColumn() -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (appearance, showcase) = {
        let axes = axes.read();
        (
            Appearance {
                accent: axes.accent,
                motion: axes.motion,
                ..Appearance::default()
            },
            axes.showcase,
        )
    };
    let mut notes = use_signal(|| {
        START
            .iter()
            .zip(0u32..)
            .map(|(&(app, summary, body), id)| Note {
                id,
                app,
                summary,
                body,
            })
            .collect::<Vec<_>>()
    });
    let mut folded = use_signal(Vec::<App>::new);
    let mut posted = use_signal(|| 0usize);
    let items = rows(&notes(), &folded())
        .into_iter()
        .map(|row| LeavingItem {
            key: row,
            row: draw(row, &notes(), &folded(), notes, folded),
        })
        .collect::<Vec<_>>();
    rsx! {
        Section { title: "LeavingList",
            note: "The notification center's column: GroupHeaders over NotificationCards in a Popover root, 360 wide. A card's close dismisses it: it folds (--t-big --e-exit) and the rows below heal by the height it measured. A header's Clear drops its group in one render: the rows fold one after another (--i x --stagger, capped at 12) and are dropped together once the last has settled, and the groups below heal by the group's summed height. Folding a group (N more) hides all but its newest card the same way. Clear all folds every row. Post adds a card that enters with row-in (--t-big --e-spring). Under Reduced a row fades out and in and the rows below take their places without sliding.",
            if showcase == Showcase::Live {
                div { class: "g-row",
                    Button { label: "Post", icon: Some(Icon::Plus),
                        onclick: move |_| {
                            let n = posted();
                            posted.set(n + 1);
                            let (app, summary, body) = POSTED[n % POSTED.len()];
                            let id = 100 + u32::try_from(n).unwrap_or(0);
                            notes.with_mut(|notes| notes.insert(0, Note { id, app, summary, body }));
                        },
                    }
                    Button { label: "Clear all", onclick: move |_| notes.set(Vec::new()) }
                    Button { bezel: Bezel::Inline, label: "Reset",
                        onclick: move |_| {
                            folded.set(Vec::new());
                            notes.set(START.iter().zip(0u32..).map(|(&(app, summary, body), id)| Note { id, app, summary, body }).collect());
                        },
                    }
                }
            }
            div { class: "g-leave",
                Ds { appearance, material: Material::Popover, stylesheet: Inject::Host,
                    LeavingList::<Row> { label: "Notifications", items }
                }
            }
        }
    }
}

/// The apps in the order their newest note arrived.
fn apps(notes: &[Note]) -> Vec<App> {
    notes.iter().fold(Vec::new(), |mut seen, note| {
        if !seen.contains(&note.app) {
            seen.push(note.app);
        }
        seen
    })
}

/// The column's rows: each group's header, then its cards (only the newest while folded).
fn rows(notes: &[Note], folded: &[App]) -> Vec<Row> {
    apps(notes)
        .into_iter()
        .flat_map(|app| {
            let cards = notes.iter().filter(move |note| note.app == app);
            let shown = match folded.contains(&app) {
                true => 1,
                false => usize::MAX,
            };
            std::iter::once(Row::Head(app)).chain(cards.take(shown).map(|note| Row::Card(note.id)))
        })
        .collect()
}

/// One row's content.
fn draw(
    row: Row,
    notes: &[Note],
    folded: &[App],
    mut all: Signal<Vec<Note>>,
    mut fold: Signal<Vec<App>>,
) -> Element {
    match row {
        Row::Head(app) => {
            let count = notes.iter().filter(|note| note.app == app).count();
            let expanded = match folded.contains(&app) {
                true => Shown::Hidden,
                false => Shown::Visible,
            };
            rsx! {
                div { class: "g-leave-row",
                    GroupHeader {
                        icon: app.mark().icon,
                        name: app.mark().name,
                        count: u32::try_from(count).unwrap_or(0),
                        expanded,
                        on_toggle: move |_| fold.with_mut(|fold| match fold.contains(&app) {
                            true => fold.retain(|held| *held != app),
                            false => fold.push(app),
                        }),
                        on_clear: move |_| all.with_mut(|notes| notes.retain(|note| note.app != app)),
                    }
                }
            }
        }
        Row::Card(id) => {
            let Some(note) = notes.iter().find(|note| note.id == id).cloned() else {
                return rsx! {};
            };
            rsx! {
                div { class: "g-leave-row",
                    NotificationCard {
                        app: note.app.mark(),
                        age: "now",
                        summary: note.summary,
                        body: note.body,
                        material: Material::Popover,
                        on_close: move |_| all.with_mut(|notes| notes.retain(|held| held.id != id)),
                        on_open: |_| {},
                    }
                }
            }
        }
    }
}
