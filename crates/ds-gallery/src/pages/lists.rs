//! Lists: a live `AnimatedList` whose rows leave by each exit, heal, and come back on undo;
//! sidebar items; account tiles; the hover strip; the appearance picker.

use super::scheduled::Scheduled;
use super::{Section, Specimen};
use crate::axes::Axes;
use dioxus::prelude::*;
use ds::{
    AccountFace, AccountTile, ActionId, Anim, AnimatedList, AppearancePicker, AvatarFace,
    AvatarShape, AvatarSize, AvatarTone, Button, ButtonVariant, Chip, ChipVariant, Colour,
    DragGhost, DropLine, Emphasis, Exit, Here, Hex, HoverStrip, Icon, ItemKind, ListPresence,
    ListRow, MarkSize, MarkStyle, PersonHue, Point, Presence, Preview, Provider, ProviderMark, Px,
    RowPitch, Selection, SidebarItem, StaggerIndex, StripAction, Switch, SystemPrefs, TimerPhase,
    UndoToken, use_motion_timer, use_pulse, use_roster, use_toast_hub,
};

/// One sample thread: sender, subject, snippet, time.
type Thread = (&'static str, &'static str, &'static str, &'static str);

const THREADS: [Thread; 8] = [
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
    fn thread(self) -> Thread {
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

/// A row's height plus the gap below it (design/04-COMPONENTS.md section 16).
const PITCH: RowPitch = RowPitch(Px(79.0));

/// The lists page.
#[component]
pub fn ListsPage() -> Element {
    rsx! {
        LiveList {}
        super::lists_search::SearchRows {}
        Sidebar {}
        Tiles {}
        Picker {}
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
    let star_pulse = use_pulse(Anim::StarPop);
    let toasts = use_toast_hub();
    let roster = use_roster(keys(), PITCH);
    let entrance = use_motion_timer(Anim::RowIn);
    use_hook(|| entrance.start(EventHandler::new(|()| {})));
    let list = match entrance.phase() {
        TimerPhase::Settled => ListPresence::Present,
        TimerPhase::Idle | TimerPhase::Running => ListPresence::Entering,
    };
    let mut remove = move |exit: Exit, text: &str| {
        let target = selected().or_else(|| keys.peek().first().copied());
        let Some(key) = target else { return };
        let Some(at) = keys.peek().iter().position(|shown| *shown == key) else {
            return;
        };
        roster.leave(key, exit, key.emphasis());
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
    rsx! {
        Section {
            title: "AnimatedList",
            note: "Remove the selected row (or the first) by each exit: unread rows exit heavier. The rows below heal into the gap; Undo, or pull the toast's tab, brings the row back.",
            div { class: "g-row",
                Button { variant: ButtonVariant::Secondary, label: "Add a row", icon: Some(Icon::Plus),
                    onclick: move |_| {
                        let id = ThreadId(next());
                        next += 1;
                        keys.with_mut(|keys| keys.insert(0, id));
                    },
                }
                Button { variant: ButtonVariant::Secondary, label: "Archive (fold)", icon: Some(Icon::Archive), onclick: move |_| remove(Exit::Fold, "Archived") }
                Button { variant: ButtonVariant::Secondary, label: "Snooze (curl)", icon: Some(Icon::Clock), onclick: move |_| remove(Exit::Curl, "Snoozed") }
                Button { variant: ButtonVariant::Danger, label: "Trash (crumple)", icon: Some(Icon::Trash), onclick: move |_| remove(Exit::Crumple, "Trashed") }
                Button { variant: ButtonVariant::Quiet, label: "Undo", icon: Some(Icon::Undo), onclick: move |_| restore(None) }
            }
            div { class: "g-list",
                AnimatedList { label: "Threads", presence: list,
                    for entry in roster.entries() {
                        ThreadRow {
                            key: "{entry.key.0}",
                            id: entry.key,
                            presence: entry.presence,
                            index: entry.index,
                            selection: if selected() == Some(entry.key) { Selection::Selected } else { Selection::Unselected },
                            star: if starred().contains(&entry.key) { Switch::On } else { Switch::Off },
                            star_pulse: star_pulse.key(),
                            onselect: move |id| selected.set(Some(id)),
                            onstar: move |(id, state)| {
                                starred.with_mut(|starred| {
                                    starred.retain(|seen| *seen != id);
                                    if state == Switch::On {
                                        starred.push(id);
                                    }
                                });
                                star_pulse.fire();
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ThreadRow(
    id: ThreadId,
    presence: Presence,
    index: StaggerIndex,
    selection: Selection,
    star: Switch,
    star_pulse: ds::PulseKey,
    onselect: EventHandler<ThreadId>,
    onstar: EventHandler<(ThreadId, Switch)>,
) -> Element {
    let (name, subject, snippet, time) = id.thread();
    rsx! {
        ListRow {
            selection,
            emphasis: id.emphasis(),
            index,
            presence,
            name,
            via: rsx! {
                ProviderMark { provider: Provider::Google, size: MarkSize::Row, style: MarkStyle::Letter }
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
            star_pulse,
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

/// The places a sidebar offers.
const PLACES: [(Icon, &str, Option<u32>); 4] = [
    (Icon::Inbox, "Inbox", Some(12)),
    (Icon::Star, "Starred", None),
    (Icon::Clock, "Snoozed", Some(2)),
    (Icon::Send, "Sent", None),
];

#[component]
fn Sidebar() -> Element {
    let mut here = use_signal(|| 0usize);
    let seal = use_pulse(Anim::SealPop);
    let gulp = use_pulse(Anim::Gulp);
    let mut today = use_signal(|| vec!["Dana Okafor", "Priya Raman"]);
    let person = |name: &str| AvatarFace {
        initial: name.chars().next().unwrap_or('?'),
        size: AvatarSize::Size18,
        tone: AvatarTone::Person(PersonHue::of(name)),
        shape: AvatarShape::Round,
    };
    rsx! {
        Section {
            title: "SidebarItem",
            note: "On the Space's frame colour. Click a place to move the seal; drop on Snoozed plays the gulp; Today items close (each close is named for its row); the scheduled draft shows its time and cancels.",
            div { class: "g-row g-row-top",
                div { class: "g-side",
                    for (index , (icon , label , count)) in PLACES.into_iter().enumerate() {
                        SidebarItem {
                            key: "{label}",
                            kind: ItemKind::Place { icon },
                            label,
                            here: if here() == index { Here::Current } else { Here::Elsewhere },
                            count,
                            presence: Presence::Present,
                            preview: None,
                            pulse: if here() == index { seal.key() } else if index == 2 { gulp.key() } else { ds::PulseKey::rest(Anim::SealPop) },
                            onclick: move |_| {
                                here.set(index);
                                seal.fire();
                            },
                            onclose: None,
                        }
                    }
                    SidebarItem {
                        kind: ItemKind::Pinned { avatar: person("Mei Chen") },
                        label: "Mei Chen",
                        here: Here::Elsewhere,
                        count: Some(1),
                        presence: Presence::Present,
                        preview: Some(Preview::Destination),
                        pulse: ds::PulseKey::rest(Anim::Gulp),
                        onclick: |_| {},
                        onclose: None,
                    }
                    for name in today() {
                        SidebarItem {
                            key: "{name}",
                            kind: ItemKind::Today { avatar: person(name) },
                            label: name,
                            here: Here::Elsewhere,
                            count: None,
                            presence: Presence::Present,
                            preview: None,
                            pulse: ds::PulseKey::rest(Anim::Gulp),
                            onclick: |_| {},
                            onclose: Some(EventHandler::new(move |()| today.with_mut(|today| today.retain(|seen| *seen != name)))),
                        }
                    }
                    Scheduled { key: "{today().len()}" }
                }
                div { class: "g-col",
                    Button { variant: ButtonVariant::Mini, label: "Drop on Snoozed (gulp)", onclick: move |_| gulp.fire() }
                    Button { variant: ButtonVariant::Mini, label: "Bring Today back", onclick: move |_| today.set(vec!["Dana Okafor", "Priya Raman"]) }
                    Specimen { name: "DropLine",
                        div { class: "g-list g-stage-pad", DropLine {} }
                    }
                }
            }
        }
    }
}

#[component]
fn Tiles() -> Element {
    let mut ghost = use_signal(|| Switch::Off);
    let colour = Colour::Solid(Hex([0x2a, 0x5d, 0xb0]));
    rsx! {
        Section { title: "Tiles, strip and drag ghost",
            div { class: "g-row",
                AccountTile { account: AccountFace::All, pressed: Switch::On, unread: 7, onclick: |_| {} }
                AccountTile { account: AccountFace::One { initial: 'F', colour, provider: Provider::Fastmail, address: None }, pressed: Switch::On, unread: 0, onclick: |_| {} }
                AccountTile { account: AccountFace::One { initial: 'F', colour, provider: Provider::Fastmail, address: None }, pressed: Switch::Off, unread: 4, onclick: |_| {} }
            }
            p { class: "g-note", "The hover strip shows on row hover in the list above. The drag ghost is fixed to the window: it follows the pointer while a row is dragged; here it is pinned near the top right." }
            div { class: "g-row",
                Button {
                    variant: ButtonVariant::Mini,
                    label: "Show the drag ghost",
                    pressed: Some(ghost()),
                    onclick: move |_| ghost.set(match ghost() { Switch::On => Switch::Off, Switch::Off => Switch::On }),
                }
            }
            if ghost() == Switch::On {
                DragGhost { title: "Re: UIDL stability across servers", sub: "Dana Okafor · 09:41", at: Point { x: Px(760.0), y: Px(140.0) } }
            }
        }
    }
}

#[component]
fn Picker() -> Element {
    let mut axes = use_context::<Signal<Axes>>();
    let value = axes().appearance();
    rsx! {
        Section { title: "AppearancePicker", note: "Bound to the gallery's own theme, accent and motion: it drives the toolbar.",
            div { class: "g-list g-stage-pad",
                AppearancePicker {
                    value,
                    system: SystemPrefs::default(),
                    onchange: move |next: ds::Appearance| {
                        axes.with_mut(|axes| {
                            axes.theme = next.theme;
                            axes.accent = next.accent;
                            axes.motion = next.motion;
                        });
                    },
                }
            }
        }
    }
}
