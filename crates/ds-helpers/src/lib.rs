//! Missing helpers (design/27 permission-prompt rules; `quire/CONSUMING.md`, "Missing helpers").
//!
//! An app runs distro tools (mpv, ffmpeg, heif-dec) that may not be installed. This crate lets
//! it say so declaratively, notice at the moment of use, and have the system install the tool,
//! like Totem's codec prompt: the person is asked first (the `HelperSheet` in `ds-shell`), then
//! [`Helpers::provide`] asks PackageKit, which asks polkit for the password itself. The app
//! hears of the new tool through [`Helpers::subscribe`] and retries without restarting.
//!
//! # The helpers file
//!
//! An app ships `$XDG_DATA_DIRS/quire/helpers/<app>.toml`. Each top-level table is a
//! capability; its fields are:
//!
//! - `tool`: the tool's name as the person knows it;
//! - `purpose`: what it is for, finishing "{app} needs {tool} to ..." (so "play videos");
//! - `probe`: executable file names; any one on `PATH` proves the tool is there;
//! - `packages.<family>`: package names for `dnf`, `apt`, `pacman` and `zypper`, each a list of
//!   alternatives in preference order. The first that exists in the repositories is installed.
//!   A family left out, or a distro that is none of the four, is [`Outcome::Unsupported`]; keys
//!   for families this version does not know are ignored.
//!
//! ```toml
//! [video-playback]
//! tool = "mpv"
//! purpose = "play videos"
//! probe = ["mpv"]
//! [video-playback.packages]
//! dnf = ["mpv"]
//! apt = ["mpv"]
//! pacman = ["mpv"]
//! zypper = ["mpv"]
//! ```
//!
//! The distro family comes from `ID`, then each `ID_LIKE` entry, of `/etc/os-release`
//! ([`Family::detect`]; read through [`Environment::os_release`], never written).
//!
//! # Why PackageKit's system transaction API, not `Modify2`
//!
//! `org.freedesktop.PackageKit.Modify2` (`InstallPackageNames`) is a session-bus interface that
//! a desktop's software centre must implement (GNOME Software does, KDE and sill's shell do not
//! necessarily), and it reports nothing but done or failed. The system service
//! `org.freedesktop.PackageKit` is the one every PackageKit install ships, is documented in the
//! interface XML, and gives what the sheet needs: `Resolve` tells whether a candidate exists
//! (so [`Outcome::NotFound`] is exact and the first alternative that exists wins),
//! `InstallPackages` asks polkit (`SetHints interactive=true` lets it prompt), and
//! `Finished`/`ErrorCode` say how it ended. The backend runs no package manager and uses no sudo.
//!
//! Limits: a Flatpak-sandboxed app cannot reach the system bus's PackageKit, so it gets
//! [`Outcome::Unsupported`]; an image-based distro (rpm-ostree) installs nothing through
//! PackageKit this way and reports its own failure.

mod capability;
mod catalog;
mod family;
mod helpers;
mod installer;
mod outcome;
mod probe;
mod watch;

pub use capability::{Capability, Executable, NameError, PackageName};
pub use catalog::{Catalog, CatalogError, Entry, data_dirs};
pub use family::Family;
pub use helpers::Helpers;
pub use installer::{FakeInstaller, Installer, PackageKit, Request, StandIn};
pub use outcome::{Missing, Outcome};
pub use probe::{Environment, Presence};
pub use watch::{Availability, Subscription};
