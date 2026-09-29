//! `use_widget`: the entry a widget shows now, from its provider's timeline (design/23-WIDGETS.md
//! section 9.2). The hook reads the design system's clock, so on a test's virtual clock the
//! entries change exactly when the test advances past their dates. It wakes only for what the
//! timeline holds (the next entry's date, the refresh) and sleeps on nothing else: a widget whose
//! provider pushes one entry at a time costs no frame between pushes (the idle-frame rule).

use crate::components::widget_kind::WidgetSize;
use crate::task::{spawn_in, try_get, try_set};
use crate::widget::contract::{Widget, fit};
use crate::widget::timeline::{RefreshAsk, Timeline, Wake};
use dioxus::core::{Task, current_scope_id};
use dioxus::prelude::*;
use std::time::Instant;

/// The entry `W` shows now from `timeline` at `size`: the timeline's current entry, or `W`'s
/// placeholder before its first. On each entry's date the caller re-renders with the next; when
/// the refresh policy comes due, `onrefresh` is asked once, and the provider answers with a new
/// timeline (an equal timeline passed again changes nothing).
pub fn use_widget<W: Widget>(
    timeline: &Timeline<W::Entry>,
    size: WidgetSize,
    onrefresh: Option<EventHandler<RefreshAsk>>,
) -> W::Entry {
    let tick = use_signal(|| 0_u64);
    // Read so that the task's write re-renders the caller.
    let _ = tick();
    let scope = use_hook(current_scope_id);
    let mut armed = use_hook(|| CopyValue::new(None::<Armed<W::Entry>>));
    let fresh = armed
        .peek()
        .as_ref()
        .is_none_or(|was| was.timeline != *timeline);
    if fresh {
        if let Some(was) = armed.write().take() {
            was.task.cancel();
        }
        let task = spawn_in(
            scope,
            follow(timeline.clone(), crate::time::clock::now(), tick, onrefresh),
        );
        armed.set(Some(Armed {
            timeline: timeline.clone(),
            task,
        }));
    }
    timeline
        .current(crate::time::clock::now())
        .cloned()
        .unwrap_or_else(|| W::placeholder(fit::<W>(size)))
}

/// The timeline being followed, and the task following it.
struct Armed<E> {
    timeline: Timeline<E>,
    task: Task,
}

/// Sleep to each wake the timeline names after `arrived`, re-rendering at each; at the refresh,
/// ask once and stop (the answer is a new timeline, which starts a new follower).
async fn follow<E: 'static>(
    timeline: Timeline<E>,
    arrived: Instant,
    tick: Signal<u64>,
    onrefresh: Option<EventHandler<RefreshAsk>>,
) {
    loop {
        let now = crate::time::clock::now();
        let Some(wake) = timeline.next_wake(now, arrived) else {
            return;
        };
        crate::time::clock::sleep(wake.at().saturating_duration_since(now)).await;
        let Ok(count) = try_get(tick) else {
            return;
        };
        if try_set(tick, count.wrapping_add(1)).is_err() {
            return;
        }
        if let Wake::Refresh(_, ask) = wake {
            if let Some(handler) = onrefresh {
                handler.call(ask);
            }
            return;
        }
    }
}
