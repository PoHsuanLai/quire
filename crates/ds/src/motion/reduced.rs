//! What each animation becomes under Reduced motion (design/27 section 3.12 rule 5; design/05
//! section 14.4): a keyframe that moves something is swapped for one that does not. An arrival
//! fades in, a departure fades out, and an emphasis (a bump, a shake, a pop) plays nothing, so
//! no state is carried by movement alone. A keyframe that already moves nothing (a fade, a colour,
//! a ring) plays as it is. The data test below fails a moving keyframe with no still form.

use super::anim::Anim;

/// Which way a cross-fade runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FadeWay {
    /// From nothing to shown: `fade`.
    In,
    /// From shown to nothing: `menu-out`.
    Out,
}

/// An animation's form under Reduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReducedForm {
    /// It moves nothing already; it plays as it is.
    Same,
    /// It moves; under Reduced it cross-fades instead.
    CrossFade(FadeWay),
    /// It moves to draw attention; under Reduced it plays `hold`, which moves nothing.
    Still,
}

impl ReducedForm {
    /// The `@keyframes` played under Reduced in place of `keyframes`.
    pub fn keyframes(self, keyframes: &'static str) -> &'static str {
        match self {
            ReducedForm::Same => keyframes,
            ReducedForm::CrossFade(FadeWay::In) => "fade",
            ReducedForm::CrossFade(FadeWay::Out) => "menu-out",
            ReducedForm::Still => "hold",
        }
    }
}

impl Anim {
    /// Its form under Reduced motion.
    pub fn reduced(self) -> ReducedForm {
        use FadeWay::{In, Out};
        use ReducedForm::{CrossFade, Same, Still};
        match self {
            Anim::PopIn
            | Anim::RowIn
            | Anim::Rise
            | Anim::ChipIn
            | Anim::TabIn
            | Anim::SlideR
            | Anim::SlideL
            | Anim::MenuIn
            | Anim::MenuPop
            | Anim::BubblePop
            | Anim::CmdkIn
            | Anim::CmdkRise
            | Anim::PeekIn
            | Anim::PeekFullIn
            | Anim::HcIn
            | Anim::LinkPillIn
            | Anim::PageIn
            | Anim::ComposeRise
            | Anim::BoatReturn
            | Anim::PillUp
            | Anim::PaneInR
            | Anim::PaneInL
            | Anim::PaneInROut
            | Anim::OsdIn
            | Anim::BannerIn
            | Anim::PanelIn
            | Anim::ShotIn
            | Anim::MorphIn
            | Anim::RollIn => CrossFade(In),
            Anim::Fold
            | Anim::FoldHeavy
            | Anim::Crumple
            | Anim::CrumpleHeavy
            | Anim::Curl
            | Anim::CurlHeavy
            | Anim::TabOut
            | Anim::HcOut
            | Anim::Park
            | Anim::ComposeSend
            | Anim::Floatup
            | Anim::Sail
            | Anim::Spark
            | Anim::PaneOutL
            | Anim::PaneOutR
            | Anim::OsdOut
            | Anim::SheetOut
            | Anim::BannerOut
            | Anim::PanelOut
            | Anim::ShotOut
            | Anim::MorphOut
            | Anim::RollOut => CrossFade(Out),
            Anim::SealPop
            | Anim::Gulp
            | Anim::StarPop
            | Anim::ChipLand
            | Anim::Heal
            | Anim::Bump
            | Anim::ShakeX
            | Anim::Nudge
            | Anim::Shake
            | Anim::Breathe
            | Anim::Spin
            | Anim::PictureAccept
            | Anim::SealOut
            | Anim::NudgeUp => Still,
            Anim::ChipFlash
            | Anim::MenuOut
            | Anim::Fade
            | Anim::PaletteFade
            | Anim::Dest
            | Anim::RingDrain
            | Anim::FadeIn
            | Anim::Busy
            | Anim::LevelTick
            | Anim::MorphFadeIn
            | Anim::MorphFadeOut
            | Anim::Hold => Same,
        }
    }
}

/// Whether a keyframes body moves anything: a transform other than `none`, or a box property.
#[cfg(test)]
fn moves(body: &str) -> bool {
    const BOX: &[&str] = &[
        "left:", "top:", "right:", "bottom:", "width:", "height:", "margin", "inset:",
    ];
    let transformed = body
        .split("transform:")
        .skip(1)
        .any(|rest| !rest.trim_start().starts_with("none"));
    transformed || BOX.iter().any(|property| body.contains(property))
}

#[cfg(test)]
mod tests {
    use super::{ReducedForm, moves};
    use crate::css::MOTION;
    use crate::css::motion_css::keyframes as motion_keyframes;
    use crate::motion::Anim;

    fn body(name: &str) -> &'static str {
        motion_keyframes(MOTION)
            .into_iter()
            .find(|(found, _)| *found == name)
            .map_or("", |(_, body)| body)
    }

    #[test]
    fn every_moving_keyframe_has_a_still_reduced_form() {
        let mut wrong = Vec::new();
        for anim in Anim::ALL {
            let recipe = anim.recipe();
            let form = anim.reduced();
            let reduced = form.keyframes(recipe.keyframes);
            if moves(body(reduced)) {
                wrong.push(format!("{anim:?}: {} under Reduced still moves", reduced));
            }
            if form == ReducedForm::Same && moves(body(recipe.keyframes)) {
                wrong.push(format!(
                    "{anim:?}: {} moves and has no Reduced form",
                    recipe.keyframes
                ));
            }
            if body(reduced).is_empty() {
                wrong.push(format!("{anim:?}: no @keyframes {reduced}"));
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    #[test]
    fn what_counts_as_moving() {
        const CASES: &[(&str, bool)] = &[
            ("{ from{ opacity:0; } to{ opacity:1; } }", false),
            (
                "{ 0%,100%{ transform:none; box-shadow:0 0 0 2px red; } }",
                false,
            ),
            (
                "{ from{ stroke-dashoffset:0; } to{ stroke-dashoffset:57; } }",
                false,
            ),
            (
                "{ from{ transform:translateX(26px); } to{ transform:none; } }",
                true,
            ),
            ("{ 0%{ max-height:40px; } 100%{ max-height:0; } }", true),
        ];
        for &(body, want) in CASES {
            assert_eq!(moves(body), want, "{body}");
        }
    }
}
