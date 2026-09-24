//! The menu tracker's vocabulary: what it remembers, what it is told, what it asks for.

use crate::geometry::Point;
use std::time::{Duration, Instant};

/// An item, by its index at each nesting level: `[2, 0]` is the first item of the third
/// item's submenu.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ItemPath(pub Vec<u16>);

/// Whether the button that opened the menu is still down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Held {
    /// Down: a release picks, or closes (press-drag-release).
    Held,
    /// Released on the title: click mode, the menu stays open.
    Released,
}

/// Whether the pointer has been inside the menu since it opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Entered {
    /// It has.
    Entered,
    /// Not yet.
    NotYet,
}

/// Whether an item can be picked (separators, headers and disabled items cannot).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Pickable {
    /// A live item.
    Enabled,
    /// A separator, header or disabled item.
    Inert,
}

/// Whether an item opens a submenu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Branch {
    /// It runs an action.
    Leaf,
    /// It opens a submenu.
    Submenu,
}

/// What is under the pointer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuTarget<K> {
    /// A menu's title (a bar title or status item): the key of the menu it opens.
    Title(K),
    /// An item of the open menu or one of its submenus.
    Item {
        /// Which item.
        path: ItemPath,
        /// Whether it can be picked.
        pick: Pickable,
        /// Whether it opens a submenu.
        branch: Branch,
    },
    /// Inside a menu, not on an item (its padding).
    Menu,
    /// Neither a menu nor a title.
    Outside,
}

/// The safe triangle (design/13 section 13.3.4): from the last pointer sample to the open
/// submenu's near-edge corners. Its hysteresis is the corners' 4 px inflation and the
/// `from` corner following the pointer, so a steady diagonal never falls off an edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeTriangle {
    /// The last pointer sample inside the triangle.
    pub from: Point,
    /// The submenu's near-edge top corner.
    pub top: Point,
    /// The submenu's near-edge bottom corner.
    pub bottom: Point,
    /// When the pointer last moved inside the triangle.
    pub still_since: Instant,
}

/// The open menu's submenu, if any.
#[derive(Debug, Clone, PartialEq)]
pub enum Submenu {
    /// None open or pending.
    None,
    /// The pointer rests on `item`; its submenu opens once the delay has passed.
    Pending {
        /// The parent item.
        item: ItemPath,
        /// When the pointer came to rest on it.
        since: Instant,
    },
    /// `item`'s submenu is open. `guard` is `None` until the caller reports where it landed.
    Open {
        /// The parent item.
        item: ItemPath,
        /// The safe triangle toward it.
        guard: Option<SafeTriangle>,
    },
}

/// One tracking session: from the press that opened a menu to the close.
#[derive(Debug, Clone, PartialEq)]
pub struct Session<K> {
    /// The open menu.
    pub menu: K,
    /// Whether the opening press is still down.
    pub held: Held,
    /// Whether the pointer has been inside the menu.
    pub entered: Entered,
    /// The highlighted item (always a pickable one).
    pub hot: Option<ItemPath>,
    /// The open or pending submenu.
    pub sub: Submenu,
    /// The last pointer sample.
    pub pointer: Point,
    /// What was under it.
    pub under: MenuTarget<K>,
}

/// Where the tracker is.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuPhase<K> {
    /// No menu open.
    Closed,
    /// A menu is open.
    Tracking(Session<K>),
}

/// The tracker's timings, from `menus.submenu_delay_ms` and
/// `menus.submenu_triangle_timeout_ms` (design/22-SETTINGS.md; proposed 200 and 300 ms).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MenuTiming {
    /// How long the pointer rests on a parent item before its submenu opens.
    pub submenu_delay: Duration,
    /// How long the pointer may rest inside the safe triangle before the item under it wins.
    pub triangle_timeout: Duration,
}

impl Default for MenuTiming {
    /// design/13 section 13.3.4's proposed 200 ms delay and 300 ms triangle timeout.
    fn default() -> Self {
        MenuTiming {
            submenu_delay: Duration::from_millis(200),
            triangle_timeout: Duration::from_millis(300),
        }
    }
}

/// The tracker: its timings and where it is.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuTrack<K> {
    /// The submenu delay and triangle timeout.
    pub timing: MenuTiming,
    /// Closed or tracking.
    pub phase: MenuPhase<K>,
}

/// The keys that cross menus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuKey {
    /// Close one level.
    Escape,
    /// Close a submenu, else move to the menu on the left.
    Left,
    /// Open the pending submenu at once, else move to the menu on the right.
    Right,
    /// Pick the highlighted item (the caller maps Space and Tab here too).
    Enter,
}

/// One thing that happened.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuTrackEvent<K> {
    /// A press on a menu's title.
    PressTitle(K),
    /// The button came up over this target.
    Release(MenuTarget<K>),
    /// A pointer sample, and what is under it.
    Move(Point, MenuTarget<K>),
    /// A key that crosses menus.
    Key(MenuKey),
    /// A press outside every menu and title (`xdg_popup.popup_done`, a click elsewhere).
    OutsidePress,
    /// The open submenu is laid out: its near-edge corners.
    SubPlaced {
        /// The near-edge top corner.
        top: Point,
        /// The near-edge bottom corner.
        bottom: Point,
    },
    /// A timer asked for with [`MenuTrackEffect::RequestTick`] fired.
    Tick,
    /// The keyboard moved the highlight to this item (Up or Down inside the menu): a submenu
    /// of another item closes, a pending one is dropped, nothing opens.
    Select(ItemPath),
    /// Open this item's submenu now: Right, Enter or a click on a parent item
    /// (design/13 section 13.3.4 "open immediately").
    Expand(ItemPath),
}

/// How a menu appears or leaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuAnim {
    /// `menu-pop --t-move --e-spring`: only the first menu of a tracking session.
    Pop,
    /// `fade --t-quick --e-exit`.
    Fade,
}

/// Left or right along the bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuDirection {
    /// Toward the start.
    Left,
    /// Toward the end.
    Right,
}

/// One thing the caller must do, in the order returned.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MenuTrackEffect<K> {
    /// Show `K`'s menu with this animation.
    Open(K, MenuAnim),
    /// Swap the open menu for `K`'s in the same frame, no animation (hover switch).
    Switch(K),
    /// Close the menu (and any submenu).
    Close(MenuAnim),
    /// Open this item's submenu, no animation.
    OpenSub(ItemPath),
    /// Close the open submenu.
    CloseSub,
    /// Highlight this item, or none.
    Highlight(Option<ItemPath>),
    /// Run the item's action; always after its `Close` (design A6 "closes then picks").
    Pick(ItemPath),
    /// Open the adjacent menu on this side (Left or Right with no submenu to act on).
    Adjacent(MenuDirection),
    /// Deliver [`MenuTrackEvent::Tick`] at this time.
    RequestTick(Instant),
}
