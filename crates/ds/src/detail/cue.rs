//! A moment that is really happening now: the only thing the primitives take.

use super::moment::Moment;
use super::touch::Touch;

/// One state change as a primitive plays it: its [`Moment`], who caused it, and which change it
/// is (so a primitive starts once per change, and a re-render replays nothing, R1). Only
/// [`crate::detail::use_detail`] makes one, from a state's own [`crate::detail::Detailed`]
/// table: a component cannot hand a primitive a moment its table does not name.
///
/// ```compile_fail,E0451
/// // A cue cannot be written by hand.
/// let cue = ds::detail::Cue { moment: ds::detail::Moment::Appear, touch: ds::detail::Touch::Remote, serial: 1 };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cue {
    moment: Moment,
    touch: Touch,
    serial: u32,
}

impl Cue {
    /// The one `use_detail` makes; `serial` counts the element's changes from 1.
    pub(crate) fn new(moment: Moment, touch: Touch, serial: u32) -> Cue {
        Cue {
            moment,
            touch,
            serial,
        }
    }

    /// What the change means.
    pub fn moment(self) -> Moment {
        self.moment
    }

    /// Who caused it.
    pub fn touch(self) -> Touch {
        self.touch
    }

    /// Which change this is: a primitive plays each serial once.
    pub(crate) fn serial(self) -> u32 {
        self.serial
    }
}
