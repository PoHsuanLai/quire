//! What a menu item or a shortcut is the face of.

use crate::command::names::{ActionName, CommandId, UiOnlyReason};
use crate::vocab::Shortcut;

/// What a command does for the person, as the conformance check sees it: it is the face of an
/// action, or it is interface only and says why.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CommandFace {
    /// The command runs this action of the app's intents manifest, so a terminal or the companion
    /// can run it too.
    Action(ActionName),
    /// The command only moves the interface (window chrome, scrolling, a panel's focus).
    UiOnly(UiOnlyReason),
}

/// The value a menu item yields and a shortcut fires, as the conformance file needs to know it:
/// an app's command enum implements this once, and every menu item and shortcut that yields it
/// is covered.
pub trait AppCommand {
    /// The app's stable id for this command.
    fn id(&self) -> CommandId;

    /// The action it is the face of, or why it is none.
    fn face(&self) -> CommandFace;
}

/// A key combination bound to a command, outside any menu: one row of an app's shortcut table.
/// A menu item's own key equivalent is read from the item.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortcutBinding<T> {
    /// The keys.
    pub keys: Shortcut,
    /// What they fire.
    pub command: T,
}
