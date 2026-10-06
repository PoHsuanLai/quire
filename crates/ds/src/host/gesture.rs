//! A touchpad's gestures with their phases: a pinch, and a scroll that says when it began and
//! when the fingers lifted. Blitz carries neither (its wheel event has no phase and it has no
//! pinch event; FINDINGS "Events"), but the window sees winit's events before the document does,
//! so the host publishes them here and a component listens with [`use_gestures`], without
//! naming a renderer.
//!
//! ```ignore
//! use_gestures(move |gesture| match gesture {
//!     Gesture::Pinch { by, at, .. } => zoom_about(at, by),
//!     Gesture::Scroll { phase: GesturePhase::Ended, .. } => settle(),
//!     Gesture::Scroll { by, .. } => pan(by),
//! });
//! ```
//!
//! A gesture is not addressed to an element: it reaches every listener, with the pointer's place,
//! and the listener decides whether the pointer is over it (`harness.rect(..)` in a test, a
//! measured rect in a component).
//!
//! **A wheel's detents.** A listener moves its own content from `Gesture::Scroll { by }`, so what
//! `by` is matters. [`use_gestures`] hears each detent as it arrives (`by` is one detent, 60 px,
//! the distance a native scroller moves), in one jump. A component that moves its own offset and
//! wants the host's smooth step (a detent eased over at most 200 ms, detents accumulating, as
//! design/11 §11.3.11 says) listens with [`use_gestures_with`] and [`WheelDelivery::Eased`]: its
//! wheel detents then arrive as one `Gesture::Scroll` per frame whose `by` is that frame's share,
//! summing to the detents' distance. A touchpad's scroll is the fingers' own motion and reaches
//! both kinds of listener unchanged.

use dioxus::prelude::*;
use ds_core::geometry::units::Point;
use std::cell::RefCell;
use std::rc::Rc;

/// Where in its run a gesture is. A gesture's state is cleared by `Ended` and by `Cancelled`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GesturePhase {
    /// The fingers touched down.
    Began,
    /// They moved.
    Changed,
    /// They lifted.
    Ended,
    /// The system took the gesture away: undo nothing, forget it.
    Cancelled,
}

/// A change of scale in thousandths of the current scale: `Magnification(50)` is 5% larger and
/// `Magnification(-50)` 5% smaller. Integer, so a gesture stays `Eq`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Magnification(pub i32);

impl Magnification {
    /// The change winit reports (a fraction of the scale, positive growing) in thousandths,
    /// rounded; zero for a value that is not a number.
    pub fn from_fraction(delta: f64) -> Self {
        if delta.is_finite() {
            // Bounded by the clamp, so the cast cannot truncate.
            Magnification((delta * 1000.0).round().clamp(-1000.0, 1000.0) as i32)
        } else {
            Magnification(0)
        }
    }
}

/// A gesture of a touchpad or a wheel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gesture {
    /// Two fingers pinching or spreading.
    Pinch {
        /// Where the gesture is in its run.
        phase: GesturePhase,
        /// How much the scale changed since the last event.
        by: Magnification,
        /// The pointer, in the window's logical pixels.
        at: Point,
    },
    /// A scroll: two fingers on a touchpad, or a wheel's detents.
    Scroll {
        /// Where the scroll is in its run; a wheel without phases reports `Changed` only.
        phase: GesturePhase,
        /// How far the content moves, in logical pixels, positive to the right and down: the
        /// direction the content travels, as winit reports it (the opposite of the web's
        /// `deltaX`). A wheel detent is 60 px, the distance a native scroller moves for it.
        by: Point,
        /// The pointer, in the window's logical pixels.
        at: Point,
        /// The modifier keys held while it happened: a viewer zooms with the wheel under
        /// Control, and scrolls with it otherwise.
        held: Modifiers,
    },
}

/// How a listener wants a wheel's detents: in one jump as they arrive, or eased over frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WheelDelivery {
    /// One `Gesture::Scroll` per detent, as the wheel turned.
    AsReceived,
    /// One `Gesture::Scroll` per frame while the host eases the detents over time.
    Eased,
}

/// The listeners of one window: provided as root context by the host, which publishes to it.
#[derive(Clone, Default)]
pub struct GestureBus(Rc<RefCell<Listeners>>);

#[derive(Default)]
struct Listeners {
    next: u64,
    entries: Vec<Entry>,
}

/// One registered listener and how it wants a wheel.
struct Entry {
    id: u64,
    delivery: WheelDelivery,
    callback: Callback<Gesture>,
}

impl std::fmt::Debug for GestureBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GestureBus").finish_non_exhaustive()
    }
}

impl GestureBus {
    /// Hand `gesture` to every listener, in the order they registered. The host calls it from
    /// its window events; a test calls it to stand in for a touchpad.
    pub fn publish(&self, gesture: Gesture) {
        self.deliver(gesture, |_| true);
    }

    /// Hand `gesture` only to the listeners that asked for `delivery`: the host sends a wheel's
    /// detents as received to one kind and eased frames to the other.
    pub fn publish_to(&self, delivery: WheelDelivery, gesture: Gesture) {
        self.deliver(gesture, |entry| entry.delivery == delivery);
    }

    /// Whether any listener asked for `delivery`: the host eases a wheel's detents only for
    /// listeners that want them.
    pub fn has_listener(&self, delivery: WheelDelivery) -> bool {
        self.0
            .borrow()
            .entries
            .iter()
            .any(|entry| entry.delivery == delivery)
    }

    fn deliver(&self, gesture: Gesture, wants: impl Fn(&Entry) -> bool) {
        let listeners: Vec<Callback<Gesture>> = self
            .0
            .borrow()
            .entries
            .iter()
            .filter(|entry| wants(entry))
            .map(|entry| entry.callback)
            .collect();
        for listener in listeners {
            listener.call(gesture);
        }
    }

    fn add(&self, delivery: WheelDelivery, callback: Callback<Gesture>) -> u64 {
        let mut listeners = self.0.borrow_mut();
        listeners.next += 1;
        let id = listeners.next;
        listeners.entries.push(Entry {
            id,
            delivery,
            callback,
        });
        id
    }

    fn remove(&self, id: u64) {
        self.0.borrow_mut().entries.retain(|entry| entry.id != id);
    }
}

/// Hear the window's gestures for as long as the calling component lives, a wheel's detents as
/// they arrive ([`WheelDelivery::AsReceived`]). With no host there is no bus and nothing arrives
/// (a server render, a document with no window).
pub fn use_gestures(on_gesture: impl FnMut(Gesture) + 'static) {
    use_gestures_with(WheelDelivery::AsReceived, on_gesture);
}

/// [`use_gestures`], choosing how a wheel's detents arrive.
pub fn use_gestures_with(delivery: WheelDelivery, on_gesture: impl FnMut(Gesture) + 'static) {
    let callback = use_callback(on_gesture);
    let bus = try_consume_context::<GestureBus>();
    use_hook(|| {
        bus.map(|bus| {
            let id = bus.add(delivery, callback);
            Rc::new(Joined { bus, id })
        })
    });
}

/// A registration that ends with the component that made it.
struct Joined {
    bus: GestureBus,
    id: u64,
}

impl Drop for Joined {
    fn drop(&mut self) {
        self.bus.remove(self.id);
    }
}

#[cfg(test)]
mod tests {
    use super::Magnification;

    #[test]
    fn a_pinch_is_counted_in_thousandths() {
        // name, winit's delta, thousandths
        const CASES: &[(&str, f64, i32)] = &[
            ("grow", 0.05, 50),
            ("shrink", -0.125, -125),
            ("none", 0.0, 0),
            ("rounds", 0.0004, 0),
            ("rounds up", 0.0006, 1),
            ("capped above", 7.0, 1000),
            ("capped below", -7.0, -1000),
            ("not a number", f64::NAN, 0),
            ("infinite", f64::INFINITY, 0),
        ];
        for (name, delta, want) in CASES {
            assert_eq!(Magnification::from_fraction(*delta).0, *want, "{name}");
        }
    }
}
