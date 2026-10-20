//! PresetSwatches: a Space's colour as one row of the eight presets and a ninth, Custom, for a
//! settings page that edits a Space in a form row (a name field above it, a theme below) rather
//! than in the editor. Custom opens the editor's colour on its own in a small popover: the field,
//! the stops and the grain ([`super::colour::SpaceColour`] without its presets, which the row
//! already holds).

use super::handles::Field;
use super::parts::{GrainRow, Stops};
use super::{DotIndex, Picker, active, edit};
use crate::components::overlays::popover::{Arrow, Popover};
use crate::host::measure::{Anchor, MountedRef};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::placement::{Align, Placement, Side};
use ds_core::geometry::units::Px;
use ds_core::vocab::Shown;
use ds_style::scope::use_scope;
use ds_style::space::dot_paint::DotPaint;
use ds_style::space::look::SpaceLook;
use ds_style::space::palette::derive;
use ds_style::space::presets::PRESETS;

/// The custom swatch's name.
const CUSTOM: &str = "Custom";

/// The eight presets as round swatches, each painted in its dots' own colours (the picks, not
/// the pale frame), named for itself, in a radio group, then Custom. The swatch whose dots
/// `look` wears is checked; a look that matches no preset checks Custom, which then wears it.
/// A preset's press is `look` with that preset's dots; grain, theme and accent stay, as the
/// editor's presets do.
///
/// Custom opens a popover under it with the colour's field, stops and grain; each drag or nudge
/// there is a whole new look through `onchange`, as fast as the hand moves. `oncommit` hears when
/// the look is settled: after a preset's press, and when the popover closes. A host that writes
/// the look somewhere writes it there, and only follows `onchange`.
#[component]
pub fn PresetSwatches(
    look: SpaceLook,
    onchange: EventHandler<SpaceLook>,
    #[props(default)] oncommit: Option<EventHandler<()>>,
    #[props(default)] common: Common,
) -> Element {
    let scheme = use_scope().scheme;
    let mut shown = use_signal(|| Shown::Hidden);
    let mut custom = use_signal(|| None::<MountedRef>);
    let picked = use_signal(|| None::<(DotIndex, DotIndex)>);
    let commit = move || {
        if let Some(oncommit) = oncommit {
            oncommit.call(());
        }
    };
    let paints: Vec<DotPaint> = PRESETS
        .iter()
        .map(|preset| DotPaint::gradient(&derive(preset.dots, scheme).picked))
        .collect();
    let preset = PRESETS.iter().position(|preset| look.dots == preset.dots);
    let own = DotPaint::gradient(&derive(&look.dots, scheme).picked);
    let picker = Picker {
        picked,
        prop: DotIndex::default(),
        report: None,
    };
    let current = active(DotIndex::default(), picked(), look.dots.len());
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
            for (index, (one, paint)) in PRESETS.iter().zip(&paints).enumerate() {
                button {
                    key: "{index}",
                    r#type: "button",
                    class: "ds-preset",
                    role: "radio",
                    "aria-checked": if preset == Some(index) { "true" } else { "false" },
                    "aria-label": one.name,
                    title: one.name,
                    "data-stops": paint.count(),
                    style: paint.style_attr(),
                    onclick: {
                        let look = look.clone();
                        move |_| {
                            onchange.call(edit::preset(&look, index));
                            commit();
                        }
                    },
                }
            }
            button {
                r#type: "button",
                class: "ds-preset ds-preset-custom",
                role: "radio",
                "aria-checked": if preset.is_none() { "true" } else { "false" },
                "aria-haspopup": "dialog",
                "aria-expanded": if shown() == Shown::Visible { "true" } else { "false" },
                "aria-label": CUSTOM,
                title: CUSTOM,
                "data-stops": if preset.is_none() { Some(own.count()) } else { None },
                style: if preset.is_none() { Some(own.style_attr()) } else { None },
                onmounted: move |event| custom.set(Some(MountedRef(event.data()))),
                onclick: move |_| shown.set(shown().flipped()),
            }
        }
        if shown() == Shown::Visible && let Some(anchor) = custom() {
            Popover {
                anchor: Anchor::Mounted(anchor),
                placement: Placement::new(Side::Bottom, Align::Center),
                gap: Px(4.0),
                arrow: Arrow::Arrow,
                common: Common { aria_label: Some(CUSTOM.to_owned()), ..Common::default() },
                onclose: move |()| {
                    shown.set(Shown::Hidden);
                    commit();
                },
                div { class: "ds-space-editor ds-custom-colour",
                    Field { look: look.clone(), scheme, current, picker, onchange }
                    Stops { look: look.clone(), scheme, current, picker, onchange }
                    GrainRow { look, onchange }
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
            PRESETS.len() + 1,
            "the presets and Custom: {html}"
        );
        let checked: Vec<&str> = html
            .split("<button")
            .filter(|button| button.contains("aria-checked=\"true\""))
            .collect();
        assert_eq!(checked.len(), 1, "{html}");
        assert!(checked[0].contains("aria-label=\"Harbour\""), "{html}");
        let custom = SpaceLook {
            dots: vec![Dot {
                hue: 10.0,
                chroma: 0.3,
            }],
            ..SpaceLook::default()
        };
        // A look made in the editor, matching no preset, checks Custom, which wears it.
        let html = page(custom);
        let checked: Vec<&str> = html
            .split("<button")
            .filter(|button| button.contains("aria-checked=\"true\""))
            .collect();
        assert_eq!(checked.len(), 1, "{html}");
        assert!(checked[0].contains("aria-label=\"Custom\""), "{html}");
        assert!(checked[0].contains("data-stops=\"1\""), "{html}");
    }
}
