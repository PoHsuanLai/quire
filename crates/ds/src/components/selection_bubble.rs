//! SelectionBubble: the inline toolbar over a text selection (design/04-COMPONENTS.md
//! section 30, design/06-INTERACTIONS.md sections 4.2 and 12). The composer supplies the
//! selection rect and the marks (O-21); ds draws the surface, places it centred 8 px above the
//! selection (clamped 8 px inside the window, never flipped below), and runs the link field:
//! Enter applies the link, Escape hides the bubble.

use crate::components::popover::{
    Dismiss, Stacking, escape_closes, position_style, use_entrance, use_float,
};
use crate::components::text_input::{InputVariant, TextInput};
use crate::components::vocab::Switch;
use crate::geometry::{Align, Placement, Px, Rect, Side};
use crate::motion::anim::Anim;
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// One bubble button.
#[derive(Debug, Clone, PartialEq)]
pub struct BubbleAction {
    /// Its face: `B`, `i`, a glyph.
    pub label: Element,
    /// Its title, with the shortcut.
    pub title: String,
    /// Whether the mark is active.
    pub pressed: Option<Switch>,
    /// Pressed.
    pub onclick: EventHandler<()>,
}

/// What the bubble shows.
#[derive(Debug, Clone, PartialEq)]
pub enum BubbleMode {
    /// Formatting buttons.
    Actions(Vec<BubbleAction>),
    /// A link field.
    Link,
}

/// The toolbar over a selection.
#[component]
pub fn SelectionBubble(
    anchor: Rect,
    mode: BubbleMode,
    onlink: EventHandler<String>,
    onclose: EventHandler<()>,
) -> Element {
    let float = use_float(ZLayer::Bubble, Stacking::Layer(Dismiss::EscOnly));
    let presence = use_entrance(Anim::MenuPop);
    let mut link = use_signal(String::new);
    let want = Placement::new(Side::Top, Align::Center).no_flip();
    let at = float.origin(Some(anchor), want, Px(8.0));
    let body = match mode {
        BubbleMode::Actions(actions) => rsx! {
            for (index , action) in actions.into_iter().enumerate() {
                button {
                    key: "{index}",
                    r#type: "button",
                    class: "ds-bubble-button",
                    title: "{action.title}",
                    "aria-pressed": action.pressed.map(Switch::aria),
                    // The selection survives a click on the bubble (`S:2135`).
                    onmousedown: move |event| event.prevent_default(),
                    onclick: move |_| action.onclick.call(()),
                    {action.label}
                }
            }
        },
        BubbleMode::Link => rsx! {
            TextInput {
                variant: InputVariant::Inline,
                label: "Link",
                value: link(),
                placeholder: "Paste a link, then Enter",
                oninput: move |text| link.set(text),
                onkey: move |key: KeyboardData| {
                    if key.key() == Key::Enter {
                        let url = link.peek().trim().to_string();
                        if !url.is_empty() {
                            onlink.call(url);
                        }
                        onclose.call(());
                    }
                },
            }
        },
    };
    let probe = float.surface();
    float.show(
        rsx! {
            div {
                class: "ds-popover ds-bubble",
                "data-elevation": "bubble",
                "data-layer": "bubble",
                "data-presence": presence.slug(),
                role: "toolbar",
                "aria-label": "Format",
                style: position_style(at),
                onmounted: move |event| probe.on_mounted(event),
                onkeydown: move |event| escape_closes(float, &event, onclose),
                {body}
            }
        },
        onclose,
    );
    rsx! {}
}
