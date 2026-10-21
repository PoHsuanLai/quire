//! Every sentence the helper sheet says. Pure, so a table tests each.

use super::model::HelperPhase;

/// The title and the line under it for `phase`.
pub(crate) struct Words {
    pub(crate) title: String,
    pub(crate) body: String,
}

/// What the sheet says in `phase` for `app` wanting `tool` to `purpose`.
pub(crate) fn words(app: &str, tool: &str, purpose: &str, phase: &HelperPhase) -> Words {
    match phase {
        HelperPhase::Ask => Words {
            title: format!("{app} needs {tool} to {purpose}."),
            body: format!(
                "{tool} is installed from your system's software sources. \
                 You may be asked for your password."
            ),
        },
        HelperPhase::Installing => Words {
            title: format!("Installing {tool}"),
            body: "This takes a moment. The system may ask for your password.".to_owned(),
        },
        HelperPhase::Failed { reason } => Words {
            title: format!("{tool} wasn’t installed"),
            body: reason.clone(),
        },
        HelperPhase::NotFound { package } => Words {
            title: format!("{tool} isn’t in your software sources"),
            body: format!(
                "Look for \u{201c}{package}\u{201d} in your software centre, then try again."
            ),
        },
        HelperPhase::Unsupported { program, command } => Words {
            title: format!("{app} can’t install {tool} here"),
            body: match command {
                Some(_) => {
                    "Run this in Terminal to install it yourself, then try again:".to_owned()
                }
                None => format!("Install \u{201c}{program}\u{201d} yourself, then try again."),
            },
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{HelperPhase, words};

    #[test]
    fn each_phase_says_what_to_do() {
        let table = [
            (
                HelperPhase::Ask,
                "Photos needs mpv to play videos.",
                "password",
            ),
            (HelperPhase::Installing, "Installing mpv", "moment"),
            (
                HelperPhase::Failed {
                    reason: "No network.".into(),
                },
                "mpv wasn’t installed",
                "No network.",
            ),
            (
                HelperPhase::NotFound {
                    package: "mpv".into(),
                },
                "mpv isn’t in your software sources",
                "\u{201c}mpv\u{201d}",
            ),
            (
                HelperPhase::Unsupported {
                    program: "mpv".into(),
                    command: None,
                },
                "Photos can’t install mpv here",
                "Install \u{201c}mpv\u{201d} yourself, then try again.",
            ),
            (
                HelperPhase::Unsupported {
                    program: "mpv".into(),
                    command: Some("sudo pacman -S mpv".into()),
                },
                "Photos can’t install mpv here",
                "Run this in Terminal to install it yourself, then try again:",
            ),
        ];
        for (phase, title, body) in table {
            let said = words("Photos", "mpv", "play videos", &phase);
            assert_eq!(said.title, title, "{phase:?}");
            assert!(said.body.contains(body), "{phase:?}: {}", said.body);
        }
    }
}
