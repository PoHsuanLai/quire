//! The corner's transition.

use super::model::{CornerIn, CornerMachine, CornerOut, CornerParams};

/// `machine` after `input`, and the timers and firing it wants. `Enter` starts a dwell from
/// `Armed`; `Leave` cancels a dwell; a live `DwellElapsed` fires and starts the re-arm; a live
/// `RearmElapsed` re-arms at once if the pointer has left, else the next `Leave` does. A stale
/// token changes nothing.
pub fn step(
    machine: CornerMachine,
    input: CornerIn,
    params: CornerParams,
) -> (CornerMachine, Vec<CornerOut>) {
    let _ = (machine, input, params);
    todo!("the hot-corner table: dwell, fire, re-arm in either order, stale tokens ignored")
}
