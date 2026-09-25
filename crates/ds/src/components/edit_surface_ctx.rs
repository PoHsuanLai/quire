//! What every handler of an [`EditSurface`](crate::EditSurface) works with: its memory, the
//! host, and the app's handlers. One value, cloned into each closure.

use crate::components::edit_surface_state::SurfaceState;
use crate::edit::host::HostEdit;
use crate::edit::input::EditInput;
use crate::edit::pointer::{EditFocus, EditPointer};
use dioxus::prelude::*;
use std::rc::Rc;

/// A surface's handlers' shared context.
#[derive(Clone)]
pub(crate) struct SurfaceCtx {
    pub(crate) state: Rc<SurfaceState>,
    pub(crate) host: Option<HostEdit>,
    pub(crate) on_input: EventHandler<EditInput>,
    pub(crate) on_pointer: Option<EventHandler<EditPointer>>,
    pub(crate) on_focus: Option<EventHandler<EditFocus>>,
}

impl SurfaceCtx {
    /// Hand each input to the app, in order.
    pub(crate) fn tell(&self, inputs: Vec<EditInput>) {
        inputs
            .into_iter()
            .for_each(|input| self.on_input.call(input));
    }
}
