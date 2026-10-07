//! Which extras of our desktop are here (design/36-PORTABLE-CORE.md).
//!
//! An app's core never asks which OS it runs on; it asks whether a capability answers. A
//! [`Capability`] names one extra and the D-Bus service that means it; [`Desktop::probe`] reports
//! each as [`Presence::Here`] or [`Presence::Absent`], and [`Desktop::watch`] follows the changes.
//! Feature `dbus` does the asking (zbus, session bus, plus the system bus for PackageKit); without
//! it everything is `Absent` and no D-Bus crate is linked. Feature `dioxus` adds [`use_desktop`].

mod capability;
mod desktop;
mod probe;
mod watch;

#[cfg(feature = "dioxus")]
mod hook;

pub use capability::{Bus, Capability, Service};
pub use desktop::{Desktop, Presence};
pub use watch::Watch;

#[cfg(feature = "dioxus")]
pub use hook::use_desktop;
