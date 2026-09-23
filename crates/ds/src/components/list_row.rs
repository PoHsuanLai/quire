//! ListRow: one item in a list, the thread row (design/04-COMPONENTS.md section 16).

use crate::components::vocab::{
    DropState, Emphasis, PulseKey, PulsePhase, Selection, StaggerIndex, Switch,
};
use crate::icon::Icon;
use crate::icon::Shape;
use crate::motion::anim::Anim;
use crate::motion::presence::Presence;
use dioxus::html::geometry::{ClientPoint, ElementPoint, PagePoint, ScreenPoint};
use dioxus::html::input_data::{MouseButton, MouseButtonSet};
use dioxus::html::{
    HasMouseData, InteractionElementOffset, InteractionLocation, Modifiers, ModifiersInteraction,
    PointerInteraction,
};
use dioxus::prelude::*;

/// The six spark angles, 0 to 300 degrees in steps of 60 (`S:1285`).
const SPARK_ANGLES: [u16; 6] = [0, 60, 120, 180, 240, 300];

/// The `data-emphasis` word.
fn emphasis_slug(emphasis: Emphasis) -> &'static str {
    match emphasis {
        Emphasis::Strong => "strong",
        Emphasis::Plain => "plain",
    }
}

/// The row's inline custom properties: the stagger `--i` always, and while healing the
/// distance `--dy` and the heal index `--d`.
fn row_style(index: StaggerIndex, presence: Presence) -> String {
    let i = index.get();
    match presence {
        Presence::Healing { dy, d } => format!("--i:{i};--dy:{}px;--d:{}", dy.0, d.get()),
        Presence::Entering | Presence::Present | Presence::Leaving(_) => format!("--i:{i}"),
    }
}

/// `data-exit`, on a leaving row only.
fn exit(presence: Presence) -> Option<&'static str> {
    match presence {
        Presence::Leaving(exit) => Some(exit.slug()),
        Presence::Entering | Presence::Present | Presence::Healing { .. } => None,
    }
}

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
fn star_button(state: Switch, onchange: EventHandler<Switch>, pulse: PulseKey) -> Element {
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

/// One row: dot, name and via, subject, snippet, tail, star, and a hover-strip slot.
///
/// `presence` comes from `use_roster`: an entering row rises staggered by `index` when its
/// list is first shown and plays `row-in` when it arrives later; a leaving row plays its exit
/// (an unread fold is the heavy one, as the roster settles it); a healing row slides up from
/// `dy`, delayed by `d` heal steps. `star_pulse` is a `use_pulse(Anim::StarPop)` key, fired on
/// every toggle. `onclick` receives the pointer's data, so the consumer can read Shift to peek.
/// `drop` is the row's part in a drag: `Source` while it is the thread being dragged (dimmed),
/// `Target` while something dragged over it would land on it.
#[component]
pub fn ListRow(
    selection: Selection,
    emphasis: Emphasis,
    index: StaggerIndex,
    presence: Presence,
    name: String,
    via: Option<Element>,
    subject: String,
    snippet: Option<String>,
    time: String,
    tags: Element,
    star: Option<(Switch, EventHandler<Switch>)>,
    star_pulse: PulseKey,
    strip: Option<Element>,
    onclick: EventHandler<MouseData>,
    #[props(default)] drop: DropState,
) -> Element {
    rsx! {
        li {
            class: "ds-row",
            role: "option",
            "aria-selected": selection.aria(),
            "data-emphasis": emphasis_slug(emphasis),
            "data-presence": presence.slug(),
            "data-exit": exit(presence),
            "data-drop": drop.drop_attr(),
            "data-drag": drop.drag_attr(),
            style: row_style(index, presence),
            onclick: move |event| onclick.call(snapshot(&event.data())),
            div { class: "ds-row-dot",
                span { class: "ds-dot" }
            }
            div { class: "ds-row-main",
                div { class: "ds-row-from",
                    span { class: "ds-row-name ds-truncate", "{name}" }
                    if let Some(via) = via {
                        span { class: "ds-row-via", {via} }
                    }
                }
                div { class: "ds-row-sub ds-truncate", "{subject}" }
                if let Some(snippet) = snippet {
                    div { class: "ds-row-snip ds-truncate", "{snippet}" }
                }
            }
            div { class: "ds-row-tail",
                span { class: "ds-row-time", "{time}" }
                span { class: "ds-row-tags", {tags} }
            }
            if let Some((state, onchange)) = star {
                {star_button(state, onchange, star_pulse)}
            }
            if let Some(strip) = strip {
                {strip}
            }
        }
    }
}

/// A pointer event's data, copied: the handler owns a `MouseData` it can keep, as the
/// `EventHandler<MouseData>` prop asks, while dioxus holds the original behind an `Rc`.
fn snapshot(data: &MouseData) -> MouseData {
    MouseData::new(Pressed {
        client: data.client_coordinates(),
        page: data.page_coordinates(),
        screen: data.screen_coordinates(),
        element: data.element_coordinates(),
        modifiers: data.modifiers(),
        held: data.held_buttons(),
        trigger: data.trigger_button(),
    })
}

/// What a click carried, kept by value.
struct Pressed {
    client: ClientPoint,
    page: PagePoint,
    screen: ScreenPoint,
    element: ElementPoint,
    modifiers: Modifiers,
    held: MouseButtonSet,
    trigger: Option<MouseButton>,
}

impl InteractionLocation for Pressed {
    fn client_coordinates(&self) -> ClientPoint {
        self.client
    }

    fn screen_coordinates(&self) -> ScreenPoint {
        self.screen
    }

    fn page_coordinates(&self) -> PagePoint {
        self.page
    }
}

impl InteractionElementOffset for Pressed {
    fn element_coordinates(&self) -> ElementPoint {
        self.element
    }
}

impl ModifiersInteraction for Pressed {
    fn modifiers(&self) -> Modifiers {
        self.modifiers
    }
}

impl PointerInteraction for Pressed {
    fn trigger_button(&self) -> Option<MouseButton> {
        self.trigger
    }

    fn held_buttons(&self) -> MouseButtonSet {
        self.held
    }
}

impl HasMouseData for Pressed {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{exit, row_style, sparks};
    use crate::components::vocab::{PulseKey, StaggerIndex, Switch};
    use crate::geometry::Px;
    use crate::motion::anim::Anim;
    use crate::motion::presence::{Exit, Presence};

    #[test]
    fn a_healing_row_carries_its_distance_and_delay() {
        let healing = Presence::Healing {
            dy: Px(79.0),
            d: StaggerIndex::new(2),
        };
        assert_eq!(
            row_style(StaggerIndex::new(4), healing),
            "--i:4;--dy:79px;--d:2"
        );
        assert_eq!(
            row_style(StaggerIndex::new(40), Presence::Present),
            "--i:12"
        );
    }

    #[test]
    fn only_a_leaving_row_names_its_exit() {
        const CASES: &[(Presence, Option<&str>)] = &[
            (Presence::Leaving(Exit::Fold), Some("fold")),
            (Presence::Leaving(Exit::Curl), Some("curl")),
            (Presence::Leaving(Exit::Crumple), Some("crumple")),
            (Presence::Leaving(Exit::TabOut), Some("tab-out")),
            (Presence::Entering, None),
            (Presence::Present, None),
        ];
        for (presence, want) in CASES {
            assert_eq!(exit(*presence), *want, "{presence:?}");
        }
    }

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
