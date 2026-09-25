//! ModuleGrid: the control center's grid of module tiles, two columns with a gap of 8 (design/13
//! section 13.3.7). A `ModuleTile { span: TileSpan::Full }` inside it takes both columns, as a
//! slider module does; the panel around it owns the padding (sill FINDINGS Q78).

use dioxus::prelude::*;

/// Two columns of tiles; `children` are `ModuleTile`s or full-width modules of the caller's.
#[component]
pub fn ModuleGrid(children: Element) -> Element {
    rsx! {
        div { class: "ds-module-grid", {children} }
    }
}
