//! The named D-Bus errors of `org.quire.SettingsModule1`.

use zbus::DBusError;

/// What a live module call can fail with. Each variant is a named D-Bus error under
/// `org.quire.SettingsModule1.Error.` so a client matches on the name, not on text.
#[derive(Debug, DBusError)]
#[zbus(prefix = "org.quire.SettingsModule1.Error")]
pub enum LiveError {
    #[zbus(error)]
    ZBus(zbus::Error),
    /// The caller may not do this: `Set` from anything but the Settings role.
    NotPermitted(String),
    /// The module has no such key.
    UnknownKey(String),
    /// The value is not of the key's type, or out of its range.
    BadValue(String),
    /// The value names a choice the key lists as `unavailable`; the text is the reason to show
    /// ("Add an account to use"). Raised by the skeleton before the module sees the call.
    Unavailable(String),
    /// The module could not do it (its own failure, e.g. a revoke that the provider refused).
    Failed(String),
}
