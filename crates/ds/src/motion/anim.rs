//! Every animation the design system plays, as data (design/05-MOTION.md section 4).
//!
//! 39 variants: the canonical keyframe set minus `part` and `fade-in` (neither has a variant;
//! S's `fade` replaces `fade-in`), plus `FoldHeavy`, which is `fold` at `--t-big-heavy`.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

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
    /// `curl`: snooze.
    Curl,
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
    pub const ALL: [Anim; 39] = [
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
        Anim::Curl,
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
        todo!()
    }

    /// The utility class a pulse renders: `a-gulp`.
    pub fn class(self) -> &'static str {
        todo!()
    }
}
