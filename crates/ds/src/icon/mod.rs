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
//! shell set's is in `geometry_shell` (design/08-ICONS.md section 1.6).
//!
//! Moved from mailo (`mail-app/src/ui/icon/mod.rs`) and made `pub`.
//!
//! No serde: an icon is never stored, and a derive would make it a persisted schema
//! (`CONVENTIONS.md` section 3).

mod geometry;
mod geometry_shell;
pub mod render;
pub mod shape;
#[cfg(test)]
mod tests;

use geometry::*;
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
}

impl Icon {
    /// The mailo set, in the order of the keys of `ICON`.
    ///
    /// The icon tests lock this list to `icons.js`.
    pub const MAILO: &[Icon] = &[
        Icon::Inbox,
        Icon::Star,
        Icon::Archive,
        Icon::Clock,
        Icon::Trash,
        Icon::Mail,
        Icon::MailOpen,
        Icon::Tag,
        Icon::Refresh,
        Icon::Send,
        Icon::Command,
        Icon::Columns,
        Icon::Group,
        Icon::Panel,
        Icon::Square,
        Icon::Maximize,
        Icon::Corner,
        Icon::Undo,
        Icon::Check,
        Icon::X,
        Icon::Paperclip,
        Icon::Pen,
        Icon::Key,
        Icon::FilePen,
        Icon::OctagonAlert,
        Icon::Pin,
        Icon::Reply,
        Icon::ReplyAll,
        Icon::Forward,
        Icon::Search,
        Icon::Settings,
        Icon::PanelLeft,
        Icon::Plus,
    ];

    /// The shell set, in design/08-ICONS.md section 1.6's order.
    pub const SHELL: &[Icon] = &[
        Icon::Wifi,
        Icon::WifiLow,
        Icon::WifiHigh,
        Icon::WifiOff,
        Icon::Battery,
        Icon::BatteryLow,
        Icon::BatteryMedium,
        Icon::BatteryFull,
        Icon::BatteryCharging,
        Icon::BatteryWarning,
        Icon::Bluetooth,
        Icon::BluetoothConnected,
        Icon::BluetoothOff,
        Icon::Volume,
        Icon::Volume1,
        Icon::Volume2,
        Icon::VolumeX,
        Icon::Mic,
        Icon::MicOff,
        Icon::Sun,
        Icon::Moon,
        Icon::SunMoon,
        Icon::Power,
        Icon::Lock,
        Icon::Bell,
        Icon::BellOff,
        Icon::Grid,
        Icon::Window,
        Icon::Monitor,
        Icon::Keyboard,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::ChevronUp,
        Icon::ChevronDown,
        Icon::Folder,
        Icon::File,
        Icon::Image,
        Icon::Terminal,
        Icon::StickyNote,
        Icon::Camera,
        Icon::Download,
        Icon::Upload,
        Icon::Copy,
        Icon::Link,
        Icon::Sparkles,
        Icon::Gauge,
        Icon::Brightness,
    ];

    /// Every glyph: the mailo set, then the shell set.
    pub const ALL: &[Icon] = &[
        Icon::Inbox,
        Icon::Star,
        Icon::Archive,
        Icon::Clock,
        Icon::Trash,
        Icon::Mail,
        Icon::MailOpen,
        Icon::Tag,
        Icon::Refresh,
        Icon::Send,
        Icon::Command,
        Icon::Columns,
        Icon::Group,
        Icon::Panel,
        Icon::Square,
        Icon::Maximize,
        Icon::Corner,
        Icon::Undo,
        Icon::Check,
        Icon::X,
        Icon::Paperclip,
        Icon::Pen,
        Icon::Key,
        Icon::FilePen,
        Icon::OctagonAlert,
        Icon::Pin,
        Icon::Reply,
        Icon::ReplyAll,
        Icon::Forward,
        Icon::Search,
        Icon::Settings,
        Icon::PanelLeft,
        Icon::Plus,
        Icon::Wifi,
        Icon::WifiLow,
        Icon::WifiHigh,
        Icon::WifiOff,
        Icon::Battery,
        Icon::BatteryLow,
        Icon::BatteryMedium,
        Icon::BatteryFull,
        Icon::BatteryCharging,
        Icon::BatteryWarning,
        Icon::Bluetooth,
        Icon::BluetoothConnected,
        Icon::BluetoothOff,
        Icon::Volume,
        Icon::Volume1,
        Icon::Volume2,
        Icon::VolumeX,
        Icon::Mic,
        Icon::MicOff,
        Icon::Sun,
        Icon::Moon,
        Icon::SunMoon,
        Icon::Power,
        Icon::Lock,
        Icon::Bell,
        Icon::BellOff,
        Icon::Grid,
        Icon::Window,
        Icon::Monitor,
        Icon::Keyboard,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::ChevronUp,
        Icon::ChevronDown,
        Icon::Folder,
        Icon::File,
        Icon::Image,
        Icon::Terminal,
        Icon::StickyNote,
        Icon::Camera,
        Icon::Download,
        Icon::Upload,
        Icon::Copy,
        Icon::Link,
        Icon::Sparkles,
        Icon::Gauge,
        Icon::Brightness,
    ];

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
            shell => geometry_shell::shapes(shell),
        }
    }
}
