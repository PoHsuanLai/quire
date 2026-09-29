//! The recipes of the keyframes quire added to design/05-MOTION.md's catalogue, each for a
//! state the catalogue has no motion for. The catalogue's own rows are in `recipe.rs`.

use super::recipe::{Fill, Iteration, Recipe, recipe};
use crate::style::tokens::{easing::EasingToken, timing::DurationToken};

/// `menu-out` (design/13 section 13.3.2): "fade over `--t-quick` with `--e-exit`".
pub(super) const MENU_OUT: Recipe = recipe(
    "menu-out",
    DurationToken::Quick,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `cmdk-rise`: `cmdk-in`'s row with its fade taken out.
pub(super) const CMDK_RISE: Recipe = recipe(
    "cmdk-rise",
    DurationToken::Big,
    EasingToken::Spring,
    Fill::None,
    Iteration::Once,
);

/// `pill-up`: a pill centred by `translateX(-50%)` springs up from below the
/// edge, at the send pill's own `--t-big --e-spring` (`S:706`). An entrance, so it holds nothing.
pub(super) const PILL_UP: Recipe = recipe(
    "pill-up",
    DurationToken::Big,
    EasingToken::Spring,
    Fill::None,
    Iteration::Once,
);

/// `ring-drain`: the undo-send ring's dash drains over the send's grace period,
/// linear (`S:2339`), and holds empty. `--t-send-ring` is a hold: Reduced does not shorten the
/// time a person has to take a send back.
pub(super) const RING_DRAIN: Recipe = recipe(
    "ring-drain",
    DurationToken::SendRing,
    EasingToken::Linear,
    Fill::Forwards,
    Iteration::Once,
);

/// `fade-in` (C:1055): an ink veil fades to its resting `--veil` at `--t-move
/// --e-out`, where `fade` runs to 1.
pub(super) const FADE_IN: Recipe = recipe(
    "fade-in",
    DurationToken::Move,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `busy`: a busy word pulses at `--t-ambient --e-in-out` and never drops below
/// .45, so it stays legible where `breathe` fades the sync halo to nothing. It loops for as long
/// as the work does; Reduced plays it once, as every loop.
pub(super) const BUSY: Recipe = recipe(
    "busy",
    DurationToken::Ambient,
    EasingToken::InOut,
    Fill::None,
    Iteration::Infinite,
);

/// `slide-r` at the detail pane's `--t-move --e-spring` (design/13 section 13.3.7):
/// the catalogue's row plays it at `--t-big` for a Space switch, too slow inside a popover.
pub(super) const PANE_IN_R: Recipe = recipe(
    "slide-r",
    DurationToken::Move,
    EasingToken::Spring,
    Fill::None,
    Iteration::Once,
);

/// `slide-l` at the same `--t-move --e-spring`: the root pane coming back.
pub(super) const PANE_IN_L: Recipe = recipe(
    "slide-l",
    DurationToken::Move,
    EasingToken::Spring,
    Fill::None,
    Iteration::Once,
);

/// `pane-out-l`: the outgoing root leaves the way the detail pushes it, over the
/// same `--t-move` so both panes settle together, at `--e-exit` since an exit does not spring
/// (design/05 principle 2). It holds its last frame until the pane is dropped.
pub(super) const PANE_OUT_L: Recipe = recipe(
    "pane-out-l",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `pane-out-r`: the outgoing detail leaves to the right as the root comes back.
pub(super) const PANE_OUT_R: Recipe = recipe(
    "pane-out-r",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `osd-in`: the OSD card's entrance at design/20 section 1.7's `--t-quick
/// --e-out`. An entrance, so it holds nothing; the card is at rest when it ends.
pub(super) const OSD_IN: Recipe = recipe(
    "osd-in",
    DurationToken::Quick,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `osd-out`: the OSD card's exit at `--t-move --e-exit` (design/20 section
/// 1.7, design/05 section 10). It holds its last, transparent frame until the host unmaps the
/// surface at `settle(OsdOut)`, so the card never flashes back between the two.
pub(super) const OSD_OUT: Recipe = recipe(
    "osd-out",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `sheet-out`: a sheet's exit at `--t-move --e-exit`, the exit design/05
/// section 10 gives shell chrome. It holds its last, transparent frame until the host unmaps
/// the surface at `settle(SheetOut)`, so the sheet never flashes back between the two.
pub(super) const SHEET_OUT: Recipe = recipe(
    "sheet-out",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `widget-out`: a widget's card leaving when the person removes it, at `--t-move
/// --e-exit` (design/05 principle 3: exits accelerate), holding its last, transparent frame
/// until the host drops the card at `settle(WidgetOut)`.
pub(super) const WIDGET_OUT: Recipe = recipe(
    "widget-out",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `banner-out`: a banner's exit at `--t-move --e-exit` (design/13 section
/// 13.3.6: "timeout and dismiss both slide right, `--t-move --e-exit`"), from the offset a swipe
/// left it at. It holds its last, transparent frame until the stack drops the row at
/// `settle(BannerOut)` and the rows below heal.
pub(super) const BANNER_OUT: Recipe = recipe(
    "banner-out",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `banner-in`: a banner's entrance at `--t-move --e-spring`. design/13 section
/// 13.3.6 proposed `--t-big` (the design toast's); the stack plays it at `--t-move`, the length
/// of the exit and the heal it may arrive beside, so the stack moves as one. An entrance, so it
/// holds nothing.
pub(super) const BANNER_IN: Recipe = recipe(
    "banner-in",
    DurationToken::Move,
    EasingToken::Spring,
    Fill::None,
    Iteration::Once,
);

/// `panel-in`: the notification center's edge panel slides in at `--t-move --e-out`.
/// Opening it is not contact with the panel, and a spring's overshoot would lift it off the edge
/// it is anchored to, so it decelerates in (design/05 principle 2). An entrance; it holds
/// nothing.
pub(super) const PANEL_IN: Recipe = recipe(
    "panel-in",
    DurationToken::Move,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `panel-out`: the edge panel slides back out at `--t-move --e-exit` (design/05
/// section 10), holding its last frame until the host unmaps it at `settle(PanelOut)`.
pub(super) const PANEL_OUT: Recipe = recipe(
    "panel-out",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `slide-r` at `--t-move --e-out`: the screenshot thumbnail's entrance, as a notification
/// banner's (design/30 section 1.3). An entrance; it holds nothing.
pub(super) const SHOT_IN: Recipe = recipe(
    "slide-r",
    DurationToken::Move,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `shot-out`: the thumbnail slides out to the right at `--t-move --e-exit`
/// (design/05 section 10: exits accelerate), holding its last frame until the host unmaps it at
/// `settle(ShotOut)`.
pub(super) const SHOT_OUT: Recipe = recipe(
    "shot-out",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `hold`: moves nothing, at `--t-tap --e-linear`. A resting state that follows an
/// animated one plays it, so the restyle that drops the running animation starts another and
/// the element is restyled from its resting style, not left at the dropped animation's value.
pub(super) const HOLD: Recipe = recipe(
    "hold",
    DurationToken::Quick,
    EasingToken::Linear,
    Fill::None,
    Iteration::Once,
);
