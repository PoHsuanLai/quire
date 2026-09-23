//! Every animation the design system plays, as data (design/05-MOTION.md section 4).
//!
//! 41 variants: the canonical keyframe set minus `part` and `fade-in` (neither has a variant;
//! S's `fade` replaces `fade-in`), plus the three heavy exits an unread row plays 15 % slower
//! (principle 6): `FoldHeavy`, `CrumpleHeavy` and `CurlHeavy`, each its base keyframes at a
//! heavy duration (freeze amendment from wave 1, so `settle()` never drops a node before its
//! CSS ends).
//!
//! Each recipe is section 5's assignment row for the element that plays it; where S and C both
//! assign a keyframe, S's row is the one (S wins). Keyframes S never assigns take C's row.

use crate::tokens::{DurationToken, EasingToken};

/// One animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anim {
    /// `seal-pop`: the current place's dot.
    SealPop,
    /// `gulp`: the place that received something.
    Gulp,
    /// `pop-in`: strip buttons.
    PopIn,
    /// `row-in`: an arriving or restored row (C).
    RowIn,
    /// `rise`: rows when a list is first shown; reader blocks.
    Rise,
    /// `star-pop`: the star.
    StarPop,
    /// `spark`: the star's sparks, starring only.
    Spark,
    /// `chip-land`: a label chip after a drop (C).
    ChipLand,
    /// `chip-in`: a person chip.
    ChipIn,
    /// `fold`: archive.
    Fold,
    /// `fold` at `--t-big-heavy`: an unread row's archive.
    FoldHeavy,
    /// `crumple`: trash and delete (C).
    Crumple,
    /// `crumple` at `--t-crumple-heavy`: an unread row's trash.
    CrumpleHeavy,
    /// `curl`: snooze.
    Curl,
    /// `curl` at `--t-curl-heavy`: an unread row's snooze.
    CurlHeavy,
    /// `heal`: rows below a removed row close the gap.
    Heal,
    /// `bump`: a count that changed.
    Bump,
    /// `tab-in`: a Today entry opens.
    TabIn,
    /// `tab-out`: a Today entry closes.
    TabOut,
    /// `slide-r`: Space switch forward, side peek.
    SlideR,
    /// `slide-l`: Space switch back.
    SlideL,
    /// `menu-in`: C's trigger-anchored menus.
    MenuIn,
    /// `menu-pop`: every floating menu, the selection bubble.
    MenuPop,
    /// `cmdk-in`: C's command menu.
    CmdkIn,
    /// `peek-in`: peek and the command menu.
    PeekIn,
    /// `fade`: scrim and backdrop.
    Fade,
    /// `hc-in`: hover card, link pill.
    HcIn,
    /// `hc-out`: hover card leaving.
    HcOut,
    /// `page-in`: composer page, inline reply.
    PageIn,
    /// `park`: the composer page parking.
    Park,
    /// `shake-x`: the To row with no recipient.
    ShakeX,
    /// `nudge`: outbox retry (C).
    Nudge,
    /// `shake`: outbox needs sign-in (C).
    Shake,
    /// `compose-rise`: the floating composer (C).
    ComposeRise,
    /// `compose-send`: the composer sending.
    ComposeSend,
    /// `floatup`: the snooze zZ.
    Floatup,
    /// `sail`: the orphaned boat (C).
    Sail,
    /// `boat-return`: the orphaned boat returning (C).
    BoatReturn,
    /// `dest`: the destination preview, alternating while hovered.
    Dest,
    /// `breathe`: the idle sync halo, looping.
    Breathe,
    /// `spin`: the busy sync halo, looping.
    Spin,
}

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
    /// Every animation, in the catalogue's order.
    pub const ALL: [Anim; 41] = [
        Anim::SealPop,
        Anim::Gulp,
        Anim::PopIn,
        Anim::RowIn,
        Anim::Rise,
        Anim::StarPop,
        Anim::Spark,
        Anim::ChipLand,
        Anim::ChipIn,
        Anim::Fold,
        Anim::FoldHeavy,
        Anim::Crumple,
        Anim::CrumpleHeavy,
        Anim::Curl,
        Anim::CurlHeavy,
        Anim::Heal,
        Anim::Bump,
        Anim::TabIn,
        Anim::TabOut,
        Anim::SlideR,
        Anim::SlideL,
        Anim::MenuIn,
        Anim::MenuPop,
        Anim::CmdkIn,
        Anim::PeekIn,
        Anim::Fade,
        Anim::HcIn,
        Anim::HcOut,
        Anim::PageIn,
        Anim::Park,
        Anim::ShakeX,
        Anim::Nudge,
        Anim::Shake,
        Anim::ComposeRise,
        Anim::ComposeSend,
        Anim::Floatup,
        Anim::Sail,
        Anim::BoatReturn,
        Anim::Dest,
        Anim::Breathe,
        Anim::Spin,
    ];

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
            // `C:714`.
            // `C:714`, at t-big x 1.15 as `S:329` weights an unread fold.
            Anim::CrumpleHeavy => recipe(
                "crumple",
                DurationToken::CrumpleHeavy,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            Anim::Crumple => recipe(
                "crumple",
                DurationToken::Big,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
            // `S:328`.
            // `S:328` at 560 x 1.15 (design/05-MOTION.md open decision 3, proposed).
            Anim::CurlHeavy => recipe(
                "curl",
                DurationToken::CurlHeavy,
                EasingToken::Exit,
                Fill::Forwards,
                Iteration::Once,
            ),
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
            // `C:1064`.
            Anim::CmdkIn => recipe(
                "cmdk-in",
                DurationToken::Big,
                EasingToken::Spring,
                Fill::None,
                Iteration::Once,
            ),
            // `S:225`.
            Anim::PeekIn => recipe(
                "peek-in",
                DurationToken::Big,
                EasingToken::Spring,
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
            // `S:401`.
            Anim::HcIn => recipe(
                "hc-in",
                DurationToken::Move,
                EasingToken::Spring,
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
        }
    }

    /// The utility class a pulse renders: `a-gulp`.
    pub fn class(self) -> &'static str {
        match self {
            Anim::SealPop => "a-seal-pop",
            Anim::Gulp => "a-gulp",
            Anim::PopIn => "a-pop-in",
            Anim::RowIn => "a-row-in",
            Anim::Rise => "a-rise",
            Anim::StarPop => "a-star-pop",
            Anim::Spark => "a-spark",
            Anim::ChipLand => "a-chip-land",
            Anim::ChipIn => "a-chip-in",
            Anim::Fold => "a-fold",
            Anim::FoldHeavy => "a-fold-heavy",
            Anim::Crumple => "a-crumple",
            Anim::CrumpleHeavy => "a-crumple-heavy",
            Anim::Curl => "a-curl",
            Anim::CurlHeavy => "a-curl-heavy",
            Anim::Heal => "a-heal",
            Anim::Bump => "a-bump",
            Anim::TabIn => "a-tab-in",
            Anim::TabOut => "a-tab-out",
            Anim::SlideR => "a-slide-r",
            Anim::SlideL => "a-slide-l",
            Anim::MenuIn => "a-menu-in",
            Anim::MenuPop => "a-menu-pop",
            Anim::CmdkIn => "a-cmdk-in",
            Anim::PeekIn => "a-peek-in",
            Anim::Fade => "a-fade",
            Anim::HcIn => "a-hc-in",
            Anim::HcOut => "a-hc-out",
            Anim::PageIn => "a-page-in",
            Anim::Park => "a-park",
            Anim::ShakeX => "a-shake-x",
            Anim::Nudge => "a-nudge",
            Anim::Shake => "a-shake",
            Anim::ComposeRise => "a-compose-rise",
            Anim::ComposeSend => "a-compose-send",
            Anim::Floatup => "a-floatup",
            Anim::Sail => "a-sail",
            Anim::BoatReturn => "a-boat-return",
            Anim::Dest => "a-dest",
            Anim::Breathe => "a-breathe",
            Anim::Spin => "a-spin",
        }
    }
}

/// One assignment row as a [`Recipe`].
const fn recipe(
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
