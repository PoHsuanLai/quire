//! Writing the file: rows in, `<AppName>.ui.toml` text out.

use crate::components::menus::ui_manifest::model::{UiCommand, UiManifest, UiOnlyRow, UiVocab};
use crate::components::menus::ui_manifest::rows::CommandRow;
use ds_core::command::{CommandFace, CommandId, IntentsApp};
use std::collections::HashMap;

/// Why the file cannot be written.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum UiManifestError {
    /// The app has no name, so no manifest to check the file against.
    #[error("the app has no name")]
    NoApp,
    /// One id is declared with two different faces, so the check could not tell which is true.
    #[error("the command {0:?} is declared with two different faces")]
    ConflictingFaces(CommandId),
    /// The text could not be produced.
    #[error("the file could not be written: {0}")]
    Write(String),
}

impl UiManifest {
    /// The file for `app` from `rows`, in the order given. A command listed twice with the same
    /// source and key equivalent (a menu item that is in the bar and in a context menu) is one
    /// row; an interface-only command has one `ui_only` row however many places it is in.
    pub fn new(
        app: IntentsApp,
        rows: impl IntoIterator<Item = CommandRow>,
    ) -> Result<Self, UiManifestError> {
        if app.0.trim().is_empty() {
            return Err(UiManifestError::NoApp);
        }
        let mut commands: Vec<UiCommand> = Vec::new();
        let mut ui_only: Vec<UiOnlyRow> = Vec::new();
        let mut faces: HashMap<CommandId, CommandFace> = HashMap::new();
        for row in rows {
            match faces.get(&row.id) {
                Some(face) if face != &row.face => {
                    return Err(UiManifestError::ConflictingFaces(row.id));
                }
                Some(_) => {}
                None => {
                    faces.insert(row.id.clone(), row.face.clone());
                    if let CommandFace::UiOnly(reason) = &row.face {
                        ui_only.push(UiOnlyRow {
                            id: row.id.clone(),
                            reason: reason.clone(),
                        });
                    }
                }
            }
            let command = UiCommand {
                id: row.id,
                source: row.source,
                chord: row.chord.map(|keys| keys.chord()),
                action: match row.face {
                    CommandFace::Action(action) => Some(action),
                    CommandFace::UiOnly(_) => None,
                },
            };
            if !commands.contains(&command) {
                commands.push(command);
            }
        }
        Ok(UiManifest {
            vocab: UiVocab::CURRENT,
            app,
            commands,
            ui_only,
        })
    }

    /// The file's text, to write as `<AppName>.ui.toml` beside the app's intents manifest.
    pub fn to_toml(&self) -> Result<String, UiManifestError> {
        toml::to_string(self).map_err(|error| UiManifestError::Write(error.to_string()))
    }
}
