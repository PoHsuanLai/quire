//! Chip: a small label that states a fact (design/04-COMPONENTS.md section 10).

use crate::components::content::avatar::{AvatarFace, face};
use crate::components::content::tip_text::TipText;
use crate::components::content::title_tip::use_tip;
use crate::components::controls::chip_dot::ChipDot;
use crate::components::controls::press::{ActivationKeys, PressListeners};
use dioxus::prelude::*;
use ds_core::colour::contrast::Verdict;
use ds_core::press::Press;
use ds_core::vocab::Check;
use ds_core::word::Word;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::tokens::label_hue::LabelHue;

/// Which chip.
#[derive(Debug, Clone, PartialEq)]
pub enum ChipVariant {
    /// Accent-soft: every label chip in the Spaces prototype.
    Accent,
    /// One Candy hue per named label.
    Label(LabelHue),
    /// Surface-2 with a line.
    Neutral,
    /// A recipient, with an avatar and a remove button.
    Person(AvatarFace),
    /// A search operator in the command menu.
    Token,
    /// A contrast check's pass or fail.
    Status(Verdict),
}

impl ChipVariant {
    /// The `data-variant` word.
    fn slug(&self) -> &'static str {
        match self {
            ChipVariant::Accent => "accent",
            ChipVariant::Label(_) => "label",
            ChipVariant::Neutral => "neutral",
            ChipVariant::Person(_) => "person",
            ChipVariant::Token => "token",
            ChipVariant::Status(_) => "status",
        }
    }

    /// `data-hue`, on a Label chip only.
    fn hue(&self) -> Option<&'static str> {
        match self {
            ChipVariant::Label(hue) => Some(hue.slug()),
            _ => None,
        }
    }

    /// `data-status`, on a Status chip only.
    fn status(&self) -> Option<&'static str> {
        match self {
            ChipVariant::Status(Verdict::Pass) => Some("ok"),
            ChipVariant::Status(Verdict::Fail) => Some("bad"),
            _ => None,
        }
    }
}

/// A small label (design/04-COMPONENTS.md section 10).
///
/// `onremove` puts a remove button (a cross, named "Remove {text}") after the text, on any
/// variant. `dot` is a small disc before the text, a tag colour (`ChipDot::Tag`, painted from
/// `--tag-<hue>`) or a CSS colour. A chip with a dot and no text is named by `label` (what
/// a screen reader says), else by the tag colour's word ("Red"). `onpress` makes the
/// chip pressable: its face becomes a real `button` that hears a click, Return or Space, and
/// `pressed` says whether it is the picked one (`aria-pressed`; leave it `None` for a chip
/// that acts and holds no state). The remove button stays a separate button beside the face,
/// so removing a chip never presses it.
#[component]
pub fn Chip(
    variant: ChipVariant,
    text: String,
    #[props(default)] onremove: Option<EventHandler<()>>,
    #[props(default)] pressed: Option<Check>,
    #[props(default)] onpress: Option<EventHandler<Press>>,
    #[props(default)] dot: Option<ChipDot>,
    #[props(default)] label: Option<String>,
) -> Element {
    let name = if text.is_empty() {
        label.or_else(|| dot.as_ref().and_then(ChipDot::name).map(str::to_owned))
    } else {
        None
    };
    let paint = dot.as_ref().map(ChipDot::paint);
    let role = match (&onpress, &name) {
        (None, Some(_)) => Some("img"),
        _ => None,
    };
    let aria = match onpress {
        None => name.clone(),
        Some(_) => None,
    };
    let avatar = match &variant {
        ChipVariant::Person(avatar) => Some(*avatar),
        _ => None,
    };
    let content = rsx! {
        if let Some(avatar) = avatar {
            {face(avatar)}
        }
        if let Some(paint) = paint {
            i { class: "ds-chip-dot", style: "--chip-dot:{paint};" }
        }
        "{text}"
    };
    rsx! {
        span {
            class: "ds-chip",
            "data-variant": variant.slug(),
            "data-hue": variant.hue(),
            "data-status": variant.status(),
            "data-pressable": onpress.map(|_| ""),
            "data-state": onpress.and(pressed).map(|state| state.slug()),
            role,
            "aria-label": aria,
            if let Some(onpress) = onpress {
                ChipFace { pressed, onpress, name, {content} }
            } else {
                {content}
            }
            if let Some(onremove) = onremove {
                ChipRemove { text: text.clone(), onremove }
            }
        }
    }
}

/// The face of a pressable chip: one button holding the chip's avatar, dot and text.
#[component]
fn ChipFace(
    pressed: Option<Check>,
    onpress: EventHandler<Press>,
    name: Option<String>,
    children: Element,
) -> Element {
    let listen = PressListeners::new(onpress);
    rsx! {
        button {
            r#type: "button",
            class: "ds-chip-press",
            "aria-label": name,
            "aria-pressed": pressed.map(Check::aria),
            onclick: move |event| listen.click(&event),
            onkeydown: move |event| listen.key_down(&event, ActivationKeys::ReturnAndSpace),
            {children}
        }
    }
}

/// The chip's remove button: named "Remove {text}", its tip just "Remove".
#[component]
fn ChipRemove(text: String, onremove: EventHandler<()>) -> Element {
    let name = format!("Remove {text}");
    let tip = use_tip(Some(TipText::new("Remove")));
    rsx! {
        button {
            r#type: "button",
            class: "ds-chip-remove",
            title: tip.native(),
            "aria-label": name,
            onclick: move |_| onremove.call(()),
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
                move |_| tip.out()
            },
            onmounted: {
                let tip = tip.clone();
                move |event| tip.mounted(&event)
            },
            Glyph { icon: Icon::X, size: IconSize::Micro }
        }
        {tip.surface()}
    }
}
