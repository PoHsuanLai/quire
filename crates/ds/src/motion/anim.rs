//! Every animation the design system plays, as data (design/05-MOTION.md section 4).
//!
//! 61 variants: the canonical keyframe set minus `part`, plus the three heavy exits an unread row plays 15 % slower
//! (principle 6): `FoldHeavy`, `CrumpleHeavy` and `CurlHeavy`, each its base keyframes at a
//! heavy duration (freeze amendment from wave 1, so `settle()` never drops a node before its
//! CSS ends), plus `ChipFlash`, quire's own keyframe for the person chip's 1200 ms ring (wave 1
//! integration amendment), plus four assignment rows that play a catalogue keyframe at another
//! recipe (wave 2 integration amendment, section 5 rows 7, 26, 37 and 64): `PaletteFade`,
//! `LinkPillIn`, `BubblePop` and `PeekFullIn`, plus `MenuOut`, quire's own fade for a menu
//! closed by Escape or an outside click (bar gaps), plus `CmdkRise`, `cmdk-in` without its fade,
//! for a command panel that must be opaque on its first frame (mailo gaps 2), plus four for
//! states the catalogue has no motion for (mailo gaps 3): `PillUp` (a centred pill's entrance),
//! `RingDrain` (the send ring's countdown), `FadeIn` (C's veil, to `--veil`, where S's `fade`
//! runs to 1) and `Busy` (a legible pulse; `Breathe` fades to nothing), plus four for the
//! control center's pane switch (sill Q80): `PaneInR` and `PaneInL` play `slide-r` and
//! `slide-l` at `--t-move` (design/13 section 13.3.7) where the catalogue's rows are `--t-big`,
//! and `PaneOutL` and `PaneOutR` carry the outgoing pane away the other way, plus the OSD's pair (sill FINDINGS Q75):
//! `OsdIn` and `OsdOut`, and `LevelTick`, the level control's quiet mark when its fill crosses a
//! step, plus `SheetOut`, the sheet's exit (sill FINDINGS Q90), plus `BannerOut`, a
//! notification banner's slide out to the right (sill Q121, Q122).
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
    /// `menu-out`: a menu closed by Escape or an outside click fades before it goes
    /// (design/13-BEHAVIOUR-menus-windows.md section 13.3.2, "fade on close"; bar gaps).
    MenuOut,
    /// `menu-pop` at `--t-quick`: the selection bubble (section 5 row 37).
    BubblePop,
    /// `cmdk-in`: C's command menu.
    CmdkIn,
    /// `cmdk-rise`: `cmdk-in`'s scale and lift with no fade, so the panel is opaque from its
    /// first frame (mailo gaps 2: a command panel is never drawn invisible).
    CmdkRise,
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
    /// `pill-up`: a pill centred by `translateX(-50%)` (a consumer's toast, a send pill of its
    /// own) springs up from below (mailo gaps 3).
    PillUp,
    /// `ring-drain`: a countdown ring's `stroke-dashoffset` drains over the send's grace
    /// period, linear, where CSS reaches the ring (the webview; on Blitz the SendPill writes
    /// the offset as an attribute, spike S6) (mailo gaps 3).
    RingDrain,
    /// `fade-in`: an ink veil fades in to `--veil` rather than to 1 (C:1055; mailo gaps 3).
    FadeIn,
    /// `busy`: a busy word pulses, never below .45, looping (mailo gaps 3).
    Busy,
    /// `slide-r` at `--t-move --e-spring`: a detail pane arriving from the right (sill Q80).
    PaneInR,
    /// `slide-l` at `--t-move --e-spring`: the root pane coming back from the left (sill Q80).
    PaneInL,
    /// `pane-out-l`: the root pane leaving to the left as its detail arrives (sill Q80).
    PaneOutL,
    /// `pane-out-r`: a detail pane leaving to the right as the root comes back (sill Q80).
    PaneOutR,
    /// `osd-in`: the OSD card comes in from `--osd-dy` (8 px above it at the top right, below
    /// it at the bottom centre) and fades in, from a .96 scale, over `--t-quick --e-out`:
    /// `pop-in`'s entrance without its overshoot, since a level shown under a key press must not
    /// bounce (design/20 section 1.7; sill FINDINGS Q75).
    OsdIn,
    /// `osd-out`: the OSD card fades out and moves half of `--osd-dy` back the way it came (a
    /// lift at the top right, a drop at the bottom centre) over `--t-move --e-exit`, the exit
    /// design/05 section 10 gives shell chrome, once its hold ends (sill FINDINGS Q75).
    OsdOut,
    /// `level-tick`: the level control's fill edge marks a step crossed, once, at `--t-tap`
    /// (`Tick::Quiet`; the sound is the shell's).
    LevelTick,
    /// `sheet-out`: a sheet leaving fades and settles 8 px back down, from a .98 scale's worth
    /// of shrink, over `--t-move --e-exit` (design/05 section 10: exits accelerate), the way it
    /// came in by `peek-in` reversed and quieter (sill FINDINGS Q90).
    SheetOut,
    /// `banner-out`: a notification banner slides out to the right from wherever a swipe left
    /// it (`--swipe-dx`) and fades, over `--t-move --e-exit`, the exit design/05 section 10
    /// gives shell chrome; the rows below it then heal (sill Q121, Q122).
    BannerOut,
}

impl Anim {
    /// Every animation, in the catalogue's order.
    pub const ALL: [Anim; 61] = [
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
        Anim::MenuOut,
        Anim::BubblePop,
        Anim::CmdkIn,
        Anim::CmdkRise,
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
        Anim::PillUp,
        Anim::RingDrain,
        Anim::FadeIn,
        Anim::Busy,
        Anim::PaneInR,
        Anim::PaneInL,
        Anim::PaneOutL,
        Anim::PaneOutR,
        Anim::OsdIn,
        Anim::OsdOut,
        Anim::LevelTick,
        Anim::SheetOut,
        Anim::BannerOut,
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
            Anim::MenuOut => "a-menu-out",
            Anim::BubblePop => "a-bubble-pop",
            Anim::CmdkIn => "a-cmdk-in",
            Anim::CmdkRise => "a-cmdk-rise",
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
            Anim::PillUp => "a-pill-up",
            Anim::RingDrain => "a-ring-drain",
            Anim::FadeIn => "a-fade-in",
            Anim::Busy => "a-busy",
            Anim::PaneInR => "a-pane-in-r",
            Anim::PaneInL => "a-pane-in-l",
            Anim::PaneOutL => "a-pane-out-l",
            Anim::PaneOutR => "a-pane-out-r",
            Anim::OsdIn => "a-osd-in",
            Anim::OsdOut => "a-osd-out",
            Anim::LevelTick => "a-level-tick",
            Anim::SheetOut => "a-sheet-out",
            Anim::BannerOut => "a-banner-out",
        }
    }
}
