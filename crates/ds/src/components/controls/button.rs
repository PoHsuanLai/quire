//! Button: `NSButton` in its push, toolbar, inline and help bezels (design/30 section 2.1).
//! Markup: `button.ds-button[data-variant=<bezel>][data-role][data-size][data-availability]`;
//! `data-state` and `aria-pressed` only for a toggle button; `data-answers` for a button that
//! answers Return or Escape; `data-image=only` when the label is not drawn; `data-pressed`
//! while a press is under way; the consumer's own `data-*` and classes after quire's.

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::button_marks::leading as leading_mark;
use crate::components::controls::button_marks::trailing as trailing_mark;
use crate::components::controls::button_marks::{Leading, Trailing, spoken_label};
use crate::components::controls::button_model::{
    Answers, Bezel, BusyLook, ButtonFocus, ButtonRole, IconSwap, ImagePosition,
};
use crate::components::controls::button_tip::use_tip;
use crate::components::controls::glyph::glyph_size;
use crate::components::controls::press::{
    ActivationKeys, PressListeners, Propagation, disabled, use_pressing,
};
use crate::components::controls::progress::busy::{busy_class, use_busy, use_busy_seen};
use crate::components::controls::progress::model::{Progress, ProgressStyle};
use crate::components::controls::progress::view::ProgressIndicator;
use crate::focus::soon::focus_soon;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::{Availability, Check, Shown};
use ds_core::word::Word;
use ds_motion::detail::{morph::MorphStyle, morph_glyph::MorphGlyph};
use ds_motion::symbol::effect::{Activity, LoopEffect, SymbolEffect};
use ds_motion::symbol::use_symbol::use_symbol_attrs;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::icon::style::GlyphStyle;
use ds_style::tokens::control_size::ControlSize;

/// The size the busy spinner takes in the leading slot: one rung under the button's own, so
/// it sits inside the label's line (design/30 section 2.9).
fn spinner_size(size: ControlSize) -> ControlSize {
    match size {
        ControlSize::Mini | ControlSize::Small => ControlSize::Mini,
        ControlSize::Regular | ControlSize::Large | ControlSize::ExtraLarge => ControlSize::Small,
    }
}

/// A labelled action. A `Help` bezel draws a question mark and is named by `label`.
///
/// `label` names the button; it is drawn unless `image` is `ImagePosition::Only`, when it is the
/// button's accessible name instead (a runs label names it by its characters). `icon` is a glyph
/// or an external icon (an `Icon` converts) at the ladder's glyph size for `size`.
/// `onclick` hears the primary, secondary (right-click) and middle buttons, and the keyboard
/// (Return or Space on the focused button) as primary. `common.mounted` hands over the element
/// once it is in the document, so a floating component can anchor to it (`Anchor::Mounted`).
///
/// `value` makes it a toggle button: `Check::On` draws it pressed in (`aria-pressed`). `shown`
/// says whether the menu or panel this button opens is up (`aria-expanded`); leave it `None` on a
/// button that opens nothing. `title` is the hover hint: a Mac tooltip through the hover hub
/// (`button_tip`), drawn below the button from the button's own pointer events, with no wrapper; inside a `Tooltip` it draws nothing (the
/// caller's tip stands), and outside a `Ds` it is the plain `title` attribute.
///
/// `trailing` puts a mark after the label, `leading` one before it (`Leading::Mark` holds an
/// element such as a `ProviderMark`).
///
/// `availability`: `Disabled` writes `aria-disabled` and `disabled`, draws the button at .35 and
/// drops every press; `Busy` does the same for input, writes `aria-busy`, and shows a spinner in
/// the leading slot, faded in over `--t-quick`.
///
/// `answers` says which window key the button answers (see [`Answers`]): a dialog that wants
/// Return and Escape to press it routes them itself (a sheet's
/// `on_return`, an alert's own keys). `focus: ButtonFocus::OnMount` gives it the keyboard as it
/// mounts. A caller-controlled disabled state is `availability`.
///
/// `busy: BusyLook::TurnIcon` replaces the busy spinner with the button's own `icon` turning
/// continuously (the symbol effect `Rotate` while busy; no leading mark; input is dropped and
/// `aria-busy` written all the same).
///
/// `swap: IconSwap::CrossFade` fades a changed `icon` into the new one.
///
/// `propagation: Propagation::Stop` keeps the press at the button: its ancestors never hear the
/// click (a header action inside a `<summary>` leaves the `<details>` as it was).
#[component]
pub fn Button(
    #[props(into)] label: TextLine,
    #[props(default)] bezel: Bezel,
    #[props(default)] role: ButtonRole,
    #[props(default)] answers: Answers,
    #[props(default)] focus: ButtonFocus,
    #[props(default)] size: ControlSize,
    #[props(default)] image: ImagePosition,
    #[props(default)] swap: IconSwap,
    #[props(default)] icon: Option<IconSource>,
    #[props(default)] value: Option<Check>,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] availability: Availability,
    #[props(default)] busy: BusyLook,
    onclick: EventHandler<Press>,
    #[props(default)] title: Option<String>,
    #[props(default)] trailing: Option<Trailing>,
    #[props(default)] leading: Option<Leading>,
    #[props(default)] propagation: Propagation,
    #[props(default)] common: Common,
) -> Element {
    let seen = use_busy_seen(availability);
    let class = busy_class(common.class("ds-button"), seen);
    let data = common.data_attributes();
    let spoken = match (bezel, image) {
        (Bezel::Help, _) | (_, ImagePosition::Only) => Some(label.plain_text()),
        (_, ImagePosition::Leading) => spoken_label(&label),
    };
    let aria_label = common.aria_label.clone().or(spoken);
    let name = aria_label.clone().unwrap_or_else(|| label.plain_text());
    let listen = PressListeners::new(onclick).with_propagation(propagation);
    let pressing = use_pressing();
    let operation = use_busy(availability);
    let live = availability == Availability::Enabled;
    let keys = match answers {
        Answers::Escape => ActivationKeys::SpaceOnly,
        Answers::Nothing | Answers::Return => ActivationKeys::ReturnAndSpace,
    };
    let turning = busy == BusyLook::TurnIcon && availability == Availability::Busy;
    let glyph = glyph_size(size);
    // The turn is the symbol effect `Rotate` while busy; the wrapper wears what it says.
    let spin = use_symbol_attrs(SymbolEffect::While(
        LoopEffect::Rotate,
        if turning {
            Activity::Active
        } else {
            Activity::Idle
        },
    ));
    let tip = use_tip(title);
    rsx! {
        button {
            r#type: "button",
            id: common.id.clone(),
            class,
            "data-variant": bezel.slug(),
            "data-role": role.slug(),
            "data-answers": answers.attr(),
            "data-size": size.slug(),
            "data-image": image.slug(),
            "data-availability": availability.slug(),
            "data-busy": turning.then_some(BusyLook::TurnIcon.slug()),
            "data-state": value.map(|state| state.slug()),
            title: tip.native(),
            "aria-label": aria_label,
            "aria-description": tip.description(&name),
            "aria-pressed": value.map(Check::aria),
            "aria-expanded": shown.map(Shown::aria),
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            disabled: disabled(availability),
            "data-pressed": if live { pressing.attr() } else { None },
            onmousedown: move |event| pressing.pointer_down(&event),
            onpointerdown: {
                let tip = tip.clone();
                move |_| tip.press()
            },
            onmouseover: {
                let tip = tip.clone();
                move |event| tip.over(&event)
            },
            onmouseleave: {
                let tip = tip.clone();
                move |_| {
                    tip.out();
                    pressing.released();
                }
            },
            onkeydown: move |event| {
                if live {
                    pressing.key_down(&event, keys);
                    listen.key_down(&event, keys);
                }
            },
            onkeyup: move |_| pressing.released(),
            onblur: move |_| pressing.released(),
            onclick: move |event| {
                if live {
                    listen.click(&event);
                }
            },
            oncontextmenu: move |event| {
                if live {
                    listen.context_menu(&event);
                }
            },
            onmouseup: move |event| {
                pressing.released();
                if live {
                    listen.mouse_up(&event);
                }
            },
            // The element, for a menu or popover anchored to it (`Anchor::Mounted`). No
            // attribute: the markup is the same with or without a handler.
            onmounted: {
                let tip = tip.clone();
                move |event| {
                    tip.mounted(&event);
                    if focus == ButtonFocus::OnMount {
                        focus_soon(event.data());
                    }
                    common.mounted(event);
                }
            },
            // The consumer's own `data-*`, last: a spread follows the named attributes.
            ..data,
            if availability == Availability::Busy && !turning {
                span { class: "ds-button-lead",
                    ProgressIndicator {
                        style: ProgressStyle::Spinner,
                        progress: Progress::Unknown(operation),
                        size: spinner_size(size),
                    }
                }
            } else if let Some(mark) = leading {
                {leading_mark(mark, glyph)}
            }
            if let Some(icon) = icon {
                span {
                    class: match spin.class() {
                        Some(anim) => format!("ds-button-icon {anim}"),
                        None => "ds-button-icon".to_owned(),
                    },
                    "data-pulse": spin.pulse,
                    {icon_view(icon, glyph, swap, value)}
                }
            }
            if bezel == Bezel::Help {
                span { class: "ds-button-label", "aria-hidden": "true", "?" }
            } else if image == ImagePosition::Leading {
                span { class: "ds-button-label", {text(&label)} }
            }
            if let Some(mark) = trailing {
                {trailing_mark(mark, glyph)}
            }
        }
        {tip.surface()}
    }
}

/// `icon` at `size`; a quire glyph under `IconSwap::CrossFade` fades into the next one it is
/// given. A toggle's `value` picks the glyph's style: a star, heart, pin or bell is outline
/// while off (`GlyphStyle::for_state`).
fn icon_view(icon: IconSource, size: IconSize, swap: IconSwap, value: Option<Check>) -> Element {
    let look = |icon| {
        value.map_or(GlyphStyle::Solid, |state| {
            GlyphStyle::for_state(icon, state)
        })
    };
    match (swap, icon) {
        (IconSwap::CrossFade, IconSource::Glyph(icon)) => rsx! {
            MorphGlyph { icon, size, style: MorphStyle::CrossFade, look: look(icon) }
        },
        (IconSwap::Instant, IconSource::Glyph(icon)) => rsx! {
            Glyph { icon, size, style: look(icon) }
        },
        (IconSwap::Instant, source) | (IconSwap::CrossFade, source) => rsx! {
            IconView { source, size }
        },
    }
}
