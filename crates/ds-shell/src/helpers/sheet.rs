//! HelperBody and HelperSheet: the missing-helper question and what follows it, as `ds`'s
//! `AlertColumn`, with the install command or the spinner in its content slot.

use super::model::HelperPhase;
use super::wording::words;
use crate::accounts::model::CopyState;
use crate::accounts::waiting::Waiting;
use dioxus::prelude::*;
use ds::components::content::icon_source::IconSource;
use ds::components::content::text_runs::TextLine;
use ds::components::controls::command_line::CommandLine;
use ds::components::overlays::alert_model::{AlertButton, AlertRole};
use ds::components::overlays::sheet::Sheet;
use ds::components::overlays::sheet_attach::Attach;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::prelude::AlertColumn;

/// The helper column, bare, so a host puts it in a window of its own. In `Ask`, Return installs
/// and Escape is "Not Now"; in the end phases Return and Escape both close; while `Installing`
/// there is nothing to press and neither key does anything. `purpose` finishes "{app} needs
/// {tool} to ...". `on_install` hears Install..., `on_dismiss` Not Now and Close. An
/// `Unsupported` phase with a `command` shows it as a line with a Copy button: `on_copy` hears
/// the command to copy (the host writes the clipboard) and `copy` is the host's word on whether
/// it has.
#[component]
pub fn HelperBody(
    #[props(into)] app: String,
    #[props(into)] tool: String,
    #[props(into)] purpose: String,
    #[props(default)] phase: HelperPhase,
    #[props(default)] icon: Option<IconSource>,
    on_install: EventHandler<()>,
    on_dismiss: EventHandler<()>,
    #[props(default)] on_copy: EventHandler<String>,
    #[props(default)] copy: CopyState,
) -> Element {
    let said = words(&app, &tool, &purpose, &phase);
    let (title, body) = (said.title, said.body);
    let ask = phase == HelperPhase::Ask;
    let busy = phase == HelperPhase::Installing;
    let command = match &phase {
        HelperPhase::Unsupported { command, .. } => command.clone(),
        _ => None,
    };
    // As NSAlert: the default first, drawn rightmost by the footer's row-reverse. While
    // installing there is nothing to press, and with no button and no dismissal neither key does
    // anything.
    let buttons = if busy {
        Vec::new()
    } else if ask {
        vec![
            AlertButton::new("Install\u{2026}", AlertRole::Normal, on_install),
            AlertButton::new("Not Now", AlertRole::Cancel, on_dismiss),
        ]
    } else {
        vec![AlertButton::new("Close", AlertRole::Normal, on_dismiss)]
    };
    rsx! {
        AlertColumn {
            title,
            message: TextLine::from(body),
            icon,
            buttons,
            announce: !ask && !busy,
            on_dismiss: if busy { None } else { Some(on_dismiss) },
            if let Some(text) = command {
                CommandLine { text, copy, on_copy }
            }
            if busy {
                div { class: "ds-helper-progress", Waiting {} }
            }
        }
    }
}

/// The helper alert in a narrow centred sheet: [`HelperBody`] for a host with no window of its
/// own. Escape closes the sheet as `on_dismiss` (never while installing). `on_copy` and `copy`
/// are [`HelperBody`]'s.
#[component]
pub fn HelperSheet(
    #[props(into)] app: String,
    #[props(into)] tool: String,
    #[props(into)] purpose: String,
    #[props(default)] phase: HelperPhase,
    #[props(default)] icon: Option<IconSource>,
    on_install: EventHandler<()>,
    on_dismiss: EventHandler<()>,
    #[props(default)] on_copy: EventHandler<String>,
    #[props(default)] copy: CopyState,
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
            HelperBody { app, tool, purpose, phase, icon, on_install, on_dismiss, on_copy, copy }
        }
    }
}
