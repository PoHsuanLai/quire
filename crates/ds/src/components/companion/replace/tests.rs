use super::model::{ReplaceIn, ReplacePhase, ReplaceProposal};
use crate::components::companion::answer::footer::{
    CardFooter, ServedByView, ServedPlace, ServedPlaceKind,
};
use crate::stack::toast_hub::UndoToken;

fn footer() -> CardFooter {
    CardFooter {
        served_by: ServedByView {
            model: "local".to_owned(),
            place: ServedPlace {
                kind: ServedPlaceKind::ThisComputer,
                provider: None,
            },
        },
        sources: Vec::new(),
        scope: Vec::new(),
    }
}

#[test]
fn a_proposal_compares_by_its_phase_and_the_undo_it_holds() {
    let make = |phase| ReplaceProposal {
        original: "teh".to_owned(),
        proposed: "the".to_owned(),
        phase,
        footer: footer(),
    };
    assert_eq!(make(ReplacePhase::Proposed), make(ReplacePhase::Proposed));
    assert_ne!(
        make(ReplacePhase::Applied { undo: UndoToken(1) }),
        make(ReplacePhase::Applied { undo: UndoToken(2) })
    );
    assert_eq!(
        ReplaceIn::Applied(UndoToken(7)),
        ReplaceIn::Applied(UndoToken(7))
    );
}
