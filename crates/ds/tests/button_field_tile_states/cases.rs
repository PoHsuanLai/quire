//! The mail-app states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::components::app::pin_tile::{PinFace, PinTile};
use ds::components::app::send_mood::SendMood;
use ds::components::app::send_pill::{PillAction, SendPill};
use ds::components::content::avatar::{AvatarFace, AvatarShape, AvatarSize, AvatarTone, PersonHue};
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::components::controls::press::Propagation;
use ds::detail::{Operation, PendingToken};
use ds::prelude::*;
use ds::root::common::Common;
use ds_core::vocab::Muting;
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::hex::{Colour, Hex};

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
        Row {
            title: "Q3 notes",
            leading: RowLeading::Avatar(CLOCKED),
            accessory: Accessory::Slot(rsx! {
                span { "Mon 9:00" }
                Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Icon::X, label: "Cancel sending Q3 notes", propagation: Propagation::Stop, onclick: |_| {} }
            }),
        }
    }
}

/// An account colour.
const VIOLET: Colour = Colour::Solid(Hex([0x5b, 0x4f, 0xc4]));

/// Poh's account on Google.
fn poh() -> PinFace {
    PinFace::Account {
        initial: 'P',
        colour: VIOLET,
        provider: MarkProvider::Google,
        address: Some("poh@acme.example".to_string()),
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
        make: || rsx! { Button { common: Common { aria_label: Some("Add account".to_string()), ..Common::default() }, size: ControlSize::Mini, label: "+", title: "Add account…", shown: Shown::Visible, onclick: |_| {} } },
    },
    Case {
        golden: "controls/button/quiet-closed.html",
        make: || rsx! { Button { bezel: Bezel::Inline, label: "More", shown: Shown::Hidden, onclick: |_| {} } },
    },
    // TextField: a secure field, empty (the placeholder) and filled (the dots).
    Case {
        golden: "controls/text_field/password-empty.html",
        make: || rsx! { TextField { kind: FieldKind::Secure, label: "Password", value: "", placeholder: "App password", oninput: |_| {} } },
    },
    Case {
        golden: "controls/text_field/password-filled.html",
        make: || rsx! { TextField { kind: FieldKind::Secure, label: "Password", value: "hunter2", oninput: |_| {} } },
    },
    // PinTile: the favicon mark, and the Add account tile, with and without a hint.
    Case {
        golden: "lists/pin_tile/one-image-mark.html",
        make: || rsx! { PinTile { face: poh(), selection: Selection::Selected, unread: 2, mark: MarkStyle::Image(ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())), onclick: |_| {} } },
    },
    Case {
        golden: "lists/pin_tile/add.html",
        make: || rsx! { PinTile { face: PinFace::Add { label: "Add account".to_string(), hint: Some("Add account…".to_string()) }, onclick: |_| {} } },
    },
    Case {
        golden: "lists/pin_tile/add-named.html",
        make: || rsx! { PinTile { face: PinFace::Add { label: "Add an account to Work".to_string(), hint: None }, onclick: |_| {} } },
    },
    // SendPill: Cancel for a held send, and a fatal refusal on two lines.
    Case {
        golden: "overlays/send_pill/cancel.html",
        make: || rsx! { SendPill { text: "Scheduled for 9:00", progress: Fraction(0), operation: Operation::Running(PendingToken::start()), action: PillAction::Cancel, onundo: |_| {} } },
    },
    Case {
        golden: "overlays/send_pill/fatal-refused.html",
        make: || rsx! { SendPill { text: "Not sent", progress: Fraction(700), operation: Operation::Running(PendingToken::start()), mood: SendMood::Fatal, refusal: "No recipients", onundo: |_| {} } },
    },
    // Row: a scheduled Today row with its time and cancel.
    Case {
        golden: "lists/row/today-scheduled.html",
        make: scheduled,
    },
    // Avatar: an account's colour muted (chroma .55, hue and lightness kept),
    // and a person's; a muted ink avatar is greyscale already and keeps its colours.
    Case {
        golden: "controls/avatar/account-28-muted.html",
        make: || rsx! { Avatar { initial: 'P', size: AvatarSize::Size28, tone: AvatarTone::Account(VIOLET), muting: Muting::Muted } },
    },
    Case {
        golden: "controls/avatar/person-18-muted.html",
        make: || rsx! { Avatar { initial: 'D', size: AvatarSize::Size18, tone: AvatarTone::Person(PersonHue(212)), muting: Muting::Muted } },
    },
    Case {
        golden: "controls/avatar/ink-28-muted.html",
        make: || rsx! { Avatar { initial: 'A', size: AvatarSize::Size28, tone: AvatarTone::Ink, muting: Muting::Muted } },
    },
];
