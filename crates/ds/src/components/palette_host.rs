//! Where the command palette's card draws and how it enters (design/04-COMPONENTS.md section
//! 25): over the window on a scrim, or in a surface of its own (sill FINDINGS Q40), with S's
//! `peek-in`, C's `cmdk-in`, or an opaque spring. Split from `command_palette`.

use crate::components::popover::Float;
use crate::components::tooltip::Shown;
use crate::motion::anim::Anim;
use crate::tokens::Corner;
use dioxus::prelude::*;

/// Where the palette draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CommandPaletteHost {
    /// Over the window, on the palette layer: a scrim, and the card 11 % down.
    #[default]
    Overlay,
    /// In place, filling its container: no scrim, no layer of its own to draw on (a shell
    /// surface that is the palette). Its card paints the enclosing material.
    Surface,
}

impl CommandPaletteHost {
    pub(crate) fn slug(self) -> &'static str {
        match self {
            CommandPaletteHost::Overlay => "overlay",
            CommandPaletteHost::Surface => "surface",
        }
    }
}

/// How the palette's card enters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PaletteEntrance {
    /// `peek-in`, S's palette.
    #[default]
    PeekIn,
    /// `cmdk-in`, C's command menu.
    CmdkIn,
    /// `cmdk-rise`: `cmdk-in`'s spring with no fade, so the card is opaque from its first
    /// frame (mailo gaps 2); over a window the scrim then appears at once too, since a fading
    /// scrim would hold the card inside it at its own opacity.
    Opaque,
}

impl PaletteEntrance {
    pub(crate) fn anim(self) -> Anim {
        match self {
            PaletteEntrance::PeekIn => Anim::PeekIn,
            PaletteEntrance::CmdkIn => Anim::CmdkIn,
            PaletteEntrance::Opaque => Anim::CmdkRise,
        }
    }

    pub(crate) fn slug(self) -> &'static str {
        match self {
            PaletteEntrance::PeekIn => "peek-in",
            PaletteEntrance::CmdkIn => "cmdk-in",
            PaletteEntrance::Opaque => "cmdk-rise",
        }
    }
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
    entrance: PaletteEntrance,
    float: Float,
    card: Element,
    shown: Shown,
    onclose: EventHandler<()>,
) -> Element {
    // Only an opaque entrance marks the wrap: the scrim must not fade the card with it.
    let still = (entrance == PaletteEntrance::Opaque).then_some(entrance.slug());
    match host {
        CommandPaletteHost::Surface => card,
        CommandPaletteHost::Overlay => {
            float.show(
                rsx! {
                    div {
                        class: "ds-palette-wrap",
                        "data-shown": shown.slug(),
                        "data-entrance": still,
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
