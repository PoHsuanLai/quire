//! The plan's pure transition.

use super::model::{PlanIn, PlanOut, PlanView};

/// The plan after `input`, and what it asks of its owner. Excluded steps are never run, Stop keeps
/// the steps already done, and Undo all is offered only after a run.
pub fn plan_step(_view: PlanView, _input: PlanIn) -> (PlanView, Vec<PlanOut>) {
    todo!(
        "plan_step: design/32 section 4 and design/agent/ux.md section 3.4; plan_step_table, excluded_steps_not_run, stop_keeps_done_steps, undo_all_only_after_run"
    )
}
