//! A handle to the running app that any thread may hold: open a window, redraw the windows, end
//! the app. `open_window` needs a component's context, so an app whose requests arrive elsewhere
//! (a D-Bus service on a tokio task, a timer, a file watcher) cannot use it; the [`AppHandle`] is
//! the way in from outside. Make one before `launch`, give a clone to the app's own threads and
//! the other to [`AppConfig::with_handle`](crate::AppConfig::with_handle); `launch` binds it to
//! its event loop. A request made before the loop runs waits for it; one made after it ended is
//! refused with [`AppEnded`].
//!
//! Every window's root can also read its app's handle with [`use_app_handle`].

use crate::open_window::WindowSpec;
use crate::window_requests::Root;
use dioxus::prelude::*;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// The app's event loop has ended (or is ending), so the request was not taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the app's event loop has ended")]
pub struct AppEnded;

/// What a thread asked of the loop. The root is built on the loop's thread (a `Root` holds an
/// `Rc`), from a closure that carries only `Send` data.
pub(crate) enum Remote {
    Open {
        spec: WindowSpec,
        make: Box<dyn FnOnce() -> Root + Send>,
    },
    Redraw,
    Quit,
    Hold,
    Release,
}

type Wake = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct State {
    pending: VecDeque<Remote>,
    /// Wakes the event loop; set when `launch` binds the handle.
    wake: Option<Wake>,
    ended: bool,
}

/// A handle to the running app, cloneable and `Send + Sync`: see the module documentation.
#[derive(Clone, Default)]
pub struct AppHandle(Arc<Mutex<State>>);

impl std::fmt::Debug for AppHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppHandle").finish_non_exhaustive()
    }
}

impl AppHandle {
    /// A handle bound to no app yet. Hand it to `AppConfig::with_handle`.
    pub fn new() -> Self {
        AppHandle::default()
    }

    /// Open a window of this app rendering `root`, as `open_window` does from a component. The
    /// window opens the next time the event loop runs, which this wakes. It returns once the
    /// request is queued, with no handle to the window: close it from its own frame or from
    /// inside its document.
    pub fn open_window(&self, spec: WindowSpec, root: fn() -> Element) -> Result<(), AppEnded> {
        self.push(Remote::Open {
            spec,
            make: Box::new(move || Root::Plain(root)),
        })
    }

    /// Open a window rendering `root` with `props`, as `open_window_with` does. The props cross
    /// threads, so they are `Send`; `root` builds from them on the event loop's thread.
    pub fn open_window_with<P: Clone + Send + 'static>(
        &self,
        spec: WindowSpec,
        root: fn(P) -> Element,
        props: P,
    ) -> Result<(), AppEnded> {
        self.push(Remote::Open {
            spec,
            make: Box::new(move || Root::Shared(Rc::new(move || root(props.clone())))),
        })
    }

    /// Ask every window to repaint, and wake the event loop: the proxy a worker thread uses to
    /// have the UI thread look again.
    pub fn redraw(&self) -> Result<(), AppEnded> {
        self.push(Remote::Redraw)
    }

    /// End the app: every window is dropped and `launch` returns, whatever the
    /// `LastWindowClosed` policy says.
    pub fn quit(&self) -> Result<(), AppEnded> {
        self.push(Remote::Quit)
    }

    /// Keep the event loop running with no window open, until the returned hold is dropped: a
    /// program that plays with no window holds the loop while it does. The `LastWindowClosed`
    /// policy applies again once the last hold is gone and no window is open (its linger starts
    /// then). It fails once the app has ended.
    pub fn hold(&self) -> Result<AppHold, AppEnded> {
        self.push(Remote::Hold)?;
        Ok(AppHold(self.clone()))
    }

    /// Whether the event loop has ended.
    pub fn has_ended(&self) -> bool {
        self.lock().ended
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn push(&self, request: Remote) -> Result<(), AppEnded> {
        let wake = {
            let mut state = self.lock();
            if state.ended {
                return Err(AppEnded);
            }
            state.pending.push_back(request);
            state.wake.clone()
        };
        if let Some(wake) = wake {
            wake();
        }
        Ok(())
    }

    /// Bind the handle to the loop `wake` wakes, waking it once if requests are already waiting.
    pub(crate) fn bind(&self, wake: impl Fn() + Send + Sync + 'static) {
        let wake: Wake = Arc::new(wake);
        let waiting = {
            let mut state = self.lock();
            state.wake = Some(Arc::clone(&wake));
            !state.pending.is_empty()
        };
        if waiting {
            wake();
        }
    }

    /// Every request since the last take, oldest first.
    pub(crate) fn take(&self) -> Vec<Remote> {
        self.lock().pending.drain(..).collect()
    }

    /// The loop ended: refuse what comes, drop what waits.
    pub(crate) fn end(&self) {
        let mut state = self.lock();
        state.ended = true;
        state.wake = None;
        state.pending.clear();
    }
}

/// A claim on the event loop staying alive, made by [`AppHandle::hold`]; dropping it lets go.
#[derive(Debug)]
pub struct AppHold(AppHandle);

impl Drop for AppHold {
    fn drop(&mut self) {
        // A loop that has ended has nothing left to hold.
        let _ended = self.0.push(Remote::Release);
    }
}

/// The app's handle, from inside any window's document: `None` where there is no app (the
/// harness, a snapshot).
pub fn use_app_handle() -> Option<AppHandle> {
    use_hook(try_consume_context::<AppHandle>)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn empty() -> Element {
        rsx! {}
    }

    fn counting(handle: &AppHandle) -> Arc<AtomicUsize> {
        let woken = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&woken);
        handle.bind(move || {
            seen.fetch_add(1, Ordering::SeqCst);
        });
        woken
    }

    fn spec() -> WindowSpec {
        WindowSpec::new("T", 10, 10)
    }

    #[test]
    fn a_request_before_the_loop_binds_waits_and_wakes_it_at_the_bind() {
        let handle = AppHandle::new();
        assert!(handle.open_window(spec(), empty).is_ok());
        let woken = counting(&handle);
        assert_eq!(woken.load(Ordering::SeqCst), 1);
        assert_eq!(handle.take().len(), 1);
    }

    #[test]
    fn a_bound_handle_wakes_the_loop_for_each_request_from_any_thread() {
        let handle = AppHandle::new();
        let woken = counting(&handle);
        assert_eq!(
            woken.load(Ordering::SeqCst),
            0,
            "nothing waiting at the bind"
        );
        let other = handle.clone();
        std::thread::spawn(move || {
            assert!(other.open_window_with(spec(), |_: u32| rsx! {}, 7).is_ok());
            assert!(other.redraw().is_ok());
            assert!(other.quit().is_ok());
        })
        .join()
        .ok();
        assert_eq!(woken.load(Ordering::SeqCst), 3);
        let kinds: Vec<&str> = handle
            .take()
            .iter()
            .map(|request| match request {
                Remote::Open { .. } => "open",
                Remote::Redraw => "redraw",
                Remote::Quit => "quit",
                Remote::Hold => "hold",
                Remote::Release => "release",
            })
            .collect();
        assert_eq!(kinds, ["open", "redraw", "quit"]);
        assert!(handle.take().is_empty());
    }

    #[test]
    fn a_request_after_the_loop_ended_is_refused() {
        let handle = AppHandle::new();
        let woken = counting(&handle);
        assert!(!handle.has_ended());
        handle.end();
        assert!(handle.has_ended());
        assert_eq!(handle.open_window(spec(), empty), Err(AppEnded));
        assert_eq!(handle.quit(), Err(AppEnded));
        assert_eq!(handle.redraw(), Err(AppEnded));
        assert_eq!(woken.load(Ordering::SeqCst), 0);
        assert!(handle.take().is_empty());
    }

    #[test]
    fn the_root_a_remote_open_carries_is_built_where_it_is_taken() {
        let handle = AppHandle::new();
        assert!(
            handle
                .open_window_with(spec(), |_: String| rsx! {}, "x".to_owned())
                .is_ok()
        );
        let built = handle.take().into_iter().find_map(|request| match request {
            Remote::Open { make, .. } => Some(make()),
            Remote::Redraw | Remote::Quit | Remote::Hold | Remote::Release => None,
        });
        assert!(matches!(built, Some(Root::Shared(_))));
    }

    #[test]
    fn a_hold_is_a_request_and_its_drop_is_another() {
        let handle = AppHandle::new();
        let _woken = counting(&handle);
        let hold = handle.hold().expect("the loop has not ended");
        drop(hold);
        let kinds: Vec<&str> = handle
            .take()
            .iter()
            .map(|request| match request {
                Remote::Hold => "hold",
                Remote::Release => "release",
                Remote::Open { .. } | Remote::Redraw | Remote::Quit => "other",
            })
            .collect();
        assert_eq!(kinds, ["hold", "release"]);
    }

    #[test]
    fn a_hold_after_the_loop_ended_is_refused_and_dropping_one_is_harmless() {
        let handle = AppHandle::new();
        let _woken = counting(&handle);
        let hold = handle.hold().expect("not ended yet");
        handle.end();
        drop(hold);
        assert!(handle.hold().is_err());
    }
}
