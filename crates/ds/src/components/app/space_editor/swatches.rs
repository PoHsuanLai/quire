//! PresetSwatches: a Space's colour as one row of the eight presets, for a settings page that
//! edits a Space in a form row (a name field above it, a theme below) rather than in the editor.
//! The field, the stops and the grain stay the editor's ([`super::colour::SpaceColour`]).

use super::edit;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_style::scope::use_scope;
use ds_style::space::dot_paint::DotPaint;
use ds_style::space::look::SpaceLook;
use ds_style::space::palette::derive;
use ds_style::space::presets::PRESETS;

/// The eight presets as round swatches, each painted in its dots' own colours (the picks, not
/// the pale frame), named for itself, in a radio group. The swatch whose dots `look` wears is checked
/// (a look made in the editor may match none). A press is `look` with that preset's dots;
/// grain, theme and accent stay, as the editor's presets do.
#[component]
pub fn PresetSwatches(
    look: SpaceLook,
    onchange: EventHandler<SpaceLook>,
    #[props(default)] common: Common,
) -> Element {
    let scheme = use_scope().scheme;
    let paints: Vec<DotPaint> = PRESETS
        .iter()
        .map(|preset| DotPaint::gradient(&derive(preset.dots, scheme).picked))
        .collect();
    let class = common.class("ds-preset-swatches");
    let label = common
        .aria_label
        .clone()
        .unwrap_or_else(|| "Colour".to_owned());
    rsx! {
        div {
            class,
            id: common.id.clone(),
            role: "radiogroup",
            "aria-label": "{label}",
            onmounted: move |event| common.mounted(event),
            for (index, (preset, paint)) in PRESETS.iter().zip(&paints).enumerate() {
                button {
                    key: "{index}",
                    r#type: "button",
                    class: "ds-preset",
                    role: "radio",
                    "aria-checked": if look.dots == preset.dots { "true" } else { "false" },
                    "aria-label": preset.name,
                    title: preset.name,
                    "data-stops": paint.count(),
                    style: paint.style_attr(),
                    onclick: {
                        let look = look.clone();
                        move |_| onchange.call(edit::preset(&look, index))
                    },
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PresetSwatches;
    use crate::assembly::ds::Ds;
    use dioxus::prelude::*;
    use ds_style::appearance::{appearance::Appearance, material::Material};
    use ds_style::space::look::SpaceLook;
    use ds_style::space::palette::Dot;
    use ds_style::space::presets::PRESETS;

    fn page(look: SpaceLook) -> String {
        // The swatches paint in the scope's scheme, which only a `Ds` provides.
        #[component]
        fn Host(look: SpaceLook) -> Element {
            rsx! {
                Ds { appearance: Appearance::default(), material: Material::Window,
                    PresetSwatches { look, onchange: |_| {} }
                }
            }
        }
        let mut dom = VirtualDom::new_with_props(Host, HostProps { look });
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_preset_the_look_wears_is_the_one_checked() {
        let harbour = SpaceLook {
            dots: PRESETS[2].dots.to_vec(),
            ..SpaceLook::default()
        };
        let html = page(harbour);
        assert_eq!(
            html.matches("role=\"radio\"").count(),
            PRESETS.len(),
            "{html}"
        );
        let checked: Vec<&str> = html
            .split("<button")
            .filter(|button| button.contains("aria-checked=\"true\""))
            .collect();
        assert_eq!(checked.len(), 1, "{html}");
        assert!(checked[0].contains("aria-label=\"Harbour\""), "{html}");
        // A look made in the editor, matching no preset, checks none.
        let custom = SpaceLook {
            dots: vec![Dot {
                hue: 10.0,
                chroma: 0.3,
            }],
            ..SpaceLook::default()
        };
        assert_eq!(page(custom).matches("aria-checked=\"true\"").count(), 0);
    }
}
