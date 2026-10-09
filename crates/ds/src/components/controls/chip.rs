//! Chip: a small label that states a fact (design/04-COMPONENTS.md section 10).

use crate::components::content::avatar::{AvatarFace, face};
use crate::components::content::title_tip::use_tip;
use dioxus::prelude::*;
use ds_core::colour::contrast::Verdict;
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
#[component]
pub fn Chip(
    variant: ChipVariant,
    text: String,
    #[props(default)] onremove: Option<EventHandler<()>>,
) -> Element {
    let avatar = match &variant {
        ChipVariant::Person(avatar) => Some(*avatar),
        _ => None,
    };
    rsx! {
        span {
            class: "ds-chip",
            "data-variant": variant.slug(),
            "data-hue": variant.hue(),
            "data-status": variant.status(),
            if let Some(avatar) = avatar {
                {face(avatar)}
            }
            "{text}"
            if let (Some(_), Some(onremove)) = (avatar, onremove) {
                ChipRemove { text: text.clone(), onremove }
            }
        }
    }
}

/// The chip's remove button: named "Remove {text}", with the same words as its tip.
#[component]
fn ChipRemove(text: String, onremove: EventHandler<()>) -> Element {
    let name = format!("Remove {text}");
    let tip = use_tip(Some(name.clone()));
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
