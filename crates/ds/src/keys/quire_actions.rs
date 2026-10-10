//! The actions quire's own components take: declared with chordkit like any app's, so a person
//! can rebind them and no app can take their chord by accident.

use super::handle::Keys;
use chordkit::{AppAction, AppId, DefaultChord};

/// quire's name as the owner of its actions.
const OWNER: &str = "quire";

/// Go back one page of a pane stack.
pub(crate) const PANE_BACK: &str = "quire.pane-back";

/// The default chord of [`PANE_BACK`]: Command and `[`.
const PANE_BACK_CHORD: &str = "Primary+[";

/// The action a pane stack goes back on.
pub(crate) fn pane_back() -> Option<AppAction> {
    AppAction::new(PANE_BACK).ok()
}

/// Declares quire's own actions on `keys`. Idempotent; a conflict (an app that already has the
/// chord) leaves the action unbound and is kept in `Keys::problems`.
pub(crate) fn register_quire_actions(keys: &Keys) {
    let Some((app, actions)) = rows() else {
        return;
    };
    if let Err(conflict) = keys.register_actions(&app, &actions) {
        keys.note_problem(conflict.to_string());
    }
}

type Rows = (AppId, Vec<(AppAction, DefaultChord)>);

/// quire's rows, or `None` when a name or chord above does not read (a test rules that out).
fn rows() -> Option<Rows> {
    let chord = PANE_BACK_CHORD.parse::<DefaultChord>().ok()?;
    Some((AppId::new(OWNER).ok()?, vec![(pane_back()?, chord)]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quires_own_rows_read() {
        let rows = rows().map(|(app, actions)| (app.as_str().to_owned(), actions.len()));
        assert_eq!(rows, Some(("quire".to_owned(), 1)));
        assert!(pane_back().is_some());
    }
}
