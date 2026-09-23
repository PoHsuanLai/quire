//! `org.quire.Appearance1`: a D-Bus interface for changing the appearance from another process
//! (the control center, a script). A stub for v2; nothing serves it yet.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

use crate::error::SettingsError;
use crate::settings::AppearanceSettings;

/// The interface name.
pub const INTERFACE: &str = "org.quire.Appearance1";

/// The object path.
pub const PATH: &str = "/org/quire/Appearance1";

/// Serve the interface, writing through `appearance.toml` on `Set`. Not built until v2.
pub async fn serve(initial: AppearanceSettings) -> Result<(), SettingsError> {
    todo!()
}
