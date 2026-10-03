//! The conformance file as data. Its serde form is the file's schema, which the checker owns, so
//! the names and tags here are its.

use ds_core::command::{ActionName, CommandId, IntentsApp, UiOnlyReason};
use ds_core::word::Word;
use serde::{Deserialize, Serialize};

/// Where a command is declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Word)]
#[serde(rename_all = "snake_case")]
pub enum CommandSource {
    /// A menu item.
    Menu,
    /// A keyboard shortcut binding.
    Shortcut,
}

/// The file's vocabulary version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UiVocab(pub u32);

impl UiVocab {
    /// The version `ds` writes.
    pub const CURRENT: UiVocab = UiVocab(1);
}

/// One menu item or shortcut binding: `[[commands]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiCommand {
    /// The app's stable id for it.
    pub id: CommandId,
    /// Menu or shortcut.
    pub source: CommandSource,
    /// The key equivalent, if it has one (`cmd+n`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chord: Option<String>,
    /// The action it is the face of; absent for an interface-only command.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionName>,
}

/// A command that is no action, and why: `[[ui_only]]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiOnlyRow {
    /// The command's id.
    pub id: CommandId,
    /// Why it has no action.
    pub reason: UiOnlyReason,
}

/// The file: every command of the app, and the reasons of the interface-only ones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiManifest {
    /// The vocabulary version.
    pub vocab: UiVocab,
    /// The app the file is for.
    pub app: IntentsApp,
    /// One row per menu item and per shortcut binding.
    pub commands: Vec<UiCommand>,
    /// One row per interface-only command id.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ui_only: Vec<UiOnlyRow>,
}
