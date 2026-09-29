//! The `.ds` root every surface draws inside, and the nested scopes under it.

pub mod chrome;

pub mod env;
pub mod extent;
pub mod scale;
pub mod surface;
pub mod typeface;

pub use crate::assembly::ds::{Ds, Inject};
pub use chrome::{FrameTint, Ground, RootChrome};
pub use env::{Env, HostModality, InputModality, use_env};
pub use extent::RootExtent;
pub use scale::{HostScale, use_scale};
pub use surface::Surface;
pub use typeface::use_typeface;
