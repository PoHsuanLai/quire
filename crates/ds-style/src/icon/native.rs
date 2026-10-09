//! The owner's per-glyph style picks (2026-10-09, design/08-ICONS.md section 1.2): how each
//! [`Icon`] is drawn when a caller does not say. One row per glyph, in `Icon`'s own order; a
//! test walks `Icon::ALL` and fails on a glyph with no row.

use super::Icon;
use super::drawn;
use super::shape::Shape;

/// How an icon is drawn by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Native {
    /// The filled form.
    Solid,
    /// Lucide's stroked form.
    Outline,
    /// A pair's half: outline while off, filled while on (`GlyphStyle::of`).
    Pair,
    /// Its own filled geometry, in place of both the solid and the outline form (`drawn`).
    Drawn(&'static [Shape]),
}

impl Icon {
    /// How this icon is drawn when nothing asks for a style.
    pub fn native(self) -> Native {
        TABLE
            .iter()
            .find(|(icon, _)| *icon == self)
            .map_or(Native::Outline, |&(_, native)| native)
    }
}

/// Every icon's default.
pub(super) const TABLE: &[(Icon, Native)] = &[
    (Icon::Inbox, Native::Outline),
    (Icon::Star, Native::Pair),
    (Icon::Archive, Native::Outline),
    (Icon::Clock, Native::Outline),
    (Icon::Trash, Native::Outline),
    (Icon::Mail, Native::Outline),
    (Icon::MailOpen, Native::Outline),
    (Icon::Tag, Native::Outline),
    (Icon::Refresh, Native::Solid),
    (Icon::Send, Native::Outline),
    (Icon::Command, Native::Outline),
    (Icon::Columns, Native::Outline),
    (Icon::Group, Native::Outline),
    (Icon::Panel, Native::Outline),
    (Icon::Square, Native::Outline),
    (Icon::Maximize, Native::Outline),
    (Icon::Corner, Native::Outline),
    (Icon::Undo, Native::Outline),
    (Icon::Check, Native::Outline),
    (Icon::X, Native::Outline),
    (Icon::Paperclip, Native::Outline),
    (Icon::Pen, Native::Solid),
    (Icon::Key, Native::Solid),
    (Icon::FilePen, Native::Outline),
    (Icon::OctagonAlert, Native::Outline),
    (Icon::Pin, Native::Pair),
    (Icon::Reply, Native::Outline),
    (Icon::ReplyAll, Native::Outline),
    (Icon::Forward, Native::Outline),
    (Icon::Search, Native::Outline),
    (Icon::Settings, Native::Outline),
    (Icon::PanelLeft, Native::Outline),
    (Icon::Plus, Native::Outline),
    (Icon::Minus, Native::Outline),
    (Icon::Wifi, Native::Outline),
    (Icon::WifiLow, Native::Outline),
    (Icon::WifiHigh, Native::Outline),
    (Icon::WifiOff, Native::Outline),
    (Icon::Ethernet, Native::Outline),
    (Icon::Battery, Native::Solid),
    (Icon::BatteryLow, Native::Solid),
    (Icon::BatteryMedium, Native::Solid),
    (Icon::BatteryFull, Native::Solid),
    (Icon::BatteryCharging, Native::Solid),
    (Icon::BatteryWarning, Native::Outline),
    (Icon::Bluetooth, Native::Outline),
    (Icon::BluetoothConnected, Native::Outline),
    (Icon::BluetoothOff, Native::Outline),
    (Icon::Volume, Native::Outline),
    (Icon::Volume1, Native::Outline),
    (Icon::Volume2, Native::Outline),
    (Icon::VolumeX, Native::Outline),
    (Icon::Mic, Native::Outline),
    (Icon::MicOff, Native::Outline),
    (Icon::Sun, Native::Outline),
    (Icon::Moon, Native::Outline),
    (Icon::SunMoon, Native::Outline),
    (Icon::Power, Native::Outline),
    (Icon::Lock, Native::Outline),
    (Icon::Bell, Native::Pair),
    (Icon::BellOff, Native::Outline),
    (Icon::Grid, Native::Outline),
    (Icon::Window, Native::Outline),
    (Icon::Monitor, Native::Outline),
    (Icon::Keyboard, Native::Outline),
    (Icon::ChevronLeft, Native::Solid),
    (Icon::ChevronRight, Native::Solid),
    (Icon::ChevronUp, Native::Solid),
    (Icon::ChevronDown, Native::Solid),
    (Icon::ChevronsUpDown, Native::Outline),
    (Icon::Folder, Native::Outline),
    (Icon::File, Native::Outline),
    (Icon::Image, Native::Outline),
    (Icon::Terminal, Native::Solid),
    (Icon::StickyNote, Native::Outline),
    (Icon::Camera, Native::Outline),
    (Icon::Download, Native::Drawn(drawn::DOWNLOAD)),
    (Icon::Upload, Native::Drawn(drawn::UPLOAD)),
    (Icon::Copy, Native::Outline),
    (Icon::Link, Native::Outline),
    (Icon::Sparkles, Native::Drawn(drawn::SPARKLES)),
    (Icon::Gauge, Native::Outline),
    (Icon::Brightness, Native::Solid),
    (Icon::Printer, Native::Outline),
    (Icon::FolderInput, Native::Outline),
    (Icon::Play, Native::Solid),
    (Icon::Pause, Native::Solid),
    (Icon::SkipBack, Native::Outline),
    (Icon::SkipForward, Native::Outline),
    (Icon::LogOut, Native::Outline),
    (Icon::Restart, Native::Solid),
    (Icon::RotateLeft, Native::Outline),
    (Icon::RotateRight, Native::Outline),
    (Icon::Music, Native::Outline),
    (Icon::Headphones, Native::Outline),
    (Icon::Speaker, Native::Outline),
    (Icon::Mouse, Native::Outline),
    (Icon::Gamepad, Native::Outline),
    (Icon::Phone, Native::Outline),
    (Icon::Ellipsis, Native::Outline),
    (Icon::EllipsisVertical, Native::Outline),
    (Icon::Switches, Native::Outline),
    (Icon::ArrowRight, Native::Outline),
    (Icon::CapsLock, Native::Outline),
    (Icon::Clipboard, Native::Outline),
    (Icon::Smile, Native::Outline),
    (Icon::Globe, Native::Outline),
    (Icon::MoonFilled, Native::Outline),
    (Icon::Bold, Native::Outline),
    (Icon::Italic, Native::Outline),
    (Icon::Underline, Native::Outline),
    (Icon::Strike, Native::Outline),
    (Icon::Code, Native::Outline),
    (Icon::Info, Native::Outline),
    (Icon::CircleCheck, Native::Outline),
    (Icon::TriangleAlert, Native::Outline),
    (Icon::Heart, Native::Pair),
];
