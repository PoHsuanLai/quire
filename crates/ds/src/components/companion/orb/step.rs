//! The pure mapping from presence to look.

use super::model::OrbLook;
use ds_core::vocab::CompanionPresence;
use ds_style::appearance::motion::MotionLevel;

/// The look of the orb for `presence` at `motion`. Idle and Waiting are always inactive (zero
/// frames), and Reduced motion makes every presence inactive.
pub fn orb_look(_presence: CompanionPresence, _motion: MotionLevel) -> OrbLook {
    todo!(
        "orb_look: the look table in design/32 section 3; orb_look_table pins every presence at every motion level"
    )
}
