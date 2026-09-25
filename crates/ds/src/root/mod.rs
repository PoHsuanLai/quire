//! The `.ds` root every surface draws inside, and the nested scopes under it.

pub mod chrome;
pub mod ds;
pub mod env;
pub mod extent;
pub mod scale;
pub mod surface;

pub use chrome::{FrameTint, Ground, RootChrome};
pub use ds::{Ds, Inject};
pub use env::{Env, HostModality, InputModality, use_env};
pub use extent::RootExtent;
pub use scale::{HostScale, use_scale};
pub use surface::Surface;
