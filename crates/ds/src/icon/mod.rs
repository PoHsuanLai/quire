//! Lucide glyphs, as data.
//!
//! The geometry is Lucide (<https://lucide.dev>), ISC licence, notice in
//! `crates/ds/assets/icons/LICENSE-lucide.txt`.
//!
//! The style is a 24 grid, 2px stroke, round caps and joins, one colour, no fills. Unlike
//! mailo, the stroke is written as SVG attributes by [`render::Glyph`], because usvg in Blitz
//! may not resolve CSS on SVG (design/08-ICONS.md section 1.3, spike S6).
//!
//! The icons are data and not markup strings, so nothing here needs a raw HTML sink.
//! The mailo set's geometry is in `geometry`, transcribed from the design's `ICON` table; the
//! shell set's is in `geometry_shell` (design/08-ICONS.md section 1.6), the control center's in
//! `geometry_control` (sill FINDINGS Q81), quire's own marks on Lucide's grid in `geometry_own`
//! (sill FINDINGS Q103); the named sets are in `sets`.
//!
//! Moved from mailo (`mail-app/src/ui/icon/mod.rs`) and made `pub`.
//!
//! No serde: an icon is never stored, and a derive would make it a persisted schema
//! (`CONVENTIONS.md` section 3).

pub mod classify;
pub mod external;
pub mod family;
mod geometry;
mod geometry_actions;
mod geometry_control;
mod geometry_own;
mod geometry_shell;
pub mod plate;
pub mod plate_tint;
pub mod render;
pub mod retint;
mod sets;
pub mod shape;
pub mod stroke;
#[cfg(test)]
mod tests;

pub use classify::{ChromaLimit, IconKind, classify, classify_with};
pub use external::{ExternalIcon, IconSource, IconUrl};
pub use family::PlateFamily;
use geometry::*;
pub use plate_tint::{PlateStops, PlateTint};
pub use retint::{IconStyle, Tint, retint};
pub use shape::Shape;

/// One glyph: the mailo set, named for its key in the design's `ICON` table, then the shell
/// set, named for its Lucide glyph.
#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub enum Icon {
    /// The mailo `Inbox` glyph.
    Inbox,
    /// The mailo `Star` glyph.
    Star,
    /// The mailo `Archive` glyph.
    Archive,
    /// The mailo `Clock` glyph.
    Clock,
    /// The mailo `Trash` glyph.
    Trash,
    /// The mailo `Mail` glyph.
    Mail,
    /// The mailo `MailOpen` glyph.
    MailOpen,
    /// The mailo `Tag` glyph.
    Tag,
    /// The mailo `Refresh` glyph.
    Refresh,
    /// The mailo `Send` glyph.
    Send,
    /// The mailo `Command` glyph.
    Command,
    /// The mailo `Columns` glyph.
    Columns,
    /// The mailo `Group` glyph.
    Group,
    /// The mailo `Panel` glyph.
    Panel,
    /// The mailo `Square` glyph.
    Square,
    /// The mailo `Maximize` glyph.
    Maximize,
    /// The mailo `Corner` glyph.
    Corner,
    /// The mailo `Undo` glyph.
    Undo,
    /// The mailo `Check` glyph.
    Check,
    /// The mailo `X` glyph.
    X,
    /// The mailo `Paperclip` glyph.
    Paperclip,
    /// The mailo `Pen` glyph.
    Pen,
    /// The mailo `Key` glyph.
    Key,
    /// The mailo `FilePen` glyph.
    FilePen,
    /// The mailo `OctagonAlert` glyph.
    OctagonAlert,
    /// The mailo `Pin` glyph.
    Pin,
    /// The mailo `Reply` glyph.
    Reply,
    /// The mailo `ReplyAll` glyph.
    ReplyAll,
    /// The mailo `Forward` glyph.
    Forward,
    /// The mailo `Search` glyph.
    Search,
    /// The mailo `Settings` glyph.
    Settings,
    /// The mailo `PanelLeft` glyph.
    PanelLeft,
    /// The mailo `Plus` glyph.
    Plus,
    /// Lucide `wifi`.
    Wifi,
    /// Lucide `wifi-low`.
    WifiLow,
    /// Lucide `wifi-high`.
    WifiHigh,
    /// Lucide `wifi-off`.
    WifiOff,
    /// Lucide `ethernet-port`: a wired network.
    Ethernet,
    /// Lucide `battery`.
    Battery,
    /// Lucide `battery-low`.
    BatteryLow,
    /// Lucide `battery-medium`.
    BatteryMedium,
    /// Lucide `battery-full`.
    BatteryFull,
    /// Lucide `battery-charging`.
    BatteryCharging,
    /// Lucide `battery-warning`.
    BatteryWarning,
    /// Lucide `bluetooth`.
    Bluetooth,
    /// Lucide `bluetooth-connected`.
    BluetoothConnected,
    /// Lucide `bluetooth-off`.
    BluetoothOff,
    /// Lucide `volume`.
    Volume,
    /// Lucide `volume-1`.
    Volume1,
    /// Lucide `volume-2`.
    Volume2,
    /// Lucide `volume-x`.
    VolumeX,
    /// Lucide `mic`.
    Mic,
    /// Lucide `mic-off`.
    MicOff,
    /// Lucide `sun`.
    Sun,
    /// Lucide `moon`.
    Moon,
    /// Lucide `sun-moon`.
    SunMoon,
    /// Lucide `power`.
    Power,
    /// Lucide `lock`.
    Lock,
    /// Lucide `bell`.
    Bell,
    /// Lucide `bell-off`.
    BellOff,
    /// Lucide `layout-grid`.
    Grid,
    /// Lucide `app-window`.
    Window,
    /// Lucide `monitor`.
    Monitor,
    /// Lucide `keyboard`.
    Keyboard,
    /// Lucide `chevron-left`.
    ChevronLeft,
    /// Lucide `chevron-right`.
    ChevronRight,
    /// Lucide `chevron-up`.
    ChevronUp,
    /// Lucide `chevron-down`.
    ChevronDown,
    /// Lucide `folder`.
    Folder,
    /// Lucide `file`.
    File,
    /// Lucide `image`.
    Image,
    /// Lucide `square-terminal`.
    Terminal,
    /// Lucide `sticky-note`.
    StickyNote,
    /// Lucide `camera`.
    Camera,
    /// Lucide `download`.
    Download,
    /// Lucide `upload`.
    Upload,
    /// Lucide `copy`.
    Copy,
    /// Lucide `link`.
    Link,
    /// Lucide `sparkles`.
    Sparkles,
    /// Lucide `gauge`.
    Gauge,
    /// Tabler `brightness-half`, used only if Lucide `sun` reads wrong at 22 px.
    Brightness,
    /// Lucide `printer`: mailo's print action.
    Printer,
    /// Lucide `folder-input`: mailo's move-to-folder action.
    FolderInput,
    /// Lucide `play`: Now Playing.
    Play,
    /// Lucide `pause`: Now Playing.
    Pause,
    /// Lucide `skip-back`: Now Playing's previous track.
    SkipBack,
    /// Lucide `skip-forward`: Now Playing's next track.
    SkipForward,
    /// Lucide `log-out`: the power menu's Log Out.
    LogOut,
    /// Lucide `rotate-ccw`: the power menu's Restart.
    Restart,
    /// Lucide `headphones`: an audio output or a Bluetooth device.
    Headphones,
    /// Lucide `speaker`: an audio output.
    Speaker,
    /// Lucide `mouse`: a Bluetooth device.
    Mouse,
    /// Lucide `gamepad`: a Bluetooth device.
    Gamepad,
    /// Lucide `smartphone`: a Bluetooth device.
    Phone,
    // mailo gaps 6: the "more" glyphs. Kept at the end so the control set (sill Q81) and these
    // merge without touching each other's lines.
    /// Lucide `ellipsis`: a row's or a header's overflow menu, laid across.
    Ellipsis,
    /// Lucide `ellipsis-vertical`: the same menu where the row is narrow and tall.
    EllipsisVertical,
    // Control center parts 2 (sill FINDINGS Q103): quire's own glyphs on Lucide's grid.
    /// Two toggles with their knobs at opposite ends: the control center's bar item
    /// (`geometry_own`, which documents the geometry).
    Switches,
    // Lock and switcher parts (M11): the lock field's enter arrow and its caps-lock mark.
    /// Lucide `arrow-right`: the lock field's enter button.
    ArrowRight,
    /// Lucide `arrow-big-up-dash`: caps lock is on.
    CapsLock,
    // Launcher v2 (sill Q296): the clipboard, emoji and web providers.
    /// Lucide `clipboard`: the clipboard history.
    Clipboard,
    /// Lucide `smile`: emoji.
    Smile,
    /// Lucide `globe`: the web.
    Globe,
}

impl Icon {
    /// The children of this glyph, in the design's order.
    pub fn shapes(self) -> &'static [Shape] {
        match self {
            Icon::Inbox => INBOX,
            Icon::Star => STAR,
            Icon::Archive => ARCHIVE,
            Icon::Clock => CLOCK,
            Icon::Trash => TRASH,
            Icon::Mail => MAIL,
            Icon::MailOpen => MAIL_OPEN,
            Icon::Tag => TAG,
            Icon::Refresh => REFRESH,
            Icon::Send => SEND,
            Icon::Command => COMMAND,
            Icon::Columns => COLUMNS,
            Icon::Group => GROUP,
            Icon::Panel => PANEL,
            Icon::Square => SQUARE,
            Icon::Maximize => MAXIMIZE,
            Icon::Corner => CORNER,
            Icon::Undo => UNDO,
            Icon::Check => CHECK,
            Icon::X => X_MARK,
            Icon::Paperclip => PAPERCLIP,
            Icon::Pen => PEN,
            Icon::Key => KEY,
            Icon::FilePen => FILE_PEN,
            Icon::OctagonAlert => OCTAGON_ALERT,
            Icon::Pin => PIN,
            Icon::Reply => REPLY,
            Icon::ReplyAll => REPLY_ALL,
            Icon::Forward => FORWARD,
            Icon::Search => SEARCH,
            Icon::Settings => SETTINGS,
            Icon::PanelLeft => PANEL_LEFT,
            Icon::Plus => PLUS,
            Icon::Printer => geometry_actions::PRINTER,
            Icon::FolderInput => geometry_actions::FOLDER_INPUT,
            Icon::Play
            | Icon::Pause
            | Icon::SkipBack
            | Icon::SkipForward
            | Icon::LogOut
            | Icon::Restart
            | Icon::Headphones
            | Icon::Speaker
            | Icon::Mouse
            | Icon::Gamepad
            | Icon::Phone => geometry_control::shapes(self),
            // mailo gaps 6.
            Icon::Ellipsis => geometry_actions::ELLIPSIS,
            Icon::EllipsisVertical => geometry_actions::ELLIPSIS_VERTICAL,
            Icon::Switches => geometry_own::SWITCHES,
            Icon::ArrowRight => geometry_actions::ARROW_RIGHT,
            Icon::CapsLock => geometry_actions::CAPS_LOCK,
            Icon::Clipboard => geometry_actions::CLIPBOARD,
            Icon::Smile => geometry_actions::SMILE,
            Icon::Globe => geometry_actions::GLOBE,
            shell => geometry_shell::shapes(shell),
        }
    }
}
