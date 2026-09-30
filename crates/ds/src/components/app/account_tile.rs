//! AccountTile: an account in this Space, as a square tile on the frame
//! (design/04-COMPONENTS.md section 27).

use crate::components::content::avatar::{Avatar, AvatarSize, AvatarTone};
use crate::components::content::muted::muted;
use crate::components::content::provider_mark::{MarkProvider, MarkSize, MarkStyle, ProviderMark};
use crate::components::controls::badge::Badge;
use crate::components::controls::badge::BadgeContent;
use crate::components::controls::badge::BadgeTone;
use dioxus::prelude::*;
use ds_core::vocab::Check;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::hex::Colour;

/// Whose tile.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountFace {
    /// Every account: the inbox glyph.
    All,
    /// One account.
    One {
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
}

/// The avatar colour a tile shows: the account's own when pressed, muted when not (S's
/// `saturate(.55)`, the same as `Avatar { muting: Muting::Muted }`).
fn tile_colour(colour: Colour, pressed: Check) -> Colour {
    match pressed {
        Check::On => colour,
        Check::Off | Check::Mixed => muted(colour),
    }
}

/// The tile's `aria-label`: the account's address (`S:1236`), or, when the consumer gave none,
/// its letter and provider.
fn label(account: &AccountFace) -> String {
    match account {
        AccountFace::All => "All accounts".to_string(),
        AccountFace::One {
            address: Some(address),
            ..
        } => address.clone(),
        AccountFace::One {
            initial,
            provider,
            address: None,
            ..
        } => format!("{initial}, {} account", provider.name()),
    }
}

/// An account tile: a toolbar Button holding the account's avatar, its provider mark and its
/// unread count. Pressed when it is the list's filter, or the Space's only account.
///
/// `mark` is how the provider is drawn, the letter or the favicon the app supplies: the caller's
/// own "provider marks" setting, so every tile and row follows one choice.
#[component]
pub fn AccountTile(
    account: AccountFace,
    pressed: Check,
    unread: u32,
    onclick: EventHandler<()>,
    #[props(default = MarkStyle::Letter)] mark: MarkStyle,
) -> Element {
    let label = label(&account);
    let face = match account {
        AccountFace::All => rsx! {
            span { class: "ds-avatar", "data-size": "28", "data-tone": "all",
                Glyph { icon: Icon::Inbox, size: IconSize::Large }
            }
        },
        AccountFace::One {
            initial,
            colour,
            provider,
            ..
        } => rsx! {
            Avatar {
                initial,
                size: AvatarSize::Size28,
                tone: AvatarTone::Account(tile_colour(colour, pressed)),
            }
            ProviderMark { provider, size: MarkSize::Tile, style: mark }
        },
    };
    rsx! {
        button {
            r#type: "button",
            class: "ds-button ds-account-tile",
            "data-variant": "toolbar",
            "data-size": "large",
            "data-image": "only",
            "aria-pressed": pressed.aria(),
            "aria-label": label,
            onclick: move |_| onclick.call(()),
            {face}
            Badge { content: BadgeContent::Number(unread), tone: BadgeTone::Alert, size: ControlSize::Mini}
        }
    }
}

/// The tile after the accounts that adds one: the same Pin plate, holding a plus in a dashed
/// ring where an avatar would be, quiet until hovered. It is an action, not a filter, so it is
/// never pressed and has no count. `label` names it to assistive technology; `title` is the
/// hover hint ("Add account…").
#[component]
pub fn AddAccountTile(
    #[props(default = "Add account".to_string())] label: String,
    #[props(default)] title: Option<String>,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            r#type: "button",
            class: "ds-button ds-account-tile",
            "data-variant": "toolbar",
            "data-size": "large",
            "data-image": "only",
            "data-face": "add",
            "aria-label": label,
            title,
            onclick: move |_| onclick.call(()),
            span { class: "ds-avatar", "data-size": "28", "data-tone": "add",
                Glyph { icon: Icon::Plus, size: IconSize::Compact }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::tile_colour;
    use ds_core::vocab::Check;
    use ds_style::tokens::hex::{Colour, Hex};

    #[test]
    fn only_an_unpressed_tile_is_desaturated() {
        let colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));
        assert_eq!(tile_colour(colour, Check::On), colour);
        assert_ne!(tile_colour(colour, Check::Off), colour);
    }
}
