//! A menu's shape and how it appears (design/04-COMPONENTS.md section 20): the four kinds, where
//! each goes against its anchor, its entrance and its item layout. Split from `menu`.

use crate::components::menu_item::Row;
use crate::geometry::{Align, Placement, Point, Px, Rect, Side};
use crate::motion::anim::Anim;

/// Which menu shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuKind {
    /// 280 wide, 34 px tiles, title and help and shortcut.
    Rich,
    /// 220 wide, 22 px tiles.
    Slim,
    /// C's check-column menu, anchored under its button's right edge.
    Dropdown,
    /// Anchored at the pointer.
    Context,
}

impl MenuKind {
    /// The `data-kind` word.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            MenuKind::Rich => "rich",
            MenuKind::Slim => "slim",
            MenuKind::Dropdown => "dropdown",
            MenuKind::Context => "context",
        }
    }

    /// Where the menu goes against its anchor: S's floating menus at `left - 8`, 6 below
    /// (`S:2060-2065`); C's dropdown under the trigger's right edge, 7 below; a context menu
    /// at the pointer (design/06-INTERACTIONS.md section 4).
    pub(crate) fn placement(self, anchor: Rect) -> (Rect, Placement, Px) {
        match self {
            MenuKind::Rich | MenuKind::Slim => (
                Rect {
                    origin: Point {
                        x: anchor.origin.x - Px(8.0),
                        ..anchor.origin
                    },
                    ..anchor
                },
                Placement::new(Side::Bottom, Align::Start),
                Px(6.0),
            ),
            MenuKind::Dropdown => (anchor, Placement::new(Side::Bottom, Align::End), Px(7.0)),
            MenuKind::Context => (anchor, Placement::new(Side::Bottom, Align::Start), Px(0.0)),
        }
    }

    /// The entrance: `menu-in` for C's dropdown, `menu-pop` for the rest.
    pub(crate) fn entrance(self) -> Anim {
        match self {
            MenuKind::Dropdown => Anim::MenuIn,
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Anim::MenuPop,
        }
    }

    /// The panel's padding (design/04-COMPONENTS.md section 20): 6 for the Dropdown, 5 for the
    /// rest. A submenu's top sits this far above its parent row.
    pub(crate) fn pad(self) -> Px {
        match self {
            MenuKind::Dropdown => Px(6.0),
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Px(5.0),
        }
    }

    /// The item layout.
    pub(crate) fn row(self) -> Row {
        match self {
            MenuKind::Dropdown => Row::Checked,
            MenuKind::Rich | MenuKind::Slim | MenuKind::Context => Row::Tiled,
        }
    }
}

/// How a menu appears: with its entrance (`menu-pop`, or `menu-in` for a Dropdown), or at once.
/// A bar menu opens at once (design/13-BEHAVIOUR-menus-windows.md section 13.3.2: the menu bar
/// has no open animation, and a hover switch shows the next menu in the same frame).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuEntrance {
    /// Play the kind's entrance.
    #[default]
    Animated,
    /// Appear at rest, with no entrance.
    Instant,
}
