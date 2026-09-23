//! Which domain of `appearance.toml` changed between two reads (design/22-SETTINGS.md section
//! 4.4): a pure diff so a live-reloading surface repaints only the domains that moved.
//!
//! `sill` and `palmrest` own the rest of `SettingsChange`'s vocabulary (bar, dock, scroll, ...)
//! in their own crates, per section 4.4's `apply`; `ds-settings` only ever compares the two
//! domains its own `AppearanceFile` holds.

use crate::settings::AppearanceFile;

/// One variant per domain `ds-settings` owns. Pure: no I/O, no clock
/// (`CONVENTIONS.md#6-time` — this is comparison, not a timed effect).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsChange {
    /// `[appearance]`.
    Appearance,
    /// `[icons]`.
    Icons,
}

/// Which domains differ between `old` and `new`, in table order (`appearance`, then `icons`).
/// Whole-struct `PartialEq`, not per-field: a domain is small enough that "one field changed"
/// and "recompute the domain" cost the same (design/22-SETTINGS.md section 4.4).
pub fn apply(old: &AppearanceFile, new: &AppearanceFile) -> Vec<SettingsChange> {
    let mut changed = Vec::new();
    if old.appearance != new.appearance {
        changed.push(SettingsChange::Appearance);
    }
    if old.icons != new.icons {
        changed.push(SettingsChange::Icons);
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::{SettingsChange, apply};
    use crate::settings::AppearanceFile;
    use ds::Theme;

    fn with(edit: impl FnOnce(&mut AppearanceFile)) -> AppearanceFile {
        let mut file = AppearanceFile::default();
        edit(&mut file);
        file
    }

    #[test]
    fn apply_yields_only_the_domains_that_changed() {
        let base = AppearanceFile::default();
        let cases: &[(&str, AppearanceFile, AppearanceFile, &[SettingsChange])] = &[
            (
                "identical files change nothing",
                base.clone(),
                base.clone(),
                &[],
            ),
            (
                "an appearance-only edit",
                base.clone(),
                with(|f| f.appearance.theme = Theme::Dark),
                &[SettingsChange::Appearance],
            ),
            (
                "an icons-only edit",
                base.clone(),
                with(|f| f.icons.plate_inset_percent = crate::units::Percent(50)),
                &[SettingsChange::Icons],
            ),
            (
                "both domains edited",
                base.clone(),
                with(|f| {
                    f.appearance.theme = Theme::Dark;
                    f.icons.plate_inset_percent = crate::units::Percent(50);
                }),
                &[SettingsChange::Appearance, SettingsChange::Icons],
            ),
            (
                "an unknown top-level table is not a tracked domain",
                base.clone(),
                with(|f| {
                    f.extra.insert("later".into(), toml::Value::Integer(1));
                }),
                &[],
            ),
        ];
        for (name, old, new, want) in cases {
            assert_eq!(&apply(old, new), want, "{name}");
        }
    }
}
