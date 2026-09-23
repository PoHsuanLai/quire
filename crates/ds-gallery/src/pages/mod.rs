//! The pages, one file each, and the pieces they share.

pub mod controls;
pub mod gaps;
pub mod lists;
pub mod materials;
pub mod matrix;
pub mod motion;
pub mod motion_lab;
pub mod overlays;
pub mod pills;
pub mod space;
pub mod tokens;
pub mod type_ramp;

use crate::axes::Axes;
use dioxus::prelude::*;
use ds::{
    Accent, Appearance, BlurState, Ds, HeaderKind, Inject, Material, Scheme, SectionHeader,
    SpaceLook, Theme,
};

/// A titled group of specimens, with an optional note under the title.
#[component]
pub fn Section(
    title: String,
    #[props(default)] note: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        section { class: "g-section",
            SectionHeader { kind: HeaderKind::Group, text: title }
            if let Some(note) = note {
                p { class: "g-note", "{note}" }
            }
            {children}
        }
    }
}

/// A specimen's caption: its name, and the code or value beside it.
#[component]
pub fn Caption(name: String, #[props(default)] code: Option<String>) -> Element {
    rsx! {
        div { class: "g-col",
            span { class: "g-name", "{name}" }
            if let Some(code) = code {
                span { class: "g-code", "{code}" }
            }
        }
    }
}

/// A specimen above its caption.
#[component]
pub fn Specimen(
    name: String,
    #[props(default)] code: Option<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "g-col",
            {children}
            Caption { name, code }
        }
    }
}

/// A nested root in another scheme, accent, material or blur state.
///
/// `Surface` can override only the material and the scheme, so a specimen that needs another
/// accent or blur state is drawn in a nested `Ds` that shares the page's stylesheet (a gap the
/// Gaps page lists). It keeps the gallery's motion level and Space.
#[component]
pub fn Scope(
    scheme: Scheme,
    accent: Accent,
    material: Material,
    #[props(default)] blur: BlurState,
    children: Element,
) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (motion, look) = {
        let axes = axes.read();
        (axes.motion, axes.look.clone())
    };
    let theme = match scheme {
        Scheme::Light => Theme::Light,
        Scheme::Dark => Theme::Dark,
    };
    rsx! {
        Ds {
            appearance: Appearance { theme, accent, motion },
            look: SpaceLook { theme, ..look },
            material,
            blur,
            stylesheet: Inject::Host,
            {children}
        }
    }
}
