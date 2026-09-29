//! The host's caret seams, until the document host replaces them.

use crate::host::caret::{Caret, FieldSelection, InitialCaret};
use crate::host::focused::Focused;
use dioxus::prelude::MountedData;

/// The host's caret read, provided as root context by ds-native (`launch`, its harness and
/// `ds_native::focus::provide`): where the caret is in the field `element`.
#[derive(Debug, Clone, Copy)]
pub struct HostCaret(pub fn(&MountedData) -> Caret);

/// The host's caret write, provided as root context by ds-native beside [`HostCaret`]
/// (`launch`, its harness and `ds_native::focus::provide`): put the caret of the field `element`
/// at an [`InitialCaret`] place. Without one the caret stays where the renderer put it. `Focused::Busy` asks to be tried again a frame later.
#[derive(Debug, Clone, Copy)]
pub struct HostPlaceCaret(pub fn(&MountedData, InitialCaret) -> Focused);

/// The host's selection read, provided as root context by ds-native beside [`HostCaret`]
/// (`launch`, its harness and `ds_native::focus::provide`). Without one a masked field
/// leaves the caret to the renderer.
#[derive(Debug, Clone, Copy)]
pub struct HostSelection(pub fn(&MountedData) -> FieldSelection);
