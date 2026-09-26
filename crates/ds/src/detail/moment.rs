//! What a state change means to the person looking (design/26-DETAILS.md section 3.1).

/// The meaning of one state change. A component's [`crate::detail::Detailed`] table names one for
/// every transition of its own state; the primitives play what it names and nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Moment {
    /// Nothing changed as drawn: 0 frames (R2, R3).
    Rest,
    /// A value is shown for the first time on a surface the person just opened (R1).
    Appear,
    /// An operation someone started is running and its end is unknown (R4).
    Pending,
    /// An operation with a known share done has advanced (R9).
    Progress,
    /// The operation ended as asked.
    Success,
    /// The operation did not happen: one reaction, never escalating (R6).
    Failure,
    /// A value changed without our asking.
    Change,
    /// The person moved a choice.
    Select,
    /// The element needs the person: once per request (R6).
    Attention,
    /// The element cannot act now.
    Unavailable,
    /// The pointer or focus is on the element: what a press would do.
    Preview,
    /// The element leaves.
    Dismiss,
}

impl Moment {
    /// Every moment, in section 3.1's order.
    pub const ALL: [Moment; 12] = [
        Moment::Rest,
        Moment::Appear,
        Moment::Pending,
        Moment::Progress,
        Moment::Success,
        Moment::Failure,
        Moment::Change,
        Moment::Select,
        Moment::Attention,
        Moment::Unavailable,
        Moment::Preview,
        Moment::Dismiss,
    ];
}
