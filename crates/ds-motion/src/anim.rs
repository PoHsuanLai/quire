//! Every animation the design system plays, as data (design/05-MOTION.md section 4).
//!
//! 92 variants: the canonical keyframe set minus `part`, plus the three heavy exits an unread row plays 15 % slower
//! (principle 6): `FoldHeavy`, `CrumpleHeavy` and `CurlHeavy`, each its base keyframes at a
//! heavy duration (so `settle()` never drops a node before its
//! CSS ends), plus `ChipFlash`, quire's own keyframe for the person chip's 1200 ms ring, plus four
//! assignment rows that play a catalogue keyframe at another
//! recipe (section 5 rows 7, 26, 37 and 64): `PaletteFade`,
//! `PeekFullIn`, plus `MenuOut`, quire's own fade for a menu
//! closed by Escape or an outside click (bar gaps), plus `CmdkRise`, `cmdk-in` without its fade,
//! for a command panel that must be opaque on its first frame, plus four for
//! states the catalogue has no motion for: `FadeIn` (C's veil, to `--veil`, where S's `fade`
//! runs to 1) and `Busy` (a legible pulse; `Breathe` fades to nothing), plus four for the
//! control center's pane switch: `PaneInR` and `PaneInL` play `slide-r` and
//! `slide-l` at `--t-move` (design/13 section 13.3.7) where the catalogue's rows are `--t-big`,
//! and `PaneOutL` and `PaneOutR` carry the outgoing pane away the other way, plus the OSD's exit:
//! `OsdOut`, and `LevelTick`, the level control's quiet mark when its fill crosses a
//! step, plus `SheetOut`, the sheet's exit, plus `PanelIn` and `PanelOut`, the slide in and out
//! of an edge panel, a notification banner, a toast and a screenshot thumbnail, plus
//! the small-state details' eight (design/26-DETAILS.md
//! section 3.2): `MorphIn`, `MorphOut`, `MorphFadeIn`, `MorphFadeOut`,
//! `SealOut` and `NudgeUp`, plus `Hold`, a keyframe that moves nothing, which a resting state
//! plays so the restyle that drops a running animation starts another, plus
//! `PaneInROut`, the preview pane's entrance nobody touched (design/26 R5), plus
//! `MorphInSpring`, `morph-in` on the glyph the person pressed (design/26 R5, play/pause, a
//! Focus disc), plus the symbol effects' CSS-driven thirteen (design/35-SYMBOL-EFFECTS.md):
//! `Bounce`, `Pulse`, `Wiggle` and `Breathe` once and as a loop, `RotateOnce` (its loop is
//! `Turn`), `ScaleUp`, `ScaleDown`, `Appear` and `Disappear`.
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
    /// `menu-out`: a menu closed by Escape or an outside click fades before it goes
    /// (design/13-BEHAVIOUR-menus-windows.md section 13.3.2, "fade on close"; bar gaps).
    MenuOut,
    /// `peek-in`: peek and the command menu.
    PeekIn,
    /// `peek-in` at `--t-move --e-out`: the reader entering Full peek (section 5 row 64, C).
    PeekFullIn,
    /// `fade`: the scrim.
    Fade,
    /// `fade` at `--t-quick`: the command palette's backdrop, and the fade in of a popover, a
    /// tooltip and a hover card (design/30 section 1.3).
    PaletteFade,
    /// `shake-x`: the To row with no recipient.
    ShakeX,
    /// `shake`: outbox needs sign-in (C).
    Shake,
    /// `fade-in`: an ink veil fades in to `--veil` rather than to 1 (C:1055).
    FadeIn,
    /// `slide-r` at `--t-move --e-spring`: a detail pane arriving from the right.
    PaneInR,
    /// `slide-l` at `--t-move --e-spring`: the root pane coming back from the left.
    PaneInL,
    /// `pane-out-l`: the root pane leaving to the left as its detail arrives.
    PaneOutL,
    /// `pane-out-r`: a detail pane leaving to the right as the root comes back.
    PaneOutR,
    /// `osd-out`: the OSD card fades out over `--t-move --e-exit` once its hold ends
    /// (design/30 section 1.3: the OSD enters with `PaletteFade`, `--t-quick`).
    OsdOut,
    /// `sheet-in` at `--t-quick --e-out`: a sheet or alert fades and scales in place (design/34
    /// section 7).
    SheetIn,
    /// `sheet-out`: a sheet or alert fades and scales out over `--t-quick --e-exit` (design/34
    /// section 7).
    SheetOut,
    /// `panel-in`: an edge panel (the notification center) slides in from past the right edge
    /// over `--t-move --e-out`: a large surface decelerates in, since a spring's overshoot would
    /// pull it off the edge it is anchored to.
    PanelIn,
    /// `panel-out`: the edge panel, or a toast, slides back out past the edge over
    /// `--t-quick --e-exit` (design/30 section 1.3), holding its last frame until the host
    /// unmaps it at `settle(PanelOut)`.
    PanelOut,
    /// `morph-in`: a glyph growing into a new state (design/26 `MorphGlyph`).
    MorphIn,
    /// `morph-out`: the glyph it replaces shrinking away.
    MorphOut,
    /// `fade` at `--t-quick`: a cross-fade's incoming glyph.
    MorphFadeIn,
    /// `morph-fade-out`: a cross-fade's outgoing glyph.
    MorphFadeOut,
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
    /// `turn` at `--t-turn` linear, forever: a busy button's icon turning (a Get Mail arrow).
    /// Under Reduced it plays `hold` once, so the glyph stays still.
    Turn,
    /// `symbol-bounce` once at `--t-big`: a symbol hops and settles (design/35 section 2).
    Bounce,
    /// `symbol-bounce` at `--t-big`, forever: a symbol bouncing while its work goes on.
    BounceLoop,
    /// `symbol-pulse` once: a symbol dims and returns.
    Pulse,
    /// `symbol-pulse` at `--t-turn`, forever.
    PulseLoop,
    /// `symbol-wiggle` once: a symbol turns a few degrees each way, each smaller.
    Wiggle,
    /// `symbol-wiggle` at `--t-turn`, forever.
    WiggleLoop,
    /// `symbol-breathe` once: a symbol swells and settles.
    Breathe,
    /// `symbol-breathe` at `--t-turn`, forever.
    BreatheLoop,
    /// `turn` once at `--t-big`: one revolution of a symbol.
    RotateOnce,
    /// `symbol-scale-up`: a symbol held larger while its effect is active.
    ScaleUp,
    /// `symbol-scale-down`: a symbol held smaller while its effect is active.
    ScaleDown,
    /// `morph-in` at `--t-move`: a symbol arriving.
    Appear,
    /// `morph-out` at `--t-quick --e-exit`: a symbol leaving, holding its last frame.
    Disappear,
}

impl Anim {
    /// Every animation, in the catalogue's order.
    pub const ALL: [Anim; 43] = [
        Anim::RowIn,
        Anim::RowOut,
        Anim::Heal,
        Anim::SlideR,
        Anim::SlideL,
        Anim::MenuOut,
        Anim::PeekIn,
        Anim::PeekFullIn,
        Anim::Fade,
        Anim::PaletteFade,
        Anim::ShakeX,
        Anim::Shake,
        Anim::FadeIn,
        Anim::PaneInR,
        Anim::PaneInL,
        Anim::PaneOutL,
        Anim::PaneOutR,
        Anim::OsdOut,
        Anim::SheetIn,
        Anim::SheetOut,
        Anim::PanelIn,
        Anim::PanelOut,
        Anim::MorphIn,
        Anim::MorphOut,
        Anim::MorphFadeIn,
        Anim::MorphFadeOut,
        Anim::Hold,
        Anim::PaneInROut,
        Anim::MorphInSpring,
        Anim::Turn,
        Anim::Bounce,
        Anim::BounceLoop,
        Anim::Pulse,
        Anim::PulseLoop,
        Anim::Wiggle,
        Anim::WiggleLoop,
        Anim::Breathe,
        Anim::BreatheLoop,
        Anim::RotateOnce,
        Anim::ScaleUp,
        Anim::ScaleDown,
        Anim::Appear,
        Anim::Disappear,
    ];

    /// The utility class a pulse renders: `a-gulp`.
    pub fn class(self) -> &'static str {
        match self {
            Anim::RowIn => "a-row-in",
            Anim::RowOut => "a-row-out",
            Anim::Heal => "a-heal",
            Anim::SlideR => "a-slide-r",
            Anim::SlideL => "a-slide-l",
            Anim::MenuOut => "a-menu-out",
            Anim::PeekIn => "a-peek-in",
            Anim::PeekFullIn => "a-peek-full-in",
            Anim::Fade => "a-fade",
            Anim::PaletteFade => "a-palette-fade",
            Anim::ShakeX => "a-shake-x",
            Anim::Shake => "a-shake",
            Anim::FadeIn => "a-fade-in",
            Anim::PaneInR => "a-pane-in-r",
            Anim::PaneInL => "a-pane-in-l",
            Anim::PaneOutL => "a-pane-out-l",
            Anim::PaneOutR => "a-pane-out-r",
            Anim::OsdOut => "a-osd-out",
            Anim::SheetIn => "a-sheet-in",
            Anim::SheetOut => "a-sheet-out",
            Anim::PanelIn => "a-panel-in",
            Anim::PanelOut => "a-panel-out",
            Anim::MorphIn => "a-morph-in",
            Anim::MorphOut => "a-morph-out",
            Anim::MorphFadeIn => "a-morph-fade-in",
            Anim::MorphFadeOut => "a-morph-fade-out",
            Anim::Hold => "a-hold",
            Anim::PaneInROut => "a-pane-in-r-out",
            Anim::MorphInSpring => "a-morph-in-spring",
            Anim::Turn => "a-turn",
            Anim::Bounce => "a-bounce",
            Anim::BounceLoop => "a-bounce-loop",
            Anim::Pulse => "a-pulse",
            Anim::PulseLoop => "a-pulse-loop",
            Anim::Wiggle => "a-wiggle",
            Anim::WiggleLoop => "a-wiggle-loop",
            Anim::Breathe => "a-breathe",
            Anim::BreatheLoop => "a-breathe-loop",
            Anim::RotateOnce => "a-rotate-once",
            Anim::ScaleUp => "a-scale-up",
            Anim::ScaleDown => "a-scale-down",
            Anim::Appear => "a-appear",
            Anim::Disappear => "a-disappear",
        }
    }
}
