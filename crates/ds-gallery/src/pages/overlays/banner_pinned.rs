//! BannerActionRow (a banner asking for text) and PinnedBar (an offer bar over a scroller's edge).

use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;

/// The BannerActionRow and PinnedBar sections.
#[component]
pub fn BannerAndPinned() -> Element {
    let mut name = use_signal(|| "Work".to_owned());
    rsx! {
        Section { title: "BannerActionRow", note: "BannerActionRow {{ label, field, children }}: a banner's text field, which takes the width left, and the buttons that act on it. It wraps the buttons under the field when the pane is narrow.",
            div { class: "g-row g-row-top",
                Specimen { name: "In a banner".to_string(),
                    div { style: "width:480px",
                        InlineBanner {
                            text: "Name this space",
                            actions: rsx! {
                                BannerActionRow {
                                    label: "Name the space",
                                    field: rsx! {
                                        TextField { label: "Name", value: name(), size: ControlSize::Small, oninput: move |next| name.set(next) }
                                    },
                                    Button { label: "Save", size: ControlSize::Small, onclick: |_| {} }
                                    Button { label: "Cancel", size: ControlSize::Small, onclick: |_| {} }
                                }
                            },
                        }
                    }
                }
            }
        }
        Section { title: "PinnedBar", note: "PinnedBar {{ label, bar, edge }}: an offer bar pinned over the top or bottom edge of a scroller. The scroller scrolls beneath it; the bar is outside the scroller because Blitz has no position: sticky.",
            div { class: "g-row g-row-top",
                for (edge , title) in [(PinEdge::Top, "Top"), (PinEdge::Bottom, "Bottom")] {
                    Specimen { name: title.to_string(),
                        div { style: "width:300px;height:180px;display:flex",
                            PinnedBar {
                                label: "Suggestion",
                                edge,
                                bar: rsx! { span { "Move to Receipts?" } },
                                div { style: "flex:1 1 auto;overflow:auto;min-height:0",
                                    for row in 0..12 {
                                        p { key: "{row}", "Row {row}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
