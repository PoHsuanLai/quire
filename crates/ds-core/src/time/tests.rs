use crate::time::clock::{VirtualClock, install_wall, now, sleep};
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Wake, Waker};
use std::time::Duration;

#[derive(Default)]
struct Count(AtomicUsize);

impl Wake for Count {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn now_reads_the_installed_clock_and_the_guard_restores_the_wall() {
    let clock = VirtualClock::new();
    let guard = clock.install();
    let start = now();
    clock.advance_to(ms(250));
    assert_eq!(now() - start, ms(250));
    drop(guard);
    assert!(now() >= clock.now() - ms(250));
}

#[test]
fn a_sleep_is_due_exactly_at_its_length_and_wakes_its_task_once() {
    let clock = VirtualClock::new();
    let _guard = clock.install();
    let count = Arc::new(Count::default());
    let waker = Waker::from(Arc::clone(&count));
    let mut cx = Context::from_waker(&waker);
    let mut wait = pin!(sleep(ms(100)));
    assert_eq!(wait.as_mut().poll(&mut cx), Poll::Pending);
    assert_eq!(clock.next_due(), Some(ms(100)));
    clock.advance_to(ms(99));
    assert_eq!(count.0.load(Ordering::SeqCst), 0);
    clock.advance_to(ms(100));
    assert_eq!(count.0.load(Ordering::SeqCst), 1);
    assert_eq!(wait.as_mut().poll(&mut cx), Poll::Ready(()));
    assert_eq!(clock.waiting(), 0);
}

#[test]
fn a_dropped_sleep_leaves_the_queue_and_time_never_runs_backwards() {
    let clock = VirtualClock::new();
    let _guard = clock.install();
    clock.advance_to(ms(50));
    let early = sleep(ms(10));
    let late = sleep(ms(30));
    assert_eq!(clock.next_due(), Some(ms(60)));
    drop(early);
    assert_eq!(clock.next_due(), Some(ms(80)));
    drop(late);
    assert_eq!(clock.next_due(), None);
    clock.advance_to(ms(20));
    assert_eq!(clock.elapsed(), ms(50));
}

#[test]
fn stamps_count_milliseconds_from_the_origin() {
    use crate::time::stamp::{FrameClock, Stamp};
    use std::time::Instant;
    let origin = Instant::now();
    let clock = FrameClock::new(origin);
    assert_eq!(clock.stamp(origin), Stamp(0));
    assert_eq!(clock.stamp(origin + ms(1234)), Stamp(1234));
    let before = origin.checked_sub(ms(5)).unwrap_or(origin);
    assert_eq!(clock.stamp(before), Stamp(0));
}

#[test]
fn stamp_arithmetic_saturates() {
    use crate::time::stamp::Stamp;
    assert_eq!(Stamp(900).since(Stamp(400)), 500);
    assert_eq!(Stamp(400).since(Stamp(900)), 0);
    assert_eq!(Stamp(10).after(5), Stamp(15));
    assert_eq!(Stamp(u64::MAX).after(1), Stamp(u64::MAX));
}

#[test]
fn a_frame_clock_follows_the_installed_virtual_clock() {
    use crate::time::stamp::{FrameClock, Stamp};
    let clock = VirtualClock::new();
    let _guard = clock.install();
    let frames = FrameClock::started();
    assert_eq!(frames.now(), Stamp(0));
    clock.advance_to(ms(250));
    assert_eq!(frames.now(), Stamp(250));
}

#[test]
fn a_stamp_serialises_as_its_bare_number() {
    use crate::time::stamp::Stamp;
    assert_eq!(serde_json::to_string(&Stamp(42)).unwrap(), "42");
    assert_eq!(serde_json::from_str::<Stamp>("42").unwrap(), Stamp(42));
}

/// Whether `now()` reads from this clock.
fn reads(clock: &VirtualClock) -> bool {
    now() == clock.now()
}

#[test]
fn guards_dropped_in_creation_order_leave_no_stale_clock() {
    let (first, second) = (VirtualClock::new(), VirtualClock::new());
    first.advance_to(ms(3));
    second.advance_to(ms(7));
    let (g1, g2) = (first.install(), second.install());
    assert!(reads(&second));
    // The older guard goes first: the newer clock stays in force, not the older one restored
    // over it.
    drop(g1);
    assert!(reads(&second));
    drop(g2);
    assert!(!reads(&first) && !reads(&second), "back on the wall clock");
}

#[test]
fn guards_dropped_newest_first_restore_the_older_clock() {
    let (first, second) = (VirtualClock::new(), VirtualClock::new());
    first.advance_to(ms(3));
    second.advance_to(ms(7));
    let (g1, g2) = (first.install(), second.install());
    drop(g2);
    assert!(reads(&first));
    drop(g1);
    assert!(!reads(&first));
}

#[test]
fn a_wall_install_shadows_a_live_virtual_clock_and_lifts_with_its_guard() {
    let clock = VirtualClock::new();
    clock.advance_to(Duration::from_secs(3600));
    let virtual_guard = clock.install();
    let wall = install_wall();
    assert!(!reads(&clock), "the wall install shadows the virtual clock");
    // The virtual guard going first must not bring the virtual clock back over the wall one.
    drop(virtual_guard);
    assert!(!reads(&clock));
    drop(wall);
    assert!(!reads(&clock));
}

#[test]
fn a_wall_install_lifts_to_the_virtual_clock_below_it() {
    let clock = VirtualClock::new();
    clock.advance_to(ms(5));
    let _virtual = clock.install();
    drop(install_wall());
    assert!(reads(&clock));
}
