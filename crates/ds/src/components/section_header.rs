//! SectionHeader: a small-caps label that names a group (design/04-COMPONENTS.md section 13).

use crate::components::vocab::Selection;
use dioxus::prelude::*;

/// Where the header sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeaderKind {
    /// Sidebar groups on the frame, with a trailing rule and an optional action.
    Frame,
    /// List groups on the card, with a count and a trailing rule.
    Group,
    /// A label above a control, with an optional value.
    Field,
    /// A group title inside a menu or palette.
    Menu,
}

impl HeaderKind {
    /// The `data-kind` word.
    fn slug(self) -> &'static str {
        match self {
            HeaderKind::Frame => "frame",
            HeaderKind::Group => "group",
            HeaderKind::Field => "field",
            HeaderKind::Menu => "menu",
        }
    }

    /// Whether a trailing rule follows the text: Frame and Group draw one.
    fn rule(self) -> Rule {
        match self {
            HeaderKind::Frame | HeaderKind::Group => Rule::Drawn,
            HeaderKind::Field | HeaderKind::Menu => Rule::None,
        }
    }
}

/// Whether a header draws its trailing rule.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Rule {
    Drawn,
    None,
}

/// A group's name. The visual order is text, value, rule, action (S puts the rule between the
/// text and the Frame's action button, `S:120-125`). `action_selection: Selected` draws the
/// action as a keyboard selection (`data-selected`): a command palette's cursor resting on a
/// group's "Show More" (sill Q294). `on_action_mounted` hears the action's element as it mounts:
/// the palette keeps it in view when its cursor rests there (sill Q340).
#[component]
pub fn SectionHeader(
    kind: HeaderKind,
    text: String,
    #[props(default)] value: Option<String>,
    #[props(default)] action: Option<(String, EventHandler<()>)>,
    #[props(default)] action_selection: Selection,
    #[props(default)] on_action_mounted: Option<EventHandler<MountedEvent>>,
) -> Element {
    let selected = (action_selection == Selection::Selected).then_some("true");
    // The rule is a real span, not `::after`: pseudo-elements are unverified in Blitz (O-22's
    // fallback, `ds-section-header-rule`).
    rsx! {
        div { class: "ds-section-header", "data-kind": kind.slug(),
            span { class: "ds-section-header-text", "{text}" }
            if let Some(value) = value {
                span { class: "ds-section-header-value", "{value}" }
            }
            if kind.rule() == Rule::Drawn {
                span { class: "ds-section-header-rule" }
            }
            if let Some((label, onclick)) = action {
                button {
                    r#type: "button",
                    class: "ds-section-header-action",
                    "data-selected": selected,
                    onclick: move |_| onclick.call(()),
                    onmounted: move |event| {
                        if let Some(mounted) = on_action_mounted {
                            mounted.call(event);
                        }
                    },
                    "{label}"
                }
            }
        }
    }
}
