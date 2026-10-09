//! What can go wrong opening a window.

/// A window [`open_window`](crate::open_window) could not ask for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, thiserror::Error)]
pub enum OpenWindowError {
    /// Not inside a window `launch` runs: the harness, a snapshot, or outside any
    /// component.
    #[error("no window event loop to open a window on")]
    NoHost,
}

/// A Tokio runtime for the host could not be started.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RuntimeError {
    /// The OS refused to start the runtime's worker threads.
    #[error("the host's tokio runtime could not be started: {0}")]
    Start(#[from] std::io::Error),
}

/// Why [`launch`](crate::launch) or [`launch_idle`](crate::launch_idle) could not run the app.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum LaunchError {
    /// No runtime for the app's `tokio::spawn` calls (and none was given with
    /// [`AppConfig::with_runtime`](crate::AppConfig::with_runtime)).
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    /// The windowing event loop could not run: a broken host, with no window to show anything in.
    #[error("the window's event loop could not run: {0}")]
    EventLoop(#[source] Box<dyn std::error::Error + Send + Sync>),
}
