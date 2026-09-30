//! Each [`Anim`]'s canonical declaration: the keyframes, duration and easing tokens, fill and
//! iteration that the stylesheet writes and [`crate::settle`] times (design/05-MOTION.md
//! section 5).
//!
//! Each recipe is section 5's assignment row for the element that plays it; where S and C both
//! assign a keyframe, S's row is the one (S wins). Keyframes S never assigns take C's row.

use super::anim::Anim;
use super::recipe_detail as detail;
use super::recipe_own as own;
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

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
            // design/30 section 1.3: an inserted row fades and slides down over `--t-move`.
            Anim::RowIn => recipe(
                "row-in",
                DurationToken::Move,
                EasingToken::Out,
                Fill::Backwards,
                Iteration::Once,
            ),
            // A removed row fades and slides up over `--t-quick`, accelerating.
            Anim::RowOut => recipe(
                "row-out",
                DurationToken::Quick,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // The rows below close the gap over `--t-move`.
            Anim::Heal => recipe(
                "heal",
                DurationToken::Move,
                EasingToken::InOut,
                Fill::Backwards,
                Iteration::Once,
            ),
            // `S:102`.
            Anim::SlideR => recipe(
                "slide-r",
                DurationToken::Big,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:103`.
            Anim::SlideL => recipe(
                "slide-l",
                DurationToken::Big,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `C:1007`.
            Anim::MenuIn => recipe(
                "menu-in",
                DurationToken::Move,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:666`.
            Anim::MenuPop => recipe(
                "menu-pop",
                DurationToken::Move,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            Anim::MenuOut => own::MENU_OUT,
            // `S:684`.
            Anim::BubblePop => recipe(
                "menu-pop",
                DurationToken::Quick,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:225`.
            Anim::PeekIn => recipe(
                "peek-in",
                DurationToken::Big,
                EasingToken::Out,
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
            // `S:433`.
            Anim::LinkPillIn => recipe(
                "hc-in",
                DurationToken::Quick,
                EasingToken::Out,
                Fill::None,
                Iteration::Once,
            ),
            // `S:568`.
            Anim::PageIn => recipe(
                "page-in",
                DurationToken::Big,
                EasingToken::Out,
                Fill::None,
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
            // `C:546`.
            Anim::Shake => recipe(
                "shake",
                DurationToken::Shake,
                EasingToken::Shake,
                Fill::None,
                Iteration::Once,
            ),
            Anim::PillUp => own::PILL_UP,
            Anim::RingDrain => own::RING_DRAIN,
            Anim::FadeIn => own::FADE_IN,
            Anim::PaneInR => own::PANE_IN_R,
            Anim::PaneInL => own::PANE_IN_L,
            Anim::PaneOutL => own::PANE_OUT_L,
            Anim::PaneOutR => own::PANE_OUT_R,
            Anim::OsdOut => own::OSD_OUT,
            Anim::SheetIn => own::SHEET_IN,
            Anim::SheetOut => own::SHEET_OUT,
            Anim::PanelIn => own::PANEL_IN,
            Anim::PanelOut => own::PANEL_OUT,
            Anim::MorphIn => detail::MORPH_IN,
            Anim::MorphOut => detail::MORPH_OUT,
            Anim::MorphFadeIn => detail::MORPH_FADE_IN,
            Anim::MorphFadeOut => detail::MORPH_FADE_OUT,
            Anim::Hold => own::HOLD,
            Anim::PaneInROut => detail::PANE_IN_R_OUT,
            Anim::MorphInSpring => detail::MORPH_IN_SPRING,
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
