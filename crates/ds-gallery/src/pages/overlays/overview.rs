//! Overlays: the palette, popovers under each dismiss policy, peek and sheet.
//! The pills, the toast and the hover cards are in `pills.rs`.

use crate::axes::{Axes, Showcase};
use crate::pages::overlays::pills::{Cards, Pills};
use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::components::app::peek::Peek;
use ds::components::controls::button_model::Answers;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::components::overlays::popover::Arrow;
use ds::host::measure::{Anchor, MountedRef};
use ds::prelude::*;
use ds::stack::toast_hub::UndoToken;
use ds::stack::toast_hub::use_toast_hub;
use ds_core::geometry::placement::Align;
use ds_core::geometry::placement::Side;
use ds_core::vocab::Dismiss;
use ds_style::appearance::peek::PeekMode;
use ds_style::tokens::control_size::ControlSize;

/// Everything the page can open, one at a time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Opened {
    Palette,
    Popover(Dismiss, Arrow),
    Peek(PeekMode),
    Sheet,
}

const POPOVERS: [(Dismiss, Arrow, &str); 3] = [
    (
        Dismiss::Transient,
        Arrow::Arrow,
        "Popover: transient, arrow",
    ),
    (
        Dismiss::Semitransient,
        Arrow::None,
        "Popover: semitransient",
    ),
    (Dismiss::Manual, Arrow::None, "Popover: manual"),
];

/// Where a posed snapshot opens its menu: over the page, clear of the toolbar.
const POSED_AT: Point = Point {
    x: Px(40.0),
    y: Px(330.0),
};

/// The overlays page.
#[component]
pub fn OverlaysPage() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let mut opened = use_signal(|| match showcase {
        Showcase::Posed => None,
        Showcase::Live => None,
    });
    let mut anchor = use_signal(|| None::<MountedRef>);
    let toasts = use_toast_hub();
    use_hook(move || {
        if showcase == Showcase::Posed {
            toasts.push("Archived “Invoice #2291”".to_string(), Some(UndoToken(1)));
        }
    });
    let at = match (showcase, anchor()) {
        (Showcase::Live, Some(mounted)) => Anchor::Mounted(mounted),
        _ => Anchor::Point(POSED_AT),
    };
    let close = move |_| opened.set(None);
    let button = move |what: Opened, label: &'static str| {
        rsx! {
            Button {
                label,
                value: Some(if opened() == Some(what) { Check::On } else { Check::Off }),
                onclick: move |_| opened.set(Some(what)),
            }
        }
    };
    rsx! {
        Section { title: "Palette, popovers, peek, sheet",
            div {
                class: "g-row",
                onmounted: move |event| anchor.set(Some(MountedRef(event.data()))),
                {button(Opened::Palette, "Command palette")}
                for (dismiss , arrow , label) in POPOVERS {
                    {button(Opened::Popover(dismiss, arrow), label)}
                }
                {button(Opened::Peek(PeekMode::Center), "Peek: center")}
                {button(Opened::Peek(PeekMode::Full), "Peek: full")}
                {button(Opened::Sheet, "Sheet")}
            }
        }
        Cards {}
        Pills { showcase }
        crate::pages::overlays::launcher::EmbeddedPalette {}
        crate::pages::overlays::launcher_hints::SpotlightHints {}
        crate::pages::overlays::palette_and_menu::RecentPalette {}
        crate::pages::overlays::hover_card_hooks::HookKeyedCards {}
        crate::pages::shell::control_center::ControlCenter {}
        crate::pages::overlays::sheet::PowerMenu {}
        crate::pages::overlays::alert::Alerts {}
        crate::pages::overlays::notifications::Notifications {}
        crate::pages::shell::calendar::Calendar {}
        crate::pages::shell::widgets::Widgets {}
        crate::pages::overlays::screenshot_thumb::ShotThumbnails {}
        match opened() {
            Some(Opened::Palette) => rsx! { Palette { onclose: close } },
            Some(Opened::Popover(dismiss, arrow)) => rsx! {
                Popover {
                    key: "{dismiss:?}",
                    anchor: at.clone(),
                    placement: Placement::new(Side::Bottom, Align::Start),
                    gap: Px(8.0),
                    arrow,
                    dismiss,
                    onclose: close,
                    div { class: "g-panel",
                        Specimen { name: format!("{dismiss:?} dismiss"), code: "Transient: Escape or an outside click closes; Semitransient: Escape only; Manual: its owner".to_string(),
                            Button { size: ControlSize::Mini, label: "Close", onclick: move |_| opened.set(None) }
                        }
                    }
                }
            },
            Some(Opened::Peek(mode)) => rsx! {
                Peek { key: "{mode:?}", mode, label: "Thread peek", onclose: close,
                    div { class: "g-panel",
                        h2 { "Re: UIDL stability across servers" }
                        p { class: "g-note", "The reader in a panel over the card. The scrim, the close tool or Escape closes it." }
                    }
                }
            },
            Some(Opened::Sheet) => rsx! {
                Sheet { label: "Settings", onclose: close,
                    div { class: "g-panel",
                        h2 { "A sheet" }
                        p { class: "g-note", "Hangs from the top edge and dims nothing; Escape closes it." }
                        Button { answers: Answers::Return, label: "Done", onclick: move |_| opened.set(None) }
                    }
                }
            },
            None => rsx! {},
        }
    }
}

/// The palette, holding its own query.
#[component]
fn Palette(onclose: EventHandler<()>) -> Element {
    let mut query = use_signal(String::new);
    let item = |value: u8, title: &str, icon: Icon, keys: Vec<ShortcutKey>| PaletteRow {
        leading: RowLeading::Icon(icon),
        accessory: Accessory::Text(Shortcut(keys).glyphs()),
        availability: Availability::Enabled,
        ..PaletteRow::new(value, title)
    };
    let actions = vec![
        item(
            0,
            "Compose a message",
            Icon::Pen,
            vec![ShortcutKey::Char('c')],
        ),
        item(1, "Archive", Icon::Archive, vec![ShortcutKey::Char('e')]),
        item(
            2,
            "Snooze until tomorrow",
            Icon::Clock,
            vec![ShortcutKey::Char('h')],
        ),
    ];
    let places = vec![
        item(
            3,
            "Go to Inbox",
            Icon::Inbox,
            vec![ShortcutKey::Char('g'), ShortcutKey::Char('i')],
        ),
        item(
            4,
            "Go to Starred",
            Icon::Star,
            vec![ShortcutKey::Char('g'), ShortcutKey::Char('s')],
        ),
    ];
    let typed = query();
    let keep = |rows: Vec<PaletteRow<u8>>| -> Vec<PaletteRow<u8>> {
        rows.into_iter()
            .filter(|row| {
                row.title
                    .plain_text()
                    .to_lowercase()
                    .contains(&typed.to_lowercase())
            })
            .collect()
    };
    rsx! {
        CommandPalette::<u8> {
            label: "Command palette",
            placeholder: "Type a command",
            query: typed.clone(),
            tokens: Vec::new(),
            groups: vec![PaletteGroup::list("Actions", keep(actions)), PaletteGroup::list("Places", keep(places))],
            empty: "Nothing matches.",
            oninput: move |next| query.set(next),
            onpick: move |_| {},
            onclose,
        }
    }
}
