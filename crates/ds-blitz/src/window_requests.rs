//! What components ask of the event loop about the app's other windows: open one, close it,
//! raise it. A request is queued here and the event loop is woken; the loop answers it the next
//! time it wakes (`crate::window_shell`), since only the loop can create a window.

use crate::open_window::WindowSpec;
use dioxus::prelude::Element;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

/// One window `open_window` asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct WindowKey(u64);

/// Where a window opened with [`open_window`](crate::open_window) is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowLife {
    /// Asked for; the event loop has not created it yet.
    Opening,
    /// On screen, with its own document.
    Open,
    /// Closed (by its handle, its frame, the compositor, or the main window closing); its
    /// VirtualDom is dropped.
    Closed,
}

/// A key for a test that needs one.
#[cfg(test)]
pub(crate) fn tests_key(n: u64) -> WindowKey {
    WindowKey(n)
}

/// What a window's root renders: a plain root, or one carrying the props it was opened with.
#[derive(Clone)]
pub(crate) enum Root {
    /// `fn() -> Element`, rendered as a component of its own.
    Plain(fn() -> Element),
    /// A root closed over its props.
    Shared(Rc<dyn Fn() -> Element>),
}

impl Root {
    /// The same root: the same function, or the same closure.
    pub(crate) fn same(&self, other: &Root) -> bool {
        match (self, other) {
            (Root::Plain(a), Root::Plain(b)) => std::ptr::fn_addr_eq(*a, *b),
            (Root::Shared(a), Root::Shared(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

/// One thing asked of the event loop.
pub(crate) enum Request {
    Open {
        key: WindowKey,
        spec: WindowSpec,
        root: Root,
    },
    Close(WindowKey),
    Focus(WindowKey),
}

/// The app's queue of window requests, shared by every window's document (a root context) and
/// the event loop.
#[derive(Clone)]
pub(crate) struct Requests(Rc<Queue>);

struct Queue {
    pending: RefCell<VecDeque<Request>>,
    lives: RefCell<HashMap<WindowKey, WindowLife>>,
    next: Cell<u64>,
    /// Wakes the event loop (winit's proxy), so a request is answered now, not at the next
    /// event.
    wake: Box<dyn Fn()>,
}

impl Requests {
    /// An empty queue that calls `wake` whenever a request is added.
    pub(crate) fn new(wake: impl Fn() + 'static) -> Self {
        Requests(Rc::new(Queue {
            pending: RefCell::new(VecDeque::new()),
            lives: RefCell::new(HashMap::new()),
            next: Cell::new(0),
            wake: Box::new(wake),
        }))
    }

    /// Ask for a window; its key names it from now on.
    pub(crate) fn open(&self, spec: WindowSpec, root: Root) -> WindowKey {
        let key = WindowKey(self.0.next.get());
        self.0.next.set(key.0 + 1);
        self.0.lives.borrow_mut().insert(key, WindowLife::Opening);
        self.push(Request::Open { key, spec, root });
        key
    }

    /// Ask for the window `key` to close.
    pub(crate) fn close(&self, key: WindowKey) {
        self.push(Request::Close(key));
    }

    /// Ask for the window `key` to be raised and focused.
    pub(crate) fn focus(&self, key: WindowKey) {
        self.push(Request::Focus(key));
    }

    /// Every request since the last take, oldest first.
    pub(crate) fn take(&self) -> Vec<Request> {
        self.0.pending.borrow_mut().drain(..).collect()
    }

    /// Where the window `key` is in its life.
    pub(crate) fn life(&self, key: WindowKey) -> WindowLife {
        self.0
            .lives
            .borrow()
            .get(&key)
            .copied()
            .unwrap_or(WindowLife::Closed)
    }

    /// Record where the window `key` is in its life.
    pub(crate) fn set_life(&self, key: WindowKey, life: WindowLife) {
        self.0.lives.borrow_mut().insert(key, life);
    }

    fn push(&self, request: Request) {
        self.0.pending.borrow_mut().push_back(request);
        (self.0.wake)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dioxus::prelude::*;

    fn empty() -> Element {
        rsx! {}
    }

    fn woken() -> (Requests, Rc<Cell<u32>>) {
        let count = Rc::new(Cell::new(0));
        let seen = Rc::clone(&count);
        (Requests::new(move || seen.set(seen.get() + 1)), count)
    }

    fn kinds(requests: &[Request]) -> Vec<String> {
        requests
            .iter()
            .map(|request| match request {
                Request::Open { key, spec, .. } => format!("open {} {}", key.0, spec.title()),
                Request::Close(key) => format!("close {}", key.0),
                Request::Focus(key) => format!("focus {}", key.0),
            })
            .collect()
    }

    #[test]
    fn every_request_wakes_the_loop_and_is_answered_in_order() {
        let (requests, woken) = woken();
        let first = requests.open(WindowSpec::new("One", 300, 200), Root::Plain(empty));
        let second = requests.open(WindowSpec::new("Two", 300, 200), Root::Plain(empty));
        requests.focus(first);
        requests.close(second);

        assert_eq!(woken.get(), 4, "each request woke the loop once");
        assert_eq!(
            kinds(&requests.take()),
            ["open 0 One", "open 1 Two", "focus 0", "close 1"]
        );
        assert!(requests.take().is_empty(), "a take empties the queue");
    }

    #[test]
    fn a_window_is_opening_until_the_loop_says_otherwise() {
        let (requests, _) = woken();
        let key = requests.open(WindowSpec::new("One", 300, 200), Root::Plain(empty));
        assert_eq!(requests.life(key), WindowLife::Opening);
        requests.set_life(key, WindowLife::Open);
        assert_eq!(requests.life(key), WindowLife::Open);
        requests.set_life(key, WindowLife::Closed);
        assert_eq!(requests.life(key), WindowLife::Closed);
        assert_eq!(
            requests.life(WindowKey(99)),
            WindowLife::Closed,
            "a key never opened is no window"
        );
    }
}
