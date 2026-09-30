//! Every control in every state, as data: the table the golden test walks.

use dioxus::prelude::*;
use ds::detail::{Operation, PendingToken};
use ds::{
    Answers, Availability, Bezel, ButtonRole, Check, ControlSize, Fraction, ImagePosition,
    Progress, ProgressIndicator, ProgressStyle, Shortcut, ShortcutKey, Shown,
};
use ds::{
    Avatar, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Button, Chip, ChipVariant, Colour,
    Common, ExternalIcon, FieldFocus, Hex, Icon, IconPx, IconSize, IconSource, IconUrl, IconView,
    LabelHue, PersonHue, SectionHeader, SegmentedControl, Slider, Toggle, Verdict,
};
use ds::{Badge, BadgeContent, BadgeTone, KeyEquivalent, KeyStyle};
use ds::{Choice, Tracking};
use ds::{FieldBezel, FieldKind, TextField};

/// A symbolic SVG, 16 px.
fn symbolic() -> IconSource {
    IconSource::Symbolic(ExternalIcon {
        url: IconUrl::svg(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'><circle cx='8' cy='8' r='6' fill='#bebebe'/></svg>",
        ),
        size: IconSize::Base,
    })
}

/// A PNG pixmap, 22 px (its bytes stand in: the golden is the markup, not the picture).
fn image() -> IconSource {
    IconSource::Image(ExternalIcon {
        url: IconUrl::png(b"\x89PNG"),
        size: IconSize::Bar,
    })
}

/// One component in one state.
pub struct Case {
    pub component: &'static str,
    pub state: &'static str,
    pub make: fn() -> Element,
}

fn options() -> Vec<(u8, String)> {
    ["System", "Light", "Dark"]
        .iter()
        .enumerate()
        .map(|(i, text)| (u8::try_from(i).unwrap_or_default(), (*text).to_string()))
        .collect()
}

const DANA: AvatarFace = AvatarFace {
    initial: 'D',
    size: AvatarSize::Size18,
    tone: AvatarTone::Person(PersonHue(212)),
    shape: AvatarShape::Round,
};

pub const CASES: &[Case] = &[
    // Button: five variants, an icon, the toggle Mini, disabled.
    Case {
        component: "button",
        state: "primary",
        make: || rsx! { Button { answers: Answers::Return, label: "Send", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "primary-icon",
        make: || rsx! { Button { answers: Answers::Return, label: "Send", icon: Icon::Send, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "secondary",
        make: || rsx! { Button { label: "Cancel", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "mini",
        make: || rsx! { Button { size: ControlSize::Mini, label: "Reply", icon: Icon::Reply, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "mini-pressed",
        make: || rsx! { Button { size: ControlSize::Mini, label: "Yes", value: Check::On, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "mini-unpressed",
        make: || rsx! { Button { size: ControlSize::Mini, label: "No", value: Check::Off, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "quiet",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "Show details", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "danger",
        make: || rsx! { Button { role: ButtonRole::Destructive, size: ControlSize::Mini, label: "Delete", icon: Icon::Trash, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "disabled",
        make: || rsx! { Button { answers: Answers::Return, label: "Send", availability: Availability::Disabled, onclick: |_| {} } },
    },
    // Sheet and modal parts: a size apart from the variant, and a disabled
    // Danger at Regular (a power menu's unavailable Suspend).
    Case {
        component: "button",
        state: "danger-regular",
        make: || rsx! { Button { role: ButtonRole::Destructive, size: ControlSize::Regular, label: "Restart", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "primary-mini",
        make: || rsx! { Button { answers: Answers::Return, size: ControlSize::Mini, label: "Send", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "danger-regular-disabled",
        make: || rsx! { Button { role: ButtonRole::Destructive, size: ControlSize::Regular, label: "Suspend", availability: Availability::Disabled, onclick: |_| {} } },
    },
    // Toolbar buttons with only an image: four variants, expanded, pressed, tooltip, disabled.
    Case {
        component: "button",
        state: "toolbar-tool",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Archive, label: "Archive", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-tool-expanded",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Tag, label: "Labels", shown: Shown::Visible, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-tool-collapsed",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Tag, label: "Labels", shown: Shown::Hidden, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-foot",
        make: || rsx! { Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::PanelLeft, label: "Hide the sidebar", title: "Hide the sidebar (Ctrl S)".to_string(), onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-strip",
        make: || rsx! { Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::Clock, label: "Snooze", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-pin",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Inbox, label: "Inbox", value: Check::Off, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-pin-pressed",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Inbox, label: "Inbox", value: Check::On, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-status",
        make: || rsx! { Button { common: Common { id: Some("net".to_string()), ..Common::default() }, bezel: Bezel::StatusItem, image: ImagePosition::Only, icon: Icon::Ethernet, label: "Wired network", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-status-open",
        make: || rsx! { Button { bezel: Bezel::StatusItem, image: ImagePosition::Only, icon: Icon::BatteryCharging, label: "Battery", shown: Shown::Visible, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-disabled",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Trash, label: "Delete", availability: Availability::Disabled, onclick: |_| {} } },
    },
    // A toolbar button with an external icon and an element id.
    Case {
        component: "button",
        state: "toolbar-symbolic",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: symbolic(), label: "Input method", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-image",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: image(), label: "Status", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-with-id",
        make: || rsx! { Button { common: Common { id: Some("tray-0".to_string()), ..Common::default() }, bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, icon: Icon::Star, label: "Tray item", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "with-id-symbolic",
        make: || rsx! { Button { common: Common { id: Some("updates".to_string()), ..Common::default() }, size: ControlSize::Mini, label: "Updates", icon: symbolic(), onclick: |_| {} } },
    },
    // IconView: a glyph, a symbolic icon, an image.
    Case {
        component: "icon_view",
        state: "glyph",
        make: || rsx! { IconView { source: Icon::Bell.into(), size: IconSize::Bar } },
    },
    // The action glyphs: Lucide printer and folder-input.
    Case {
        component: "icon_view",
        state: "printer",
        make: || rsx! { IconView { source: Icon::Printer.into() } },
    },
    Case {
        component: "icon_view",
        state: "folder-input",
        make: || rsx! { IconView { source: Icon::FolderInput.into() } },
    },
    Case {
        component: "icon_view",
        state: "symbolic",
        make: || rsx! { IconView { source: symbolic() } },
    },
    Case {
        component: "icon_view",
        state: "image",
        make: || rsx! { IconView { source: image() } },
    },
    // The dock's sizes: a glyph at a tile's full magnification, an image at
    // a size the caller resolved.
    Case {
        component: "icon_view",
        state: "glyph-tile96",
        make: || rsx! { IconView { source: Icon::Folder.into(), size: IconSize::Tile96 } },
    },
    Case {
        component: "icon_view",
        state: "image-px",
        make: || rsx! { IconView { source: IconSource::Image(ExternalIcon { url: IconUrl::png(b"\x89PNG"), size: IconSize::Px(IconPx(71)) }) } },
    },
    // SegmentedControl: both sizes, each with a different choice pressed.
    Case {
        component: "segmented",
        state: "regular",
        make: || rsx! { SegmentedControl { label: "Appearance", choices: Choice::pairs(options()), tracking: Tracking::SelectOne(0u8), onchange: |_: u8| {} } },
    },
    Case {
        component: "segmented",
        state: "mini",
        make: || rsx! { SegmentedControl { label: "View", choices: Choice::pairs(options()), tracking: Tracking::SelectOne(2u8), size: ControlSize::Mini, onchange: |_: u8| {} } },
    },
    // Toggle: off, on, disabled.
    Case {
        component: "toggle",
        state: "off",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Check::Off, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "on",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Check::On, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "disabled",
        make: || rsx! { Toggle { label: "Bluetooth", value: Check::Off, availability: Availability::Disabled, onchange: |_| {} } },
    },
    // TextField: both bezels, placeholder shown and hidden, disabled.
    Case {
        component: "text_field",
        state: "bezeled-empty",
        make: || rsx! { TextField { label: "To", value: "", placeholder: "Add a person", oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "bezeled-filled",
        make: || rsx! { TextField { label: "To", value: "dana@", placeholder: "Add a person", oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "plain-empty",
        make: || rsx! { TextField { bezel: FieldBezel::Plain, label: "Link", value: "", placeholder: "Paste a link", oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "no-placeholder",
        make: || rsx! { TextField { bezel: FieldBezel::Plain, label: "Name", value: "", oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "disabled",
        make: || rsx! { TextField { label: "Name", value: "Dana", availability: Availability::Disabled, oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "focus-on-mount",
        make: || rsx! { TextField { bezel: FieldBezel::Plain, label: "Link", value: "", placeholder: "Paste a link", focus: FieldFocus::OnMount, oninput: |_| {} } },
    },
    // A search field: empty, and typed with tokens.
    Case {
        component: "text_field",
        state: "search-empty",
        make: || rsx! { TextField { label: "Search", value: "", placeholder: "Search mail, people, actions", tokens: Vec::new(), oninput: |_| {}, onkey: |_| {} , kind: FieldKind::Search, bezel: FieldBezel::Plain} },
    },
    Case {
        component: "text_field",
        state: "search-tokens",
        make: || rsx! { TextField { label: "Search", value: "uidl", placeholder: "Search mail, people, actions", tokens: vec!["from dana".to_string(), "has:attachment".to_string()], oninput: |_| {}, onkey: |_| {} , kind: FieldKind::Search, bezel: FieldBezel::Plain} },
    },
    // Key caps: each size, every modifier.
    Case {
        component: "key_equivalent",
        state: "regular",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')]) , style: KeyStyle::Cap} },
    },
    Case {
        component: "key_equivalent",
        state: "small",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Ctrl, ShortcutKey::Super, ShortcutKey::Char('s')]), style: KeyStyle::Cap, size: ControlSize::Mini} },
    },
    Case {
        component: "key_equivalent",
        state: "modifiers",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Shift, ShortcutKey::Alt, ShortcutKey::Ctrl, ShortcutKey::Enter]) , style: KeyStyle::Cap} },
    },
    // Chip: every variant, the removable person, a pulse at rest.
    Case {
        component: "chip",
        state: "accent",
        make: || rsx! { Chip { variant: ChipVariant::Accent, text: "spec" } },
    },
    Case {
        component: "chip",
        state: "label",
        make: || rsx! { Chip { variant: ChipVariant::Label(LabelHue::Blue), text: "spec" } },
    },
    Case {
        component: "chip",
        state: "neutral",
        make: || rsx! { Chip { variant: ChipVariant::Neutral, text: "draft" } },
    },
    Case {
        component: "chip",
        state: "token",
        make: || rsx! { Chip { variant: ChipVariant::Token, text: "from dana" } },
    },
    Case {
        component: "chip",
        state: "status-ok",
        make: || rsx! { Chip { variant: ChipVariant::Status(Verdict::Pass), text: "4.8:1" } },
    },
    Case {
        component: "chip",
        state: "status-bad",
        make: || rsx! { Chip { variant: ChipVariant::Status(Verdict::Fail), text: "2.1:1" } },
    },
    Case {
        component: "chip",
        state: "person",
        make: || rsx! { Chip { variant: ChipVariant::Person(DANA), text: "Dana Okafor" } },
    },
    Case {
        component: "chip",
        state: "person-removable",
        make: || rsx! { Chip { variant: ChipVariant::Person(DANA), text: "Dana Okafor", onremove: |_| {} } },
    },
    // Avatar: every tone, both shapes, the sizes with special rules.
    Case {
        component: "avatar",
        state: "ink-28",
        make: || rsx! { Avatar { initial: 'D', size: AvatarSize::Size28, tone: AvatarTone::Ink } },
    },
    Case {
        component: "avatar",
        state: "account-28",
        make: || rsx! { Avatar { initial: 'W', size: AvatarSize::Size28, tone: AvatarTone::Account(Colour::Solid(Hex([0x5b, 0x4f, 0xc4]))) } },
    },
    Case {
        component: "avatar",
        state: "person-18",
        make: || rsx! { Avatar { initial: 'D', size: AvatarSize::Size18, tone: AvatarTone::Person(PersonHue::of("dana@example.org")) } },
    },
    Case {
        component: "avatar",
        state: "stack-20",
        make: || rsx! { Avatar { initial: 'M', size: AvatarSize::Size20, tone: AvatarTone::Stack } },
    },
    Case {
        component: "avatar",
        state: "square-16",
        make: || rsx! { Avatar { initial: 'G', size: AvatarSize::Size16, tone: AvatarTone::Person(PersonHue(30)), shape: AvatarShape::Square } },
    },
    Case {
        component: "avatar",
        state: "square-34",
        make: || rsx! { Avatar { initial: 'L', size: AvatarSize::Size34, tone: AvatarTone::Ink, shape: AvatarShape::Square } },
    },
    // SectionHeader: one look, with a value, an action, and collapsing.
    Case {
        component: "section_header",
        state: "title",
        make: || rsx! { SectionHeader { title: "Places" } },
    },
    Case {
        component: "section_header",
        state: "value",
        make: || rsx! { SectionHeader { title: "Grain", value: "35%".to_string() } },
    },
    Case {
        component: "section_header",
        state: "action",
        make: || rsx! { SectionHeader { title: "Today", action: ("Clear".to_string(), EventHandler::new(|_| {})) } },
    },
    Case {
        component: "section_header",
        state: "action-selected",
        make: || rsx! { SectionHeader { title: "Files", action: ("Show More".to_string(), EventHandler::new(|_| {})), action_selection: ds::Selection::Selected } },
    },
    Case {
        component: "section_header",
        state: "collapsible-open",
        make: || rsx! { SectionHeader { title: "Favourites", collapse: (ds::Shown::Visible, EventHandler::new(|_| {})) } },
    },
    Case {
        component: "section_header",
        state: "collapsible-closed",
        make: || rsx! { SectionHeader { title: "Favourites", collapse: (ds::Shown::Hidden, EventHandler::new(|_| {})) } },
    },
    // Count: both places, empty at zero.
    Case {
        component: "badge",
        state: "item",
        make: || rsx! { Badge { content: BadgeContent::Number(4) , tone: BadgeTone::Quiet, size: ControlSize::Mini} },
    },
    Case {
        component: "badge",
        state: "tile",
        make: || rsx! { Badge { content: BadgeContent::Number(12), tone: BadgeTone::Alert, size: ControlSize::Mini} },
    },
    Case {
        component: "badge",
        state: "zero",
        make: || rsx! { Badge { content: BadgeContent::Number(0) , tone: BadgeTone::Quiet, size: ControlSize::Mini} },
    },
    // Spinner.
    Case {
        component: "progress",
        state: "spinner",
        make: || rsx! { ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(running()), size: ControlSize::Regular } },
    },
];

/// An operation that has just started: a spinner's first frame is drawn before its timer runs, so
/// it draws idle.
fn running() -> Operation {
    Operation::Running(PendingToken::start())
}

/// Cases driven by the motion module: the slider's drag tracker.
pub const MOTION_CASES: &[Case] = &[
    Case {
        component: "slider",
        state: "default",
        make: || rsx! { Slider { label: "Grain", value: Fraction(350), onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "empty",
        make: || rsx! { Slider { label: "Volume", value: Fraction(0), onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "full",
        make: || rsx! { Slider { label: "Volume", value: Fraction(1000), onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "clamped",
        make: || rsx! { Slider { label: "Gain", value: Fraction(1400), step: Fraction(10), onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "disabled",
        make: || rsx! { Slider { label: "Brightness", value: Fraction(500), availability: Availability::Disabled, onchange: |_| {} } },
    },
];
