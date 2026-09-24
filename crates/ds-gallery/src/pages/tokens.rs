//! Tokens: every colour with its name and hex in both schemes, the six accents, the label hues,
//! the spacing scale, radii, shadows and z layers.

use super::{Caption, Scope, Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Accent, ColourToken, HueMember, LabelHue, Material, Radius, Scheme, Shadow, Surface, ZLayer,
    quad,
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
        for scheme in Scheme::ALL {
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
                for accent in Accent::ALL {
                    AccentQuad { accent }
                }
            }
        }
        Section { title: "Label hues", note: "--c-<hue>, -deep and -soft, as the root's scheme paints them.",
            div { class: "g-grid6",
                for hue in LabelHue::ALL {
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

/// One accent's four properties, in both schemes.
#[component]
fn AccentQuad(accent: Accent) -> Element {
    rsx! {
        div { class: "g-col",
            span { class: "g-name", "{accent.label()}" }
            for scheme in Scheme::ALL {
                Scope { scheme, accent, material: Material::Popover, frame: Some(ds::FrameTint::None),
                    div { class: "g-cell",
                        div { class: "g-row",
                            for (name , hex) in quad_parts(accent, scheme) {
                                div {
                                    class: "g-chip-swatch",
                                    title: "{name} {hex}",
                                    style: "background:var({name})",
                                }
                            }
                        }
                        span { class: "g-code", "{quad(accent, scheme).accent.css()}" }
                    }
                }
            }
        }
    }
}

/// The accent quad as `(name, hex)`.
fn quad_parts(accent: Accent, scheme: Scheme) -> [(&'static str, String); 4] {
    let quad = quad(accent, scheme);
    [
        ("--accent", quad.accent.css()),
        ("--accent-soft", quad.soft.css()),
        ("--accent-ink", quad.ink.css()),
        ("--seal", quad.seal.css()),
    ]
}
