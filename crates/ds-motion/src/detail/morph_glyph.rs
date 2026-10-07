//! MorphGlyph: a glyph that cross-fades into the next one it is given, as two stacked layers,
//! each its own `svg` in an HTML wrapper the stylesheet can fade (design/26-DETAILS.md section
//! 3.2, design/30 section 1.3).

use super::level::use_level;
use super::morph::{MorphStyle, Slashed};
use crate::timeline::glide::Glide;
use crate::timeline::playback::{Playback, use_playback};
use crate::{
    anim::Anim,
    timer::{TimerPhase, use_motion_timer},
};
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::word::Word;
use ds_style::appearance::motion::MotionLevel;
use ds_style::icon::Icon;
use ds_style::icon::render::{Glyph, IconSize};
use ds_style::icon::stroke::stroke_width;
use ds_style::icon::style::GlyphStyle;
use ds_style::scale::use_scale;
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// The incoming glyph's fade.
const FADE_IN: Anim = Anim::MorphFadeIn;
/// The outgoing glyph's fade.
const FADE_OUT: Anim = Anim::MorphFadeOut;

/// The slash's length on the 24-unit grid (`M2 2l20 20`), for its dash.
const SLASH_LENGTH: f32 = 28.29;

/// What the glyph shows, compared as drawn (R2): the same icon and slash is no morph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Shown {
    icon: Icon,
    slashed: Slashed,
    /// The glyph it replaced, while the morph plays.
    before: Option<Icon>,
    /// Which change this is: its layers' keys and pulse alias.
    round: u32,
}

/// `icon` at `size`, cross-fading into each new icon it is given, and for `MorphStyle::Slash`
/// drawing its slash on or off as `slashed` changes (over `--t-quick`). The first frame is still;
/// a render with the same icon and slash plays nothing (R2); under Reduced every change snaps
/// (R7). `look` is the glyph's style (solid unless a pair's off state). Decorative (`aria-hidden`): the words beside it carry the state.
#[component]
pub fn MorphGlyph(
    icon: Icon,
    size: IconSize,
    style: MorphStyle,
    #[props(default)] slashed: Slashed,
    #[props(default)] look: GlyphStyle,
) -> Element {
    let timer = use_motion_timer(FADE_IN);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let env = use_level();
    let slash = use_playback(Glide::still(slash_target(slashed)));
    let mut seen = use_hook(|| {
        CopyValue::new(Shown {
            icon,
            slashed,
            before: None,
            round: 0,
        })
    });
    let was = *seen.peek();
    if (was.icon, was.slashed) != (icon, slashed) {
        let reduced = env.now() == MotionLevel::Reduced;
        let before = (was.icon != icon && !reduced).then_some(was.icon);
        seen.set(Shown {
            icon,
            slashed,
            before,
            round: was.round.wrapping_add(1),
        });
        let turned = was.slashed != slashed;
        queue_effect(move || {
            if before.is_some() {
                timer.start(settled);
            }
            if turned {
                slide_slash(slash, slashed, reduced);
            }
        });
    }
    let shown = *seen.peek();
    let playing = timer.phase() == TimerPhase::Running && shown.before.is_some();
    let px = size.px();
    let stroke = stroke_width(size, use_scale());
    // The slash is a stroke laid across the glyph, which a solid body would hide: a slash morph
    // draws its glyph as an outline.
    let look = match style {
        MorphStyle::Slash => GlyphStyle::Outline,
        _ => look,
    };
    let drawn = slash.frame().value.clamp(0, 1000);
    let offset = SLASH_LENGTH * (1.0 - drawn as f32 / 1000.0);
    let alias = if shown.round.is_multiple_of(2) {
        "b"
    } else {
        "a"
    };
    rsx! {
        span { class: "ds-morph-glyph", "data-style": style.slug(), "aria-hidden": "true",
            if let (true, Some(before)) = (playing, shown.before) {
                span {
                    key: "out-{shown.round}",
                    class: "ds-morph-layer {FADE_OUT.class()}",
                    "data-morph": "out",
                    "data-pulse": alias,
                    Glyph { icon: before, size, style: look }
                }
            }
            span {
                key: "in-{shown.round}",
                class: if playing { format!("ds-morph-layer {}", FADE_IN.class()) } else { "ds-morph-layer".to_owned() },
                "data-morph": if playing { "in" } else { "still" },
                "data-pulse": playing.then_some(alias),
                Glyph { icon, size, style: look }
            }
            if style == MorphStyle::Slash && drawn > 0 {
                svg {
                    class: "ds-ic ds-morph-slash",
                    width: "{px}",
                    height: "{px}",
                    view_box: "0 0 24 24",
                    "stroke": "currentColor",
                    "stroke-width": stroke,
                    "stroke-linecap": "round",
                    "fill": "none",
                    path {
                        d: "M2 2l20 20",
                        "stroke-dasharray": "{SLASH_LENGTH}",
                        "stroke-dashoffset": "{offset:.2}",
                    }
                }
            }
        }
    }
}

/// How much of the slash is drawn, in thousandths, for `slashed`.
fn slash_target(slashed: Slashed) -> i64 {
    match slashed {
        Slashed::On => 1000,
        Slashed::Off => 0,
    }
}

/// Draw the slash on or off over `--t-quick --e-out`, or at once under Reduced.
fn slide_slash(slash: Playback<Glide>, slashed: Slashed, reduced: bool) {
    let to = slash_target(slashed);
    if reduced {
        return slash.play(Glide::still(to));
    }
    slash.play(Glide::between(
        slash.peek().map_or(to, |pose| pose.value),
        to,
        DurationToken::Quick.duration(MotionLevel::Standard),
        EasingToken::Out.easing(MotionLevel::Standard),
    ));
}
