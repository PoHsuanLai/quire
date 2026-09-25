//! Where quire's app-icon files are, and which file draws an app at a size and a style (sill
//! FINDINGS Q71; design/08-ICONS.md 2.11).
//!
//! quire ships `assets/icons/apps/<app>/[muted|monochrome/]<px>.png`; every consumer used to
//! guess where that directory lands (sill searched its own variable, then the data dirs, then a
//! sibling checkout). The order is settled here, once, and documented in CONSUMING.md:
//!
//! 1. `$QUIRE_ICON_ASSETS`, the apps directory itself (a packager's or a test's override);
//! 2. `$XDG_DATA_HOME/quire/icons/apps` (`$HOME/.local/share` when unset), where
//!    `icons install` copies the set;
//! 3. each `$XDG_DATA_DIRS` entry's `quire/icons/apps` (`/usr/local/share:/usr/share` when
//!    unset), for a system package;
//! 4. the repository's own `assets/icons/apps`, resolved from this crate's `CARGO_MANIFEST_DIR`
//!    at compile time: development only, for a consumer that takes quire as a path dependency.
//!
//! The first directory that exists wins, whole: a set is installed together, so a file missing
//! from it is not looked for further down. This lives in `ds-settings`, not `ds`, because it
//! reads the environment and the disk and `ds` stays effect-free; each step is a pure function
//! of an [`AssetsEnv`] and a probe, which the tests drive with tables.

mod lookup;
mod pick;

pub use lookup::{
    AssetsDir, AssetsEnv, AssetsOrigin, Presence, apps_dir, candidates, find_apps_dir, install_dir,
};
pub use pick::{APP_ICON_PX, AppIconName, app_icon_path, find_app_icon, sizes_to_try};
