//! Where an app's Spaces come from.

/// Whether an app's Spaces are its own or follow the desktop's workspaces.
///
/// `Desktop` is an experiment, off by default and not implemented: the seam only. The intended
/// contract (design/21-SPACES.md section 13.6): sill publishes each workspace's id, name and
/// `SpaceLook` over a bus interface, and an app maps each of its Spaces to one workspace. Until a
/// later lane builds it, an app passes `Local`, which is every behaviour in this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SpacesSource {
    /// The app keeps its own list (`spaces.json`). The only source implemented.
    #[default]
    Local,
    /// EXPERIMENT, not implemented: the list follows the desktop's workspaces.
    Desktop,
}
