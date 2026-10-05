//! A `List` driven by its own settle timers: its roster inside a `Ds` environment at the Reduced
//! level (every settle 94 ms), drawn with `ThreadRow`s, the VirtualDom polled on a small
//! `block_on` until each state has settled.

use super::rows::listed_thread;
use crate::dom_time::TimedDom;
use crate::scoped::scope;
use dioxus::prelude::*;
use ds::prelude::*;
use std::time::Duration;

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
fn set_keys(dom: &mut TimedDom, keys: &[&'static str]) -> String {
    let keys = keys.to_vec();
    dom.in_scope(ScopeId::APP, || {
        consume_context::<Signal<Vec<&'static str>>>().set(keys)
    });
    dom.render_immediate_to_vec();
    dioxus_ssr::render(dom)
}

/// Step the list's timers until its presences are `want`, or the wait runs out.
fn until(dom: &mut TimedDom, want: &[&str]) -> Option<String> {
    dom.run_until(SETTLED, |dom| presences(&dioxus_ssr::render(dom)) == want)?;
    Some(dioxus_ssr::render(dom))
}

/// A list through a key arriving, a key leaving, the rows below healing and the heal resting: the
/// markup at each moment, by name. The list plays them on its own settle timers.
pub fn moments() -> Vec<(&'static str, String)> {
    let mut dom = TimedDom::new(|| VirtualDom::new(app));
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
