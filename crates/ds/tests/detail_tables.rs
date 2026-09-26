//! The moment tables of every component state that implements `Detailed` in design/26's first
//! wave, as data (design/26-DETAILS.md section 4.2; CHECKLIST 5b): every transition, including
//! the ones that must not move.

use ds::detail::{Moment, first_table, moment_table};
use ds::{ModuleState, PromptState};

#[test]
fn a_module_tiles_moments() {
    use ModuleState::{Busy, Off, On};
    moment_table(&[
        (Off, Busy, Moment::Pending),
        (On, Busy, Moment::Pending),
        (Busy, On, Moment::Success),
        (Busy, Off, Moment::Change),
        (Off, On, Moment::Change),
        (On, Off, Moment::Change),
    ]);
    first_table(&[
        (Off, Moment::Rest),
        (On, Moment::Rest),
        (Busy, Moment::Pending),
    ]);
}

#[test]
fn a_lock_prompts_moments() {
    let out = || PromptState::LockedOut {
        until: "9:52".into(),
    };
    moment_table(&[
        (PromptState::Idle, PromptState::Checking, Moment::Pending),
        (PromptState::Wrong, PromptState::Checking, Moment::Pending),
        (PromptState::Checking, PromptState::Wrong, Moment::Failure),
        (PromptState::Checking, PromptState::Idle, Moment::Change),
        (PromptState::Checking, out(), Moment::Unavailable),
        (out(), PromptState::Idle, Moment::Change),
        (PromptState::Wrong, PromptState::Idle, Moment::Change),
    ]);
    first_table(&[
        (PromptState::Idle, Moment::Rest),
        (PromptState::Checking, Moment::Pending),
        (PromptState::Wrong, Moment::Rest),
        (out(), Moment::Rest),
    ]);
}
