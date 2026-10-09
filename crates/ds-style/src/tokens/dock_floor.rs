//! The dock's reflective floor, the one dock token the style layer reads itself: the stylesheet's
//! shape section draws `.ds-dock-floor` (a soft light band, masked as the pill is) at `--dock-floor`.
//! The rest of the dock's geometry is the shell's (sill's `DockToken`); a consumer writes the input
//! (`--dock-floor-on`, 0 or 1) on any element around the dock.

use crate::tokens::token::Token;
use ds_core::word::Word;

/// The floor's opacity token, which a settings key (`dock.floor`) moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "", kind = tuned)]
#[non_exhaustive]
pub enum DockFloorToken {
    /// `--dock-floor`: the reflective floor's opacity, 0 or 1 (`dock.floor`, `Off`).
    #[token(name = "dock-floor", input = "--dock-floor-on", value = "0")]
    Floor,
}
