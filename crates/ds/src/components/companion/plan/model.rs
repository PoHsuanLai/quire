//! A plan, its phases and what moves them.

use crate::components::companion::answer::footer::CardFooter;
use crate::components::content::icon_source::IconSource;
use ds_core::vocab::EffectMark;
use ds_core::word::Word;

/// A step's id within its plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StepId(pub u32);

/// Whether a step is part of the run, `data-inclusion`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Inclusion {
    /// It runs.
    Included,
    /// The person left it out.
    Excluded,
}

/// Where a step stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanStepState {
    /// Waiting its turn.
    Pending,
    /// Running now.
    Running,
    /// Done.
    Done,
    /// Failed, and why.
    Failed(String),
    /// Not run.
    Skipped,
    /// Done, then undone.
    Undone,
}

/// One step of a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanStep {
    /// Its id.
    pub id: StepId,
    /// What it does.
    pub label: String,
    /// More about it.
    pub detail: Option<String>,
    /// The app it acts in.
    pub app: Option<IconSource>,
    /// What it does to the world.
    pub effect: EffectMark,
    /// Whether it is part of the run.
    pub inclusion: Inclusion,
    /// Where it stands.
    pub state: PlanStepState,
}

/// Steps that belong together under one effect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanGroup {
    /// What the group is called.
    pub label: String,
    /// The effect its steps share.
    pub effect: EffectMark,
    /// The steps.
    pub steps: Vec<PlanStep>,
}

/// Why a plan stopped, `data-why`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum StopWhy {
    /// The person stopped it.
    You,
    /// A step failed.
    Failed,
    /// It needs the person.
    NeedsYou,
}

/// How a finished plan ended, `data-finish`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PlanFinish {
    /// Every included step is done.
    AllDone,
    /// Some steps were left out or did not run.
    Partly,
}

/// Where a plan stands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanPhase {
    /// Proposed, not run; the person may edit it.
    Draft,
    /// Running, at this step.
    Running {
        /// The step in hand.
        at: StepId,
    },
    /// Stopped, at this step.
    Stopped {
        /// The step it stopped at.
        at: StepId,
        /// Why.
        why: StopWhy,
    },
    /// Over.
    Finished(PlanFinish),
    /// Being undone.
    Undoing,
    /// Undone.
    Undone,
}

/// A plan as a card shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanView {
    /// What the plan is for.
    pub title: String,
    /// Its steps, grouped.
    pub groups: Vec<PlanGroup>,
    /// Where it stands.
    pub phase: PlanPhase,
    /// The footer every card has.
    pub footer: CardFooter,
}

/// What moves a plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanIn {
    /// The person toggled a step in or out.
    Toggle(StepId),
    /// The person ran it.
    Run,
    /// The person asked to edit it.
    Edit,
    /// The person stopped it.
    Stop,
    /// A step started.
    Started(StepId),
    /// A step finished.
    Done(StepId),
    /// A step failed.
    Failed(StepId, String),
    /// A step needs the person.
    Asked(StepId),
    /// The person resumed it.
    Resume,
    /// The person asked to undo every step that ran.
    UndoAll,
    /// Everything is undone.
    AllUndone,
}

/// What a plan asks of its owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanOut {
    /// Run these steps.
    Run(Vec<StepId>),
    /// Open the plan for editing.
    Edit,
    /// Stop.
    Stop,
    /// Resume.
    Resume,
    /// Undo every step that ran.
    UndoAll,
}
