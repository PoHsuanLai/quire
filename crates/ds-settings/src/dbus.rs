//! `org.quire.Appearance1`: a D-Bus interface for changing the appearance from another process
//! (the control center, a script). A stub for v2; nothing serves it yet.

use crate::error::SettingsError;
use crate::settings::AppearanceSettings;

/// The interface name.
pub const INTERFACE: &str = "org.quire.Appearance1";

/// The object path.
pub const PATH: &str = "/org/quire/Appearance1";

/// Serve the interface, writing through `appearance.toml` on `Set`. Not built until v2: this is
/// a documented no-op for v1 — no object is exported, no bus name is requested, and nothing
/// this build writes can be changed over D-Bus yet. `initial` is accepted so v2 filling this
/// body in is not a signature change; today it is unused.
pub async fn serve(_initial: AppearanceSettings) -> Result<(), SettingsError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::serve;
    use crate::settings::AppearanceSettings;

    #[tokio::test]
    async fn serving_v1_is_a_no_op_that_never_fails() {
        assert!(serve(AppearanceSettings::default()).await.is_ok());
    }
}
