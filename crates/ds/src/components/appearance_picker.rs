//! AppearancePicker: THE one picker for Theme, Accent and Motion, in mailo, the control center
//! and settings (design/04-COMPONENTS.md section 26).

use crate::appearance::{Accent, Appearance, Motion, ReducedMotion, Scheme, SystemPrefs, Theme};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::segmented::SegmentedControl;
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

/// What "System" answers to for motion: Reduced when the desktop asks for it, else Standard.
fn motion_hint(motion: Motion, system: SystemPrefs) -> Option<String> {
    match (motion, system.motion) {
        (Motion::System, ReducedMotion::Reduce) => Some("Reduced".to_string()),
        (Motion::System, ReducedMotion::NoPreference) => Some("Standard".to_string()),
        (_, _) => None,
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

/// Theme, accent and motion rows. Every change is emitted at once as a whole [`Appearance`];
/// the consumer persists it. The swatches are the accent table's six (O-17), each painted with
/// its `--swatch-*` token; the Motion row offers `System` as well as the four levels, because
/// [`Motion`] defaults to it (the doc's markup lists only the four).
#[component]
pub fn AppearancePicker(
    value: Appearance,
    system: SystemPrefs,
    onchange: EventHandler<Appearance>,
) -> Element {
    let themes: Vec<(Theme, String)> = Theme::ALL
        .into_iter()
        .map(|theme| (theme, theme.label().to_string()))
        .collect();
    let motions: Vec<(Motion, String)> = Motion::ALL
        .into_iter()
        .map(|motion| (motion, motion.label().to_string()))
        .collect();
    rsx! {
        div { class: "ds-appearance", role: "group", "aria-label": "Appearance",
            div { class: "ds-appearance-row",
                SectionHeader { kind: HeaderKind::Field, text: "Theme", value: theme_hint(value.theme, system) }
                SegmentedControl::<Theme> {
                    label: "Theme",
                    options: themes,
                    value: value.theme,
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
            div { class: "ds-appearance-row",
                SectionHeader { kind: HeaderKind::Field, text: "Motion", value: motion_hint(value.motion, system) }
                SegmentedControl::<Motion> {
                    label: "Motion",
                    options: motions,
                    value: value.motion,
                    onchange: move |motion| onchange.call(Appearance { motion, ..value }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{motion_hint, theme_hint};
    use crate::appearance::{Motion, ReducedMotion, Scheme, SystemPrefs, Theme};

    #[test]
    fn system_names_what_it_follows() {
        let dark = SystemPrefs {
            scheme: Scheme::Dark,
            motion: ReducedMotion::Reduce,
            ..SystemPrefs::default()
        };
        assert_eq!(theme_hint(Theme::System, dark).as_deref(), Some("Dark"));
        assert_eq!(theme_hint(Theme::Light, dark), None);
        assert_eq!(
            motion_hint(Motion::System, dark).as_deref(),
            Some("Reduced")
        );
        assert_eq!(
            motion_hint(Motion::System, SystemPrefs::default()).as_deref(),
            Some("Standard")
        );
        assert_eq!(motion_hint(Motion::Calm, dark), None);
    }
}
