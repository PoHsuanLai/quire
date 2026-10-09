//! A thread row's star (design/04-COMPONENTS.md section 16): the button that toggles it. Split
//! from `list_row` so the row's own file holds the row.

use crate::components::content::tip_text::TipText;
use crate::components::content::title_tip::use_tip;
use crate::focus::click::kept_click;
use dioxus::prelude::*;
use ds_core::vocab::{Check, Shortcut};
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::icon::style::GlyphStyle;

/// The star's glyph: solid once starred, outline while not (the pair rule of
/// design/08-ICONS.md section 1.2).
fn star_glyph(state: Check) -> Element {
    rsx! { Glyph { icon: Icon::Star, size: IconSize::Compact, style: GlyphStyle::of(state) } }
}

/// The star button. Its accessible name is the full wording; its tip is the terse noun, with
/// `shortcut` when the host binds a key.
#[component]
pub(crate) fn StarButton(
    state: Check,
    onchange: EventHandler<Check>,
    #[props(default)] shortcut: Option<Shortcut>,
) -> Element {
    let label = match state {
        Check::On => "Unstar this thread",
        Check::Off | Check::Mixed => "Star this thread",
    };
    let noun = match state {
        Check::On => "Unstar",
        Check::Off | Check::Mixed => "Star",
    };
    let tip = use_tip(Some(TipText::new(noun).with_shortcut_opt(shortcut)));
    rsx! {
        button {
            r#type: "button",
            class: "ds-star",
            "aria-pressed": state.aria(),
            "aria-label": label,
            title: tip.native(),
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
            onclick: move |event| {
                // The star acts on its own; the row must not also open.
                event.stop_propagation();
                onchange.call(state.flipped());
                kept_click(&event);
            },
            span { class: "ds-star-glyph", {star_glyph(state)} }
        }
        {tip.surface()}
    }
}
