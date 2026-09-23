//! What Blitz does not paint, for [`super::Rule::BlitzUnsupported`] (the plan's Blitz risk
//! table).
#![allow(unused_variables)] // Freeze stubs: remove with the last todo!().

/// Why `property: value` will not paint on Blitz, or `None` when it will.
pub fn unsupported(property: &str, value: &str) -> Option<&'static str> {
    todo!()
}
