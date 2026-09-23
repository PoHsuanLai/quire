//! A roster driven by its own settle timers: `use_roster` inside a `Ds` environment at the
//! Reduced level (every settle 94 ms), drawn with `AnimatedList` and `ListRow`, the VirtualDom
//! polled on a small `block_on` until each state has settled.

use super::rows::Row;
use dioxus::prelude::*;
use ds::components::vocab::Emphasis;
use ds::{
    Accent, AnimatedList, BlurState, Env, Exit, InputModality, ListPresence, Material, MotionLevel,
    Px, Resolved, Roster, RowPitch, Scheme, use_roster,
};
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
            let mut deadline = pin!(ds::sleep(left));
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

fn env() -> Env {
    Env {
        resolved: Resolved {
            scheme: Scheme::Light,
            accent: Accent::Postmark,
            motion: MotionLevel::Reduced,
        },
        scheme: Scheme::Light,
        material: Material::Window,
        blur: BlurState::default(),
        modality: InputModality::Pointer,
    }
}

fn app() -> Element {
    use_context_provider(|| Signal::new(env()));
    let roster = use_roster(vec!["a", "b", "c"], RowPitch(Px(79.0)));
    use_context_provider(|| roster);
    rsx! {
        AnimatedList { label: "Threads", presence: ListPresence::Present,
            for entry in roster.entries() {
                Row { key: "{entry.key}", presence: entry.presence, emphasis: Emphasis::Strong, index: entry.index }
            }
        }
    }
}

/// `(key, data-presence)` of each row in `html`.
fn rows(html: &str) -> Vec<String> {
    html.split("<li ")
        .skip(1)
        .filter_map(|li| li.split("data-presence=\"").nth(1))
        .filter_map(|rest| rest.split('"').next())
        .map(str::to_owned)
        .collect()
}

/// Settles are 94 ms at Reduced; wait well past each.
const SETTLED: Duration = Duration::from_millis(400);

pub fn exit_plays_through() {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    assert_eq!(
        rows(&dioxus_ssr::render(&dom)),
        ["entering"; 3],
        "first show"
    );

    pump(&mut dom, SETTLED);
    assert_eq!(rows(&dioxus_ssr::render(&dom)), ["present"; 3], "rested");

    dom.in_scope(ScopeId::APP, || {
        consume_context::<Roster<&'static str>>().leave("a", Exit::Fold, Emphasis::Strong)
    });
    dom.render_immediate_to_vec();
    let leaving = dioxus_ssr::render(&dom);
    assert_eq!(
        rows(&leaving),
        ["leaving", "present", "present"],
        "{leaving}"
    );
    assert!(leaving.contains("data-exit=\"fold\""), "{leaving}");

    // After the fold settles the row is dropped and the two below heal; the heal then rests.
    let healing_seen = {
        let end = Instant::now() + SETTLED;
        let mut seen = false;
        while Instant::now() < end && !seen {
            pump(&mut dom, Duration::from_millis(10));
            seen = rows(&dioxus_ssr::render(&dom)) == ["healing", "healing"];
        }
        seen
    };
    assert!(healing_seen, "the rows below never healed");
    pump(&mut dom, SETTLED);
    assert_eq!(
        rows(&dioxus_ssr::render(&dom)),
        ["present"; 2],
        "healed and rested"
    );
}
