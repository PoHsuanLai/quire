//! An answer that is a refusal.

use super::action::CardAction;
use super::footer::CardFooter;

/// Why there is no answer, in words the person can act on.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RefusedView {
    /// The request needs a cloud model the person has not allowed; the action is "Allow Once".
    NeedsCloud {
        /// The class of data that would leave the computer.
        class: String,
    },
    /// Policy denied it, or no grant covers it.
    NotAllowed {
        /// What was refused.
        what: String,
    },
    /// No typed action reaches this app and computer use is off for it.
    NoWay {
        /// The app.
        app: String,
    },
    /// A budget ran out.
    OverBudget,
    /// It failed.
    Failed {
        /// Why, in short.
        why: String,
    },
}

/// A refusal card.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RefusedAnswer {
    /// Why.
    pub why: RefusedView,
    /// What the person can do about it.
    pub actions: Vec<CardAction>,
    /// The footer every card has.
    pub footer: CardFooter,
}
