//! What a root paints on its own box, and which inks its content is drawn in: three decisions
//! derived from the material, each with a documented rule and an explicit override.
//!
//! - [`RootChrome`]: whether the root box paints its material (tint, edge, shadow) at all. A
//!   root that only hosts overlays (a shell popup holding a menu) is a zero-height, full-width
//!   box; painting the material there drew a shadow band across the popup (sill FINDINGS F53,
//!   Q13). The card paints itself instead.
//! - [`FrameTint`]: whether the root draws the Space gradient, and how: opaque with its grain
//!   on a window, at the material's tint alpha on shell chrome (design/21-SPACES.md sections 3
//!   and 5; sill FINDINGS F48, Q9).
//! - [`Ground`]: whether components are drawn on paper (the Post inks) or on the frame colour
//!   (the `--f-*` inks, design/03-COLOR.md section 4 and design/04-COMPONENTS.md's sidebar
//!   item; sill FINDINGS F49, Q12).

use crate::material::Material;

/// Whether a root paints its material on its own box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RootChrome {
    /// The root box paints its material: tint (and the Space gradient where the material takes
    /// one), edge and shadow.
    Painted,
    /// The root box paints nothing; the surfaces it hosts (a menu, a popover, a sheet) paint
    /// the material's tint, edge and shadow on their own cards.
    Transparent,
}

impl RootChrome {
    /// The rule: a root in `Popover`, `Sheet` or `Toast` hosts floating cards (a shell popup,
    /// a panel, a banner), so it is transparent and each card paints itself; every other
    /// material paints its root box. A root that is itself the panel (the launcher's panel,
    /// a Popover root holding content) passes `Painted`.
    pub fn of(material: Material) -> Self {
        match material {
            Material::Popover | Material::Sheet | Material::Toast => RootChrome::Transparent,
            Material::Window
            | Material::Bar
            | Material::Dock
            | Material::Osd
            | Material::Widget => RootChrome::Painted,
        }
    }

    /// The `data-chrome` value, written only for a transparent root.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            RootChrome::Painted => None,
            RootChrome::Transparent => Some("transparent"),
        }
    }
}

/// How a painted root draws the Space gradient.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameTint {
    /// The window's frame: the gradient opaque, its two layers and grain (design/03-COLOR.md
    /// sections 4.5 and 8).
    Opaque,
    /// Shell chrome: the gradient, its layers and grain as one group at the material's tint
    /// alpha over compositor blur (`data-blur=on`), or at the solid floor .94 without it
    /// (design/21-SPACES.md section 3).
    Tinted,
    /// No gradient: the material's own tint.
    None,
}

impl FrameTint {
    /// The rule: the window paints the frame opaque; the bar, the dock, a popover panel (the
    /// launcher, the control center), the OSD and a widget paint it tinted; a sheet and a
    /// toast keep their own tint (design/21 section 3's "no" rows). A transparent root paints
    /// nothing.
    pub fn of(material: Material, chrome: RootChrome) -> Self {
        match (chrome, material) {
            (RootChrome::Transparent, _) => FrameTint::None,
            (RootChrome::Painted, Material::Window) => FrameTint::Opaque,
            (
                RootChrome::Painted,
                Material::Bar
                | Material::Dock
                | Material::Popover
                | Material::Osd
                | Material::Widget,
            ) => FrameTint::Tinted,
            (RootChrome::Painted, Material::Sheet | Material::Toast) => FrameTint::None,
        }
    }

    /// The `data-frame` value, written only for a tinted root.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            FrameTint::Tinted => Some("tinted"),
            FrameTint::Opaque | FrameTint::None => None,
        }
    }
}

/// What the content of a scope is drawn on, and so which inks it takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ground {
    /// Paper: the Post inks and surfaces (`--ink*`, `--surface*`, `--line`).
    Paper,
    /// The frame colour: every component under it reads the `--f-*` inks and fills instead
    /// (`--ink*` are `--f-ink*`, `--surface` is `--f-pill-hover`, `--surface-2` is `--f-pill`,
    /// `--line` is `--f-line`), as the sidebar item does by hand. Overlays opened from it are
    /// paper again.
    Frame,
}

impl Ground {
    /// The rule: the bar and the dock are chrome on the frame colour (design/21-SPACES.md
    /// section 3), so their content takes the frame inks; everything else is paper.
    pub fn of(material: Material) -> Self {
        match material {
            Material::Bar | Material::Dock => Ground::Frame,
            Material::Window
            | Material::Popover
            | Material::Sheet
            | Material::Toast
            | Material::Osd
            | Material::Widget => Ground::Paper,
        }
    }

    /// The `data-ground` value, written only on the frame.
    pub(crate) fn attribute(self) -> Option<&'static str> {
        match self {
            Ground::Paper => None,
            Ground::Frame => Some("frame"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameTint, Ground, RootChrome};
    use crate::material::Material;

    #[test]
    fn each_material_has_its_chrome_tint_and_ground() {
        use FrameTint as F;
        use Ground as G;
        use RootChrome as C;
        #[rustfmt::skip]
        const CASES: &[(Material, RootChrome, FrameTint, Ground)] = &[
            (Material::Window, C::Painted, F::Opaque, G::Paper),
            (Material::Bar, C::Painted, F::Tinted, G::Frame),
            (Material::Dock, C::Painted, F::Tinted, G::Frame),
            (Material::Popover, C::Transparent, F::None, G::Paper),
            (Material::Sheet, C::Transparent, F::None, G::Paper),
            (Material::Toast, C::Transparent, F::None, G::Paper),
            (Material::Osd, C::Painted, F::Tinted, G::Paper),
            (Material::Widget, C::Painted, F::Tinted, G::Paper),
        ];
        for &(material, chrome, tint, ground) in CASES {
            assert_eq!(RootChrome::of(material), chrome, "{material:?}");
            assert_eq!(FrameTint::of(material, chrome), tint, "{material:?}");
            assert_eq!(Ground::of(material), ground, "{material:?}");
        }
    }

    #[test]
    fn a_painted_popover_takes_the_tinted_frame() {
        assert_eq!(
            FrameTint::of(Material::Popover, RootChrome::Painted),
            FrameTint::Tinted
        );
        assert_eq!(
            FrameTint::of(Material::Window, RootChrome::Transparent),
            FrameTint::None
        );
    }
}
