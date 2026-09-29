//! The `.ds` root every surface draws inside, and the nested scopes under it.

pub mod chrome;

pub mod extent;

pub mod surface;
pub mod typeface;

pub use crate::assembly::ds::{Ds, Inject};
pub use crate::style::env::{Env, HostModality, InputModality, use_env};
pub use crate::style::scale::{HostScale, use_scale};
pub use chrome::{FrameTint, Ground, RootChrome};
pub use extent::RootExtent;
pub use surface::Surface;
pub use typeface::use_typeface;
