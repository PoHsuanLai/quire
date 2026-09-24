//! Which device scale a root draws for: the `Ds { scale }` prop, else the host's, else 1x.
//!
//! `ds-native` knows the scale it renders at (a snapshot's viewport, a window's scale factor)
//! and provides it as [`HostScale`] root context, the way it provides the input modality; a
//! host that is not `ds-native` (shell-host) passes `scale` to `Ds` itself. The root resolves
//! it once and provides it to its subtree, so a `Glyph` can snap its stroke.

use crate::geometry::Scale;
use dioxus::prelude::*;

/// The host's device scale, provided as root context by `ds-native`. A signal, because a
/// window's scale changes when it moves to another output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HostScale(pub Signal<Scale>);

/// The scale the enclosing root resolved, as its subtree reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeviceScale(Scale);

/// The scale a root draws for: `given`, else the host's, else [`Scale::ONE`].
pub(crate) fn use_root_scale(given: Option<Scale>) -> Scale {
    let host = use_hook(try_consume_context::<HostScale>);
    let scale = given
        .or(host.map(|HostScale(current)| current()))
        .unwrap_or(Scale::ONE);
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
