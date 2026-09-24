//! The mailo gaps 2 and 3 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::components::vocab::Switch;
use ds::{
    AccountFace, AccountTile, AddAccountTile, Button, ButtonVariant, Colour, Expanded, Fraction,
    Hex, ImageSource, InputVariant, MarkStyle, PillAction, Provider, SendMood, SendPhase, SendPill,
    SendRing, TextInput, TextInputKind,
};
use ds::{
    Anim, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Here, ItemKind, PersonHue, Presence,
    PulseKey, SidebarItem, TodayTrailing,
};
use ds::{Avatar, AvatarMuting};
use ds::{
    CardAccent, DotIndex, MeasuredIn, Motion, MotionChoice, PRESETS, Scheme, SpaceEditor,
    SpaceLook, Theme,
};

/// A scheduled draft's favicon.
const CLOCKED: AvatarFace = AvatarFace {
    initial: 'Q',
    size: AvatarSize::Size18,
    tone: AvatarTone::Person(PersonHue(212)),
    shape: AvatarShape::Round,
};

/// A scheduled Today row: its time and cancel, and no close.
fn scheduled() -> Element {
    rsx! {
        SidebarItem {
            kind: ItemKind::Today { avatar: CLOCKED },
            label: "Q3 notes",
            here: Here::Elsewhere,
            count: None,
            presence: Presence::Present,
            preview: None,
            pulse: PulseKey::rest(Anim::Gulp),
            onclick: |_| {},
            onclose: None,
            trailing: TodayTrailing { time: "Mon 9:00".to_string(), cancel: "Cancel sending Q3 notes".to_string(), on_cancel: EventHandler::new(|()| {}) },
        }
    }
}

/// An account colour.
const VIOLET: Colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));

/// Poh's account on Google.
fn poh() -> AccountFace {
    AccountFace::One {
        initial: 'P',
        colour: VIOLET,
        provider: Provider::Google,
        address: Some("poh@acme.example".to_string()),
    }
}

/// Preset `index` as a Space's look, in `theme`.
fn look(index: usize, theme: Theme) -> SpaceLook {
    SpaceLook {
        dots: PRESETS[index].dots.to_vec(),
        grain: ds::Grain(35),
        theme,
        card_accent: CardAccent::SpaceHue,
    }
}

/// The editor with every mailo gaps 2 row switched on, for a Space whose theme is `theme`.
fn editor_rows(theme: Theme) -> Element {
    rsx! {
        SpaceEditor {
            look: look(0, theme),
            scheme: Scheme::Light,
            active_dot: DotIndex(0),
            onchange: |_| {},
            name: "Work".to_string(),
            on_rename: EventHandler::new(|_: String| {}),
            motion: MotionChoice { level: Motion::Calm, on_motion: EventHandler::new(|_: Motion| {}) },
            measured: MeasuredIn::EachScheme,
        }
    }
}

/// One state and the golden it must match (relative to `tests/snapshots`).
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

pub const CASES: &[Case] = &[
    // Button: a hint, a name for assistive technology, and the open state of what it opens.
    Case {
        golden: "controls/button/mini-titled-open.html",
        make: || rsx! { Button { variant: ButtonVariant::Mini, label: "+", title: "Add account…", aria_label: "Add account", expanded: Expanded::Open, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/quiet-closed.html",
        make: || rsx! { Button { variant: ButtonVariant::Quiet, label: "More", expanded: Expanded::Closed, onclick: |_| {} } },
    },
    // TextInput: a password, empty (the placeholder) and filled (the dots).
    Case {
        golden: "controls/text_input/password-empty.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Password, label: "Password", value: "", placeholder: "App password", oninput: |_| {} } },
    },
    Case {
        golden: "controls/text_input/password-filled.html",
        make: || rsx! { TextInput { variant: InputVariant::Boxed, kind: TextInputKind::Password, label: "Password", value: "hunter2", oninput: |_| {} } },
    },
    // AccountTile: the favicon mark, and the Add account tile, with and without a hint.
    Case {
        golden: "lists/account_tile/one-image-mark.html",
        make: || rsx! { AccountTile { account: poh(), pressed: Switch::On, unread: 2, mark: MarkStyle::Image(ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())), onclick: |_| {} } },
    },
    Case {
        golden: "lists/account_tile/add.html",
        make: || rsx! { AddAccountTile { title: "Add account…", onclick: |_| {} } },
    },
    Case {
        golden: "lists/account_tile/add-named.html",
        make: || rsx! { AddAccountTile { label: "Add an account to Work", onclick: |_| {} } },
    },
    // SendPill: Cancel for a held send, a spinning ring with no button, a nudge on mount (at
    // rest: a mood plays only when it changes), and a fatal refusal on two lines.
    Case {
        golden: "overlays/send_pill/cancel.html",
        make: || rsx! { SendPill { text: "Scheduled for 9:00", progress: Fraction(0), phase: SendPhase::Counting, action: PillAction::Cancel, onundo: |_| {} } },
    },
    Case {
        golden: "overlays/send_pill/spin-nothing.html",
        make: || rsx! { SendPill { text: "Sending…", progress: Fraction(0), phase: SendPhase::Counting, ring: SendRing::Spin, action: PillAction::Nothing, onundo: |_| {} } },
    },
    Case {
        golden: "overlays/send_pill/nudge-mounted.html",
        make: || rsx! { SendPill { text: "Not sent yet · will try again", progress: Fraction(700), phase: SendPhase::Counting, mood: SendMood::Nudge, action: PillAction::Nothing, onundo: |_| {} } },
    },
    Case {
        golden: "overlays/send_pill/fatal-refused.html",
        make: || rsx! { SendPill { text: "Not sent", progress: Fraction(700), phase: SendPhase::Counting, mood: SendMood::Fatal, refusal: "No recipients", onundo: |_| {} } },
    },
    // SidebarItem: a scheduled Today row with its time and cancel.
    Case {
        golden: "lists/sidebar_item/today-scheduled.html",
        make: scheduled,
    },
    // SpaceEditor: the name field, the Motion row and the readout per scheme (both for a
    // System Space, one for a Dark one); an unnamed Space's field shows its placeholder.
    Case {
        golden: "lists/space_editor/rows-system.html",
        make: || editor_rows(Theme::System),
    },
    Case {
        golden: "lists/space_editor/rows-dark.html",
        make: || editor_rows(Theme::Dark),
    },
    Case {
        golden: "lists/space_editor/rename-unnamed.html",
        make: || rsx! { SpaceEditor { look: look(2, Theme::Light), scheme: Scheme::Light, active_dot: DotIndex(0), onchange: |_| {}, on_rename: EventHandler::new(|_: String| {}) } },
    },
    // mailo gaps 3. Avatar: an account's colour muted (chroma .55, hue and lightness kept),
    // and a person's; a muted ink avatar is greyscale already and keeps its colours.
    Case {
        golden: "controls/avatar/account-28-muted.html",
        make: || rsx! { Avatar { initial: 'P', size: AvatarSize::Size28, tone: AvatarTone::Account(VIOLET), muting: AvatarMuting::Muted } },
    },
    Case {
        golden: "controls/avatar/person-18-muted.html",
        make: || rsx! { Avatar { initial: 'D', size: AvatarSize::Size18, tone: AvatarTone::Person(PersonHue(212)), muting: AvatarMuting::Muted } },
    },
    Case {
        golden: "controls/avatar/ink-28-muted.html",
        make: || rsx! { Avatar { initial: 'A', size: AvatarSize::Size28, tone: AvatarTone::Ink, muting: AvatarMuting::Muted } },
    },
];
