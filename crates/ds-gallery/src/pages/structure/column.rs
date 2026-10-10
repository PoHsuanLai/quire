//! Column: the stepped gap, no gap, and the fill and fixed extents.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds::style::tokens::spacing::SpacingToken;

/// The Column section.
#[component]
pub fn ColumnSection() -> Element {
    rsx! {
        Section { title: "Column", note: "A vertical stack. The gap is a spacing step or none; the extent is the content's height, the parent's whole box, or a control's height.",
            div { class: "g-row",
                div { class: "g-stage",
                    Column { gap: SpacingToken::S16,
                        span { "Stepped" }
                        span { "gap" }
                    }
                }
                div { class: "g-stage",
                    Column { gap: ColumnGap::None,
                        span { "No" }
                        span { "gap" }
                    }
                }
                div { class: "g-stage", style: "height:120px",
                    Column { extent: ColumnExtent::Fill, gap: ColumnGap::None,
                        span { "Fills the stage" }
                    }
                }
                div { class: "g-stage",
                    Column { extent: ColumnExtent::Fixed(ControlSize::Large),
                        span { "One large control tall" }
                    }
                }
            }
        }
    }
}
