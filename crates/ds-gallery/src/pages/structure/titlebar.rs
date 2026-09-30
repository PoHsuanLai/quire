//! WindowTitlebar: with a subtitle, a proxy icon and the edited dot, active and inactive.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::{DocumentState, Icon, TitleParts, TrafficLights, WindowTitlebar};

/// The WindowTitlebar section.
#[component]
pub fn TitlebarSection() -> Element {
    let bar = |title: &'static str, parts: TitleParts| {
        rsx! {
            div { class: "g-titlebar", WindowTitlebar { title, parts } }
        }
    };
    rsx! {
        Section { title: "WindowTitlebar", note: "The window's titlebar: traffic lights (the green one zooms; with Option held it only zooms, and held or rested on otherwise it opens the tiling menu), the title, a subtitle after it, a proxy icon before it and the edited dot; a drag moves the window and a double-click zooms it.",
            {bar("Untitled", TitleParts::default())}
            {bar("Quarterly report", TitleParts { subtitle: Some("Edited today".to_owned()), ..TitleParts::default() })}
            {bar("Quarterly report.pdf", TitleParts { proxy: Some(Icon::File), document: DocumentState::Edited, ..TitleParts::default() })}
            div { class: "g-titlebar",
                WindowTitlebar { title: "No lights", lights: TrafficLights::Hidden }
            }
        }
    }
}
