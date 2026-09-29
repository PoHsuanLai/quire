//! Z-order inside a window (design/01-LAYOUT.md section 12). The plan names the ends,
//! `--z-raise` 1 and `--z-drag` 50; the layers between are named after what sits on them.

use super::name::VarName;

/// One stacking layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ZLayer {
    /// `--z-scene` -2: the frame's gradient layers.
    Scene,
    /// `--z-grain` -1: the grain tile.
    Grain,
    /// `--z-raise` 1: a lifted row or control.
    Raise,
    /// `--z-link-pill` 7.
    LinkPill,
    /// `--z-toast` 8.
    Toast,
    /// `--z-send-pill` 9: also the floating composer.
    SendPill,
    /// `--z-scrim` 10.
    Scrim,
    /// `--z-peek` 11.
    Peek,
    /// `--z-focus-page` 12: the focus-mode composer page.
    FocusPage,
    /// `--z-edge` 14: the left edge strip.
    Edge,
    /// `--z-side-peek` 15.
    SidePeek,
    /// `--z-palette` 20: the command menu.
    Palette,
    /// `--z-card` 30: hover cards.
    Card,
    /// `--z-menu` 40: floating menus and the zZ floater.
    Menu,
    /// `--z-bubble` 41: the selection bubble.
    Bubble,
    /// `--z-drag` 50: the drag ghost.
    Drag,
}

impl ZLayer {
    /// Every layer, lowest first.
    pub const ALL: [ZLayer; 16] = [
        ZLayer::Scene,
        ZLayer::Grain,
        ZLayer::Raise,
        ZLayer::LinkPill,
        ZLayer::Toast,
        ZLayer::SendPill,
        ZLayer::Scrim,
        ZLayer::Peek,
        ZLayer::FocusPage,
        ZLayer::Edge,
        ZLayer::SidePeek,
        ZLayer::Palette,
        ZLayer::Card,
        ZLayer::Menu,
        ZLayer::Bubble,
        ZLayer::Drag,
    ];

    /// The custom property: `--z-menu`, …
    pub fn var(self) -> VarName {
        VarName(match self {
            ZLayer::Scene => "--z-scene",
            ZLayer::Grain => "--z-grain",
            ZLayer::Raise => "--z-raise",
            ZLayer::LinkPill => "--z-link-pill",
            ZLayer::Toast => "--z-toast",
            ZLayer::SendPill => "--z-send-pill",
            ZLayer::Scrim => "--z-scrim",
            ZLayer::Peek => "--z-peek",
            ZLayer::FocusPage => "--z-focus-page",
            ZLayer::Edge => "--z-edge",
            ZLayer::SidePeek => "--z-side-peek",
            ZLayer::Palette => "--z-palette",
            ZLayer::Card => "--z-card",
            ZLayer::Menu => "--z-menu",
            ZLayer::Bubble => "--z-bubble",
            ZLayer::Drag => "--z-drag",
        })
    }

    /// The `z-index`.
    pub fn z(self) -> i16 {
        match self {
            ZLayer::Scene => -2,
            ZLayer::Grain => -1,
            ZLayer::Raise => 1,
            ZLayer::LinkPill => 7,
            ZLayer::Toast => 8,
            ZLayer::SendPill => 9,
            ZLayer::Scrim => 10,
            ZLayer::Peek => 11,
            ZLayer::FocusPage => 12,
            ZLayer::Edge => 14,
            ZLayer::SidePeek => 15,
            ZLayer::Palette => 20,
            ZLayer::Card => 30,
            ZLayer::Menu => 40,
            ZLayer::Bubble => 41,
            ZLayer::Drag => 50,
        }
    }
}
