//! The colours that mean one thing each (`--signal-success` and its four siblings): the mark
//! of a finished step, of something waiting for a person, of a failure, of a git branch or
//! repository, and of a folder.
//!
//! Each meaning has one colour and no other meaning borrows it: success is green, waiting for
//! a person is orange, an error is red, git is indigo, a folder is the secondary label colour.
//! Green, orange, red and indigo are the system hues in the scope's scheme; the folder follows
//! `--ink-soft`.

use crate::appearance::system::Contrast;
use crate::tokens::system_hue::SystemHue;
use crate::tokens::token::{CssValue, Token, TokenScope};
use ds_core::word::Word;

/// One signal colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Token)]
#[token(prefix = "signal-", kind = fixed, css = signal_css)]
#[non_exhaustive]
pub enum SignalColour {
    /// `--signal-success`: green.
    Success,
    /// `--signal-waiting`: orange, for something that waits on a person.
    Waiting,
    /// `--signal-error`: red.
    Error,
    /// `--signal-git`: indigo.
    Git,
    /// `--signal-folder`: the secondary label colour.
    Folder,
}

impl SignalColour {
    /// The system hue this signal is, or `None` for the folder, which follows the label.
    pub fn hue(self) -> Option<SystemHue> {
        match self {
            SignalColour::Success => Some(SystemHue::Green),
            SignalColour::Waiting => Some(SystemHue::Orange),
            SignalColour::Error => Some(SystemHue::Red),
            SignalColour::Git => Some(SystemHue::Indigo),
            SignalColour::Folder => None,
        }
    }
}

/// A signal as the stylesheet writes it.
fn signal_css(token: SignalColour, scope: TokenScope) -> CssValue {
    match token.hue() {
        Some(hue) => CssValue::computed(hue.value(scope.scheme, Contrast::Normal).css()),
        None => CssValue::fixed("var(--ink-soft)"),
    }
}

#[cfg(test)]
mod tests {
    use super::SignalColour;
    use crate::tokens::token::{Token, TokenScope};

    #[test]
    fn the_folder_follows_the_secondary_label_and_no_two_signals_share_a_hue() {
        assert_eq!(
            SignalColour::Folder.css_value(TokenScope::BASE).as_str(),
            "var(--ink-soft)"
        );
        let hues: Vec<_> = SignalColour::ALL.iter().filter_map(|s| s.hue()).collect();
        for (at, hue) in hues.iter().enumerate() {
            assert!(!hues[at + 1..].contains(hue), "{hue:?} means two things");
        }
    }
}
