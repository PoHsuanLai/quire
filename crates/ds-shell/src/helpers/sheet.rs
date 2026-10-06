//! HelperBody and HelperSheet: the missing-helper question and what follows it, in the alert
//! layout the consent alert uses.

use super::model::HelperPhase;
use super::wording::words;
use crate::accounts::adapter::{Action, Intent, Landing};
use crate::accounts::waiting::Waiting;
use dioxus::prelude::*;
use ds::components::content::icon_source::IconSource;
use ds::components::content::icon_view::IconView;
use ds::components::overlays::sheet::Sheet;
use ds::components::overlays::sheet_attach::Attach;
use ds::components::overlays::sheet_width::SheetWidth;
use ds_style::icon::render::IconSize;

/// The helper column, bare, so a host puts it in a window of its own. In `Ask`, Return installs
/// and Escape is "Not Now"; in the end phases Return and Escape both close; while `Installing`
/// there is nothing to press and neither key does anything. `purpose` finishes "{app} needs
/// {tool} to ...". `on_install` hears Install..., `on_dismiss` Not Now and Close.
#[component]
pub fn HelperBody(
    #[props(into)] app: String,
    #[props(into)] tool: String,
    #[props(into)] purpose: String,
    #[props(default)] phase: HelperPhase,
    #[props(default)] icon: Option<IconSource>,
    on_install: EventHandler<()>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let said = words(&app, &tool, &purpose, &phase);
    let (title, body) = (said.title, said.body);
    let phase_key = match &phase {
        HelperPhase::Ask => "ask",
        HelperPhase::Installing => "installing",
        HelperPhase::Failed { .. } => "failed",
        HelperPhase::NotFound { .. } => "not-found",
        HelperPhase::Unsupported { .. } => "unsupported",
    };
    let ask = phase == HelperPhase::Ask;
    let busy = phase == HelperPhase::Installing;
    rsx! {
        div {
            class: "ds-alert",
            "data-phase": phase_key,
            onkeydown: move |event| {
                let key = event.key();
                if busy || !matches!(key, Key::Escape | Key::Enter) {
                    return;
                }
                event.stop_propagation();
                event.prevent_default();
                match (key, ask) {
                    (Key::Enter, true) => on_install.call(()),
                    _ => on_dismiss.call(()),
                }
            },
            if let Some(icon) = icon {
                div { class: "ds-alert-icon", IconView { source: icon, size: IconSize::Tile48 } }
            }
            div { class: "ds-alert-title", "{title}" }
            div { class: "ds-alert-body", role: if ask || busy { "" } else { "alert" }, "{body}" }
            if busy {
                div { class: "ds-helper-progress", Waiting {} }
            }
            if !busy {
                div { class: "ds-alert-footer", "data-layout": "row",
                    if ask {
                        // As NSAlert: the default first, drawn rightmost by the footer's row-reverse.
                        span { class: "ds-alert-slot",
                            Action { label: "Install\u{2026}", intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_install.call(()) }
                        }
                        span { class: "ds-alert-slot",
                            Action { label: "Not Now", intent: Intent::Cancel, onclick: move |()| on_dismiss.call(()) }
                        }
                    } else {
                        span { class: "ds-alert-slot",
                            Action { label: "Close", intent: Intent::Default, landing: Landing::Here, onclick: move |()| on_dismiss.call(()) }
                        }
                    }
                }
            }
        }
    }
}

/// The helper alert in a narrow centred sheet: [`HelperBody`] for a host with no window of its
/// own. Escape closes the sheet as `on_dismiss` (never while installing).
#[component]
pub fn HelperSheet(
    #[props(into)] app: String,
    #[props(into)] tool: String,
    #[props(into)] purpose: String,
    #[props(default)] phase: HelperPhase,
    #[props(default)] icon: Option<IconSource>,
    on_install: EventHandler<()>,
    on_dismiss: EventHandler<()>,
) -> Element {
    let label = words(&app, &tool, &purpose, &phase).title;
    let busy = phase == HelperPhase::Installing;
    rsx! {
        Sheet {
            label,
            onclose: move |()| {
                if !busy {
                    on_dismiss.call(());
                }
            },
            attach: Attach::Centre,
            width: SheetWidth::Narrow,
            HelperBody { app, tool, purpose, phase, icon, on_install, on_dismiss }
        }
    }
}
