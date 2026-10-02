//! What a confirmation shows and what the person can answer.

use ds::components::companion::mark::AppMark;
use ds::components::fields::fact_list::Fact;
use ds_core::vocab::EffectMark;
use ds_core::word::Word;

/// What the confirmation says about what the companion read.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TaintLine {
    /// Nothing untrusted was read.
    Clean,
    /// It read untrusted text from this source: "It read a message from a sender you don't know."
    ReadUntrusted {
        /// Where the text came from, in words.
        source: String,
    },
}

/// What the person may choose besides this once, `data-offer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum ScopeOffer {
    /// Only this time.
    OnceOnly,
    /// This time, or always.
    OnceOrAlways,
}

/// Whether the buttons take input yet, `data-arm`: they wait out the arm delay so a stray key
/// cannot answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum ArmState {
    /// Waiting out the delay; the buttons take no input.
    Arming,
    /// Ready.
    Armed,
}

/// How the person answers, `data-gesture`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum GestureMark {
    /// A press.
    Press,
    /// A press held for a moment: for what cannot be undone.
    Hold,
}

/// Whether an allow lasts, `data-always`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum AlwaysChoice {
    /// This time only.
    Once,
    /// From now on.
    Always,
}

/// What the person answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfirmChoice {
    /// Go ahead, once or always.
    Allow {
        /// Whether it lasts.
        always: AlwaysChoice,
    },
    /// Do not.
    Deny,
}

/// The content of a confirmation. Every word is the router's, never a model's: the title is
/// generated from the action, and the facts are the arguments the action will run with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmView {
    /// Who asks: "Companion".
    pub asker: AppMark,
    /// The app and window it acts in.
    pub target: AppMark,
    /// What it is about to do: "Send this email?".
    pub title: String,
    /// The arguments: To, Subject.
    pub facts: Vec<Fact>,
    /// The default button's word: "Send".
    pub verb: String,
    /// What it does to the world.
    pub effect: EffectMark,
    /// What it read that the person cannot vouch for.
    pub taint: TaintLine,
    /// What the person may choose.
    pub offer: ScopeOffer,
    /// How the person answers.
    pub gesture: GestureMark,
    /// Whether the buttons take input yet.
    pub arm: ArmState,
}
