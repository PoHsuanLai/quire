//! Following the shell's work areas for an app (feature `desktop-outputs`): one call after the
//! handle is made, and `AppHandle::screen_area` answers with the shell's work area and scale
//! (`WorkBasis::Desktop`) for as long as the shell is on the bus. Without the shell nothing is
//! fed and the answer stays the window system's own (the portable fallback, design/36).

use crate::app_handle::AppHandle;
use ds_desktop::Outputs;

impl AppHandle {
    /// Read `org.quire.Outputs1` on the session bus now and on every change, feeding
    /// [`set_desktop_outputs`](AppHandle::set_desktop_outputs), on a thread of its own. The thread
    /// ends when the app has ended (found at the next change) or the bus goes away. Call it before
    /// `launch`; it is a no-op where there is no session bus.
    pub fn follow_desktop_outputs(&self) -> std::thread::JoinHandle<()> {
        let handle = self.clone();
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(follow(handle));
        })
    }
}

async fn follow(handle: AppHandle) {
    let Some(mut watch) = Outputs::watch().await else {
        return;
    };
    if handle.set_desktop_outputs(watch.current().await).is_err() {
        return;
    }
    while let Some(outputs) = watch.changed().await {
        if handle.set_desktop_outputs(outputs).is_err() {
            return;
        }
    }
}
