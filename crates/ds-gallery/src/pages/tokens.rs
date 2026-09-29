//! Tokens: every colour with its name and hex in both schemes, the six accents, the label hues,
//! the spacing scale, radii, shadows and z layers.

use super::{Caption, Scope, Section, Specimen};
use dioxus::prelude::*;
use ds::Word;
use ds::{
    Accent, ColourToken, HueMember, LabelHue, Material, Radius, Scheme, Shadow, Surface, ZLayer,
    accent_of,
};

/// design/01-LAYOUT.md section 2's common steps. The design names these values and quire has no
/// token for them (a gap the Gaps page lists): every padding is a raw length today.
const SPACING: [u16; 18] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 16, 18, 22, 26, 36,
];

/// The tokens page.
#[component]
pub fn TokensPage() -> Element {
    rsx! {
        for scheme in Scheme::ALL.iter().copied() {
            Section { title: format!("Colour, {}", scheme.slug()),
                Surface { material: Material::Window, theme: Some(scheme),
                    div { class: "g-card",
                        div { class: "g-grid6",
                            for token in ColourToken::ALL {
                                Specimen {
                                    name: token.var().as_str(),
                                    code: token.value(scheme).css(),
                                    div { class: "g-swatch", style: "background:{token.var().reference()}" }
                                }
                            }
                        }
                    }
                }
            }
        }
        Section {
            title: "Accents",
            note: "The four properties of each accent, light then dark. The card's own accent follows the toolbar.",
            div { class: "g-grid6",
                for accent in Accent::ALL.iter().copied() {
                    AccentRoles { accent }
                }
            }
        }
        Section { title: "Label hues", note: "--c-<hue>, -deep and -soft, as the root's scheme paints them.",
            div { class: "g-grid6",
                for hue in LabelHue::ALL.iter().copied() {
                    for member in [HueMember::Base, HueMember::Deep, HueMember::Soft] {
                        Specimen { name: hue.var(member), code: None,
                            div { class: "g-swatch", style: "background:var({hue.var(member)})" }
                        }
                    }
                }
            }
        }
        Section {
            title: "Spacing scale",
            note: "design/01-LAYOUT.md section 2. These are raw lengths: quire has no spacing token.",
            div { class: "g-col",
                for step in SPACING {
                    div { class: "g-type-line",
                        span { class: "g-code g-type-name", "{step}px" }
                        div { class: "g-space-bar", style: "width:{step * 8}px" }
                    }
                }
            }
        }
        Section { title: "Radii",
            div { class: "g-row g-row-top",
                for radius in Radius::ALL {
                    Specimen { name: radius.var().as_str(), code: radius.css(),
                        div { class: "g-radius", style: "border-radius:{radius.var().reference()}" }
                    }
                }
            }
        }
        Section { title: "Shadows",
            div { class: "g-row g-row-top",
                for shadow in Shadow::ALL {
                    Specimen { name: shadow.var().as_str(), code: None,
                        div { class: "g-shadow", style: "box-shadow:{shadow.var().reference()}" }
                    }
                }
            }
        }
        Section { title: "Layers",
            div { class: "g-grid8",
                for layer in ZLayer::ALL {
                    Caption { name: layer.var().as_str(), code: format!("z-index {}", layer.z()) }
                }
            }
        }
    }
}

/// One accent's six properties, in both schemes.
#[component]
fn AccentRoles(accent: Accent) -> Element {
    rsx! {
        div { class: "g-col",
            span { class: "g-name", "{accent.label()}" }
            for scheme in Scheme::ALL.iter().copied() {
                Scope { scheme, accent, material: Material::Popover, frame: Some(ds::FrameTint::None),
                    div { class: "g-cell",
                        div { class: "g-row",
                            for (name , hex) in role_parts(accent, scheme) {
                                div {
                                    class: "g-chip-swatch",
                                    title: "{name} {hex}",
                                    style: "background:var({name})",
                                }
                            }
                        }
                        span { class: "g-code", "{accent_of(accent, scheme).fill.css()}" }
                    }
                }
            }
        }
    }
}

/// The accent's roles as `(name, value)`.
fn role_parts(accent: Accent, scheme: Scheme) -> [(&'static str, String); 6] {
    let roles = accent_of(accent, scheme);
    [
        ("--accent", roles.fill.css()),
        ("--accent-soft", roles.wash_colour().css()),
        ("--accent-text", roles.text.css()),
        ("--accent-text-material", roles.text_material.css()),
        ("--accent-ring", roles.ring_colour().css()),
        ("--accent-ink", roles.ink.css()),
    ]
}
