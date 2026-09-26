//! Where the gallery is on each of its axes: the page, and the theme, accent, motion level,
//! material, blur and Space the root is drawn with.
//!
//! The window and each snapshot start from an [`Axes`] handed over through [`start_with`]:
//! `ds_native::launch` and `ds_native::snapshot_at` take a plain `fn() -> Element`, so the root
//! reads its first state from this thread rather than from props.

use crate::page::Page;
use ds::tokens::Alpha;
use ds::{
    Accent, Appearance, BlurState, CardAccent, Grain, Material, Motion, MotionLevel, SpaceLook,
    Theme, Typeface, default_look,
};
use ds_settings::AppearanceSettings;
use std::cell::RefCell;

/// One of the eight preset Spaces, by its place in `ds::PRESETS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PresetIndex(pub u8);

impl PresetIndex {
    /// Every preset, in the editor's order.
    pub fn all() -> impl Iterator<Item = PresetIndex> {
        (0..ds::PRESETS.len()).map(|index| PresetIndex(u8::try_from(index).unwrap_or(u8::MAX)))
    }

    /// What the toolbar calls it: the two sample Spaces by name, the rest by number.
    pub fn label(self) -> String {
        match self.0 {
            0 => "Work".to_owned(),
            1 => "Home".to_owned(),
            n => format!("Preset {}", n + 1),
        }
    }

    /// The look this preset ships as: its dots and grain, the system theme, Postmark.
    pub fn look(self) -> SpaceLook {
        default_look(usize::from(self.0), Grain::default(), CardAccent::Postmark)
    }
}

/// Everything the toolbar chooses.
#[derive(Debug, Clone, PartialEq)]
pub struct Axes {
    /// The page on show.
    pub page: Page,
    /// Light, dark or the desktop's.
    pub theme: Theme,
    /// One of the six accents.
    pub accent: Accent,
    /// The motion preference; the toolbar offers the four levels.
    pub motion: Motion,
    /// The root's material.
    pub material: Material,
    /// Whether the root paints its translucent tint (`data-blur=on`) or the solid one.
    pub blur: BlurState,
    /// The preset the Space started from.
    pub preset: PresetIndex,
    /// The Space the root is drawn with: the preset, as the Space editor has changed it.
    pub look: SpaceLook,
    /// `appearance.material_tint_alpha`, as the root writes it.
    pub tint_alpha: Alpha,
    /// Whether overlays wait for a click or open posed, for a snapshot.
    pub showcase: Showcase,
    /// The typeface the root speaks in.
    pub typeface: Typeface,
}

/// How the pages that open things on demand start.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Showcase {
    /// Closed until clicked: the window.
    #[default]
    Live,
    /// Opened on the first frame at fixed points: a snapshot, which runs no clicks.
    Posed,
}

impl Default for Axes {
    /// The settings file's defaults (design/22-SETTINGS.md section 3.1) on the tokens page, in
    /// the window material over the Work Space. The gallery shows the defaults, never the
    /// person's own file.
    fn default() -> Self {
        let settings = AppearanceSettings::default();
        Axes {
            page: Page::Tokens,
            theme: settings.theme,
            accent: settings.accent,
            motion: settings.motion_level,
            material: Material::Window,
            blur: BlurState::default(),
            preset: PresetIndex::default(),
            look: PresetIndex::default().look(),
            tint_alpha: Alpha(u16::from(settings.material_tint_alpha.0) * 10),
            showcase: Showcase::Live,
            typeface: settings.typeface,
        }
    }
}

impl Axes {
    /// The appearance the root resolves.
    pub fn appearance(&self) -> Appearance {
        Appearance {
            theme: self.theme,
            accent: self.accent,
            motion: self.motion,
        }
    }

    /// The same axes on `preset`, with the Space reset to it.
    pub fn with_preset(self, preset: PresetIndex) -> Self {
        Axes {
            preset,
            look: preset.look(),
            ..self
        }
    }
}

/// The preference that names `level` outright.
pub fn motion_of(level: MotionLevel) -> Motion {
    match level {
        MotionLevel::Calm => Motion::Calm,
        MotionLevel::Standard => Motion::Standard,
        MotionLevel::Extra => Motion::Extra,
        MotionLevel::Reduced => Motion::Reduced,
    }
}

/// The material's name in the toolbar.
pub fn material_label(material: Material) -> &'static str {
    match material {
        Material::Window => "Window",
        Material::Bar => "Bar",
        Material::Dock => "Dock",
        Material::Popover => "Popover",
        Material::Sheet => "Sheet",
        Material::Toast => "Toast",
        Material::Osd => "OSD",
        Material::Widget => "Widget",
    }
}

thread_local! {
    /// The axes the next root on this thread starts from.
    static START: RefCell<Option<Axes>> = const { RefCell::new(None) };
}

/// Make the next gallery root on this thread start at `axes`.
pub fn start_with(axes: Axes) {
    START.with(|start| *start.borrow_mut() = Some(axes));
}

/// The axes a new root starts from: the last [`start_with`] on this thread, else the defaults.
pub fn starting() -> Axes {
    START.with(|start| start.borrow().clone().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::{Axes, PresetIndex, motion_of, start_with, starting};
    use crate::page::Page;
    use ds::{MotionLevel, PRESETS};

    #[test]
    fn a_preset_resets_the_space_to_its_own_dots() {
        for preset in PresetIndex::all() {
            let mut edited = Axes::default();
            edited.look.dots.clear();
            let axes = edited.with_preset(preset);
            assert_eq!(
                axes.look.dots,
                PRESETS[usize::from(preset.0)].dots,
                "{preset:?}"
            );
        }
        assert_eq!(PresetIndex::all().count(), 8);
    }

    #[test]
    fn each_level_is_named_outright() {
        for level in MotionLevel::ALL {
            let appearance = Axes {
                motion: motion_of(level),
                ..Axes::default()
            }
            .appearance();
            let resolved = ds::resolve(appearance, ds::Theme::System, ds::SystemPrefs::default());
            assert_eq!(resolved.motion, level);
        }
    }

    #[test]
    fn a_root_starts_where_it_was_asked_to() {
        assert_eq!(starting().page, Axes::default().page);
        start_with(Axes {
            page: Page::Gaps,
            ..Axes::default()
        });
        assert_eq!(starting().page, Page::Gaps);
    }
}
