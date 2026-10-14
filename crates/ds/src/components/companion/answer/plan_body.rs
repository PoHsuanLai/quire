//! A plan: its steps grouped by effect with the state of each, the phase it is in, and the
//! buttons that phase offers.

use super::action::{ActionRole, CardAction};
use super::built_in::built_in;
use crate::components::companion::effect::view::EffectTag;
use crate::components::companion::plan::model::{
    PlanFinish, PlanGroup, PlanPhase, PlanStep, PlanStepState, PlanView, StopWhy,
};
use crate::components::content::icon_view::IconView;
use dioxus::prelude::*;
use ds_core::vocab::EffectMark;
use ds_core::word::Word;
use ds_style::icon::render::IconSize;

/// The id of the button that runs a draft plan.
pub const RUN: &str = "plan.run";
/// The id of the button that opens a draft plan for editing.
pub const EDIT: &str = "plan.edit";
/// The id of the button that stops a running plan.
pub const STOP: &str = "plan.stop";
/// The id of the button that resumes a stopped plan.
pub const RESUME: &str = "plan.resume";
/// The id of the button that undoes every step that ran.
pub const UNDO_ALL: &str = "plan.undo-all";

/// What the phase says, in a few words.
fn phase_words(phase: &PlanPhase) -> &'static str {
    match phase {
        PlanPhase::Draft => "Not started",
        PlanPhase::Running { .. } => "Running",
        PlanPhase::Stopped {
            why: StopWhy::You, ..
        } => "Stopped by you",
        PlanPhase::Stopped {
            why: StopWhy::Failed,
            ..
        } => "Stopped: a step failed",
        PlanPhase::Stopped {
            why: StopWhy::NeedsYou,
            ..
        } => "Waiting for you",
        PlanPhase::Finished(PlanFinish::AllDone) => "All done",
        PlanPhase::Finished(PlanFinish::Partly) => "Partly done",
        PlanPhase::Undoing => "Undoing",
        PlanPhase::Undone => "Undone",
    }
}

/// What a step says of itself, in a few words; nothing while it waits.
fn state_words(state: &PlanStepState) -> Option<String> {
    match state {
        PlanStepState::Pending => None,
        PlanStepState::Running => Some("Running".to_owned()),
        PlanStepState::Done => Some("Done".to_owned()),
        PlanStepState::Failed(why) => Some(format!("Failed: {why}")),
        PlanStepState::Skipped => Some("Skipped".to_owned()),
        PlanStepState::Undone => Some("Undone".to_owned()),
    }
}

fn steps(view: &PlanView) -> impl Iterator<Item = &PlanStep> {
    view.groups.iter().flat_map(|group| group.steps.iter())
}

/// The most serious effect any step has: what Run does to the world.
fn strongest(view: &PlanView) -> EffectMark {
    view.groups
        .iter()
        .map(|group| group.effect)
        .max()
        .unwrap_or(EffectMark::Read)
}

/// The buttons the phase offers. Undo all only once a step has run.
pub fn plan_actions(view: &PlanView) -> Vec<CardAction> {
    let ran = steps(view).any(|step| step.state == PlanStepState::Done);
    let undo = built_in(
        UNDO_ALL,
        "Undo All",
        ActionRole::Secondary,
        EffectMark::UndoableWrite,
    );
    match &view.phase {
        PlanPhase::Draft => vec![
            built_in(RUN, "Run", ActionRole::Primary, strongest(view)),
            built_in(EDIT, "Edit", ActionRole::Secondary, EffectMark::Read),
        ],
        PlanPhase::Running { .. } => vec![built_in(
            STOP,
            "Stop",
            ActionRole::Secondary,
            EffectMark::Read,
        )],
        PlanPhase::Stopped { .. } => {
            let resume = built_in(RESUME, "Resume", ActionRole::Primary, strongest(view));
            std::iter::once(resume).chain(ran.then_some(undo)).collect()
        }
        PlanPhase::Finished(_) => ran.then_some(undo).into_iter().collect(),
        PlanPhase::Undoing | PlanPhase::Undone => Vec::new(),
    }
}

/// The plan as the card body.
#[component]
pub fn PlanBody(view: PlanView) -> Element {
    rsx! {
        div { class: "ds-card-plan",
            div { class: "ds-card-plan-head",
                span { class: "ds-card-plan-title", "{view.title}" }
                span { class: "ds-card-plan-phase", "{phase_words(&view.phase)}" }
            }
            for (index , group) in view.groups.iter().enumerate() {
                GroupView { key: "{index}", group: group.clone() }
            }
        }
    }
}

#[component]
fn GroupView(group: PlanGroup) -> Element {
    rsx! {
        div { class: "ds-card-plan-group",
            div { class: "ds-card-plan-group-head",
                span { "{group.label}" }
                EffectTag { effect: group.effect }
            }
            ol { class: "ds-card-plan-steps",
                for step in group.steps {
                    StepView { key: "{step.id.0}", step }
                }
            }
        }
    }
}

#[component]
fn StepView(step: PlanStep) -> Element {
    let state = state_words(&step.state);
    rsx! {
        li {
            class: "ds-card-plan-step",
            "data-inclusion": step.inclusion.slug(),
            "data-state": state.as_ref().map(|_| step_slug(&step.state)),
            if let Some(app) = step.app.clone() {
                IconView { source: app, size: IconSize::Compact }
            }
            span { class: "ds-card-plan-step-label", "{step.label}" }
            if let Some(detail) = &step.detail {
                span { class: "ds-card-plan-step-detail", "{detail}" }
            }
            if let Some(state) = &state {
                span { class: "ds-card-plan-step-state", "{state}" }
            }
        }
    }
}

/// `data-state` for a step that has something to say.
fn step_slug(state: &PlanStepState) -> &'static str {
    match state {
        PlanStepState::Pending => "pending",
        PlanStepState::Running => "running",
        PlanStepState::Done => "done",
        PlanStepState::Failed(_) => "failed",
        PlanStepState::Skipped => "skipped",
        PlanStepState::Undone => "undone",
    }
}
