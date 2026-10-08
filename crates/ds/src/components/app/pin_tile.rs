//! PinTile: one square tile of the pinned grid above a sidebar's list (design/30 section 2.11):
//! an account in this Space, the "All" tile, or the tile that adds one. One component for the
//! three faces; the account's unread count is a `Badge`, and a drag over it draws the list's drop
//! line at its leading edge. Grouping, reordering and the Add tile's place are `PinTiles`.
//!
//! An account's problem mark sits on its tile, as macOS Mail puts it beside the account: a
//! [`PinStatus`] in the top-left corner, the one corner the tile leaves free (the unread `Badge`
//! is top right, the provider's mark bottom right). `Attention` is the warning glyph, named by
//! its reason (a tooltip and the glyph's `aria-label`) and, with `onstatus`, a press of its own
//! that never also presses the tile or starts its drag; `Busy` is a Mini spinner that turns only
//! while its `Operation` runs (R4).

use crate::components::content::avatar::{Avatar, AvatarSize, AvatarTone};
use crate::components::content::provider_mark::{MarkProvider, MarkStyle, ProviderMark};
use crate::components::content::title_tip::use_tip;
use crate::components::controls::badge::{Badge, BadgeContent, BadgeTone};
use crate::components::controls::press::{PressListeners, use_pressing};
use crate::components::controls::progress::model::{Progress, ProgressStyle};
use crate::components::controls::progress::view::ProgressIndicator;
use crate::components::overlays::tooltip::Tooltip;
use crate::focus::click::kept_click;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::{DropState, Muting, Selection};
use ds_motion::detail::operation::Operation;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::hex::Colour;

/// What a tile shows.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PinFace {
    /// Every account: the inbox glyph.
    All,
    /// One account.
    Account {
        /// Its letter.
        initial: char,
        /// Its colour.
        colour: Colour,
        /// Its provider's mark.
        provider: MarkProvider,
        /// Its address, which names the tile to assistive technology (`S:1236`); without one
        /// the tile is named by its letter and provider.
        address: Option<String>,
    },
    /// The tile after the accounts that adds one: a plus in a dashed ring, quiet until hovered.
    /// It is an action, not a filter: never selected, no count.
    Add {
        /// What names it to assistive technology ("Add account").
        label: String,
        /// The hover hint ("Add account…"), when the app wants one.
        hint: Option<String>,
    },
}

/// An account's problem mark on its tile.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum PinStatus {
    /// Nothing to say: no mark.
    #[default]
    Quiet,
    /// The account is working (syncing, signing in): a Mini spinner in the corner, turning only
    /// while the `Operation` is `Running` (R4).
    Busy(Operation),
    /// The account needs the person (a rejected password, an unreachable server): the warning
    /// glyph in the corner, with `why` as its tooltip and its accessible name.
    Attention {
        /// What is wrong, in a sentence.
        why: String,
    },
}

impl PinStatus {
    /// The `data-status` word, none for a quiet tile.
    fn slug(&self) -> Option<&'static str> {
        match self {
            PinStatus::Quiet => None,
            PinStatus::Busy(_) => Some("busy"),
            PinStatus::Attention { .. } => Some("attention"),
        }
    }
}

impl PinFace {
    /// The `data-face` word.
    fn slug(&self) -> &'static str {
        match self {
            PinFace::All => "all",
            PinFace::Account { .. } => "account",
            PinFace::Add { .. } => "add",
        }
    }
}

/// The tile's `aria-label`: the account's address (`S:1236`), or, when the consumer gave none,
/// its letter and provider.
fn label(face: &PinFace) -> String {
    match face {
        PinFace::All => "All accounts".to_string(),
        PinFace::Add { label, .. } => label.clone(),
        PinFace::Account {
            address: Some(address),
            ..
        } => address.clone(),
        PinFace::Account {
            initial,
            provider,
            address: None,
            ..
        } => format!("{initial}, {} account", provider.name()),
    }
}

/// How an account's avatar is drawn: in its own colour while its tile is the list's filter,
/// muted (its chroma at .55) while it is not.
fn muting(selection: Selection) -> Muting {
    match selection {
        Selection::Selected => Muting::Audible,
        Selection::Unselected => Muting::Muted,
    }
}

/// A pinned tile. `selection` is whether it is the list's filter (`aria-pressed`, the plate
/// under it, and an account's colour at full strength); `unread` is the `Badge` on its corner
/// (zero draws none). `drop` is its part in a drag: `Source` while it is the one being dragged,
/// `Target` while a drop would land before it (the drop line). `mark` is how the provider is
/// drawn, the letter or the favicon the app supplies. `status` is the account's problem mark
/// (top-left corner); `onstatus` hears a press on its `Attention` glyph, which is not a press on
/// the tile. `onpointerdown` hears the press that may become a drag; `onclick` the press itself.
#[component]
pub fn PinTile(
    face: PinFace,
    #[props(default)] selection: Selection,
    #[props(default)] unread: u32,
    #[props(default)] drop: DropState,
    #[props(default = MarkStyle::Letter)] mark: MarkStyle,
    #[props(default)] status: PinStatus,
    #[props(default)] onstatus: Option<EventHandler<()>>,
    #[props(default)] onpointerdown: Option<EventHandler<PointerEvent>>,
    onclick: EventHandler<Press>,
    #[props(default)] common: Common,
) -> Element {
    let name = common.aria_label.clone().unwrap_or_else(|| label(&face));
    let listen = PressListeners::new(onclick);
    let pressing = use_pressing();
    let class = common.class("ds-pin-tile");
    let data = common.data_attributes();
    let filter = !matches!(face, PinFace::Add { .. });
    let slug = face.slug();
    let status_slug = status.slug();
    let mark_of_status = match status {
        PinStatus::Quiet => rsx! {},
        PinStatus::Busy(operation) => rsx! {
            span { class: "ds-pin-tile-status", "data-status": "busy",
                ProgressIndicator {
                    style: ProgressStyle::Spinner,
                    progress: Progress::Unknown(operation),
                    size: ControlSize::Mini,
                    common: Common { aria_label: Some("Working".to_owned()), ..Common::default() },
                }
            }
        },
        PinStatus::Attention { why } => rsx! {
            Tooltip { text: why.clone(),
                span {
                    class: "ds-pin-tile-status",
                    "data-status": "attention",
                    role: if onstatus.is_some() { "button" } else { "img" },
                    "aria-label": why,
                    // A press here is the badge's own: it neither presses the tile nor starts
                    // its drag.
                    onpointerdown: move |event| event.stop_propagation(),
                    onmousedown: move |event| event.stop_propagation(),
                    onmouseup: move |event| event.stop_propagation(),
                    onclick: move |event| {
                        event.stop_propagation();
                        if let Some(onstatus) = onstatus {
                            onstatus.call(());
                        }
                        kept_click(&event);
                    },
                    Glyph { icon: Icon::TriangleAlert, size: IconSize::Tiny }
                }
            }
        },
    };
    let hint = match &face {
        PinFace::Add { hint, .. } => hint.clone(),
        PinFace::All | PinFace::Account { .. } => None,
    };
    let tip = use_tip(hint);
    let picture = match face {
        PinFace::All => rsx! {
            span { class: "ds-pin-tile-face", Glyph { icon: Icon::Inbox, size: IconSize::Large } }
        },
        PinFace::Account {
            initial,
            colour,
            provider,
            ..
        } => rsx! {
            Avatar {
                initial,
                size: AvatarSize::Size28,
                tone: AvatarTone::Account(colour),
                muting: muting(selection),
            }
            ProviderMark { provider, size: ControlSize::Regular, style: mark }
        },
        PinFace::Add { .. } => rsx! {
            span { class: "ds-pin-tile-face", Glyph { icon: Icon::Plus, size: IconSize::Compact } }
        },
    };
    rsx! {
        button {
            r#type: "button",
            id: common.id.clone(),
            class,
            "data-face": slug,
            "data-selected": (filter && selection == Selection::Selected).then_some("true"),
            "data-drop": drop.drop_attr(),
            "data-drag": drop.drag_attr(),
            "data-status": status_slug,
            "data-pressed": pressing.attr(),
            "aria-pressed": filter.then(|| selection.aria()),
            "aria-label": name,
            title: tip.native(),
            onmouseover: { let tip = tip.clone(); move |event| tip.over(&event) },
            onmousedown: move |event| pressing.pointer_down(&event),
            onmouseleave: {
                let tip = tip.clone();
                move |_| {
                    tip.out();
                    pressing.released();
                }
            },
            onmouseup: move |event| {
                pressing.released();
                listen.mouse_up(&event);
            },
            onpointerdown: {
                let tip = tip.clone();
                move |event| {
                    tip.press();
                    if let Some(handler) = onpointerdown {
                        handler.call(event);
                    }
                }
            },
            onclick: move |event| listen.click(&event),
            oncontextmenu: move |event| listen.context_menu(&event),
            onmounted: {
                let tip = tip.clone();
                move |event| {
                    tip.mounted(&event);
                    common.mounted(event);
                }
            },
            ..data,
            if drop == DropState::Target {
                span { class: "ds-pin-tile-drop", "aria-hidden": "true" }
            }
            {picture}
            Badge { content: BadgeContent::Number(unread), tone: BadgeTone::Alert, size: ControlSize::Mini }
            {mark_of_status}
        }
        {tip.surface()}
    }
}

#[cfg(test)]
mod tests {
    use super::{PinFace, PinStatus, label, muting};
    use crate::components::content::provider_mark::MarkProvider;
    use ds_core::vocab::{Muting, Selection};
    use ds_motion::detail::operation::Operation;
    use ds_style::tokens::hex::{Colour, Hex};

    #[test]
    fn only_a_tile_that_is_not_the_filter_mutes_its_colour() {
        assert_eq!(muting(Selection::Selected), Muting::Audible);
        assert_eq!(muting(Selection::Unselected), Muting::Muted);
    }

    #[test]
    fn a_tile_is_quiet_unless_it_has_a_status() {
        assert_eq!(PinStatus::default(), PinStatus::Quiet);
        assert_eq!(PinStatus::Quiet.slug(), None);
        assert_eq!(PinStatus::Busy(Operation::Idle).slug(), Some("busy"));
        let why = "Password rejected".to_string();
        assert_eq!(PinStatus::Attention { why }.slug(), Some("attention"));
    }

    #[test]
    fn a_tile_is_named_by_its_address_else_its_letter_and_provider() {
        let colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));
        let account = |address: Option<&str>| PinFace::Account {
            initial: 'P',
            colour,
            provider: MarkProvider::Fastmail,
            address: address.map(str::to_string),
        };
        let cases = [
            (PinFace::All, "All accounts"),
            (account(Some("poh@acme.example")), "poh@acme.example"),
            (account(None), "P, Fastmail account"),
            (
                PinFace::Add {
                    label: "Add account".to_string(),
                    hint: None,
                },
                "Add account",
            ),
        ];
        for (face, want) in cases {
            assert_eq!(label(&face), want);
        }
    }
}
