//! The catalogue's controls and fields in every state they can express (design/30 section 2), as
//! data: the table the golden test walks beside `cases.rs`. One `Case` is one component in one
//! state; its golden is `tests/snapshots/controls/<component>/<state>.html`.

use super::cases::Case;
use dioxus::prelude::*;
use ds::components::content::label::{LabelRole, LabelStyle};
use ds::components::content::level_glyph::vocab::LevelGlyph;
use ds::components::content::text_runs::RunTone;
use ds::components::controls::badge::{Badge, BadgeContent, BadgeTone};
use ds::components::controls::button_model::{Answers, Bezel, ButtonRole, IconSwap, ImagePosition};
use ds::components::controls::checkbox::Checkbox;
use ds::components::controls::key_equivalent::{KeyEquivalent, KeyStyle};
use ds::components::controls::level_indicator::{Bands, LevelIndicator, LevelStyle};
use ds::components::controls::progress::model::{Progress, ProgressStyle};
use ds::components::controls::progress::view::ProgressIndicator;
use ds::components::controls::radio_group::Arrangement;
use ds::components::controls::segmented::Tracking;
use ds::components::controls::slider_model::{SliderLook, Ticks};
use ds::components::fields::text_field_model::Invalid;
use ds::detail::{EventStamp, Operation, PendingToken};
use ds::prelude::*;
use ds_core::vocab::Muting;
use ds_style::tokens::control_size::ControlSize;

/// An operation that has just started: a spinner's first frame is drawn before its timer runs, so
/// it draws at rest.
fn running() -> Operation {
    Operation::Running(PendingToken::start())
}

fn words() -> Vec<Choice<u8>> {
    Choice::pairs([(0u8, "System"), (1, "Light"), (2, "Dark")])
}

fn with_images() -> Vec<Choice<u8>> {
    vec![
        Choice::new(0u8, "Grid").with_icon(Icon::Grid),
        Choice::new(1u8, "List").with_icon(Icon::Columns),
        Choice::new(2u8, "Panel")
            .with_icon(Icon::Panel)
            .with_availability(Availability::Disabled),
    ]
}

fn rejected() -> Validity {
    Validity::Invalid(Invalid {
        message: TextLine::from("That address is not valid."),
        stamp: EventStamp(1),
    })
}

pub const CASES: &[Case] = &[
    // Label: each role, each step, disabled, runs.
    Case {
        component: "label",
        state: "primary",
        make: || rsx! { Label { text: "Primary" } },
    },
    Case {
        component: "label",
        state: "secondary",
        make: || rsx! { Label { text: "Secondary", role: LabelRole::Secondary } },
    },
    Case {
        component: "label",
        state: "tertiary",
        make: || rsx! { Label { text: "Tertiary", role: LabelRole::Tertiary } },
    },
    Case {
        component: "label",
        state: "quaternary",
        make: || rsx! { Label { text: "Quaternary", role: LabelRole::Quaternary } },
    },
    Case {
        component: "label",
        state: "caption",
        make: || rsx! { Label { text: "Caption", style: LabelStyle::Caption } },
    },
    Case {
        component: "label",
        state: "footnote",
        make: || rsx! { Label { text: "Footnote", style: LabelStyle::Footnote } },
    },
    Case {
        component: "label",
        state: "headline",
        make: || rsx! { Label { text: "Headline", style: LabelStyle::Headline } },
    },
    Case {
        component: "label",
        state: "title",
        make: || rsx! { Label { text: "Title", style: LabelStyle::Title } },
    },
    Case {
        component: "label",
        state: "display",
        make: || rsx! { Label { text: "42", style: LabelStyle::Display } },
    },
    Case {
        component: "label",
        state: "disabled",
        make: || rsx! { Label { text: "Disabled", availability: Availability::Disabled } },
    },
    Case {
        component: "label",
        state: "runs",
        make: || {
            rsx! {
                Label { text: TextLine::Runs(vec![
                    TextRun::new("Dana Okafor", RunTone::Strong),
                    TextRun::new(" wrote on Tue", RunTone::Faint),
                ]) }
            }
        },
    },
    // Button: the bezels at each size, the roles, the states.
    Case {
        component: "button",
        state: "push-mini",
        make: || rsx! { Button { size: ControlSize::Mini, label: "Mini", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "push-small",
        make: || rsx! { Button { size: ControlSize::Small, label: "Small", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "push-regular",
        make: || rsx! { Button { label: "Regular", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "push-large",
        make: || rsx! { Button { size: ControlSize::Large, label: "Large", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "default-return",
        make: || rsx! { Button { answers: Answers::Return, label: "Send", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "cancel-escape",
        make: || rsx! { Button { answers: Answers::Escape, label: "Cancel", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "destructive",
        make: || rsx! { Button { role: ButtonRole::Destructive, label: "Delete", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "destructive-default",
        make: || rsx! { Button { role: ButtonRole::Destructive, answers: Answers::Return, label: "Erase", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "busy",
        make: || rsx! { Button { label: "Sending", availability: Availability::Busy, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "default-busy",
        make: || rsx! { Button { answers: Answers::Return, label: "Sending", availability: Availability::Busy, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toggle-on",
        make: || rsx! { Button { label: "Bold", value: Check::On, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toggle-mixed",
        make: || rsx! { Button { label: "Bold", value: Check::Mixed, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "inline-regular",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "Show details", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "inline-destructive",
        make: || rsx! { Button { bezel: Bezel::Inline, role: ButtonRole::Destructive, label: "Remove", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "help",
        make: || rsx! { Button { bezel: Bezel::Help, label: "Help", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-titled",
        make: || rsx! { Button { bezel: Bezel::Toolbar, label: "Settings", icon: Icon::Settings, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-image-mini",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Mini, image: ImagePosition::Only, icon: Icon::Star, label: "Star", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-image-small",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Small, image: ImagePosition::Only, icon: Icon::Star, label: "Star", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-image-busy",
        make: || rsx! { Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::Refresh, label: "Refresh", availability: Availability::Busy, onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "toolbar-cross-fade",
        make: || rsx! { Button { bezel: Bezel::Toolbar, size: ControlSize::Large, image: ImagePosition::Only, swap: IconSwap::CrossFade, icon: Icon::Play, label: "Play", onclick: |_| {} } },
    },
    Case {
        component: "button",
        state: "shown",
        make: || rsx! { Button { label: "More", icon: Icon::ChevronDown, shown: Shown::Visible, onclick: |_| {} } },
    },
    // Toggle.
    Case {
        component: "toggle",
        state: "mini-on",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Check::On, size: ControlSize::Mini, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "small-off",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Check::Off, size: ControlSize::Small, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "large-on",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Check::On, size: ControlSize::Large, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "mixed-draws-as-off",
        make: || rsx! { Toggle { label: "Wi-Fi", value: Check::Mixed, onchange: |_| {} } },
    },
    Case {
        component: "toggle",
        state: "busy",
        make: || rsx! { Toggle { label: "Bluetooth", value: Check::On, availability: Availability::Busy, onchange: |_| {} } },
    },
    // Checkbox.
    Case {
        component: "checkbox",
        state: "off",
        make: || rsx! { Checkbox { label: "Remember me", value: Check::Off, onchange: |_| {} } },
    },
    Case {
        component: "checkbox",
        state: "on",
        make: || rsx! { Checkbox { label: "Remember me", value: Check::On, onchange: |_| {} } },
    },
    Case {
        component: "checkbox",
        state: "mixed",
        make: || rsx! { Checkbox { label: "Include subfolders", value: Check::Mixed, onchange: |_| {} } },
    },
    Case {
        component: "checkbox",
        state: "mini-on",
        make: || rsx! { Checkbox { label: "Remember me", value: Check::On, size: ControlSize::Mini, onchange: |_| {} } },
    },
    Case {
        component: "checkbox",
        state: "large-on",
        make: || rsx! { Checkbox { label: "Remember me", value: Check::On, size: ControlSize::Large, onchange: |_| {} } },
    },
    Case {
        component: "checkbox",
        state: "disabled",
        make: || rsx! { Checkbox { label: "Remember me", value: Check::On, availability: Availability::Disabled, onchange: |_| {} } },
    },
    Case {
        component: "checkbox",
        state: "busy",
        make: || rsx! { Checkbox { label: "Remember me", value: Check::Off, availability: Availability::Busy, onchange: |_| {} } },
    },
    // RadioGroup.
    Case {
        component: "radio_group",
        state: "column",
        make: || rsx! { RadioGroup::<u8> { label: "Appearance", choices: words(), value: 1, onchange: |_| {} } },
    },
    Case {
        component: "radio_group",
        state: "row-with-images",
        make: || rsx! { RadioGroup::<u8> { label: "Layout", choices: with_images(), value: 0, arrangement: Arrangement::Row, onchange: |_| {} } },
    },
    Case {
        component: "radio_group",
        state: "mini",
        make: || rsx! { RadioGroup::<u8> { label: "Appearance", choices: words(), value: 2, size: ControlSize::Mini, onchange: |_| {} } },
    },
    Case {
        component: "radio_group",
        state: "disabled",
        make: || rsx! { RadioGroup::<u8> { label: "Appearance", choices: words(), value: 0, availability: Availability::Disabled, onchange: |_| {} } },
    },
    Case {
        component: "radio_group",
        state: "busy",
        make: || rsx! { RadioGroup::<u8> { label: "Appearance", choices: words(), value: 0, availability: Availability::Busy, onchange: |_| {} } },
    },
    // SegmentedControl.
    Case {
        component: "segmented",
        state: "small",
        make: || rsx! { SegmentedControl::<u8> { label: "View", choices: words(), tracking: Tracking::SelectOne(1), size: ControlSize::Small, onchange: |_| {} } },
    },
    Case {
        component: "segmented",
        state: "large",
        make: || rsx! { SegmentedControl::<u8> { label: "View", choices: words(), tracking: Tracking::SelectOne(1), size: ControlSize::Large, onchange: |_| {} } },
    },
    Case {
        component: "segmented",
        state: "select-any",
        make: || rsx! { SegmentedControl::<u8> { label: "Style", choices: words(), tracking: Tracking::SelectAny(vec![0, 2]), onchange: |_| {} } },
    },
    Case {
        component: "segmented",
        state: "momentary",
        make: || rsx! { SegmentedControl::<u8> { label: "Step", choices: words(), tracking: Tracking::Momentary, onchange: |_| {} } },
    },
    Case {
        component: "segmented",
        state: "images-one-disabled",
        make: || rsx! { SegmentedControl::<u8> { label: "Layout", choices: with_images(), tracking: Tracking::SelectOne(0), onchange: |_| {} } },
    },
    Case {
        component: "segmented",
        state: "disabled",
        make: || rsx! { SegmentedControl::<u8> { label: "View", choices: words(), tracking: Tracking::SelectOne(0), availability: Availability::Disabled, onchange: |_| {} } },
    },
    Case {
        component: "segmented",
        state: "busy",
        make: || rsx! { SegmentedControl::<u8> { label: "View", choices: words(), tracking: Tracking::SelectOne(0), availability: Availability::Busy, onchange: |_| {} } },
    },
    // Slider.
    Case {
        component: "slider",
        state: "linear-mini",
        make: || rsx! { Slider { label: "Level", value: Fraction(500), size: ControlSize::Mini, onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "linear-large",
        make: || rsx! { Slider { label: "Level", value: Fraction(500), size: ControlSize::Large, onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "ticks-quarters",
        make: || rsx! { Slider { label: "Level", value: Fraction(500), ticks: Ticks::Every(Fraction(250)), onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "busy",
        make: || rsx! { Slider { label: "Level", value: Fraction(500), availability: Availability::Busy, onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "capsule-volume",
        make: || rsx! { Slider { label: "Volume", value: Fraction(400), look: SliderLook::Capsule, glyph: LevelGlyph::Volume(Muting::Audible), onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "capsule-knob-brightness",
        make: || rsx! { Slider { label: "Brightness", value: Fraction(300), look: SliderLook::CapsuleKnob, glyph: LevelGlyph::Brightness, onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "capsule-muted",
        make: || rsx! { Slider { label: "Volume", value: Fraction(400), look: SliderLook::Capsule, glyph: LevelGlyph::Volume(Muting::Muted), onchange: |_| {} } },
    },
    Case {
        component: "slider",
        state: "capsule-disabled",
        make: || rsx! { Slider { label: "Brightness", value: Fraction(0), look: SliderLook::CapsuleKnob, glyph: LevelGlyph::Brightness, availability: Availability::Disabled } },
    },
    // TextField.
    Case {
        component: "text_field",
        state: "bezeled-mini",
        make: || rsx! { TextField { label: "To", value: "dana", size: ControlSize::Mini, oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "bezeled-large",
        make: || rsx! { TextField { label: "To", value: "dana", size: ControlSize::Large, oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "secure-typed-nothing-written",
        make: || rsx! { TextField { label: "Password", value: "", kind: FieldKind::Secure, placeholder: "Password", oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "secure-rejected",
        make: || rsx! { TextField { label: "Password", value: "", kind: FieldKind::Secure, validity: rejected(), oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "help",
        make: || rsx! { TextField { label: "Email", value: "dana@example.com", help: TextLine::from("We never share it."), oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "rejected",
        make: || rsx! { TextField { label: "Email", value: "dana@", validity: rejected(), help: TextLine::from("We never share it."), oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "prefix-suffix",
        make: || {
            rsx! {
                TextField {
                    label: "Website",
                    value: "example",
                    prefix: rsx! { IconView { source: Icon::Globe.into() } },
                    suffix: rsx! { span { ".com" } },
                    oninput: |_| {},
                }
            }
        },
    },
    Case {
        component: "text_field",
        state: "search-filled-clear",
        make: || rsx! { TextField { label: "Search", value: "uidl", kind: FieldKind::Search, oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "search-bezeled-empty",
        make: || rsx! { TextField { label: "Search", value: "", kind: FieldKind::Search, placeholder: "Search", oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "busy",
        make: || rsx! { TextField { label: "Name", value: "Checking", availability: Availability::Busy, oninput: |_| {} } },
    },
    Case {
        component: "text_field",
        state: "plain-secure",
        make: || rsx! { TextField { label: "Password", value: "", kind: FieldKind::Secure, bezel: FieldBezel::Plain, oninput: |_| {} } },
    },
    // ProgressIndicator.
    Case {
        component: "progress",
        state: "bar-zero",
        make: || rsx! { ProgressIndicator { progress: Progress::Known(Fraction(0)) } },
    },
    Case {
        component: "progress",
        state: "bar-forty",
        make: || rsx! { ProgressIndicator { progress: Progress::Known(Fraction(400)) } },
    },
    Case {
        component: "progress",
        state: "bar-full",
        make: || rsx! { ProgressIndicator { progress: Progress::Known(Fraction(1000)) } },
    },
    Case {
        component: "progress",
        state: "bar-mini",
        make: || rsx! { ProgressIndicator { progress: Progress::Known(Fraction(400)), size: ControlSize::Mini } },
    },
    Case {
        component: "progress",
        state: "bar-unknown",
        make: || rsx! { ProgressIndicator { progress: Progress::Unknown(running()) } },
    },
    Case {
        component: "progress",
        state: "bar-unknown-idle",
        make: || rsx! { ProgressIndicator { progress: Progress::Unknown(Operation::Idle) } },
    },
    Case {
        component: "progress",
        state: "spinner-small",
        make: || rsx! { ProgressIndicator { style: ProgressStyle::Spinner, progress: Progress::Unknown(running()), size: ControlSize::Small } },
    },
    Case {
        component: "progress",
        state: "ring-forty",
        make: || rsx! { ProgressIndicator { style: ProgressStyle::Ring, progress: Progress::Known(Fraction(400)) } },
    },
    Case {
        component: "progress",
        state: "ring-full",
        make: || rsx! { ProgressIndicator { style: ProgressStyle::Ring, progress: Progress::Known(Fraction(1000)) } },
    },
    Case {
        component: "progress",
        state: "ring-glyph",
        make: || rsx! { ProgressIndicator { style: ProgressStyle::Ring, progress: Progress::Known(Fraction(600)), size: ControlSize::Large, glyph: IconSource::Glyph(Icon::Download) } },
    },
    Case {
        component: "progress",
        state: "ring-unknown",
        make: || rsx! { ProgressIndicator { style: ProgressStyle::Ring, progress: Progress::Unknown(running()) } },
    },
    // LevelIndicator.
    Case {
        component: "level_indicator",
        state: "continuous-volume",
        make: || rsx! { LevelIndicator { label: "Volume", value: Fraction(600), glyph: LevelGlyph::Volume(Muting::Audible) } },
    },
    Case {
        component: "level_indicator",
        state: "continuous-brightness",
        make: || rsx! { LevelIndicator { label: "Brightness", value: Fraction(300), glyph: LevelGlyph::Brightness } },
    },
    Case {
        component: "level_indicator",
        state: "discrete-volume",
        make: || rsx! { LevelIndicator { label: "Volume", value: Fraction(600), style: LevelStyle::Discrete, glyph: LevelGlyph::Volume(Muting::Audible) } },
    },
    Case {
        component: "level_indicator",
        state: "continuous-no-glyph",
        make: || rsx! { LevelIndicator { label: "Storage", value: Fraction(500) } },
    },
    Case {
        component: "level_indicator",
        state: "band-warning",
        make: || rsx! { LevelIndicator { label: "Battery", value: Fraction(250), bands: Bands { warning: Fraction(300), critical: Fraction(150) } } },
    },
    Case {
        component: "level_indicator",
        state: "band-critical",
        make: || rsx! { LevelIndicator { label: "Battery", value: Fraction(100), bands: Bands { warning: Fraction(300), critical: Fraction(150) } } },
    },
    Case {
        component: "level_indicator",
        state: "mini",
        make: || rsx! { LevelIndicator { label: "Storage", value: Fraction(500), size: ControlSize::Mini } },
    },
    // Badge.
    Case {
        component: "badge",
        state: "alert-999-plus",
        make: || rsx! { Badge { content: BadgeContent::Number(1200) } },
    },
    Case {
        component: "badge",
        state: "alert-dot",
        make: || rsx! { Badge { content: BadgeContent::Dot } },
    },
    Case {
        component: "badge",
        state: "quiet-dot",
        make: || rsx! { Badge { content: BadgeContent::Dot, tone: BadgeTone::Quiet } },
    },
    Case {
        component: "badge",
        state: "quiet-small",
        make: || rsx! { Badge { content: BadgeContent::Number(7), tone: BadgeTone::Quiet, size: ControlSize::Small } },
    },
    Case {
        component: "badge",
        state: "quiet-regular",
        make: || rsx! { Badge { content: BadgeContent::Number(42), tone: BadgeTone::Quiet } },
    },
    // KeyEquivalent.
    Case {
        component: "key_equivalent",
        state: "text",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Shift, ShortcutKey::Super, ShortcutKey::Char('z')]) } },
    },
    Case {
        component: "key_equivalent",
        state: "text-empty",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![]) } },
    },
    Case {
        component: "key_equivalent",
        state: "cap-regular",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')]), style: KeyStyle::Cap } },
    },
    Case {
        component: "key_equivalent",
        state: "cap-small",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Escape]), style: KeyStyle::Cap, size: ControlSize::Small } },
    },
    Case {
        component: "key_equivalent",
        state: "cap-mini-arrows",
        make: || rsx! { KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Up, ShortcutKey::Down]), style: KeyStyle::Cap, size: ControlSize::Mini } },
    },
];
