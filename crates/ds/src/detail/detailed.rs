//! How a component declares what its state changes mean (design/26-DETAILS.md section 4.2).

use super::moment::Moment;
use std::fmt::Debug;

/// A component's own state, which knows what each of its transitions means.
///
/// Write both functions as exhaustive `match`es over the variants, never with a `_` arm, so a
/// new state variant fails to compile until its moments are decided. [`crate::detail::use_detail`]
/// calls `moment` only when the state changed, and quantises nothing itself: compare what the
/// person sees (three bars, 80 %), not the raw value (R2).
///
/// ```
/// use ds::detail::{Detailed, Moment};
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum Link { Off, Joining, Joined }
///
/// impl Detailed for Link {
///     fn moment(from: &Self, to: &Self) -> Moment {
///         match (from, to) {
///             (_, Link::Joining) => Moment::Pending,
///             (Link::Joining, Link::Joined) => Moment::Success,
///             (Link::Off | Link::Joined, Link::Joined) => Moment::Change,
///             (Link::Off | Link::Joining | Link::Joined, Link::Off) => Moment::Unavailable,
///         }
///     }
///     fn first(state: &Self) -> Moment {
///         match state {
///             Link::Joining => Moment::Pending,
///             Link::Off | Link::Joined => Moment::Rest,
///         }
///     }
/// }
///
/// ds::detail::moment_table(&[
///     (Link::Off, Link::Joining, Moment::Pending),
///     (Link::Joining, Link::Joined, Moment::Success),
///     (Link::Joined, Link::Off, Moment::Unavailable),
/// ]);
/// ```
///
/// A variant the table does not decide is a compile error, not a silent `Rest`:
///
/// ```compile_fail,E0004
/// use ds::detail::{Detailed, Moment};
///
/// #[derive(Debug, Clone, PartialEq)]
/// enum Link { Off, Joining, Joined }
///
/// impl Detailed for Link {
///     fn moment(from: &Self, to: &Self) -> Moment {
///         match (from, to) {
///             (_, Link::Joining) => Moment::Pending,
///             (Link::Joining, Link::Joined) => Moment::Success,
///             (Link::Off | Link::Joined, Link::Joined) => Moment::Change,
///             // `Link::Off` as a destination is undecided.
///         }
///     }
///     fn first(state: &Self) -> Moment {
///         match state {
///             Link::Joining => Moment::Pending,
///             Link::Off | Link::Joined => Moment::Rest,
///         }
///     }
/// }
/// ```
pub trait Detailed: Clone + PartialEq + 'static {
    /// The moment `from -> to` is, on what the person sees (quantised first: R2).
    fn moment(from: &Self, to: &Self) -> Moment;

    /// The moment of the state's first frame when the surface was just opened; `Moment::Rest`
    /// for chrome that is always there (R1).
    fn first(state: &Self) -> Moment;
}

/// Assert `S`'s moment table as data: each `(from, to, moment)` row is what `S::moment` says.
/// Panics naming every row that differs; for a component's own tests.
pub fn moment_table<S: Detailed + Debug>(cases: &[(S, S, Moment)]) {
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(|(from, to, want)| {
            let got = S::moment(from, to);
            (got != *want).then(|| format!("  {from:?} -> {to:?}: {got:?}, want {want:?}"))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "moment table differs:\n{}",
        wrong.join("\n")
    );
}

/// Assert `S`'s first-frame table as data: each `(state, moment)` row is what `S::first` says.
pub fn first_table<S: Detailed + Debug>(cases: &[(S, Moment)]) {
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(|(state, want)| {
            let got = S::first(state);
            (got != *want).then(|| format!("  {state:?}: {got:?}, want {want:?}"))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "first-frame table differs:\n{}",
        wrong.join("\n")
    );
}
