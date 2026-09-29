//! AppearancePicker: THE one picker for Theme and Accent, in mailo, the control center and
//! settings (design/04-COMPONENTS.md section 26). Motion is not offered: the user decided the
//! motion level need not be a choice (2026-09-28), so [`Appearance::motion`] rides through a
//! change untouched and the system's reduced-motion preference still maps into it.

use crate::appearance::{
    accent::Accent,
    appearance::Appearance,
    system::SystemPrefs,
    theme::{Scheme, Theme},
};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::segmented::{SegSize, SegmentedControl};
use crate::components::vocab::Switch;
use crate::css::accents_css::swatch_var;
use dioxus::prelude::*;

/// What "System" answers to right now, shown as the Theme header's value while the theme
/// follows the desktop. Not specified (O-16): the doc gives `system` no role of its own.
fn theme_hint(theme: Theme, system: SystemPrefs) -> Option<String> {
    match (theme, system.scheme) {
        (Theme::System, Scheme::Light) => Some("Light".to_string()),
        (Theme::System, Scheme::Dark) => Some("Dark".to_string()),
        (Theme::Light | Theme::Dark, _) => None,
    }
}

/// How the picker lays its rows out for the width it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PickerLayout {
    /// Each segmented row as wide as its words at the regular size: a settings page or a sheet.
    #[default]
    Full,
    /// The rows fill the width they are given and no more, at the control center's Regular
    /// size (22, design/29-SIZING.md), with narrow sides on equal segments that share it: the
    /// control center's 300 px module.
    Compact,
}

impl PickerLayout {
    /// The `data-layout` word.
    fn slug(self) -> &'static str {
        match self {
            PickerLayout::Full => "full",
            PickerLayout::Compact => "compact",
        }
    }

    /// The segmented rows' size.
    fn seg_size(self) -> SegSize {
        match self {
            PickerLayout::Full => SegSize::Regular,
            PickerLayout::Compact => SegSize::Regular,
        }
    }
}

/// `aria-pressed` for a swatch.
fn pressed(accent: Accent, value: Accent) -> Switch {
    if accent == value {
        Switch::On
    } else {
        Switch::Off
    }
}

/// Theme and accent rows. Every change is emitted at once as a whole [`Appearance`], its
/// `motion` as it came in; the consumer persists it. The swatches are the accent table's six
/// (O-17), each painted with its `--swatch-*` token. `layout` fits it to a narrow host
/// ([`PickerLayout::Compact`]).
#[component]
pub fn AppearancePicker(
    value: Appearance,
    system: SystemPrefs,
    onchange: EventHandler<Appearance>,
    #[props(default)] layout: PickerLayout,
) -> Element {
    let size = layout.seg_size();
    let themes: Vec<(Theme, String)> = Theme::ALL
        .into_iter()
        .map(|theme| (theme, theme.label().to_string()))
        .collect();
    rsx! {
        div { class: "ds-appearance", role: "group", "aria-label": "Appearance", "data-layout": layout.slug(),
            div { class: "ds-appearance-row",
                SectionHeader { kind: HeaderKind::Field, text: "Theme", value: theme_hint(value.theme, system) }
                SegmentedControl::<Theme> {
                    label: "Theme",
                    options: themes,
                    value: value.theme,
                    size,
                    onchange: move |theme| onchange.call(Appearance { theme, ..value }),
                }
            }
            div { class: "ds-appearance-row",
                SectionHeader { kind: HeaderKind::Field, text: "Accent" }
                div { class: "ds-appearance-swatches", role: "group", "aria-label": "Accent",
                    for accent in Accent::ALL {
                        button {
                            r#type: "button",
                            class: "ds-space-dot",
                            "aria-pressed": pressed(accent, value.accent).aria(),
                            "aria-label": accent.label(),
                            style: format!("background:var({})", swatch_var(accent)),
                            onclick: move |_| onchange.call(Appearance { accent, ..value }),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::theme_hint;
    use crate::appearance::{
        system::{ReducedMotion, SystemPrefs},
        theme::{Scheme, Theme},
    };

    #[test]
    fn system_names_what_it_follows() {
        let dark = SystemPrefs {
            scheme: Scheme::Dark,
            motion: ReducedMotion::Reduce,
            ..SystemPrefs::default()
        };
        assert_eq!(theme_hint(Theme::System, dark).as_deref(), Some("Dark"));
        assert_eq!(theme_hint(Theme::Light, dark), None);
    }
}
