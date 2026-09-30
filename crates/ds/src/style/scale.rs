//! Which device scale a root draws for: the `Ds { scale }` prop, else the host's, else 1x.
//!
//! `ds-native` knows the scale it renders at (a snapshot's viewport, a window's scale factor)
//! and provides it in `HostSignals` root context, beside the input modality; a
//! host that is not `ds-native` (shell-host) passes `scale` to `Ds` itself. The root resolves
//! it once and provides it to its subtree, so a `Glyph` can snap its stroke.

use dioxus::prelude::*;
use ds_core::geometry::scale::Scale;

/// The scale the enclosing root resolved, as its subtree reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeviceScale(Scale);

/// The scale a root draws for: `given`, else `host`'s, else [`Scale::ONE`].
pub fn use_root_scale(given: Option<Scale>, host: Option<Scale>) -> Scale {
    let scale = given.or(host).unwrap_or(Scale::ONE);
    let mut provided = use_context_provider(|| Signal::new(DeviceScale(scale)));
    if *provided.peek() != DeviceScale(scale) {
        provided.set(DeviceScale(scale));
    }
    scale
}

/// The device scale of the enclosing root; [`Scale::ONE`] outside one (a glyph rendered on its
/// own, in a test).
pub fn use_scale() -> Scale {
    try_use_context::<Signal<DeviceScale>>().map_or(Scale::ONE, |provided| provided().0)
}
