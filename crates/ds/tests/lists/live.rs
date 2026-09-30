//! A `List` driven by its own settle timers: its roster inside a `Ds` environment at the Reduced
//! level (every settle 94 ms), drawn with `ThreadRow`s, the VirtualDom polled on a small
//! `block_on` until each state has settled.

use super::rows::listed_thread;
use crate::scoped::scope;
use dioxus::prelude::*;
use ds::prelude::*;
use ds_core::time::clock::sleep;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::Thread;
use std::time::{Duration, Instant};

struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Run `future` on this thread, parking between polls.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let waker = Waker::from(Arc::new(Unpark(std::thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(value) = future.as_mut().poll(&mut cx) {
            return value;
        }
        std::thread::park_timeout(Duration::from_millis(5));
    }
}

/// Poll the dom's tasks and re-render for `length`.
fn pump(dom: &mut VirtualDom, length: Duration) {
    let end = Instant::now() + length;
    while Instant::now() < end {
        let left = end.saturating_duration_since(Instant::now());
        block_on(async {
            let mut work = pin!(dom.wait_for_work());
            let mut deadline = pin!(sleep(left));
            std::future::poll_fn(|cx| {
                if work.as_mut().poll(cx).is_ready() || deadline.as_mut().poll(cx).is_ready() {
                    Poll::Ready(())
                } else {
                    Poll::Pending
                }
            })
            .await;
        });
        dom.render_immediate_to_vec();
    }
}

fn app() -> Element {
    use_context_provider(|| Signal::new(scope()));
    let keys = use_context_provider(|| Signal::new(vec!["a", "b", "c"]));
    rsx! {
        List::<&'static str> {
            label: "Threads",
            items: keys().into_iter().map(|key| listed_thread(key, Emphasis::Strong)).collect::<Vec<_>>(),
        }
    }
}

/// Every `data-presence` a rendered list's items carry, in order.
pub fn presences(html: &str) -> Vec<String> {
    html.split("class=\"ds-list-item\"")
        .skip(1)
        .filter_map(|item| item.split("data-presence=\"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect()
}

/// Settles are 94 ms at Reduced; wait well past each.
const SETTLED: Duration = Duration::from_millis(400);

/// Set the list's keys, as a consumer's state change does, and render.
fn set_keys(dom: &mut VirtualDom, keys: &[&'static str]) -> String {
    let keys = keys.to_vec();
    dom.in_scope(ScopeId::APP, || {
        consume_context::<Signal<Vec<&'static str>>>().set(keys)
    });
    dom.render_immediate_to_vec();
    dioxus_ssr::render(dom)
}

/// Pump until the list's presences are `want`, or the wait runs out.
fn until(dom: &mut VirtualDom, want: &[&str]) -> Option<String> {
    let end = Instant::now() + SETTLED;
    while Instant::now() < end {
        pump(dom, Duration::from_millis(10));
        let html = dioxus_ssr::render(dom);
        if presences(&html) == want {
            return Some(html);
        }
    }
    None
}

/// A list through a key arriving, a key leaving, the rows below healing and the heal resting: the
/// markup at each moment, by name. The list plays them on its own settle timers.
pub fn moments() -> Vec<(&'static str, String)> {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let first = dioxus_ssr::render(&dom);
    assert_eq!(presences(&first), ["present"; 3], "first show");
    let entering = set_keys(&mut dom, &["a", "b", "c", "d"]);
    assert_eq!(
        presences(&entering),
        ["present", "present", "present", "entering"],
        "{entering}"
    );
    let rested = until(&mut dom, &["present"; 4]).expect("the arrival never rested");
    let leaving = set_keys(&mut dom, &["a", "c", "d"]);
    assert_eq!(
        presences(&leaving),
        ["present", "leaving", "present", "present"],
        "{leaving}"
    );
    assert!(leaving.contains("data-exit=\"row\""), "{leaving}");
    // After the fold settles the row is dropped and the two below heal; the heal then rests.
    let healing =
        until(&mut dom, &["present", "healing", "healing"]).expect("the rows below never healed");
    let healed = until(&mut dom, &["present"; 3]).expect("the heal never rested");
    vec![
        ("first-show", first),
        ("entering", entering),
        ("rested", rested),
        ("leaving", leaving),
        ("healing", healing),
        ("healed", healed),
    ]
}
