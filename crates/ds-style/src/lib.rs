//! The design system's appearance: the token table and its `Token` trait, appearance choices and
//! their resolution, materials, Space palettes, fonts, icons, the stylesheet's own sections and
//! the `Kit` seam other crates add theirs through, and the tasks a scope owns (`task`, `busy`),
//! over `ds-core` and nothing else of ours. Feature `dioxus` adds what needs Dioxus (`scope`,
//! `scale`, `task`, `busy`, `icon::render::Glyph`); without it the crate is plain data.

// `#[derive(Token)]` names `::ds_style::tokens`, so inside this crate `ds_style` is the crate
// itself.
extern crate self as ds_style;

pub mod appearance;
#[cfg(feature = "dioxus")]
pub mod busy;
pub mod css;
pub mod emit;
pub mod fonts;
pub mod icon;
pub mod kit;
pub mod material;
#[cfg(feature = "dioxus")]
pub mod scale;
#[cfg(feature = "dioxus")]
pub mod scope;
pub mod space;
#[cfg(test)]
mod stored_words;
#[cfg(feature = "dioxus")]
pub mod task;
pub mod tokens;
