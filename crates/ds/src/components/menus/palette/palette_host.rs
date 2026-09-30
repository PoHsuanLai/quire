//! Where the command palette's card draws and how it enters (design/04-COMPONENTS.md section
//! 25): over the window on a scrim, or in a surface of its own, with S's
//! `peek-in`, C's `cmdk-in`, or an opaque spring. Split from `command_palette`.

use crate::components::overlays::popover::Float;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_style::tokens::shape::Corner;

/// Where the palette draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum CommandPaletteHost {
    /// Over the window, on the palette layer: a scrim, and the card 11 % down.
    #[default]
    Overlay,
    /// In place, filling its container: no scrim, no layer of its own to draw on (a shell
    /// surface that is the palette). Its card paints the enclosing material.
    Surface,
}

/// The card's own corner: its radius, and a squircle's extent and shadow circle.
pub(crate) fn card_corner(corner: Corner) -> String {
    match corner {
        Corner::Squircle(_) => corner.squircle_style(),
        Corner::Token(_) | Corner::Px(_) => format!("border-radius:{};", corner.css()),
    }
}

/// The card where `host` draws it: in place, or on the palette layer over its scrim, which
/// closes the palette on a pointer down outside the card while it is the topmost layer.
pub(crate) fn hosted(
    host: CommandPaletteHost,
    float: Float,
    card: Element,
    shown: Shown,
    onclose: EventHandler<()>,
) -> Element {
    match host {
        CommandPaletteHost::Surface => card,
        CommandPaletteHost::Overlay => {
            float.show(
                rsx! {
                    div {
                        class: "ds-palette-wrap",
                        "data-shown": shown.slug(),
                        onpointerdown: move |_| {
                            if float.is_top() {
                                onclose.call(());
                            }
                        },
                        {card}
                    }
                },
                onclose,
            );
            rsx! {}
        }
    }
}
