//! A thread row's star (design/04-COMPONENTS.md section 16): the button that pops on every
//! toggle and the sparks that fly only when starring. Split from `list_row` so the row's own
//! file holds the row.

use crate::components::vocab::{PulseKey, PulsePhase, Switch};
use crate::icon::Icon;
use crate::icon::Shape;
use crate::motion::anim::Anim;
use dioxus::prelude::*;

/// The six spark angles, 0 to 300 degrees in steps of 60 (`S:1285`).
const SPARK_ANGLES: [u16; 6] = [0, 60, 120, 180, 240, 300];

/// The sparks play the star's pulse alias, but only when starring (`S:1526`): the `spark`
/// pulse in the same phase as `star`, or nothing.
fn sparks(state: Switch, star: PulseKey) -> Option<(String, &'static str)> {
    let rest = PulseKey::rest(Anim::Spark);
    let spark = match star.phase() {
        PulsePhase::Rest => rest,
        PulsePhase::A => rest.fired(),
        PulsePhase::B => rest.fired().fired(),
    };
    match state {
        Switch::On => spark.attrs(),
        Switch::Off => None,
    }
}

/// The star's glyph: the outline, filled with its own colour once starred. `Glyph` only
/// strokes, and a CSS `fill` never reaches SVG on Blitz (spike S6), so the fill is written as an
/// attribute here.
fn star_glyph(state: Switch) -> Element {
    let fill = match state {
        Switch::On => "currentColor",
        Switch::Off => "none",
    };
    rsx! {
        svg {
            class: "ds-ic",
            "data-size": "14",
            width: "14",
            height: "14",
            view_box: "0 0 24 24",
            "aria-hidden": "true",
            "stroke": "currentColor",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            "fill": fill,
            for shape in Icon::Star.shapes() {
                if let Shape::Path(d) = shape {
                    path { d: "{d}" }
                }
            }
        }
    }
}

/// The star button: pops on every toggle, sparks only when starring.
pub(crate) fn star_button(
    state: Switch,
    onchange: EventHandler<Switch>,
    pulse: PulseKey,
) -> Element {
    let label = match state {
        Switch::On => "Unstar this thread",
        Switch::Off => "Star this thread",
    };
    let (pop_class, pop_alias) = match pulse.attrs() {
        Some((anim, alias)) => (format!("ds-star-glyph {anim}"), Some(alias)),
        None => ("ds-star-glyph".to_string(), None),
    };
    let (spark_class, spark_alias) = match sparks(state, pulse) {
        Some((anim, alias)) => (Some(anim), Some(alias)),
        None => (None, None),
    };
    rsx! {
        button {
            r#type: "button",
            class: "ds-star",
            "aria-pressed": state.aria(),
            "aria-label": label,
            onclick: move |event| {
                // The star acts on its own; the row must not also open.
                event.stop_propagation();
                onchange.call(state.flipped());
            },
            span { class: pop_class, "data-pulse": pop_alias, {star_glyph(state)} }
            span { class: "ds-sparks",
                for angle in SPARK_ANGLES {
                    i {
                        class: spark_class.clone(),
                        "data-pulse": spark_alias,
                        style: "--a:{angle}deg",
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::sparks;
    use crate::components::vocab::{PulseKey, Switch};
    use crate::motion::anim::Anim;

    #[test]
    fn sparks_follow_the_pop_only_when_starring() {
        let rest = PulseKey::rest(Anim::StarPop);
        assert_eq!(sparks(Switch::On, rest), None, "at rest nothing plays");
        assert_eq!(
            sparks(Switch::On, rest.fired()),
            Some(("a-spark".to_string(), "a"))
        );
        assert_eq!(
            sparks(Switch::On, rest.fired().fired()),
            Some(("a-spark".to_string(), "b"))
        );
        assert_eq!(sparks(Switch::Off, rest.fired()), None, "unstarring");
    }
}
