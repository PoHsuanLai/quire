//! One `.ds[data-material=…]` block per material and scheme, and the `data-blur` switch
//! between `--m-tint` and `--m-tint-solid`.
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

/// The material recipes. The tint alpha over blur is a settings key
/// (`appearance.material_tint_alpha`), so the stylesheet reads it from a variable the root
/// writes inline rather than baking in `.80`.
pub fn materials_css() -> String {
    todo!()
}
