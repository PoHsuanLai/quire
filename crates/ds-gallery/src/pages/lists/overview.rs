//! Lists: a live `List` of mail rows whose rows leave by the roster, heal and come back on undo;
//! the mail rows a search draws; `Row` in every state; a source list; the notification column;
//! pinned tiles; the hover strip.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::app::hover_strip::{ActionId, HoverStrip, StripAction};
use ds::components::app::pin_tile::{PinFace, PinTile};
use ds::components::app::thread_row::ThreadRow;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};
use ds::components::controls::button_model::Bezel;
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::prelude::*;
use ds::stack::toast_hub::{UndoToken, use_toast_hub};
use ds::style::tokens::control_size::ControlSize;
use ds::style::tokens::hex::{Colour, Hex};
use ds_core::vocab::RowState;

/// One sample thread: sender, subject, snippet, time.
type ThreadSample = (&'static str, &'static str, &'static str, &'static str);

const THREADS: [ThreadSample; 8] = [
    (
        "Dana Okafor",
        "Re: UIDL stability across servers",
        "Treat UIDL as stable only while UIDVALIDITY holds.",
        "09:41",
    ),
    (
        "Sam Lindqvist",
        "Notes from the sync review",
        "Three things to change before the next wave.",
        "09:12",
    ),
    (
        "Priya Raman",
        "Invoice #2291 for September",
        "Attached, as discussed on the call.",
        "Tue",
    ),
    (
        "Léa Martin",
        "Lunch on Thursday?",
        "The place by the river opens again this week.",
        "Mon",
    ),
    (
        "GitHub",
        "[quire] Wave 2 merged",
        "Seven branches, forty-one files changed.",
        "Sun",
    ),
    (
        "Tomás Ferreira",
        "Photos from the trip",
        "Twelve of them, the good ones anyway.",
        "Sat",
    ),
    (
        "Mei Chen",
        "Draft: the adoption guide",
        "Could you read section three before Friday?",
        "Fri",
    ),
    (
        "Hannah Weiss",
        "Your parcel is on its way",
        "Delivery expected tomorrow before noon.",
        "Thu",
    ),
];

/// A thread by the list's own key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ThreadId(u32);

impl ThreadId {
    fn thread(self) -> ThreadSample {
        THREADS[self.0 as usize % THREADS.len()]
    }

    /// Every other thread is unread, so both exit weights can be watched.
    fn emphasis(self) -> Emphasis {
        if self.0.is_multiple_of(2) {
            Emphasis::Strong
        } else {
            Emphasis::Plain
        }
    }
}

/// The lists page.
#[component]
pub fn ListsPage() -> Element {
    rsx! {
        LiveList {}
        Grouped {}
        crate::pages::lists::rows::RowGallery {}
        crate::pages::lists::headers::HeaderGallery {}
        crate::pages::lists::virtual_rows::VirtualGallery {}
        crate::pages::lists::search::SearchRows {}
        crate::pages::lists::strip::StripPress {}
        crate::pages::lists::sidebar::Sidebar {}
        crate::pages::lists::leaving::LeavingColumn {}
        Tiles {}
    }
}

/// Removed rows, newest last, each with the place it left and the undo token its toast carries.
type Removed = Vec<(usize, ThreadId, UndoToken)>;

#[component]
fn LiveList() -> Element {
    let mut keys = use_signal(|| (0..5).map(ThreadId).collect::<Vec<_>>());
    let mut next = use_signal(|| 5u32);
    let mut removed = use_signal(Removed::new);
    let mut selected = use_signal(|| None::<ThreadId>);
    let mut starred = use_signal(Vec::<ThreadId>::new);
    let toasts = use_toast_hub();
    let mut remove = move |text: &str| {
        let target = selected().or_else(|| keys.peek().first().copied());
        let Some(key) = target else { return };
        let Some(at) = keys.peek().iter().position(|shown| *shown == key) else {
            return;
        };
        keys.with_mut(|keys| keys.retain(|shown| *shown != key));
        let token = UndoToken(u64::from(key.0));
        removed.with_mut(|removed| removed.push((at, key, token)));
        selected.set(None);
        toasts.push(format!("{text} “{}”", key.thread().1), Some(token));
    };
    let mut restore = move |token: Option<UndoToken>| {
        let found = removed.with_mut(|removed| {
            let at = match token {
                Some(token) => removed.iter().rposition(|(_, _, seen)| *seen == token)?,
                None => removed.len().checked_sub(1)?,
            };
            Some(removed.remove(at))
        });
        if let Some((at, key, _)) = found {
            keys.with_mut(|keys| keys.insert(at.min(keys.len()), key));
        }
    };
    let pulled = toasts.last_undo();
    let mut handled = use_signal(|| None::<UndoToken>);
    use_effect(move || {
        if pulled.is_some() && *handled.peek() != pulled {
            handled.set(pulled);
            restore(pulled);
        }
    });
    let items: Vec<ListItem<ThreadId>> = keys()
        .into_iter()
        .map(|id| {
            ListItem::row(
                id,
                id.thread().1,
                rsx! {
                    ThreadLine {
                        id,
                        selection: if selected() == Some(id) { Selection::Selected } else { Selection::Unselected },
                        star: if starred().contains(&id) { Check::On } else { Check::Off },
                        onselect: move |id| selected.set(Some(id)),
                        onstar: move |(id, state)| {
                            starred.with_mut(|starred| {
                                starred.retain(|seen| *seen != id);
                                if state == Check::On {
                                    starred.push(id);
                                }
                            });
                        },
                    }
                },
            )
        })
        .collect();
    rsx! {
        Section {
            title: "List: rows that come and go",
            note: "Remove the selected row (or the first) The row fades and slides up. The rows below close into the gap; Undo, or pull the toast's tab, brings the row back. Up and Down move the cursor and stop at the ends; letters jump to the next subject that starts with them.",
            div { class: "g-row",
                Button { label: "Add a row", icon: Some(Icon::Plus),
                    onclick: move |_| {
                        let id = ThreadId(next());
                        next += 1;
                        keys.with_mut(|keys| keys.insert(0, id));
                    },
                }
                Button { label: "Archive", icon: Some(Icon::Archive), onclick: move |_| remove("Archived") }
                Button { bezel: Bezel::Inline, label: "Undo", icon: Some(Icon::Undo), onclick: move |_| restore(None) }
            }
            div { class: "g-list",
                List::<ThreadId> {
                    label: "Threads",
                    items,
                    cursor: selected(),
                    onselect: move |id| selected.set(Some(id)),
                }
            }
        }
    }
}

#[component]
fn ThreadLine(
    id: ThreadId,
    selection: Selection,
    star: Check,
    onselect: EventHandler<ThreadId>,
    onstar: EventHandler<(ThreadId, Check)>,
) -> Element {
    let (name, subject, snippet, time) = id.thread();
    rsx! {
        ThreadRow {
            state: RowState { selection, emphasis: id.emphasis(), ..RowState::default() },
            name,
            via: rsx! {
                ProviderMark { provider: MarkProvider::Google, size: ControlSize::Mini, style: MarkStyle::Letter }
                "gmail"
            },
            subject,
            snippet: snippet.to_string(),
            time,
            tags: rsx! {
                if id.0.is_multiple_of(3) {
                    Chip { variant: ChipVariant::Accent, text: "spec" }
                }
            },
            star: (star, EventHandler::new(move |state| onstar.call((id, state)))),
            strip: rsx! { HoverStrip { actions: strip_actions() } },
            onclick: move |_| onselect.call(id),
        }
    }
}

/// The four strip actions every row offers.
fn strip_actions() -> Vec<StripAction> {
    [
        (
            "archive",
            Icon::Archive,
            "Archive",
            "Archive → out of Inbox",
        ),
        ("snooze", Icon::Clock, "Snooze", "Snooze until…"),
        ("label", Icon::Tag, "Label", "Label…"),
        ("read", Icon::MailOpen, "Mark read", "Mark read"),
    ]
    .into_iter()
    .map(|(id, icon, label, fly)| StripAction {
        id: ActionId(id.to_string()),
        icon,
        label: label.to_string(),
        fly: fly.to_string(),
        onhover: None,
        onclick: EventHandler::new(|_| {}),
    })
    .collect()
}

#[component]
fn Tiles() -> Element {
    let mut ghost = use_signal(|| Check::Off);
    let colour = Colour::Solid(Hex([0x2a, 0x5d, 0xb0]));
    rsx! {
        Section { title: "Tiles, strip and drag ghost",
            div { class: "g-row",
                PinTile { face: PinFace::All, selection: Selection::Selected, unread: 7, onclick: |_| {} }
                PinTile { face: PinFace::Account { initial: 'F', colour, provider: MarkProvider::Fastmail, address: None }, selection: Selection::Selected, unread: 0, onclick: |_| {} }
                PinTile { face: PinFace::Account { initial: 'F', colour, provider: MarkProvider::Fastmail, address: None }, unread: 4, onclick: |_| {} }
            }
            p { class: "g-note", "The hover strip shows on row hover in the list above. The drag ghost is fixed to the window: it follows the pointer while a row is dragged; here it is pinned near the top right." }
            div { class: "g-row",
                Button {
                    size: ControlSize::Mini,
                    label: "Show the drag ghost",
                    value: Some(ghost()),
                    onclick: move |_| ghost.set(ghost().flipped()),
                }
            }
            if ghost() == Check::On {
                DragGhost { title: "Re: UIDL stability across servers", sub: "Dana Okafor · 09:41", at: Point { x: Px(760.0), y: Px(140.0) } }
            }
        }
    }
}

/// The grouped list (design/34-MODERN-LOOK.md section 3.5): System Settings' inset groups.
#[component]
fn Grouped() -> Element {
    rsx! {
        Section {
            title: "List: grouped",
            note: "ListStyle::Grouped (the old Inset, drawn the same): one rounded group on the grouped ground, no outline; rows 44 high with a 24 px icon tile or a 32 px avatar, a chevron on rows that open something, a hairline from the row's text to the edge with none above the first row or below the last, and the quiet wash for the selection.",
            div { class: "g-stage-pad", style: "width:420px;background:var(--surface-2)",
                crate::pages::forms::grouped::Panes {}
            }
        }
    }
}
