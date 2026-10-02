//! A layer's standing with the texture it shows: which window to repaint when the app writes a
//! new frame (effects: it registers with the texture's handle and releases on drop).

use super::gpu::TextureHandle;
use blitz_traits::shell::ShellProvider;
use std::sync::Arc;

/// The window a layer is in and the texture it shows; the handle repaints that window when the
/// texture changes while both are known.
#[derive(Default)]
pub(crate) struct Watching {
    shell: Option<Arc<dyn ShellProvider>>,
    handle: Option<TextureHandle>,
    /// Whether the pair above is registered with the handle.
    registered: Option<(Arc<dyn ShellProvider>, TextureHandle)>,
}

impl std::fmt::Debug for Watching {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Watching")
            .field("shell", &self.shell.is_some())
            .field("handle", &self.handle)
            .finish_non_exhaustive()
    }
}

impl Watching {
    /// The window the layer mounted in.
    pub(crate) fn in_window(&mut self, shell: Arc<dyn ShellProvider>) {
        self.shell = Some(shell);
        self.settle();
    }

    /// The texture the layer shows now.
    pub(crate) fn showing(&mut self, handle: Option<TextureHandle>) {
        self.handle = handle;
        self.settle();
    }

    /// Ask the window to repaint, if it is known.
    pub(crate) fn repaint(&self) {
        if let Some(shell) = &self.shell {
            shell.request_redraw();
        }
    }

    /// The layer is unmounted.
    pub(crate) fn release(&mut self) {
        self.shell = None;
        self.handle = None;
        self.settle();
    }

    /// Make the handle's registration match the window and texture known now.
    fn settle(&mut self) {
        let wanted = self.shell.clone().zip(self.handle.clone());
        let same = match (&self.registered, &wanted) {
            (Some((shell, handle)), Some((wanted_shell, wanted_handle))) => {
                Arc::ptr_eq(shell, wanted_shell) && handle == wanted_handle
            }
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }
        if let Some((shell, handle)) = self.registered.take() {
            handle.unwatch(&shell);
        }
        if let Some((shell, handle)) = wanted {
            handle.watch(&shell);
            self.registered = Some((shell, handle));
        }
    }
}
