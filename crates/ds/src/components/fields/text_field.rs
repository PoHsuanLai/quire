//! TextField: `NSTextField`, `NSSecureTextField` and `NSSearchField` in one component (design/30
//! section 2.2), the one field every text input uses.
//!
//! Markup: `span.ds-text-field[data-kind][data-variant][data-size]` holding `span.ds-text-field-frame`
//! (the bezel: an optional `icon`, the `input` in its wrapper, a `clear` button on a search field,
//! an optional `suffix`) and, under it, the `help` note and the search tokens. `data-focus="ring"`
//! is written while the input has the caret, `data-validity="invalid"` while the value is rejected.

use crate::components::content::icon_view::IconView;
use crate::components::content::label::{Label, LabelRole, LabelStyle};
use crate::components::content::text_runs::TextLine;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::components::controls::chip::{Chip, ChipVariant};
use crate::components::controls::glyph::glyph_size;
use crate::components::controls::progress::busy::use_busy;
use crate::components::controls::progress::model::{Progress, ProgressStyle};
use crate::components::controls::progress::view::ProgressIndicator;
use crate::components::fields::text_field_focus::{FieldFocus, FieldFocuser};
use crate::components::fields::text_field_mask::MaskCaret;
use crate::components::fields::text_field_model::{
    FieldBezel, FieldKind, FieldRows, FieldText, Validity,
};
use crate::components::fields::text_field_parts::{Field, Handlers, line};
use crate::focus::field::FieldHandle;
use crate::focus::targets::Told;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Availability, FocusStyle};
use ds_core::word::Word;
use ds_motion::detail::{once::use_shake, touch::Touch, use_detail::use_detail};
use ds_style::icon::Icon;
use ds_style::tokens::control_size::ControlSize;

/// Whether the field's input has the caret: the ring is drawn from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Holds {
    /// The input has the caret.
    Caret,
    /// Something else has the keyboard.
    Elsewhere,
}

impl Holds {
    /// `data-focus`: the field shows its focus as a ring while it holds the caret.
    fn attr(self) -> Option<&'static str> {
        match self {
            Holds::Caret => Some(FocusStyle::Ring.slug()),
            Holds::Elsewhere => None,
        }
    }
}

/// The text a field shows: a secure field's own state unless the caller holds it, else the
/// caller's `value`.
fn held(kind: FieldKind, text: FieldText, value: String, typed: Signal<String>) -> String {
    match (kind, text) {
        (FieldKind::Secure, FieldText::Own) => typed(),
        (FieldKind::Secure, FieldText::Caller)
        | (FieldKind::Plain | FieldKind::Search | FieldKind::Multiline, _) => value,
    }
}

/// Which incarnation of the line the field draws. A secure field whose text the caller holds
/// cannot be told its text was emptied (the renderer keeps its own copy), so the line is drawn
/// again, empty, under the next generation each time the caller's text goes from some to none.
#[derive(Clone, Copy)]
struct Round {
    generation: CopyValue<u32>,
    /// Whether the caret was in the field when the current generation began.
    caret: CopyValue<Holds>,
}

impl Round {
    fn generation(self) -> u32 {
        *self.generation.peek()
    }

    /// How the new generation's input takes the keyboard: the caret stays where it was.
    fn focus(self, asked: FieldFocus) -> FieldFocus {
        match (self.generation() > 0, *self.caret.peek()) {
            (true, Holds::Caret) => FieldFocus::OnMount,
            (true, Holds::Elsewhere) | (false, _) => asked,
        }
    }
}

/// The field's round: `caller_holds` is whether the caller holds a secret's text, `value` that text.
fn use_round(caller_holds: bool, value: &str, holds: Holds) -> Round {
    let mut generation = use_hook(|| CopyValue::new(0u32));
    let mut filled = use_hook(|| CopyValue::new(false));
    let mut caret = use_hook(|| CopyValue::new(Holds::Elsewhere));
    let emptied = caller_holds && value.is_empty() && *filled.peek();
    if emptied {
        let next = *generation.peek() + 1;
        generation.set(next);
        caret.set(holds);
    }
    filled.set(caller_holds && !value.is_empty());
    Round { generation, caret }
}

/// `line` under `generation` as its key, so a new generation mounts a new input.
fn keyed(generation: u32, line: Element) -> Element {
    rsx! {
        Fragment { key: "{generation}", {line} }
    }
}

/// The size of the clear button on a field of `size`: the rung under it, as far as there is one.
fn clear_size(size: ControlSize) -> ControlSize {
    match size {
        ControlSize::Mini | ControlSize::Small => ControlSize::Mini,
        ControlSize::Regular => ControlSize::Small,
        ControlSize::Large => ControlSize::Regular,
        ControlSize::ExtraLarge => ControlSize::Large,
    }
}

/// A text field. `focus: FieldFocus::OnMount` puts the caret in it when it mounts;
/// `FieldFocus::Controlled(request)` does too, and again at each `request.request()`. `onkey`
/// hears each key as the event itself, so a caller that takes a key can `prevent_default` it.
///
/// `onfocus` and `onblur` hear the caret arrive and leave, so a caller can tell "the person is
/// typing in a field" from "a key for the window". They fire for a click or Tab (the renderer's
/// own events) and for the focus seam: when `FieldFocus::OnMount` or `FieldFocus::Controlled`
/// puts the caret in the field through a host that dispatches no event (Blitz), the field calls
/// `onfocus` itself. A range is [`Slider`](crate::components::controls::slider::Slider).
///
/// `handle` ([`use_field_handle`](crate::focus::field::use_field_handle)) is the caller's grip on the field
/// from any handler: `focus(Select)`, `blur()` and the mounted element. A focus or blur through
/// it (or through [`focus_by_selector`](crate::focus::selector::focus_by_selector)) calls `onfocus`/`onblur` once
/// on Blitz too.
///
/// `kind` picks what it holds ([`FieldKind`]): `Secure` keeps its text out of the markup, `Search`
/// draws a magnifier, a clear button and `tokens` under it, and `Multiline` is a `textarea`
/// `rows` tall (Enter adds a line; the caret leaving commits). `bezel` is its edge ([`FieldBezel`]).
/// `onchange` hears the value committed: Enter, or the caret leaving (Blitz sends no `change`
/// event, so the field makes its own, the same on both renderers). `onsubmit` hears the value on
/// Enter alone (never on the caret leaving, never in a multi-line field, never while the field is
/// not enabled). The controlled shape is `value` in, `oninput` out for each change, `onsubmit`
/// for Enter; a secure field takes `value` too under `text: FieldText::Caller`, and then keeps
/// no copy of the text (its renderer's own input aside). `prefix` and `suffix` are
/// marks inside the frame before and after the text; `help` is a note under it, and `validity`
/// says the value is rejected: its message replaces the help in the danger ink, and a secure
/// field shakes once for each new rejection. `Busy` takes no input and shows a spinner after
/// the text. `common.id` is the input's own, so `focus_by_selector` can find it.
#[component]
pub fn TextField(
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default)] kind: FieldKind,
    #[props(default)] text: FieldText,
    #[props(default)] bezel: FieldBezel,
    #[props(default)] rows: FieldRows,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    #[props(default)] validity: Validity,
    #[props(default)] help: Option<TextLine>,
    #[props(default)] prefix: Option<Element>,
    #[props(default)] suffix: Option<Element>,
    #[props(default)] tokens: Vec<String>,
    oninput: EventHandler<String>,
    #[props(default)] onkey: EventHandler<KeyboardEvent>,
    #[props(default)] focus: FieldFocus,
    #[props(default)] onfocus: EventHandler<()>,
    #[props(default)] onblur: EventHandler<()>,
    #[props(default)] onchange: EventHandler<String>,
    #[props(default)] onsubmit: EventHandler<String>,
    #[props(default)] handle: Option<FieldHandle>,
    #[props(default)] common: Common,
) -> Element {
    let focuser = FieldFocuser::use_new(handle);
    let caret = MaskCaret::use_new();
    let mut typed = use_signal(String::new);
    let mut holds = use_signal(|| Holds::Elsewhere);
    let operation = use_busy(availability);
    let detail = use_detail(validity.clone(), Touch::Remote);
    let shake = use_shake(detail.cue()).attrs();
    if let FieldFocus::Controlled(request) = focus {
        focuser.follow(request, onfocus);
    }
    let round = use_round(
        kind == FieldKind::Secure && text == FieldText::Caller,
        &value,
        holds(),
    );
    let held_text = held(kind, text, value, typed);
    let committed = held_text.clone();
    let submitted = held_text.clone();
    let blurred = held_text.clone();
    let focus_in = EventHandler::new(move |()| {
        holds.set(Holds::Caret);
        onfocus.call(());
    });
    let focus_out = EventHandler::new(move |()| {
        holds.set(Holds::Elsewhere);
        onblur.call(());
    });
    let told = Told {
        focus: use_callback(move |()| {
            caret.refresh();
            focus_in.call(())
        }),
        blur: use_callback(move |()| {
            caret.refresh();
            onchange.call(blurred.clone());
            focus_out.call(());
        }),
    };
    let clearing =
        kind == FieldKind::Search && !held_text.is_empty() && availability == Availability::Enabled;
    let note = validity.message().cloned().or(help);
    let field = Field {
        bezel,
        rows,
        size,
        label,
        id: common.id.clone(),
        placeholder,
        availability,
        invalid: validity.attr().map(|_| "true"),
        focus: round.focus(focus),
        focuser: focuser.clone(),
        caret,
        handlers: Handlers {
            oninput: EventHandler::new(move |next: String| {
                if kind == FieldKind::Secure && text == FieldText::Own {
                    typed.set(next.clone());
                }
                oninput.call(next);
            }),
            onkey,
            onfocus: focus_in,
            onblur: focus_out,
            onchange: EventHandler::new(move |()| onchange.call(committed.clone())),
            onsubmit: EventHandler::new(move |()| onsubmit.call(submitted.clone())),
        },
        told,
    };
    let leading = match (prefix, kind) {
        (Some(mark), _) => Some(mark),
        (None, FieldKind::Search) => Some(rsx! {
            IconView { source: Icon::Search.into(), size: glyph_size(size) }
        }),
        (None, FieldKind::Plain | FieldKind::Secure | FieldKind::Multiline) => None,
    };
    let class = match (&shake, kind) {
        (Some((anim, _)), FieldKind::Secure) => common.class(&format!("ds-text-field {anim}")),
        _ => common.class("ds-text-field"),
    };
    let alias = match kind {
        FieldKind::Secure => shake.map(|(_, alias)| alias),
        FieldKind::Plain | FieldKind::Search | FieldKind::Multiline => None,
    };
    let data = common.data_attributes();
    rsx! {
        span {
            class,
            "data-pulse": alias,
            "data-kind": kind.data_kind(),
            "data-variant": bezel.slug(),
            "data-size": size.slug(),
            "data-availability": availability.slug(),
            "data-focus": holds().attr(),
            "data-validity": validity.attr(),
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            onmounted: move |event| common.mounted(event),
            ..data,
            span { class: "ds-text-field-frame",
                if let Some(mark) = leading {
                    span { class: "ds-text-field-icon", {mark} }
                }
                {keyed(round.generation(), line(field, kind, held_text))}
                if clearing {
                    Button {
                        bezel: Bezel::Toolbar,
                        size: clear_size(size),
                        image: ImagePosition::Only,
                        icon: Icon::X,
                        label: "Clear",
                        common: Common {
                            extra_class: None,
                            ..Common::default()
                        },
                        onclick: move |_| {
                            typed.set(String::new());
                            oninput.call(String::new());
                            onchange.call(String::new());
                            focuser.refocus();
                        },
                    }
                }
                if let Some(mark) = suffix {
                    span { class: "ds-text-field-suffix", {mark} }
                }
                if availability == Availability::Busy {
                    span { class: "ds-text-field-suffix",
                        ProgressIndicator {
                            style: ProgressStyle::Spinner,
                            progress: Progress::Unknown(operation),
                            size: ControlSize::Mini,
                        }
                    }
                }
            }
            if let Some(note) = note {
                span { class: "ds-text-field-help",
                    Label { text: note, role: LabelRole::Secondary, style: LabelStyle::Footnote }
                }
            }
            if !tokens.is_empty() {
                div { class: "ds-text-field-tokens",
                    for token in tokens {
                        Chip { variant: ChipVariant::Token, text: token }
                    }
                }
            }
        }
    }
}
