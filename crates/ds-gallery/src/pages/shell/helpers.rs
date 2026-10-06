//! The Helpers page: the missing-helper sheet in every phase, light and dark, each in a sheet root
//! of its own over the wallpaper. The values are the page's own: nothing here installs anything.

use crate::axes::Axes;
use crate::pages::Section;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::prelude::*;
use ds_shell::helpers::model::HelperPhase;
use ds_shell::prelude::*;

fn sheet(phase: HelperPhase) -> Element {
    rsx! {
        HelperSheet { app: "Anyview", tool: "mpv", purpose: "play videos", phase, on_install: |_| {}, on_dismiss: |_| {} }
    }
}

/// The page.
#[component]
pub fn HelpersPage() -> Element {
    rsx! {
        Section { title: "The question", note: "HelperSheet, Ask: an app tried something that needs a tool this machine lacks, and asks at that moment (never at launch): the purpose as one sentence, a line saying where it comes from and that the system may ask for the password, Not Now (Escape) and Install... (Return). Install... calls PackageKit, which asks polkit for the password itself.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: sheet(HelperPhase::Ask) }
                Stage { theme: Theme::Dark, body: sheet(HelperPhase::Ask) }
            }
        }
        Section { title: "Installing", note: "Installing: a spinner and nothing to press, because the install is the system's and is not cancelled from here; Escape does nothing. The host moves on when the install ends.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: sheet(HelperPhase::Installing) }
                Stage { theme: Theme::Dark, body: sheet(HelperPhase::Installing) }
            }
        }
        Section { title: "Failed", note: "Failed: the package manager's own words, and Close (Return or Escape).",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: sheet(HelperPhase::Failed { reason: "The network is unreachable, so the package could not be downloaded.".to_owned() }) }
                Stage { theme: Theme::Dark, body: sheet(HelperPhase::Failed { reason: "The network is unreachable, so the package could not be downloaded.".to_owned() }) }
            }
        }
        Section { title: "Not found and unsupported", note: "NotFound: no software source has the package; the sheet names the package to look for. Unsupported: this system cannot install from here (an unknown distro, or no PackageKit); it names the program a package must provide.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: sheet(HelperPhase::NotFound { package: "mpv".to_owned() }) }
                Stage { theme: Theme::Dark, body: sheet(HelperPhase::Unsupported { program: "mpv".to_owned() }) }
            }
        }
    }
}

/// A sheet root of `theme` over the wallpaper holding `body`.
#[component]
fn Stage(theme: Theme, body: Element) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion) = {
        let axes = axes.read();
        (axes.accent, axes.motion)
    };
    rsx! {
        div { class: "g-modal g-helpers", style: "background-image:url(\"{wallpaper::uri()}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Sheet,
                stylesheet: Inject::Host,
                div { class: "g-helpers-stage" }
                {body}
            }
        }
    }
}
