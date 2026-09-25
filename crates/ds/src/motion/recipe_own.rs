//! The recipes of the keyframes quire added to design/05-MOTION.md's catalogue, each for a
//! state the catalogue has no motion for. The catalogue's own rows are in `recipe.rs`.

use super::recipe::{Fill, Iteration, Recipe, recipe};
use crate::tokens::{DurationToken, EasingToken};

/// `chip-flash` (`S:2119`): the flash is a hold, so it runs linear and leaves nothing behind.
pub(super) const CHIP_FLASH: Recipe = recipe(
    "chip-flash",
    DurationToken::Flash,
    EasingToken::Linear,
    Fill::None,
    Iteration::Once,
);

/// `menu-out` (design/13 section 13.3.2): "fade over `--t-quick` with `--e-exit`".
pub(super) const MENU_OUT: Recipe = recipe(
    "menu-out",
    DurationToken::Quick,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);

/// `cmdk-rise`: `cmdk-in`'s row with its fade taken out (mailo gaps 2).
pub(super) const CMDK_RISE: Recipe = recipe(
    "cmdk-rise",
    DurationToken::Big,
    EasingToken::Spring,
    Fill::None,
    Iteration::Once,
);

/// `pill-up` (mailo gaps 3): a pill centred by `translateX(-50%)` springs up from below the
/// edge, at the send pill's own `--t-big --e-spring` (`S:706`). An entrance, so it holds nothing.
pub(super) const PILL_UP: Recipe = recipe(
    "pill-up",
    DurationToken::Big,
    EasingToken::Spring,
    Fill::None,
    Iteration::Once,
);

/// `ring-drain` (mailo gaps 3): the undo-send ring's dash drains over the send's grace period,
/// linear (`S:2339`), and holds empty. `--t-send-ring` is a hold: Reduced does not shorten the
/// time a person has to take a send back.
pub(super) const RING_DRAIN: Recipe = recipe(
    "ring-drain",
    DurationToken::SendRing,
    EasingToken::Linear,
    Fill::Forwards,
    Iteration::Once,
);

/// `fade-in` (C:1055, mailo gaps 3): an ink veil fades to its resting `--veil` at `--t-move
/// --e-out`, where `fade` runs to 1.
pub(super) const FADE_IN: Recipe = recipe(
    "fade-in",
    DurationToken::Move,
    EasingToken::Out,
    Fill::None,
    Iteration::Once,
);

/// `busy` (mailo gaps 3): a busy word pulses at `--t-ambient --e-in-out` and never drops below
/// .45, so it stays legible where `breathe` fades the sync halo to nothing. It loops for as long
/// as the work does; Reduced plays it once, as every loop.
pub(super) const BUSY: Recipe = recipe(
    "busy",
    DurationToken::Ambient,
    EasingToken::InOut,
    Fill::None,
    Iteration::Infinite,
);

/// `sheet-out` (sill FINDINGS Q90): a sheet's exit at `--t-move --e-exit`, the exit design/05
/// section 10 gives shell chrome. It holds its last, transparent frame until the host unmaps
/// the surface at `settle(SheetOut)`, so the sheet never flashes back between the two.
pub(super) const SHEET_OUT: Recipe = recipe(
    "sheet-out",
    DurationToken::Move,
    EasingToken::Exit,
    Fill::Forwards,
    Iteration::Once,
);
