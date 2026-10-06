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
            title: format!("{tool} was not installed"),
            body: reason.clone(),
        },
        HelperPhase::NotFound { package } => Words {
            title: format!("{tool} is not in your software sources"),
            body: format!(
                "Look for \u{201c}{package}\u{201d} in your software centre or package \
                 manager, then try again."
            ),
        },
        HelperPhase::Unsupported { program } => Words {
            title: format!("{app} cannot install {tool} here"),
            body: format!(
                "Install a package that provides \u{201c}{program}\u{201d}, then try again."
            ),
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
                "mpv was not installed",
                "No network.",
            ),
            (
                HelperPhase::NotFound {
                    package: "mpv".into(),
                },
                "mpv is not in your software sources",
                "\u{201c}mpv\u{201d}",
            ),
            (
                HelperPhase::Unsupported {
                    program: "mpv".into(),
                },
                "Photos cannot install mpv here",
                "provides \u{201c}mpv\u{201d}",
            ),
        ];
        for (phase, title, body) in table {
            let said = words("Photos", "mpv", "play videos", &phase);
            assert_eq!(said.title, title, "{phase:?}");
            assert!(said.body.contains(body), "{phase:?}: {}", said.body);
        }
    }
}
