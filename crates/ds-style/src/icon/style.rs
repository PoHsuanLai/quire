//! How a glyph is drawn: solid, or as an outline (design/08-ICONS.md section 1.2).
//!
//! Solid is the default everywhere. Outline is the "off" half of a pair that states on and off
//! (a star, a heart, a bookmark, a pin, a flag, a bell), the way macOS draws them.

use super::Icon;
use super::shape::Shape;
use ds_core::vocab::Check;

/// Which geometry of an [`Icon`] to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlyphStyle {
    /// Filled paths in `currentColor`, no stroke.
    #[default]
    Solid,
    /// Lucide's 2 px stroke, `fill="none"`: the off state of a pair.
    Outline,
}

impl Icon {
    /// The glyphs whose off state is their outline, the pairs macOS draws that way: a star, a
    /// heart, a pin, a bell (design/08-ICONS.md section 1.2). Every other glyph is solid in
    /// both states, its state carried by colour or by the control around it.
    pub const OFF_IS_OUTLINE: &[Icon] = &[Icon::Star, Icon::Heart, Icon::Pin, Icon::Bell];

    /// The glyphs drawn as an outline in both states, a stated per-icon override: the owner
    /// prefers the stroked rune (design/08-ICONS.md section 1.2).
    pub const ALWAYS_OUTLINE: &[Icon] = &[
        Icon::Bluetooth,
        Icon::BluetoothConnected,
        Icon::BluetoothOff,
    ];
}

impl GlyphStyle {
    /// The style `icon` draws in whatever it is asked for: outline for a glyph of
    /// [`Icon::ALWAYS_OUTLINE`], else `asked`.
    pub fn resolve(icon: Icon, asked: GlyphStyle) -> GlyphStyle {
        match Icon::ALWAYS_OUTLINE.contains(&icon) {
            true => GlyphStyle::Outline,
            false => asked,
        }
    }

    /// The style `icon` draws in when the control that holds it is in `state`: for a glyph of
    /// [`Icon::OFF_IS_OUTLINE`] what [`GlyphStyle::of`] says, else solid.
    pub fn for_state(icon: Icon, state: Check) -> GlyphStyle {
        match Icon::OFF_IS_OUTLINE.contains(&icon) {
            true => GlyphStyle::of(state),
            false => GlyphStyle::resolve(icon, GlyphStyle::Solid),
        }
    }

    /// The style of a pair's half: solid when it is on, outline when it is off or mixed.
    pub fn of(state: Check) -> GlyphStyle {
        match state {
            Check::On => GlyphStyle::Solid,
            Check::Off | Check::Mixed => GlyphStyle::Outline,
        }
    }
}

impl Icon {
    /// The children of this glyph in `style`.
    pub fn shapes_in(self, style: GlyphStyle) -> &'static [Shape] {
        match style {
            GlyphStyle::Solid => self.solid_shapes(),
            GlyphStyle::Outline => self.shapes(),
        }
    }

    /// The solid form: filled paths on the 24 grid, never empty.
    pub fn solid_shapes(self) -> &'static [Shape] {
        [
            super::solid_fan::shapes,
            super::solid_mailo::shapes,
            super::solid_shell::shapes,
            super::solid_more::shapes,
        ]
        .iter()
        .map(|table| table(self))
        .find(|shapes| !shapes.is_empty())
        .unwrap_or(&[])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_solid_filled_geometry() {
        for &icon in Icon::ALL {
            let solid = icon.solid_shapes();
            assert!(!solid.is_empty(), "{icon:?}: no solid form");
            for shape in solid {
                assert!(
                    matches!(shape, Shape::Solid(d) if d.len() > 8),
                    "{icon:?}: {shape:?}"
                );
            }
        }
    }

    #[test]
    fn solid_is_the_default() {
        assert_eq!(GlyphStyle::default(), GlyphStyle::Solid);
        assert_eq!(GlyphStyle::of(Check::Off), GlyphStyle::Outline);
        assert_eq!(
            GlyphStyle::for_state(Icon::Star, Check::Off),
            GlyphStyle::Outline
        );
        assert_eq!(
            GlyphStyle::for_state(Icon::Star, Check::On),
            GlyphStyle::Solid
        );
        assert_eq!(
            GlyphStyle::for_state(Icon::Trash, Check::Off),
            GlyphStyle::Solid
        );
        for state in [Check::On, Check::Off] {
            assert_eq!(
                GlyphStyle::for_state(Icon::Bluetooth, state),
                GlyphStyle::Outline
            );
        }
        assert_eq!(
            GlyphStyle::resolve(Icon::Bluetooth, GlyphStyle::Solid),
            GlyphStyle::Outline
        );
    }
}
