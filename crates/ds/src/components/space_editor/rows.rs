//! The editor's rows a consumer switches on (mailo gaps 2): the title as a name field, the
//! Motion row, and the contrast measured in each scheme under its own heading.

use super::parts::CheckRows;
use crate::appearance::{Motion, Scheme, Theme};
use crate::components::section_header::{HeaderKind, SectionHeader};
use crate::components::segmented::SegmentedControl;
use crate::components::text_input::{InputVariant, TextInput};
use crate::space::dot_paint::DotPaint;
use crate::space::{Dot, SpaceLook, derive};
use dioxus::prelude::*;

/// The Motion row's value and where a pick goes. The Space's own motion is the person's
/// choice, `System` included, which the consumer passes on as its root's `appearance.motion`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionChoice {
    /// The Space's motion now.
    pub level: Motion,
    /// The person picked another.
    pub on_motion: EventHandler<Motion>,
}

/// Which schemes the contrast readout measures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasuredIn {
    /// The scheme the editor is drawn in, under "Measured, this Space, this theme".
    #[default]
    ThisScheme,
    /// Every scheme the Space's theme can show (both for System, else its one), each under a
    /// small-caps heading of its own: a System Space is read in the dark as well as the light.
    EachScheme,
}

/// The schemes a Space with `theme` is seen in, and what each is called.
fn schemes_of(theme: Theme) -> &'static [(Scheme, &'static str)] {
    match theme {
        Theme::System => &[(Scheme::Light, "Light"), (Scheme::Dark, "Dark")],
        Theme::Light => &[(Scheme::Light, "Light")],
        Theme::Dark => &[(Scheme::Dark, "Dark")],
    }
}

/// The panel's title: the swatch and "{name} Space", or, with `on_rename`, the swatch and an
/// inline field holding the name.
#[component]
pub(super) fn Title(
    dots: Vec<Dot>,
    scheme: Scheme,
    name: Option<String>,
    on_rename: Option<EventHandler<String>>,
) -> Element {
    let swatch = DotPaint::gradient(&derive(&dots, scheme).stops);
    let heading = match (&name, on_rename) {
        (_, Some(rename)) => rsx! {
            TextInput {
                variant: InputVariant::Inline,
                label: "Space name",
                value: name.clone().unwrap_or_default(),
                placeholder: "Name this Space",
                oninput: move |text: String| rename.call(text),
            }
        },
        (Some(name), None) => rsx! { "{name} Space" },
        (None, None) => rsx! { "Space" },
    };
    rsx! {
        h3 { class: "ds-space-editor-title",
            span { class: "ds-space-swatch", "data-stops": swatch.count(), style: swatch.style_attr() }
            {heading}
        }
    }
}

/// The Motion row: a segmented control over the person's five choices.
#[component]
pub(super) fn MotionRow(choice: MotionChoice) -> Element {
    rsx! {
        div {
            SectionHeader { kind: HeaderKind::Field, text: "Motion" }
            SegmentedControl::<Motion> {
                label: "Motion",
                options: Motion::ALL.into_iter().map(|level| (level, level.label().to_string())).collect::<Vec<_>>(),
                value: choice.level,
                onchange: move |level| choice.on_motion.call(level),
            }
        }
    }
}

/// The contrast readout in each scheme the Space's theme shows, each under its own heading.
#[component]
pub(super) fn EachScheme(look: SpaceLook) -> Element {
    rsx! {
        div {
            SectionHeader { kind: HeaderKind::Field, text: "Measured, this Space" }
            for &(scheme , heading) in schemes_of(look.theme) {
                div { key: "{heading}", class: "ds-checks-scheme",
                    div { class: "ds-checks-heading", "{heading}" }
                    CheckRows { look: look.clone(), scheme }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::schemes_of;
    use crate::appearance::{Scheme, Theme};

    #[test]
    fn a_system_space_is_measured_in_both_schemes_and_a_fixed_one_in_its_own() {
        const CASES: &[(Theme, &[Scheme])] = &[
            (Theme::System, &[Scheme::Light, Scheme::Dark]),
            (Theme::Light, &[Scheme::Light]),
            (Theme::Dark, &[Scheme::Dark]),
        ];
        for &(theme, want) in CASES {
            let got: Vec<Scheme> = schemes_of(theme).iter().map(|(s, _)| *s).collect();
            assert_eq!(got, want, "{theme:?}");
        }
    }
}
