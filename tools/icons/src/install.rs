//! `icons install`: copy the shipped app-icon set to where `ds_settings::apps_dir` looks
//! (sill FINDINGS Q71): by default from this repository's `assets/icons/apps` to
//! `$XDG_DATA_HOME/quire/icons/apps`, the lookup's second step, so an installed shell finds the
//! icons without a checkout beside it.

use icons::IconsError;
use std::path::{Path, PathBuf};

/// The repository's shipped set.
pub fn repository_set() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/icons/apps")
}

/// Where the set goes when `--to` is not given: the lookup's data-home step.
pub fn default_target() -> Result<PathBuf, IconsError> {
    ds_settings::icon_assets::install_dir(&ds_settings::icon_assets::AssetsEnv::current())
        .ok_or(IconsError::NoInstallDir)
}

/// Copies every `.png` under `from` to the same place under `to`, making directories as it
/// goes and replacing files already there. Returns how many files it copied.
pub fn install(from: &Path, to: &Path) -> Result<usize, IconsError> {
    std::fs::create_dir_all(to)?;
    std::fs::read_dir(from)?.try_fold(0, |copied, entry| {
        let entry = entry?;
        let (source, target) = (entry.path(), to.join(entry.file_name()));
        let more = if entry.file_type()?.is_dir() {
            install(&source, &target)?
        } else if source.extension().is_some_and(|ext| ext == "png") {
            std::fs::copy(&source, &target)?;
            1
        } else {
            0
        };
        Ok(copied + more)
    })
}
