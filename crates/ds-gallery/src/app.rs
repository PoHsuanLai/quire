//! The gallery's root: a `Ds` drawn with the toolbar's axes, the toolbar, and the page.

use crate::axes::starting;
use crate::page::Page;
use crate::registry;
use crate::style;
use crate::toolbar::Toolbar;
use dioxus::prelude::*;
use ds::{Ds, HeaderKind, SectionHeader};

/// The gallery, starting from the axes this thread was handed (`crate::axes::start_with`).
#[allow(non_snake_case)] // A component: launch and snapshot name it like a type.
pub fn App() -> Element {
    let axes = use_signal(starting);
    use_context_provider(|| axes);
    // `Ds` reads `appearance.material_tint_alpha` from this context; the materials page moves it.
    use_context_provider(|| Signal::new(axes.peek().tint_alpha));
    let now = axes();
    rsx! {
        Ds {
            appearance: now.appearance(),
            look: now.look.clone(),
            material: now.material,
            blur: now.blur,
            style { {style::CSS} }
            div { class: "g-app",
                Toolbar {}
                div { class: "g-card",
                    PageBody { key: "{now.page.slug()}", page: now.page }
                }
            }
        }
    }
}

/// One page: its title, what it shows, and its body.
#[component]
fn PageBody(page: Page) -> Element {
    let entry = registry::entry(page);
    #[allow(non_snake_case)] // Rendered as a component.
    let Body = entry.body;
    rsx! {
        SectionHeader { kind: HeaderKind::Frame, text: entry.title }
        p { class: "g-lede", "{entry.lede}" }
        Body {}
    }
}
