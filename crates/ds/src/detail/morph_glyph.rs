//! MorphGlyph: a glyph that morphs into the next one it is given, as two stacked layers, each its
//! own `svg` in an HTML wrapper the stylesheet can move (design/26-DETAILS.md section 3.2).

use super::glide::Glide;
use super::level::use_level;
use super::morph::{MorphStyle, Slashed};
use super::motor::use_motor;
use crate::appearance::MotionLevel;
use crate::icon::Icon;
use crate::icon::render::{Glyph, IconSize};
use crate::icon::stroke::stroke_width;
use crate::motion::{Anim, TimerPhase, use_motion_timer};
use crate::root::use_scale;
use crate::tokens::{DurationToken, EasingToken};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

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

/// `icon` at `size`, morphing into each new icon it is given by `style`, and for
/// `MorphStyle::Slash` drawing its slash on or off as `slashed` changes (over `--t-quick`). The
/// first frame is still; a render with the same icon and slash plays nothing (R2); under Reduced
/// every change snaps (R7). Decorative (`aria-hidden`): the words beside it carry the state.
#[component]
pub fn MorphGlyph(
    icon: Icon,
    size: IconSize,
    style: MorphStyle,
    #[props(default)] slashed: Slashed,
) -> Element {
    let (incoming, outgoing) = anims(style);
    let timer = use_motion_timer(incoming);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let env = use_level();
    let slash = use_motor(slash_target(slashed));
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
    let drawn = slash.pose().value.clamp(0, 1000);
    let offset = SLASH_LENGTH * (1.0 - drawn as f32 / 1000.0);
    let alias = if shown.round.is_multiple_of(2) {
        "b"
    } else {
        "a"
    };
    rsx! {
        span { class: "ds-morph-glyph", "data-style": style.slug(), "aria-hidden": "true",
            if let (true, Some(before), Some(out)) = (playing, shown.before, outgoing) {
                span {
                    key: "out-{shown.round}",
                    class: "ds-morph-layer {out.class()}",
                    "data-morph": "out",
                    "data-pulse": alias,
                    Glyph { icon: before, size }
                }
            }
            span {
                key: "in-{shown.round}",
                class: if playing { format!("ds-morph-layer {}", incoming.class()) } else { "ds-morph-layer".to_owned() },
                "data-morph": if playing { "in" } else { "still" },
                "data-pulse": playing.then_some(alias),
                Glyph { icon, size }
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

/// The incoming layer's animation and the outgoing one's (none: it goes at once).
fn anims(style: MorphStyle) -> (Anim, Option<Anim>) {
    match style {
        MorphStyle::DownUp => (Anim::MorphIn, Some(Anim::MorphOut)),
        MorphStyle::OffUp => (Anim::MorphIn, None),
        MorphStyle::CrossFade | MorphStyle::Slash => (Anim::MorphFadeIn, Some(Anim::MorphFadeOut)),
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
fn slide_slash(slash: super::motor::Motor, slashed: Slashed, reduced: bool) {
    let to = slash_target(slashed);
    if reduced {
        return slash.snap(to);
    }
    slash.play(Glide {
        from: slash.peek().value,
        to,
        length: DurationToken::Quick.duration(MotionLevel::Standard),
        easing: EasingToken::Out.easing(MotionLevel::Standard),
    });
}

#[cfg(test)]
mod tests {
    use super::anims;
    use crate::detail::MorphStyle;
    use crate::motion::Anim;

    #[test]
    fn each_style_plays_its_own_pair() {
        const CASES: &[(MorphStyle, Anim, Option<Anim>)] = &[
            (MorphStyle::DownUp, Anim::MorphIn, Some(Anim::MorphOut)),
            (MorphStyle::OffUp, Anim::MorphIn, None),
            (
                MorphStyle::CrossFade,
                Anim::MorphFadeIn,
                Some(Anim::MorphFadeOut),
            ),
            (
                MorphStyle::Slash,
                Anim::MorphFadeIn,
                Some(Anim::MorphFadeOut),
            ),
        ];
        for &(style, incoming, outgoing) in CASES {
            assert_eq!(anims(style), (incoming, outgoing), "{style:?}");
        }
    }
}
