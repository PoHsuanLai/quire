//! A toast's swipe to dismiss: the glue between its pointer events and the pure machine
//! (`ds_motion::swipe`), and the spring that carries it home when a drag ends under both
//! thresholds (design/30 section 1.4, "Swipe"; design/27 section 3.12).
//!
//! Released past 80 px or 600 px/s to the right it is dismissed from where it is; the toast's
//! exit (`panel-out`) starts at the offset the drag left (`--swipe-dx`), so it flies on rather
//! than starting over. Under both it springs back carrying the hand's release velocity.

use crate::components::controls::press::button_of;
use dioxus::core::queue_effect;
use dioxus::html::input_data::MouseButton;
use dioxus::prelude::*;
use ds_core::geometry::units::Px;
use ds_core::press::PointerButton;
use ds_core::vocab::PressPhase;
use ds_motion::detail::touch::{Contact, Touch};
use ds_motion::swipe::{Click, SwipeInput, SwipeLook, SwipeMetrics, SwipeState};
use ds_motion::use_swipe::{Swiper, use_swipe};
use ds_motion::{
    spring_spec::SpringSpec,
    timeline::spring::PxPerUnit,
    use_spring::{SpringMotion, use_spring_motion},
    velocity::VelocityMeter,
};

/// The toast's swipe for one render.
#[derive(Clone, Copy)]
pub(crate) struct ToastSwipe {
    swiper: Swiper,
    spring: SpringMotion,
    meter: Signal<VelocityMeter>,
    released: CopyValue<Touch>,
    followed: CopyValue<SwipeLook>,
}

/// The toast's swipe: `on_dismiss` hears a release past a threshold.
pub(crate) fn use_toast_swipe(on_dismiss: EventHandler<()>) -> ToastSwipe {
    ToastSwipe {
        swiper: use_swipe(SwipeMetrics::default(), on_dismiss),
        spring: use_spring_motion(0.0, PxPerUnit(1.0)),
        meter: use_signal(VelocityMeter::default),
        released: use_hook(|| CopyValue::new(Touch::Remote)),
        followed: use_hook(|| CopyValue::new(SwipeLook::Rest)),
    }
}

impl ToastSwipe {
    /// `data-swipe`: `live` while held, `rest`, or `gone`.
    pub(crate) fn look(&self) -> &'static str {
        use ds_core::word::Word;
        self.swiper.state().look().slug()
    }

    /// The toast's inline offset, `--swipe-dx`, while it is off its place (on its way back too).
    /// Call once per render: it moves the return spring.
    pub(crate) fn style(&self) -> Option<String> {
        let state = self.swiper.state();
        let drawn = self.drawn(state);
        let off = drawn.0 != 0.0 || state.look() == SwipeLook::Gone;
        let rounded = (drawn.0 * 100.0).round() / 100.0;
        off.then(|| format!("--swipe-dx:{rounded}px"))
    }

    /// The offset to draw for `state`, moving the spring to follow it.
    fn drawn(&self, state: SwipeState) -> Px {
        let (spring, mut followed) = (self.spring, self.followed);
        let look = state.look();
        let before = *followed.peek();
        followed.set(look);
        match look {
            SwipeLook::Live => {
                let at = state.offset().0;
                queue_effect(move || spring.track(at));
                state.offset()
            }
            SwipeLook::Gone => state.offset(),
            SwipeLook::Rest => {
                if before != SwipeLook::Rest {
                    let mut released = self.released;
                    let touch = *released.peek();
                    released.set(Touch::Remote);
                    queue_effect(move || spring.go(0.0, SpringSpec::for_touch(touch)));
                }
                Px(spring.frame().position())
            }
        }
    }

    /// A click on the toast is a press only when it is not the end of a drag.
    pub(crate) fn click_passes(&self) -> bool {
        self.swiper.take_click() == Click::Passes
    }

    /// A pointer went down on the toast: only the primary button starts a drag.
    pub(crate) fn down(&self, event: &PointerEvent) {
        if button_of(event.trigger_button()) == Some(PointerButton::Primary) {
            let x = Px(event.client_coordinates().x as f32);
            let mut meter = self.meter;
            meter.set(VelocityMeter::default().moved(x, ds_core::time::clock::now()));
            self.swiper.feed(SwipeInput::Down {
                x,
                at: self.swiper.now(),
            });
        }
    }

    /// The pointer moved over the toast.
    pub(crate) fn moved(&self, event: &PointerEvent) {
        let held = if event.held_buttons().contains(MouseButton::Primary) {
            PressPhase::Pressed
        } else {
            PressPhase::Idle
        };
        let x = Px(event.client_coordinates().x as f32);
        let mut meter = self.meter;
        let measured = meter.peek().moved(x, ds_core::time::clock::now());
        meter.set(measured);
        self.swiper.pointer_moved(x, held);
    }

    /// The pointer left the toast: a release it could not hear, so no contact goes with it.
    pub(crate) fn left(&self) {
        self.swiper.feed(SwipeInput::Up {
            at: self.swiper.now(),
        });
    }

    /// The pointer was released over the toast: the release is the hand's contact, carrying the
    /// pointer's velocity into the return.
    pub(crate) fn released(&self, event: &PointerEvent) {
        let velocity = self.meter.peek().released(ds_core::time::clock::now());
        let mut released = self.released;
        released.set(Touch::Contact(
            Contact::from_event(event).with_velocity(velocity),
        ));
        self.swiper.feed(SwipeInput::Up {
            at: self.swiper.now(),
        });
    }
}
