//! TodayTabs: the temporary tabs under the pinned tiles (design/30 section 2.11): pages the
//! person opened today, each a `Row` of the `SourceList` shape that carries what it has left and
//! is dropped when that runs out. A tab enters and leaves by the `List`'s roster: it fades and
//! slides in, and on its close (or its expiry) it fades and slides out while the rows below close
//! the gap. The tabs are the caller's; this only stops listing one once its time is up.

use crate::components::lists::list::list::List;
use crate::components::lists::list::model::{ListItem, ListStyle};
use crate::components::lists::row::action::RowAction;
use crate::components::lists::row::leading::RowLeading;
use crate::components::lists::row::row::Row;
use crate::components::lists::row::shape::{Expiry, RowShape};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::time::clock::{now, sleep};
use ds_core::vocab::{RowState, Selection};
use ds_style::icon::Icon;
use ds_style::tokens::control_size::SidebarSize;
use std::time::{Duration, Instant};

/// How little time a tab has left before it is drawn quieter.
const SOON: Duration = Duration::from_secs(30 * 60);

/// The longest the expiry clock sleeps: the "left" words are minutes, so a minute is fine.
const LONGEST_WAIT: Duration = Duration::from_secs(60);

/// The shortest it sleeps, so a tab about to go does not spin the clock.
const SHORTEST_WAIT: Duration = Duration::from_secs(1);

/// One tab: its identity, its page, what leads it and when it goes.
#[derive(Debug, Clone, PartialEq)]
pub struct TodayTab<K> {
    /// The tab's identity, stable across renders.
    pub key: K,
    /// The page's title.
    pub title: String,
    /// What leads it: the page's icon.
    pub leading: RowLeading,
    /// When it expires, on the design system's clock.
    pub expires: Instant,
}

/// What a tab has left, in words: whole hours from an hour on, else minutes rounded up (at least
/// one). Nothing left reads "0 min".
pub fn left_text(left: Duration) -> String {
    let secs = left.as_secs();
    if secs >= 3600 {
        format!("{} h", secs / 3600)
    } else {
        format!("{} min", secs.div_ceil(60))
    }
}

/// How near a tab with `left` is to going.
pub fn expiry_of(left: Duration) -> Expiry {
    if left < SOON {
        Expiry::Soon
    } else {
        Expiry::Later
    }
}

/// How long to sleep before the words or the list next change, at `now`: to the soonest expiry,
/// or a minute for the words, never under a second.
fn next_wait(expiries: impl IntoIterator<Item = Instant>, now: Instant) -> Duration {
    expiries
        .into_iter()
        .map(|at| at.saturating_duration_since(now))
        .filter(|left| !left.is_zero())
        .min()
        .map_or(LONGEST_WAIT, |left| left.min(LONGEST_WAIT))
        .max(SHORTEST_WAIT)
}

/// The tabs still listed at `now`, with what each has left.
fn live<K: Clone>(tabs: &[TodayTab<K>], now: Instant) -> Vec<(TodayTab<K>, Duration)> {
    tabs.iter()
        .filter_map(|tab| {
            let left = tab.expires.saturating_duration_since(now);
            (!left.is_zero()).then(|| (tab.clone(), left))
        })
        .collect()
}

/// The Today tabs of a sidebar. `selected` is the open one; `onpick` hears a press on a tab,
/// `onclose` its close button, and `onexpire` a tab whose time ran out (it stops being listed by
/// itself, and leaves as a closed one does; the caller drops it from its own list).
#[component]
pub fn TodayTabs<K: Clone + PartialEq + std::hash::Hash + 'static>(
    label: String,
    tabs: Vec<TodayTab<K>>,
    #[props(default)] selected: Option<K>,
    #[props(default)] sidebar: SidebarSize,
    onpick: EventHandler<K>,
    onclose: EventHandler<K>,
    #[props(default)] onexpire: Option<EventHandler<K>>,
    #[props(default)] common: Common,
) -> Element {
    let mut clock = use_signal(now);
    let mut held = use_hook(|| CopyValue::new(Vec::<(K, Instant)>::new()));
    held.set(
        tabs.iter()
            .map(|tab| (tab.key.clone(), tab.expires))
            .collect(),
    );
    let mut told = use_hook(|| CopyValue::new(Vec::<K>::new()));
    use_hook(move || {
        spawn(async move {
            loop {
                let wait = next_wait(held.peek().iter().map(|(_, at)| *at), now());
                sleep(wait).await;
                let at = now();
                clock.set(at);
                let gone: Vec<K> = held
                    .peek()
                    .iter()
                    .filter(|(key, expires)| *expires <= at && !told.peek().contains(key))
                    .map(|(key, _)| key.clone())
                    .collect();
                for key in gone {
                    told.write().push(key.clone());
                    if let Some(onexpire) = onexpire {
                        onexpire.call(key);
                    }
                }
            }
        })
    });
    let at = clock();
    let items: Vec<ListItem<K>> = live(&tabs, at)
        .into_iter()
        .map(|(tab, left)| {
            let TodayTab {
                key,
                title,
                leading,
                ..
            } = tab;
            let state = RowState {
                selection: Selection::of(&Some(key.clone()), &selected),
                ..RowState::default()
            };
            let (picked, closed) = (key.clone(), key.clone());
            let content = rsx! {
                Row {
                    title: title.clone(),
                    leading,
                    shape: RowShape::Today { left: left_text(left), expiry: expiry_of(left) },
                    state,
                    action: RowAction::new(
                        Icon::X,
                        format!("Close {title}"),
                        EventHandler::new(move |_: Press| onclose.call(closed.clone())),
                    ),
                    onclick: move |_| onpick.call(picked.clone()),
                }
            };
            ListItem::row(key, title.clone(), content)
        })
        .collect();
    rsx! {
        List::<K> {
            label,
            items,
            style: ListStyle::SourceList,
            sidebar,
            cursor: selected,
            onselect: move |key| onpick.call(key),
            common,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Expiry, TodayTab, expiry_of, left_text, live, next_wait};
    use crate::components::lists::row::leading::RowLeading;
    use std::time::{Duration, Instant};

    fn secs(n: u64) -> Duration {
        Duration::from_secs(n)
    }

    #[test]
    fn what_is_left_reads_in_hours_then_minutes() {
        const CASES: &[(u64, &str)] = &[
            (7200, "2 h"),
            (3600, "1 h"),
            (3599, "60 min"),
            (720, "12 min"),
            (61, "2 min"),
            (1, "1 min"),
            (0, "0 min"),
        ];
        for &(left, want) in CASES {
            assert_eq!(left_text(secs(left)), want, "{left}");
        }
    }

    #[test]
    fn a_tab_with_under_half_an_hour_is_about_to_go() {
        assert_eq!(expiry_of(secs(1799)), Expiry::Soon);
        assert_eq!(expiry_of(secs(1800)), Expiry::Later);
    }

    #[test]
    fn the_clock_sleeps_to_the_soonest_expiry_within_a_minute() {
        let now = Instant::now();
        let at = |n: u64| now + secs(n);
        assert_eq!(next_wait([at(20), at(500)], now), secs(20));
        assert_eq!(next_wait([at(500)], now), secs(60));
        assert_eq!(next_wait([], now), secs(60));
        assert_eq!(next_wait([at(0)], now), secs(60));
        assert_eq!(next_wait([at(0), at(0)], now), secs(60));
        assert_eq!(next_wait([now + Duration::from_millis(100)], now), secs(1));
    }

    #[test]
    fn a_tab_stops_being_listed_when_its_time_is_up() {
        let now = Instant::now();
        let tab = |key: u8, left: u64| TodayTab {
            key,
            title: String::new(),
            leading: RowLeading::None,
            expires: now + secs(left),
        };
        let listed = live(&[tab(1, 0), tab(2, 90)], now);
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].0.key, 2);
        assert_eq!(listed[0].1, secs(90));
    }
}
