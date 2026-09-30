//! The Controls page's mail-app buttons: words on the Space frame, a dropdown value with
//! its caret, and the selection bubble's B, i, U and S faces. Split from `controls.rs` to keep
//! that page under its size.

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{Bezel, ControlSize};
use ds::{Button, ButtonFace, Check, Icon, Shown, Trailing};

/// The four mark faces with their labels and shortcuts, as the bubble titles them.
const FACES: [(ButtonFace, &str, &str); 4] = [
    (ButtonFace::Bold, "Bold", "Bold (Ctrl B)"),
    (ButtonFace::Italic, "Italic", "Italic (Ctrl I)"),
    (ButtonFace::Underline, "Underline", "Underline (Ctrl U)"),
    (
        ButtonFace::Strike,
        "Strikethrough",
        "Strikethrough (Ctrl Shift S)",
    ),
];

/// Flip a switch.
fn flip(state: Check) -> Check {
    state.flipped()
}

#[component]
pub fn ButtonFaces() -> Element {
    let mut marks = use_signal(|| [Check::On, Check::Off, Check::Off, Check::Off]);
    let mut open = use_signal(|| Shown::Hidden);
    rsx! {
        Section { title: "Button: frame words, a trailing caret, mark faces", note: "Frame: words on the Space frame in the sidebar item's chrome (hover it; the second is pressed). Trailing: a dropdown's value and its caret, and a glyph after a label. Faces: the bubble's marks, each named by its label (click to toggle).",
            div { class: "g-row g-row-top",
                Specimen { name: "frame",
                    div { class: "g-side",
                        Button { bezel: Bezel::Toolbar, label: "Settings", icon: Some(Icon::Settings), onclick: |_| {} }
                        Button { bezel: Bezel::Toolbar, label: "Today", value: Some(Check::On), onclick: |_| {} }
                        Button { bezel: Bezel::Toolbar, label: "All mail", trailing: Trailing::Caret, onclick: |_| {} }
                    }
                }
                Specimen { name: "trailing",
                    div { class: "g-row",
                        Button { bezel: Bezel::Inline, label: "poh@acme.example", trailing: Trailing::Caret, shown: open(),
                            onclick: move |_| open.set(match open() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }) }
                        Button { size: ControlSize::Mini, label: "Sends: Now", trailing: Trailing::Caret, onclick: |_| {} }
                        Button { label: "Open", trailing: Trailing::Glyph(Icon::Link), onclick: |_| {} }
                    }
                }
                Specimen { name: "faces",
                    div { class: "g-row",
                        for (index , (face , label , title)) in FACES.into_iter().enumerate() {
                            Button { bezel: Bezel::Inline, label, face, title, value: Some(marks()[index]),
                                onclick: move |_| marks.with_mut(|all| all[index] = flip(all[index])) }
                        }
                    }
                }
            }
        }
    }
}
