//! Each [`Anim`]'s canonical declaration: the keyframes, duration and easing tokens, fill and
//! iteration that the stylesheet writes and [`crate::settle`] times (design/05-MOTION.md
//! section 5).
//!
//! Each recipe is section 5's assignment row for the element that plays it; where S and C both
//! assign a keyframe, S's row is the one (S wins). Keyframes S never assigns take C's row.

use super::anim::Anim;
use super::recipe_own as own;
use crate::tokens::{DurationToken, EasingToken};

/// `animation-fill-mode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Fill {
    /// `none`.
    None,
    /// `forwards`: exits hold their last frame until the roster drops the node.
    Forwards,
    /// `backwards`: staggered entrances hold their first frame through the delay.
    Backwards,
}

/// `animation-iteration-count` and direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Iteration {
    /// Once.
    Once,
    /// `infinite`.
    Infinite,
    /// `infinite alternate`.
    InfiniteAlternate,
}

/// The canonical declaration of one animation: what the stylesheet writes and what
/// [`crate::settle`] times.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Recipe {
    /// The `@keyframes` name.
    pub keyframes: &'static str,
    /// The duration token.
    pub duration: DurationToken,
    /// The easing token.
    pub easing: EasingToken,
    /// The fill mode.
    pub fill: Fill,
    /// The iteration.
    pub iteration: Iteration,
}

impl Anim {
    /// The canonical declaration.
    pub fn recipe(self) -> Recipe {
        match self {
            // `S:346`.
            Anim::SealPop => recipe(
                "seal-pop",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:339`.
            Anim::Gulp => recipe(
                "gulp",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:319`.
            Anim::PopIn => recipe(
                "pop-in",
                DurationToken::Move,
                EasingToken::Spring,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `C:373`.
            Anim::RowIn => recipe(
                "row-in",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::Backwards,
                Iteration::Once,
            ),
            // `S:294`.
            Anim::Rise => recipe(
                "rise",
                DurationToken::Move,
                EasingToken::Out,
                Fill::Backwards,
                Iteration::Once,
            ),
            // `S:304`.
            Anim::StarPop => recipe(
                "star-pop",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:307`.
            Anim::Spark => recipe(
                "spark",
                DurationToken::Spark,
                EasingToken::Out,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `C:402`.
            Anim::ChipLand => recipe(
                "chip-land",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:593`.
            Anim::ChipIn => recipe(
                "chip-in",
                DurationToken::Move,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            Anim::ChipFlash => own::CHIP_FLASH,
            // `S:327`.
            Anim::Fold => recipe(
                "fold",
                DurationToken::Big,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:329`.
            Anim::FoldHeavy => recipe(
                "fold",
                DurationToken::BigHeavy,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `C:714`, at t-big x 1.15 as `S:329` weights an unread fold.
            Anim::CrumpleHeavy => recipe(
                "crumple",
                DurationToken::CrumpleHeavy,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `C:714`.
            Anim::Crumple => recipe(
                "crumple",
                DurationToken::Big,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:328` at 560 x 1.15 (design/05-MOTION.md open decision 3, proposed).
            Anim::CurlHeavy => recipe(
                "curl",
                DurationToken::CurlHeavy,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:328`.
            Anim::Curl => recipe(
                "curl",
                DurationToken::Curl,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:330`.
            Anim::Heal => recipe(
                "heal",
                DurationToken::Move,
                EasingToken::Spring,
                Fill::Backwards,
                Iteration::Once,
            ),
            // `S:341`.
            Anim::Bump => recipe(
                "bump",
                DurationToken::Move,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:350`.
            Anim::TabIn => recipe(
                "tab-in",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:351`.
            Anim::TabOut => recipe(
                "tab-out",
                DurationToken::Move,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:102`.
            Anim::SlideR => recipe(
                "slide-r",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:103`.
            Anim::SlideL => recipe(
                "slide-l",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `C:1007`.
            Anim::MenuIn => recipe(
                "menu-in",
                DurationToken::Move,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:666`.
            Anim::MenuPop => recipe(
                "menu-pop",
                DurationToken::Move,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            Anim::MenuOut => own::MENU_OUT,
            // `S:684`.
            Anim::BubblePop => recipe(
                "menu-pop",
                DurationToken::Quick,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `C:1064`.
            Anim::CmdkIn => recipe(
                "cmdk-in",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            Anim::CmdkRise => own::CMDK_RISE,
            // `S:225`.
            Anim::PeekIn => recipe(
                "peek-in",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `C:1050`.
            Anim::PeekFullIn => recipe(
                "peek-in",
                DurationToken::Move,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:222`.
            Anim::Fade => recipe(
                "fade",
                DurationToken::Move,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:231`.
            Anim::PaletteFade => recipe(
                "fade",
                DurationToken::Quick,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:401`.
            Anim::HcIn => recipe(
                "hc-in",
                DurationToken::Move,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:433`.
            Anim::LinkPillIn => recipe(
                "hc-in",
                DurationToken::Quick,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:402`.
            Anim::HcOut => recipe(
                "hc-out",
                DurationToken::HcOut,
                EasingToken::Out,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:568`.
            Anim::PageIn => recipe(
                "page-in",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:570`.
            Anim::Park => recipe(
                "park",
                DurationToken::Park,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:590`.
            Anim::ShakeX => recipe(
                "shake-x",
                DurationToken::Shake,
                EasingToken::Shake,
                Fill::None,
                Iteration::Once,
            ),
            // `C:545`.
            Anim::Nudge => recipe(
                "nudge",
                DurationToken::Nudge,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `C:546`.
            Anim::Shake => recipe(
                "shake",
                DurationToken::ShakeLong,
                EasingToken::Shake,
                Fill::None,
                Iteration::Once,
            ),
            // `C:506`.
            Anim::ComposeRise => recipe(
                "compose-rise",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:572`.
            Anim::ComposeSend => recipe(
                "compose-send",
                DurationToken::Send,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:335`.
            Anim::Floatup => recipe(
                "floatup",
                DurationToken::Float,
                EasingToken::Out,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `C:535`.
            Anim::Sail => recipe(
                "sail",
                DurationToken::Sail,
                EasingToken::Out,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `C:536`.
            Anim::BoatReturn => recipe(
                "boat-return",
                DurationToken::BoatReturn,
                EasingToken::Out,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:447`.
            Anim::Dest => recipe(
                "dest",
                DurationToken::Float,
                EasingToken::Out,
                Fill::None,
                Iteration::InfiniteAlternate,
            ),
            // `C:288`.
            Anim::Breathe => recipe(
                "breathe",
                DurationToken::Ambient,
                EasingToken::InOut,
                Fill::None,
                Iteration::Infinite,
            ),
            // `C:291`.
            Anim::Spin => recipe(
                "spin",
                DurationToken::Spin,
                EasingToken::Linear,
                Fill::None,
                Iteration::Infinite,
            ),
            Anim::PillUp => own::PILL_UP,
            Anim::RingDrain => own::RING_DRAIN,
            Anim::FadeIn => own::FADE_IN,
            Anim::Busy => own::BUSY,
            Anim::OsdIn => own::OSD_IN,
            Anim::OsdOut => own::OSD_OUT,
        }
    }
}

/// One assignment row as a [`Recipe`].
pub(super) const fn recipe(
    keyframes: &'static str,
    duration: DurationToken,
    easing: EasingToken,
    fill: Fill,
    iteration: Iteration,
) -> Recipe {
    Recipe {
        keyframes,
        duration,
        easing,
        fill,
        iteration,
    }
}
