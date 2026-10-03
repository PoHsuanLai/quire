//! The switcher's transition (design/13 §13.4).
//!
//! The quick tap when the release may go unseen: what arrives inside the show delay decides. A
//! `ModifierReleased` switches to the armed selection with no panel; so does a second chord (a
//! second tap whose first release went unseen: the first tap wins, the second is spent). A key
//! seen while armed (Tab, Grave, the arrows) proves the chord is held and steps as it would
//! shown. Past the delay a chord steps.

use ds_core::time::stamp::Stamp;

use super::model::{SwIn, SwOut, Switcher, SwitcherParams};

/// `s` after `input` at `now`, over the MRU list `apps` (the current app first). An empty list
/// is no switcher: the machine goes to [`Switcher::Hidden`], hiding the panel if it was shown.
pub fn step<K: Clone + Eq>(
    s: Switcher,
    input: SwIn,
    now: Stamp,
    apps: &[K],
    p: &SwitcherParams,
) -> (Switcher, Vec<SwOut<K>>) {
    let _ = (s, input, now, apps, p);
    todo!("the design/13 13.4 table, plus Up/Down shown: hide and expose the selection")
}
