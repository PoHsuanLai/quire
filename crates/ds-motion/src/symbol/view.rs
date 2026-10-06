//! `Symbol`: an icon that plays a symbol effect (design/35-SYMBOL-EFFECTS.md).

use super::effect::SymbolEffect;
use super::pose::pose;
use super::route::{Route, route};
use super::timing::DRAW;
use super::use_symbol::{use_part_progress, use_symbol_attrs};
use crate::detail::morph::MorphStyle;
use crate::detail::morph_glyph::MorphGlyph;
use crate::detail::tween::use_tween;
use dioxus::prelude::*;
use ds_core::vocab::{Fraction, Shown};
use ds_style::icon::Icon;
use ds_style::icon::posed::{PartPose, PosedGlyph, Thousandths};
use ds_style::icon::render::{Glyph, IconSize};

/// How much of the strokes a draw-on has drawn when `route` is at rest.
fn drawn_target(route: Route) -> Fraction {
    match route {
        Route::Draw(Shown::Hidden) => Fraction(0),
        _ => Fraction(1000),
    }
}

/// The pose of each of `icon`'s parts `progress` through its cycle.
fn poses(icon: Icon, route: Route, progress: Thousandths) -> Vec<PartPose> {
    let Route::Parts(gesture, _) = route else {
        return Vec::new();
    };
    let count = icon.parts().map_or(0, |parts| parts.parts.len());
    (0..count)
        .map(|index| pose(gesture, index, count, progress))
        .collect()
}

/// `icon` at `size` playing `effect`. The wrapper carries `data-symbol` (the effect), `data-run`
/// (how it runs) and, while the stylesheet plays it, the effect's `a-<anim>` class and
/// `data-pulse` alias; a part effect moves the icon's annotated shapes inside the `svg`; a
/// `Replace` cross-fades. The first frame is still (R2), under Reduced the whole-icon effects
/// hold or fade and the part effects stand still (R7). Decorative (`aria-hidden`).
#[component]
pub fn Symbol(icon: Icon, #[props(default)] size: IconSize, effect: SymbolEffect) -> Element {
    let route = route(effect, icon);
    let attrs = use_symbol_attrs(match route {
        Route::Css(effect) => effect,
        _ => SymbolEffect::NONE,
    });
    let progress = use_part_progress(route);
    let drawn = use_tween(drawn_target(route), DRAW);
    let glyph = match route {
        Route::Css(_) => rsx! { Glyph { icon, size } },
        Route::Replace => rsx! { MorphGlyph { icon, size, style: MorphStyle::CrossFade } },
        Route::Parts(..) => rsx! { PosedGlyph { icon, size, poses: poses(icon, route, progress) } },
        Route::Draw(_) => rsx! {
            PosedGlyph { icon, size, poses: Vec::new(), drawn: Thousandths(i32::from(drawn.0)) }
        },
    };
    let class = match attrs.class() {
        Some(anim) => format!("ds-symbol {anim}"),
        None => "ds-symbol".to_owned(),
    };
    rsx! {
        span {
            class,
            "data-symbol": effect.slug(),
            "data-run": effect.run(),
            "data-pulse": attrs.pulse,
            "data-visibility": attrs.hidden.then_some("hidden"),
            "aria-hidden": "true",
            {glyph}
        }
    }
}
