//! The Forms page (design/34-MODERN-LOOK.md sections 3.5 and 4): a `Form` of `FormSection`s the
//! way System Settings draws a pane, from the grouped list, the icon tile and the field rows.

pub(crate) mod grouped;

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::fields::field_row::FieldRow;
use ds::prelude::*;
use ds::style::tokens::hex::Hex;

/// The Forms page.
#[component]
pub fn FormsPage() -> Element {
    rsx! {
        Section {
            title: "Form",
            note: "Form stacks FormSections 20 apart inside a 20 margin. A section is a title (13/600, secondary), one rounded group on the grouped ground with no outline, and a footer of help text. The rows' separator starts at their text, past the tile.",
            div { class: "g-stage-pad", style: "width:460px;background:var(--surface-2)",
                Form {
                    FormSection { title: "Connectivity", footer: "Known networks are joined automatically; the others ask first.",
                        grouped::Panes {}
                    }
                    FormSection { title: "General",
                        FieldRow { label: TextLine::from("Wi-Fi"), help: Some(TextLine::from("Join known networks automatically.")),
                            Toggle { label: "Wi-Fi", value: Check::On, onchange: |_| {} }
                        }
                        FieldRow { label: TextLine::from("Airplane mode"),
                            Toggle { label: "Airplane mode", value: Check::Off, onchange: |_| {} }
                        }
                    }
                }
            }
        }
        Section {
            title: "IconTile",
            note: "A 24 px rounded tile: the caller's colour under a gradient from a lighter step of it, a white glyph, no outline. The other face is the circular Avatar a row of people leads with.",
            div { class: "g-row",
                IconTile { icon: Icon::Wifi, colour: Hex([0x0a, 0x84, 0xff]) }
                IconTile { icon: Icon::Bell, colour: Hex([0xff, 0x3b, 0x30]) }
                IconTile { icon: Icon::Speaker, colour: Hex([0xff, 0x9f, 0x0a]) }
                IconTile { icon: Icon::Lock, colour: Hex([0x8e, 0x8e, 0x93]) }
                IconTile { icon: Icon::Bluetooth, colour: Hex([0x30, 0xb0, 0x50]) }
            }
        }
    }
}
