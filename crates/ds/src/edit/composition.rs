//! Turning the IME's raw events into [`Composition`] steps. Pure: the surface keeps the state
//! and feeds each event through [`on_ime`].
//!
//! winit clears the preedit (an empty `Preedit`) right before every commit, and an IME that
//! cancels a composition clears it and commits nothing. So a cleared preedit is told at once
//! (`Update` with empty text: the app stops drawing it), and the composition ends at the commit,
//! or at the next key, blur or IME detach with nothing committed.

use crate::edit::host::ImeEvent;
use crate::edit::input::{Composition, EditInput, PreeditCursor};

/// Where the surface is in a composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Composing {
    /// None under way.
    #[default]
    Idle,
    /// A preedit is showing.
    Active,
    /// The preedit was cleared; a commit may follow, or nothing.
    Cleared,
}

/// The state after `event`, and what the app hears.
pub fn on_ime(state: Composing, event: ImeEvent) -> (Composing, Vec<EditInput>) {
    match (state, event) {
        (Composing::Idle, ImeEvent::Preedit { text, cursor }) if !text.is_empty() => (
            Composing::Active,
            vec![
                EditInput::Composition(Composition::Start),
                update(text, cursor),
            ],
        ),
        (Composing::Active | Composing::Cleared, ImeEvent::Preedit { text, cursor })
            if !text.is_empty() =>
        {
            (Composing::Active, vec![update(text, cursor)])
        }
        (Composing::Active, ImeEvent::Preedit { .. }) => {
            (Composing::Cleared, vec![update(String::new(), None)])
        }
        (Composing::Idle, ImeEvent::Commit(text)) if !text.is_empty() => {
            (Composing::Idle, vec![EditInput::Text(text)])
        }
        (Composing::Active | Composing::Cleared, ImeEvent::Commit(text)) => (
            Composing::Idle,
            vec![EditInput::Composition(Composition::End { text })],
        ),
        (Composing::Active | Composing::Cleared, ImeEvent::Disabled) => settle(state),
        (state, _) => (state, Vec::new()),
    }
}

/// End a composition that is still open, committing nothing: before a key is handled, and as the
/// surface loses the keyboard.
pub fn settle(state: Composing) -> (Composing, Vec<EditInput>) {
    match state {
        Composing::Idle => (Composing::Idle, Vec::new()),
        Composing::Active | Composing::Cleared => (
            Composing::Idle,
            vec![EditInput::Composition(Composition::End {
                text: String::new(),
            })],
        ),
    }
}

fn update(text: String, cursor: Option<(usize, usize)>) -> EditInput {
    EditInput::Composition(Composition::Update {
        text,
        cursor: cursor.map(|(start, end)| PreeditCursor { start, end }),
    })
}

#[cfg(test)]
mod tests {
    use super::{Composing, on_ime, settle};
    use crate::edit::host::ImeEvent;
    use crate::edit::input::{Composition, EditInput, PreeditCursor};

    fn preedit(text: &str) -> ImeEvent {
        ImeEvent::Preedit {
            text: text.to_owned(),
            cursor: Some((text.len(), text.len())),
        }
    }

    fn update(text: &str) -> EditInput {
        EditInput::Composition(Composition::Update {
            text: text.to_owned(),
            cursor: Some(PreeditCursor {
                start: text.len(),
                end: text.len(),
            }),
        })
    }

    fn cleared() -> EditInput {
        EditInput::Composition(Composition::Update {
            text: String::new(),
            cursor: None,
        })
    }

    fn end(text: &str) -> EditInput {
        EditInput::Composition(Composition::End {
            text: text.to_owned(),
        })
    }

    /// Feed `events` from idle; the state at the end and everything heard.
    fn run(events: Vec<ImeEvent>) -> (Composing, Vec<EditInput>) {
        events.into_iter().fold(
            (Composing::Idle, Vec::new()),
            |(state, mut heard), event| {
                let (next, more) = on_ime(state, event);
                heard.extend(more);
                (next, heard)
            },
        )
    }

    #[test]
    fn a_zhuyin_composition_starts_updates_clears_and_ends_with_the_commit() {
        let (state, heard) = run(vec![
            ImeEvent::Enabled,
            preedit("ㄓ"),
            preedit("ㄓㄨ"),
            preedit(""),
            ImeEvent::Commit("注".to_owned()),
        ]);
        assert_eq!(state, Composing::Idle);
        assert_eq!(
            heard,
            vec![
                EditInput::Composition(Composition::Start),
                update("ㄓ"),
                update("ㄓㄨ"),
                cleared(),
                end("注"),
            ]
        );
    }

    #[test]
    fn a_commit_with_no_composition_is_plain_text() {
        let (state, heard) = run(vec![ImeEvent::Commit("a".to_owned())]);
        assert_eq!(state, Composing::Idle);
        assert_eq!(heard, vec![EditInput::Text("a".to_owned())]);
    }

    #[test]
    fn a_cancelled_composition_ends_empty_at_the_next_key_or_detach() {
        let (state, heard) = run(vec![preedit("か"), preedit("")]);
        assert_eq!(state, Composing::Cleared);
        assert_eq!(heard.len(), 3);
        assert_eq!(settle(state), (Composing::Idle, vec![end("")]));
        let (after, detached) = on_ime(state, ImeEvent::Disabled);
        assert_eq!((after, detached), (Composing::Idle, vec![end("")]));
    }

    #[test]
    fn a_preedit_after_a_clear_resumes_the_same_composition() {
        let (state, heard) = run(vec![preedit("ni"), preedit(""), preedit("你")]);
        assert_eq!(state, Composing::Active);
        assert_eq!(heard.last(), Some(&update("你")));
        assert_eq!(
            heard
                .iter()
                .filter(|input| **input == EditInput::Composition(Composition::Start))
                .count(),
            1
        );
    }

    #[test]
    fn settling_with_nothing_open_says_nothing() {
        assert_eq!(settle(Composing::Idle), (Composing::Idle, Vec::new()));
        assert_eq!(
            run(vec![preedit(""), ImeEvent::Disabled]),
            (Composing::Idle, Vec::new())
        );
    }
}
