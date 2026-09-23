//! Every animation the design system plays, as data (design/05-MOTION.md section 4).
//!
//! 46 variants: the canonical keyframe set minus `part` and `fade-in` (neither has a variant;
//! S's `fade` replaces `fade-in`), plus the three heavy exits an unread row plays 15 % slower
//! (principle 6): `FoldHeavy`, `CrumpleHeavy` and `CurlHeavy`, each its base keyframes at a
//! heavy duration (freeze amendment from wave 1, so `settle()` never drops a node before its
//! CSS ends), plus `ChipFlash`, quire's own keyframe for the person chip's 1200 ms ring (wave 1
//! integration amendment), plus four assignment rows that play a catalogue keyframe at another
//! recipe (wave 2 integration amendment, section 5 rows 7, 26, 37 and 64): `PaletteFade`,
//! `LinkPillIn`, `BubblePop` and `PeekFullIn`.
//!
//! Each recipe is section 5's assignment row for the element that plays it; where S and C both
//! assign a keyframe, S's row is the one (S wins). Keyframes S never assigns take C's row.

pub use super::recipe::{Fill, Iteration, Recipe};

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
    /// `chip-flash`: a mentioned person chip's ring, held for `--t-flash` (design/04-COMPONENTS.md
    /// section 10, design/06-INTERACTIONS.md section 2.5; wave 1 amendment).
    ChipFlash,
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
    /// `menu-pop`: every floating menu.
    MenuPop,
    /// `menu-pop` at `--t-quick`: the selection bubble (section 5 row 37).
    BubblePop,
    /// `cmdk-in`: C's command menu.
    CmdkIn,
    /// `peek-in`: peek and the command menu.
    PeekIn,
    /// `peek-in` at `--t-move --e-out`: the reader entering Full peek (section 5 row 64, C).
    PeekFullIn,
    /// `fade`: the scrim.
    Fade,
    /// `fade` at `--t-quick`: the command palette's backdrop (section 5 row 7).
    PaletteFade,
    /// `hc-in`: hover card, tooltip.
    HcIn,
    /// `hc-in` at `--t-quick --e-out`: the link pill (section 5 row 26).
    LinkPillIn,
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

impl Anim {
    /// Every animation, in the catalogue's order.
    pub const ALL: [Anim; 46] = [
        Anim::SealPop,
        Anim::Gulp,
        Anim::PopIn,
        Anim::RowIn,
        Anim::Rise,
        Anim::StarPop,
        Anim::Spark,
        Anim::ChipLand,
        Anim::ChipIn,
        Anim::ChipFlash,
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
        Anim::BubblePop,
        Anim::CmdkIn,
        Anim::PeekIn,
        Anim::PeekFullIn,
        Anim::Fade,
        Anim::PaletteFade,
        Anim::HcIn,
        Anim::LinkPillIn,
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
            Anim::ChipFlash => "a-chip-flash",
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
            Anim::BubblePop => "a-bubble-pop",
            Anim::CmdkIn => "a-cmdk-in",
            Anim::PeekIn => "a-peek-in",
            Anim::PeekFullIn => "a-peek-full-in",
            Anim::Fade => "a-fade",
            Anim::PaletteFade => "a-palette-fade",
            Anim::HcIn => "a-hc-in",
            Anim::LinkPillIn => "a-link-pill-in",
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
