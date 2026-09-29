//! Every animation the design system plays, as data (design/05-MOTION.md section 4).
//!
//! 79 variants: the canonical keyframe set minus `part`, plus the three heavy exits an unread row plays 15 % slower
//! (principle 6): `FoldHeavy`, `CrumpleHeavy` and `CurlHeavy`, each its base keyframes at a
//! heavy duration (so `settle()` never drops a node before its
//! CSS ends), plus `ChipFlash`, quire's own keyframe for the person chip's 1200 ms ring, plus four
//! assignment rows that play a catalogue keyframe at another
//! recipe (section 5 rows 7, 26, 37 and 64): `PaletteFade`,
//! `LinkPillIn`, `BubblePop` and `PeekFullIn`, plus `MenuOut`, quire's own fade for a menu
//! closed by Escape or an outside click (bar gaps), plus `CmdkRise`, `cmdk-in` without its fade,
//! for a command panel that must be opaque on its first frame, plus four for
//! states the catalogue has no motion for: `PillUp` (a centred pill's entrance),
//! `RingDrain` (the send ring's countdown), `FadeIn` (C's veil, to `--veil`, where S's `fade`
//! runs to 1) and `Busy` (a legible pulse; `Breathe` fades to nothing), plus four for the
//! control center's pane switch: `PaneInR` and `PaneInL` play `slide-r` and
//! `slide-l` at `--t-move` (design/13 section 13.3.7) where the catalogue's rows are `--t-big`,
//! and `PaneOutL` and `PaneOutR` carry the outgoing pane away the other way, plus the OSD's pair:
//! `OsdIn` and `OsdOut`, and `LevelTick`, the level control's quiet mark when its fill crosses a
//! step, plus `SheetOut`, the sheet's exit, plus `BannerOut`, a
//! notification banner's slide out by its entry edge, and `BannerIn`, its
//! slide in, plus `PanelIn` and `PanelOut`, the notification center's edge panel,
//! plus `ShotIn` and `ShotOut`, the screenshot thumbnail's pair, plus
//! `PictureAccept`, the user picture's accept beat on unlock (design/25-EMOJI.md section 7), plus the small-state details' eight (design/26-DETAILS.md
//! section 3.2): `MorphIn`, `MorphOut`, `MorphFadeIn`, `MorphFadeOut`, `RollIn`, `RollOut`,
//! `SealOut` and `NudgeUp`, plus `Hold`, a keyframe that moves nothing, which a resting state
//! plays so the restyle that drops a running animation starts another, plus
//! `PaneInROut`, the preview pane's entrance nobody touched (design/26 R5), plus
//! `MorphInSpring`, `morph-in` on the glyph the person pressed (design/26 R5, play/pause, a
//! Focus disc), plus `WidgetOut`, a widget's card leaving when the person removes it.
//!
//! Each recipe is section 5's assignment row for the element that plays it; where S and C both
//! assign a keyframe, S's row is the one (S wins). Keyframes S never assigns take C's row.

/// One animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Anim {
    /// `row-in`: an arriving or restored row (C).
    RowIn,
    /// `row-out`: a row leaving a roster fades and slides up.
    RowOut,
    /// `heal`: rows below a removed row close the gap.
    Heal,
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
    /// first frame (a command panel is never drawn invisible).
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
    /// own) springs up from below.
    PillUp,
    /// `ring-drain`: a countdown ring's `stroke-dashoffset` drains over the send's grace
    /// period, linear (on Blitz the SendPill writes the offset as an attribute).
    RingDrain,
    /// `fade-in`: an ink veil fades in to `--veil` rather than to 1 (C:1055).
    FadeIn,
    /// `busy`: a busy word pulses, never below .45, looping.
    Busy,
    /// `slide-r` at `--t-move --e-spring`: a detail pane arriving from the right.
    PaneInR,
    /// `slide-l` at `--t-move --e-spring`: the root pane coming back from the left.
    PaneInL,
    /// `pane-out-l`: the root pane leaving to the left as its detail arrives.
    PaneOutL,
    /// `pane-out-r`: a detail pane leaving to the right as the root comes back.
    PaneOutR,
    /// `osd-in`: the OSD card comes in from `--osd-dy` (8 px above it at the top right, below
    /// it at the bottom centre) and fades in, from a .96 scale, over `--t-quick --e-out`:
    /// `pop-in`'s entrance without its overshoot, since a level shown under a key press must not
    /// bounce (design/20 section 1.7).
    OsdIn,
    /// `osd-out`: the OSD card fades out and moves half of `--osd-dy` back the way it came (a
    /// lift at the top right, a drop at the bottom centre) over `--t-move --e-exit`, the exit
    /// design/05 section 10 gives shell chrome, once its hold ends.
    OsdOut,
    /// `sheet-out`: a sheet leaving fades and settles 8 px back down, from a .98 scale's worth
    /// of shrink, over `--t-move --e-exit` (design/05 section 10: exits accelerate), the way it
    /// came in by `peek-in` reversed and quieter.
    SheetOut,
    /// `banner-out`: a notification banner slides out by its stack's entry edge (to the right
    /// by default, `--banner-dx`/`--banner-dy`) from wherever a swipe left it (`--swipe-dx`)
    /// and fades, over `--t-move --e-exit`, the exit design/05 section 10
    /// gives shell chrome; the rows below it then heal.
    BannerOut,
    /// `banner-in`: a notification banner slides in from its stack's entry edge (past the right
    /// edge by default, or from below; `--banner-dx`/`--banner-dy`) to its place,
    /// over `--t-move --e-spring` (design/13 section 13.3.6's entrance, at the
    /// stack's `--t-move` so a banner that arrives as another leaves moves with it).
    BannerIn,
    /// `panel-in`: an edge panel (the notification center) slides in from past the right edge
    /// over `--t-move --e-out`: a large surface decelerates in, since a spring's overshoot would
    /// pull it off the edge it is anchored to.
    PanelIn,
    /// `panel-out`: the edge panel slides back out past the edge over `--t-move --e-exit`,
    /// holding its last frame until the host unmaps it at `settle(PanelOut)`.
    PanelOut,
    /// `rise` at `--t-big --e-spring`: the screenshot thumbnail arrives as a surface, at the
    /// toast's spring rather than a row's `--t-move --e-out` (design/20 section 1.13).
    ShotIn,
    /// `shot-out`: the screenshot thumbnail slides out to the right past its own width and
    /// fades, over `--t-move --e-exit`, holding its last frame until the host unmaps it at
    /// `settle(ShotOut)` (design/20 section 1.13: "slide-r out").
    ShotOut,
    /// `morph-in`: a glyph growing into a new state (design/26 `MorphGlyph`).
    MorphIn,
    /// `morph-out`: the glyph it replaces shrinking away.
    MorphOut,
    /// `fade` at `--t-quick`: a cross-fade's incoming glyph.
    MorphFadeIn,
    /// `morph-fade-out`: a cross-fade's outgoing glyph.
    MorphFadeOut,
    /// `roll-in`: a changed digit rolling into place (`RollDigits`).
    RollIn,
    /// `roll-out`: the digit it replaces rolling away.
    RollOut,
    /// `hold`: moves nothing, for `--t-tap`. A state that comes after an animated one (a
    /// surface present after its entrance, a hide taken back, a row at rest) plays it, so the
    /// restyle that drops the running animation always starts another one. Blitz at the pinned
    /// rev keeps a cancelled animation's last value on the element until something restyles it
    /// again, and a new animation is that restyle.
    Hold,
    /// `slide-r` at `--t-move --e-out`: a preview pane shown by something other than the
    /// person's contact (design/26 R5), where `PaneInR` springs.
    PaneInROut,
    /// `morph-in` at `--t-quick --e-spring`: the incoming glyph of a DownUp or OffUp morph the
    /// person's own press caused (design/26 R5, play/pause, the Focus disc); `MorphIn` is the
    /// same growth at `--e-out` for a change from elsewhere.
    MorphInSpring,
    /// `widget-out` at `--t-move --e-exit`, forwards: a widget's card shrinking and fading as
    /// the person removes it from the desktop, held gone until the host drops it at
    /// `settle(WidgetOut)`.
    WidgetOut,
}

impl Anim {
    /// Every animation, in the catalogue's order.
    pub const ALL: [Anim; 58] = [
        Anim::RowIn,
        Anim::RowOut,
        Anim::Heal,
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
        Anim::SheetOut,
        Anim::BannerOut,
        Anim::BannerIn,
        Anim::PanelIn,
        Anim::PanelOut,
        Anim::ShotIn,
        Anim::ShotOut,
        Anim::MorphIn,
        Anim::MorphOut,
        Anim::MorphFadeIn,
        Anim::MorphFadeOut,
        Anim::RollIn,
        Anim::RollOut,
        Anim::Hold,
        Anim::PaneInROut,
        Anim::MorphInSpring,
        Anim::WidgetOut,
    ];

    /// The utility class a pulse renders: `a-gulp`.
    pub fn class(self) -> &'static str {
        match self {
            Anim::RowIn => "a-row-in",
            Anim::RowOut => "a-row-out",
            Anim::Heal => "a-heal",
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
            Anim::SheetOut => "a-sheet-out",
            Anim::BannerOut => "a-banner-out",
            Anim::BannerIn => "a-banner-in",
            Anim::PanelIn => "a-panel-in",
            Anim::PanelOut => "a-panel-out",
            Anim::ShotIn => "a-shot-in",
            Anim::ShotOut => "a-shot-out",
            Anim::MorphIn => "a-morph-in",
            Anim::MorphOut => "a-morph-out",
            Anim::MorphFadeIn => "a-morph-fade-in",
            Anim::MorphFadeOut => "a-morph-fade-out",
            Anim::RollIn => "a-roll-in",
            Anim::RollOut => "a-roll-out",
            Anim::Hold => "a-hold",
            Anim::PaneInROut => "a-pane-in-r-out",
            Anim::MorphInSpring => "a-morph-in-spring",
            Anim::WidgetOut => "a-widget-out",
        }
    }
}
