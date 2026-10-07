//! Every closed vocabulary of the companion's components parses back from its own slug, and no
//! two members share one.

use ds::components::companion::answer::action::ActionRole;
use ds::components::companion::answer::footer::ServedPlaceKind;
use ds::components::companion::answer::model::Streaming;
use ds::components::companion::form::model::FieldNeed;
use ds::components::companion::memory::model::{FactOrigin, FactStanding};
use ds::components::companion::orb::model::{OrbPeriod, OrbSize, OrbTone};
use ds::components::companion::plan::model::{Inclusion, PlanFinish, StopWhy};
use ds::components::companion::prompt::host::PromptHost;
use ds::components::companion::prompt::model::PromptPhase;
use ds::components::companion::proposed_event::model::BusyKind;
use ds::components::companion::run_row::model::{RunPlace, RunState};
use ds_core::word::Word;

fn round_trips<T: Word + std::fmt::Debug>() {
    for &word in T::ALL {
        assert_eq!(T::parse(word.slug()), Some(word), "{word:?}");
    }
    let slugs: std::collections::HashSet<_> = T::ALL.iter().map(|word| word.slug()).collect();
    assert_eq!(slugs.len(), T::ALL.len(), "{:?}", T::ALL);
}

#[test]
fn every_companion_word_parses_back_from_its_slug() {
    round_trips::<ActionRole>();
    round_trips::<BusyKind>();
    round_trips::<FactOrigin>();
    round_trips::<FactStanding>();
    round_trips::<FieldNeed>();
    round_trips::<Inclusion>();
    round_trips::<OrbPeriod>();
    round_trips::<OrbSize>();
    round_trips::<OrbTone>();
    round_trips::<PlanFinish>();
    round_trips::<PromptHost>();
    round_trips::<PromptPhase>();
    round_trips::<RunPlace>();
    round_trips::<RunState>();
    round_trips::<ServedPlaceKind>();
    round_trips::<StopWhy>();
    round_trips::<Streaming>();
}
