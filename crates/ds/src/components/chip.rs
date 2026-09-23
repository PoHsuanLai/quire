//! Chip: a small label that states a fact (design/04-COMPONENTS.md section 10).

use crate::components::avatar::{AvatarFace, face};
use crate::components::vocab::PulseKey;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::space::Verdict;
use crate::tokens::LabelHue;
use dioxus::prelude::*;

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
            ChipVariant::Label(hue) => Some(hue_slug(*hue)),
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

/// The `data-hue` word. `LabelHue::slug` says the same; it is the tokens wave's to fill, and a
/// chip must render before that lands, so the five words are spelled here too.
fn hue_slug(hue: LabelHue) -> &'static str {
    match hue {
        LabelHue::Red => "red",
        LabelHue::Amber => "amber",
        LabelHue::Green => "green",
        LabelHue::Blue => "blue",
        LabelHue::Violet => "violet",
    }
}

/// A small label. `pulse` plays `chip-land` or `chip-in` again; the Person chip's 1200 ms
/// flash has no `Anim` and no prop, so it is not drawn (reported against the frozen props).
#[component]
pub fn Chip(
    variant: ChipVariant,
    text: String,
    #[props(default)] onremove: Option<EventHandler<()>>,
    #[props(default)] pulse: Option<PulseKey>,
) -> Element {
    let (class, alias) = match pulse.and_then(PulseKey::attrs) {
        Some((anim, alias)) => (format!("ds-chip {anim}"), Some(alias)),
        None => ("ds-chip".to_string(), None),
    };
    let avatar = match &variant {
        ChipVariant::Person(avatar) => Some(*avatar),
        _ => None,
    };
    rsx! {
        span {
            class,
            "data-variant": variant.slug(),
            "data-hue": variant.hue(),
            "data-status": variant.status(),
            "data-pulse": alias,
            if let Some(avatar) = avatar {
                {face(avatar)}
            }
            "{text}"
            if let (Some(_), Some(onremove)) = (avatar, onremove) {
                button {
                    r#type: "button",
                    class: "ds-chip-remove",
                    "aria-label": "Remove {text}",
                    onclick: move |_| onremove.call(()),
                    Glyph { icon: Icon::X, size: IconSize::Micro }
                }
            }
        }
    }
}
