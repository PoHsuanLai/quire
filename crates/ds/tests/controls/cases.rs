//! Every control in every state, as data: the table the golden test walks.

use dioxus::prelude::*;
use ds::components::vocab::{Availability, Fraction, Key, PulseKey, Shortcut, Switch};
use ds::{
    Anim, Avatar, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Button, ButtonSize,
    ButtonVariant, Chip, ChipVariant, Colour, Count, CountPlace, ExternalIcon, Focus, HeaderKind,
    Hex, Icon, IconButton, IconButtonVariant, IconPx, IconSize, IconSource, IconUrl, IconView,
    InputVariant, Kbd, KbdSize, LabelHue, PersonHue, SearchField, SectionHeader, SegSize,
    SegmentedControl, Slider, Spinner, SpinnerKind, Tabs, TextInput, Toggle, Verdict,
};

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
        make: || rsx! { Button { variant: ButtonVariant::Primary, label: "Send", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "primary-icon",
        make: || rsx! { Button { variant: ButtonVariant::Primary, label: "Send", icon: Icon::Send, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "secondary",
        make: || rsx! { Button { variant: ButtonVariant::Secondary, label: "Cancel", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "mini",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "Reply", icon: Icon::Reply, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "mini-pressed",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "Yes", pressed: Switch::On, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "mini-unpressed",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "No", pressed: Switch::Off, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "quiet",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "Show details", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "danger",
        make: || rsx! { Button { variant: ButtonVariant::Danger, label: "Delete", icon: Icon::Trash, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "disabled",
        make: || rsx! { Button { variant: ButtonVariant::Primary, label: "Send", availability: Availability::Disabled, onclick: |_| {} } },
    },
    // Sheet and modal parts (sill Q92, Q93): a size apart from the variant, and a disabled
    // Danger at Regular (a power menu's unavailable Suspend).
    Case {
        component: "button",
        state: "danger-regular",
        make: || rsx! { Button { variant: ButtonVariant::Danger, size: ButtonSize::Regular, label: "Restart", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "primary-mini",
        make: || rsx! { Button { variant: ButtonVariant::Primary, size: ButtonSize::Mini, label: "Send", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "danger-regular-disabled",
        make: || rsx! { Button { variant: ButtonVariant::Danger, size: ButtonSize::Regular, label: "Suspend", availability: Availability::Disabled, onclick: |_| {} } },
    },
    // IconButton: four variants, expanded, pressed, tooltip, disabled.
    Case {
        component: "icon_button",
        state: "tool",
        make: || rsx! { IconButton { variant: IconButtonVariant::Tool, icon: Icon::Archive, label: "Archive", onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "tool-expanded",
        make: || rsx! { IconButton { variant: IconButtonVariant::Tool, icon: Icon::Tag, label: "Labels", expanded: Switch::On, onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "tool-collapsed",
        make: || rsx! { IconButton { variant: IconButtonVariant::Tool, icon: Icon::Tag, label: "Labels", expanded: Switch::Off, onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "foot",
        make: || rsx! { IconButton { variant: IconButtonVariant::Foot, icon: Icon::PanelLeft, label: "Hide the sidebar", tooltip: "Hide the sidebar (Ctrl S)".to_string(), onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "strip",
        make: || rsx! { IconButton { variant: IconButtonVariant::Strip, icon: Icon::Clock, label: "Snooze", onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "pin",
        make: || rsx! { IconButton { variant: IconButtonVariant::Pin, icon: Icon::Inbox, label: "Inbox", pressed: Switch::Off, onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "pin-pressed",
        make: || rsx! { IconButton { variant: IconButtonVariant::Pin, icon: Icon::Inbox, label: "Inbox", pressed: Switch::On, onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "status",
        make: || rsx! { IconButton { variant: IconButtonVariant::Status, icon: Icon::Ethernet, label: "Wired network", id: "net".to_string(), onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "status-open",
        make: || rsx! { IconButton { variant: IconButtonVariant::Status, icon: Icon::BatteryCharging, label: "Battery", expanded: Switch::On, onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "disabled",
        make: || rsx! { IconButton { variant: IconButtonVariant::Tool, icon: Icon::Trash, label: "Delete", availability: Availability::Disabled, onclick: |_| {} } },
    },
    // IconButton with an external icon (quire gap Q6) and an element id (Q8).
    Case {
        component: "icon_button",
        state: "symbolic",
        make: || rsx! { IconButton { variant: IconButtonVariant::Tool, icon: symbolic(), label: "Input method", onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "image",
        make: || rsx! { IconButton { variant: IconButtonVariant::Tool, icon: image(), label: "Status", onclick: |_| {} } },
    },
    Case {
        component: "icon_button",
        state: "with-id",
        make: || rsx! { IconButton { variant: IconButtonVariant::Tool, icon: Icon::Star, label: "Tray item", id: "tray-0", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "with-id-symbolic",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "Updates", icon: symbolic(), id: "updates", onclick: |_| {} } },
    },
    // IconView: a glyph, a symbolic icon, an image.
    Case {
        component: "icon_view",
        state: "glyph",
        make: || rsx! { IconView { source: Icon::Bell.into(), size: IconSize::Bar } },
    },
    // The action glyphs (FINDINGS "mailo gaps"): Lucide printer and folder-input.
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
    // The dock's sizes (sill FINDINGS Q16): a glyph at a tile's full magnification, an image at
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
        make: || rsx! { SegmentedControl { label: "Appearance", options: options(), value: 0u8, onchange: |_: u8| {} } },
    },
    Case {
        component: "segmented",
        state: "small",
        make: || rsx! { SegmentedControl { label: "View", options: options(), value: 2u8, size: SegSize::Small, onchange: |_: u8| {} } },
    },
    // Toggle: off, on, disabled.
    Case {
        component: "toggle",
        state: "off",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Switch::Off, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "on",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Switch::On, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "disabled",
        make: || rsx! { Toggle { label: "Bluetooth", value: Switch::Off, availability: Availability::Disabled, onchange: |_| {} } },
    },
    // TextInput: both variants, placeholder shown and hidden, disabled.
    Case {
        component: "text_input",
        state: "boxed-empty",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, label: "To", value: "", placeholder: "Add a person", oninput: |_| {} } },
    },
    Case {
        component: "text_input",
        state: "boxed-filled",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, label: "To", value: "dana@", placeholder: "Add a person", oninput: |_| {} } },
    },
    Case {
        component: "text_input",
        state: "inline-empty",
        make: || rsx! { TextInput { variant: InputVariant::Inline, label: "Link", value: "", placeholder: "Paste a link", oninput: |_| {} } },
    },
    Case {
        component: "text_input",
        state: "no-placeholder",
        make: || rsx! { TextInput { variant: InputVariant::Inline, label: "Name", value: "", oninput: |_| {} } },
    },
    Case {
        component: "text_input",
        state: "disabled",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, label: "Name", value: "Dana", availability: Availability::Disabled, oninput: |_| {} } },
    },
    Case {
        component: "text_input",
        state: "focus-on-mount",
        make: || rsx! { TextInput { variant: InputVariant::Inline, label: "Link", value: "", placeholder: "Paste a link", focus: Focus::OnMount, oninput: |_| {} } },
    },
    // SearchField: empty, and typed with tokens.
    Case {
        component: "search_field",
        state: "empty",
        make: || rsx! { SearchField { label: "Search", value: "", placeholder: "Search mail, people, actions", tokens: Vec::new(), oninput: |_| {}, onkey: |_| {} } },
    },
    Case {
        component: "search_field",
        state: "tokens",
        make: || rsx! { SearchField { label: "Search", value: "uidl", placeholder: "Search mail, people, actions", tokens: vec!["from dana".to_string(), "has:attachment".to_string()], oninput: |_| {}, onkey: |_| {} } },
    },
    // Kbd: both sizes, every modifier.
    Case {
        component: "kbd",
        state: "regular",
        make: || rsx! { Kbd { shortcut: Shortcut(vec![Key::Ctrl, Key::Char('t')]) } },
    },
    Case {
        component: "kbd",
        state: "small",
        make: || rsx! { Kbd { shortcut: Shortcut(vec![Key::Ctrl, Key::Char('k')]), size: KbdSize::Small } },
    },
    Case {
        component: "kbd",
        state: "modifiers",
        make: || rsx! { Kbd { shortcut: Shortcut(vec![Key::Super, Key::Shift, Key::Alt, Key::Ctrl, Key::Enter]) } },
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
    Case {
        component: "chip",
        state: "pulse-rest",
        make: || rsx! { Chip { variant: ChipVariant::Accent, text: "spec", pulse: PulseKey::rest(Anim::ChipLand) } },
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
    // Tabs: each tab selected in turn.
    Case {
        component: "tabs",
        state: "first",
        make: || rsx! { Tabs { label: "Sections", tabs: vec![(1u8, "Inbox".to_string()), (2u8, "Motion catalogue".to_string())], value: 1u8, onchange: |_: u8| {} } },
    },
    Case {
        component: "tabs",
        state: "second",
        make: || rsx! { Tabs { label: "Sections", tabs: vec![(1u8, "Inbox".to_string()), (2u8, "Motion catalogue".to_string())], value: 2u8, onchange: |_: u8| {} } },
    },
    // SectionHeader: four kinds, the value and the action.
    Case {
        component: "section_header",
        state: "frame",
        make: || rsx! { SectionHeader { kind: HeaderKind::Frame, text: "Places" } },
    },
    Case {
        component: "section_header",
        state: "frame-action",
        make: || rsx! { SectionHeader { kind: HeaderKind::Frame, text: "Today", action: ("Clear".to_string(), EventHandler::new(|_| {})) } },
    },
    Case {
        component: "section_header",
        state: "group",
        make: || rsx! { SectionHeader { kind: HeaderKind::Group, text: "Yesterday", value: "35".to_string() } },
    },
    Case {
        component: "section_header",
        state: "field",
        make: || rsx! { SectionHeader { kind: HeaderKind::Field, text: "Grain", value: "35%".to_string() } },
    },
    Case {
        component: "section_header",
        state: "menu",
        make: || rsx! { SectionHeader { kind: HeaderKind::Menu, text: "Snooze until" } },
    },
    // Count: both places, empty at zero.
    Case {
        component: "count",
        state: "item",
        make: || rsx! { Count { value: 4 } },
    },
    Case {
        component: "count",
        state: "tile",
        make: || rsx! { Count { value: 12, place: CountPlace::Tile } },
    },
    Case {
        component: "count",
        state: "zero",
        make: || rsx! { Count { value: 0 } },
    },
    // Spinner: both kinds.
    Case {
        component: "spinner",
        state: "spin",
        make: || rsx! { Spinner { kind: SpinnerKind::Spin } },
    },
    Case {
        component: "spinner",
        state: "breathe",
        make: || rsx! { Spinner { kind: SpinnerKind::Breathe } },
    },
];

/// Cases driven by the motion module: the slider's drag tracker and the pulse classes. The
/// count's bump needs a second render, so it is `components_controls::a_count_bumps_on_change`.
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
    Case {
        component: "chip",
        state: "landing",
        make: || rsx! { Chip { variant: ChipVariant::Label(LabelHue::Amber), text: "rust", pulse: PulseKey::rest(Anim::ChipLand).fired() } },
    },
    Case {
        component: "chip",
        state: "person-flash",
        make: || rsx! { Chip { variant: ChipVariant::Person(DANA), text: "Dana Okafor", pulse: PulseKey::rest(Anim::ChipFlash).fired() } },
    },
];
