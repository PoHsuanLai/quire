//! Bump on change: a value a widget shows (a battery's percentage, a clock's minute) plays
//! `bump` once each time it changes, and is at rest again once the bump has settled (design/20
//! section 1.14: "none in steady state; value change `bump`"; sill FINDINGS Q182).
//!
//! `Count` decides its bump while rendering and never returns to rest; a widget cannot, because
//! nothing may keep a pulse class on a surface that should idle. So the pulse is fired through
//! `use_pulse` after the render that saw the change, and a `MotionTimer` for `Anim::Bump` puts
//! it back at rest at `settle(Bump)`. A change while one bump plays restarts it on the other
//! alias; nothing loops.

use crate::components::vocab::PulseKey;
use crate::motion::{Anim, TimerPhase, use_motion_timer, use_pulse};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The `Anim::Bump` pulse for `value`: at rest on mount, fired each time `value` differs from
/// the value last rendered, and at rest again `settle(Bump)` after the latest firing. Render
/// the key with [`bump_attrs`] or wear it through [`Bumped`]. Needs an enclosing `Ds` (the
/// settle reads its motion level).
///
/// The last value lives in a plain value, not a signal: the change is noticed while rendering,
/// and a signal written during render would schedule a second render for nothing.
pub fn use_bump_on<T: PartialEq + Clone + 'static>(value: T) -> PulseKey {
    let pulse = use_pulse(Anim::Bump);
    let timer = use_motion_timer(Anim::Bump);
    let settled = use_hook(|| EventHandler::new(|()| {}));
    let mut seen = use_hook(|| CopyValue::new(value.clone()));
    if *seen.peek() != value {
        seen.set(value);
        // Fired after this render, as an event handler would: a signal written while rendering
        // would re-render this component from inside its own render.
        queue_effect(move || {
            pulse.fire();
            timer.start(settled);
        });
    }
    worn(pulse.key(), timer.phase())
}

/// What a bump wears while its timer is in `phase`: the fired key while it runs, rest otherwise.
fn worn(key: PulseKey, phase: TimerPhase) -> PulseKey {
    match phase {
        TimerPhase::Running => key,
        TimerPhase::Idle | TimerPhase::Settled => PulseKey::rest(key.anim()),
    }
}

/// `base` with the pulse's class added while it plays, and its `data-pulse` alias.
pub(crate) fn bump_attrs(base: &str, key: PulseKey) -> (String, Option<&'static str>) {
    match key.attrs() {
        Some((anim, alias)) => (format!("{base} {anim}"), Some(alias)),
        None => (base.to_string(), None),
    }
}

/// `children` in a `span.ds-bumped` that bumps once each time `on` changes: for a run of text a
/// widget draws itself (a device's percentage, a zone's time), where no quire component owns
/// the value.
#[component]
pub fn Bumped<T: PartialEq + Clone + 'static>(on: T, children: Element) -> Element {
    let (class, alias) = bump_attrs("ds-bumped", use_bump_on(on));
    rsx! {
        span { class, "data-pulse": alias, {children} }
    }
}

#[cfg(test)]
mod tests {
    use super::{bump_attrs, worn};
    use crate::components::vocab::{PulseKey, PulsePhase};
    use crate::motion::{Anim, TimerPhase};

    #[test]
    fn only_a_running_timer_wears_the_fired_key() {
        let fired = PulseKey::rest(Anim::Bump).fired();
        const PHASES: [(TimerPhase, PulsePhase); 3] = [
            (TimerPhase::Idle, PulsePhase::Rest),
            (TimerPhase::Running, PulsePhase::A),
            (TimerPhase::Settled, PulsePhase::Rest),
        ];
        for (phase, want) in PHASES {
            assert_eq!(worn(fired, phase).phase(), want, "{phase:?}");
        }
    }

    #[test]
    fn a_playing_key_adds_its_class_and_alias() {
        let rest = PulseKey::rest(Anim::Bump);
        assert_eq!(bump_attrs("ds-bumped", rest), ("ds-bumped".into(), None));
        assert_eq!(
            bump_attrs("ds-bumped", rest.fired()),
            ("ds-bumped a-bump".into(), Some("a"))
        );
    }
}
