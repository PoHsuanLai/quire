//! A run row's data.

use crate::components::companion::answer::action::CardAction;
use crate::components::companion::answer::footer::ServedByView;
use crate::components::companion::mark::AppMark;
use ds_core::vocab::Tally;
use ds_core::word::Word;

/// Where a run acts, `data-place`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum RunPlace {
    /// In the window the person sees, hands off.
    InPlace,
    /// In an agent workspace, out of sight, with a glow and a peek.
    AgentWorkspace,
    /// In a nested session.
    NestedSession,
}

/// Where a run stands, `data-state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum RunState {
    /// Getting ready.
    Starting,
    /// Working.
    Running,
    /// Paused.
    Paused,
    /// Waiting for the person.
    NeedsYou,
    /// The person has taken over.
    TakenOver,
    /// Finished.
    Done,
    /// Failed.
    Failed,
    /// The person stopped it.
    Stopped,
    /// A budget ran out.
    BudgetOut,
}

/// One run, as a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRowView {
    /// What it is trying to do.
    pub goal: String,
    /// The app it works in.
    pub app: AppMark,
    /// The step it is at.
    pub step: Tally,
    /// How many steps it may take.
    pub budget: Option<Tally>,
    /// What it last thought.
    pub thought: Option<String>,
    /// Where it acts.
    pub place: RunPlace,
    /// Where it stands.
    pub state: RunState,
    /// Which model drives it.
    pub served_by: ServedByView,
    /// What the person can do: the labels and keys are the shell's.
    pub actions: Vec<CardAction>,
}
