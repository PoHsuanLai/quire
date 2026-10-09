//! How a glyph is drawn: solid, or as an outline (design/08-ICONS.md section 1.2).
//!
//! What an icon draws when nothing asks is the owner's per-icon pick, `Icon::native`
//! (`native.rs`). The four pairs macOS states with a hollow glyph (a star, a heart, a pin, a
//! bell) are outline while off and [`PAIR_ON`] while on.

use super::Icon;
use super::native::Native;
use super::shape::Shape;
use ds_core::vocab::Check;

/// Which geometry of an [`Icon`] to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlyphStyle {
    /// The icon's own pick (`Icon::native`).
    #[default]
    Native,
    /// Filled paths in `currentColor`, no stroke.
    Solid,
    /// Lucide's 2 px stroke, `fill="none"`.
    Outline,
}

/// How the on half of a pair is drawn: outline, like the off half (owner, 2026-10-09), so a star,
/// heart, pin or bell is outline in both states and only its colour marks the on state. Set it to
/// `Solid` to fill the on half again.
pub const PAIR_ON: GlyphStyle = GlyphStyle::Outline;

impl GlyphStyle {
    /// The paint `icon` draws with when it is asked for `asked`: solid or outline, never
    /// `Native`. An explicit ask wins; `Native` reads the icon's pick, and a pair, with no state
    /// given, is its off half.
    pub fn resolve(icon: Icon, asked: GlyphStyle) -> GlyphStyle {
        match (asked, icon.native()) {
            (GlyphStyle::Native, Native::Solid | Native::Drawn(_)) => GlyphStyle::Solid,
            (GlyphStyle::Native, Native::Outline) => GlyphStyle::Outline,
            (GlyphStyle::Native, Native::Pair) => GlyphStyle::of(Check::Off),
            (asked, _) => asked,
        }
    }

    /// The style `icon` asks for when the control that holds it is in `state`: a pair's half for
    /// a pair, else its own pick.
    pub fn for_state(icon: Icon, state: Check) -> GlyphStyle {
        match icon.native() {
            Native::Pair => GlyphStyle::of(state),
            Native::Solid | Native::Outline | Native::Drawn(_) => GlyphStyle::Native,
        }
    }

    /// The style of a pair's half: [`PAIR_ON`] when it is on (outline today), outline when it is
    /// off or mixed.
    pub fn of(state: Check) -> GlyphStyle {
        match state {
            Check::On => PAIR_ON,
            Check::Off | Check::Mixed => GlyphStyle::Outline,
        }
    }
}

impl Icon {
    /// The children of this glyph when asked for `style`.
    pub fn shapes_in(self, style: GlyphStyle) -> &'static [Shape] {
        match (style, self.native()) {
            (GlyphStyle::Native, Native::Drawn(shapes)) => shapes,
            _ => match GlyphStyle::resolve(self, style) {
                GlyphStyle::Outline => self.shapes(),
                GlyphStyle::Solid | GlyphStyle::Native => self.solid_shapes(),
            },
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
    fn every_icon_has_both_geometries() {
        for &icon in Icon::ALL {
            assert!(!icon.shapes().is_empty(), "{icon:?}: no outline form");
            assert!(!icon.solid_shapes().is_empty(), "{icon:?}: no solid form");
        }
    }

    #[test]
    fn the_table_covers_every_icon_once() {
        use super::super::native::TABLE;
        assert_eq!(TABLE.len(), Icon::ALL.len());
        for &icon in Icon::ALL {
            assert_eq!(
                TABLE.iter().filter(|(i, _)| *i == icon).count(),
                1,
                "{icon:?}"
            );
        }
    }

    #[test]
    fn the_owner_picks_are_the_defaults() {
        let count = |wanted: Native| Icon::ALL.iter().filter(|i| i.native() == wanted).count();
        assert_eq!(count(Native::Solid), 16);
        assert_eq!(count(Native::Pair), 4);
        assert_eq!(
            Icon::ALL
                .iter()
                .filter(|i| matches!(i.native(), Native::Drawn(_)))
                .count(),
            3
        );
        assert_eq!(GlyphStyle::default(), GlyphStyle::Native);
        for solid in [Icon::Battery, Icon::ChevronDown, Icon::Play, Icon::Terminal] {
            assert_eq!(
                GlyphStyle::resolve(solid, GlyphStyle::Native),
                GlyphStyle::Solid
            );
        }
        for outline in [Icon::Trash, Icon::Bluetooth, Icon::Wifi, Icon::LogOut] {
            assert_eq!(
                GlyphStyle::resolve(outline, GlyphStyle::Native),
                GlyphStyle::Outline
            );
        }
    }

    #[test]
    fn a_pair_is_outline_off_and_follows_pair_on_when_on() {
        for icon in [Icon::Star, Icon::Heart, Icon::Pin, Icon::Bell] {
            assert_eq!(GlyphStyle::for_state(icon, Check::Off), GlyphStyle::Outline);
            assert_eq!(GlyphStyle::for_state(icon, Check::On), PAIR_ON);
            assert_eq!(
                GlyphStyle::resolve(icon, GlyphStyle::Native),
                GlyphStyle::Outline
            );
        }
        assert_eq!(
            GlyphStyle::for_state(Icon::Trash, Check::On),
            GlyphStyle::Native
        );
    }

    #[test]
    fn an_explicit_style_is_honoured() {
        assert_eq!(
            GlyphStyle::resolve(Icon::Bluetooth, GlyphStyle::Solid),
            GlyphStyle::Solid
        );
        assert_eq!(
            GlyphStyle::resolve(Icon::Play, GlyphStyle::Outline),
            GlyphStyle::Outline
        );
        assert_eq!(
            Icon::Download.shapes_in(GlyphStyle::Solid),
            Icon::Download.solid_shapes()
        );
        assert_eq!(
            Icon::Download.shapes_in(GlyphStyle::Outline),
            Icon::Download.shapes()
        );
    }
}
