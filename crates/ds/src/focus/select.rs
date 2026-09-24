//! What a focus change does with a field's text once the caret is in it (mailo Phase B, G6): a
//! rename field opened on a folder's name selects it, so typing replaces the name. Blitz has no
//! script to call `input.select()` with, so the host does it: ds-native's [`HostSelect`] selects
//! the field's whole value in the document, after its `HostFocus` write has landed.

use dioxus::prelude::MountedData;

use crate::focus::host::Focused;

/// The field's text after the focus lands in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Select {
    /// Left as it is: the caret where the renderer puts it.
    #[default]
    None,
    /// All of it selected, so the first key typed replaces it.
    All,
}

/// The host's select-all write, provided as root context by `ds-native` beside `HostFocus`
/// (`ds_native::launch`, its harness, `ds_native::focus::provide`). Without one, [`Select::All`]
/// does nothing: a webview has no seam for it.
#[derive(Debug, Clone, Copy)]
pub struct HostSelect(pub fn(&MountedData) -> Focused);
