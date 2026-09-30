//! PinTile: one square tile of the pinned grid above a sidebar's list (design/30 section 2.11):
//! an account in this Space, the "All" tile, or the tile that adds one. One component for the
//! three faces; the account's unread count is a `Badge`, and a drag over it draws the list's drop
//! line at its leading edge. Grouping, reordering and the Add tile's place are `PinTiles`.

use crate::components::content::avatar::{Avatar, AvatarSize, AvatarTone};
use crate::components::content::provider_mark::{MarkProvider, MarkStyle, ProviderMark};
use crate::components::controls::badge::{Badge, BadgeContent, BadgeTone};
use crate::components::controls::press::{PressListeners, use_pressing};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::{DropState, Muting, Selection};
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
/// drawn, the letter or the favicon the app supplies. `onpointerdown` hears the press that may
/// become a drag; `onclick` the press itself.
#[component]
pub fn PinTile(
    face: PinFace,
    #[props(default)] selection: Selection,
    #[props(default)] unread: u32,
    #[props(default)] drop: DropState,
    #[props(default = MarkStyle::Letter)] mark: MarkStyle,
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
    let hint = match &face {
        PinFace::Add { hint, .. } => hint.clone(),
        PinFace::All | PinFace::Account { .. } => None,
    };
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
            "data-pressed": pressing.attr(),
            "aria-pressed": filter.then(|| selection.aria()),
            "aria-label": name,
            title: hint,
            onmousedown: move |event| pressing.pointer_down(&event),
            onmouseleave: move |_| pressing.released(),
            onmouseup: move |event| {
                pressing.released();
                listen.mouse_up(&event);
            },
            onpointerdown: move |event| {
                if let Some(handler) = onpointerdown {
                    handler.call(event);
                }
            },
            onclick: move |event| listen.click(&event),
            oncontextmenu: move |event| listen.context_menu(&event),
            onmounted: move |event| common.mounted(event),
            ..data,
            if drop == DropState::Target {
                span { class: "ds-pin-tile-drop", "aria-hidden": "true" }
            }
            {picture}
            Badge { content: BadgeContent::Number(unread), tone: BadgeTone::Alert, size: ControlSize::Mini }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PinFace, label, muting};
    use crate::components::content::provider_mark::MarkProvider;
    use ds_core::vocab::{Muting, Selection};
    use ds_style::tokens::hex::{Colour, Hex};

    #[test]
    fn only_a_tile_that_is_not_the_filter_mutes_its_colour() {
        assert_eq!(muting(Selection::Selected), Muting::Audible);
        assert_eq!(muting(Selection::Unselected), Muting::Muted);
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
