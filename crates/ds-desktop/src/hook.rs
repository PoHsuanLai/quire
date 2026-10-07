//! The hook: the desktop's capabilities as a signal.

use crate::Desktop;
use dioxus::prelude::*;

/// The capabilities now, probed once the component mounts and kept current. The signal starts
/// all `Absent`, so an extra appears when its service is found and never flashes a placeholder.
pub fn use_desktop() -> ReadSignal<Desktop> {
    let mut desktop = use_signal(Desktop::default);
    use_future(move || async move {
        desktop.set(Desktop::probe().await);
        if let Some(mut watch) = Desktop::watch().await {
            while let Some(next) = watch.changed().await {
                desktop.set(next);
            }
        }
    });
    ReadSignal::new(desktop)
}
