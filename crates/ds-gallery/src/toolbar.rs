//! The toolbar: every axis the gallery sweeps, built from quire's own controls: segmented
//! controls for theme, typeface, accent and motion level, menus for the material and the Space, a toggle
//! for blur, tabs for the page.

use crate::axes::{Axes, PresetIndex, material_label, motion_of};
use crate::page::Page;
use crate::registry;
use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::components::menus::pop_up_button::{PopUpButton, PopUpKind};
use ds::prelude::*;
use ds::style::appearance::blur::BlurState;
use ds::style::tokens::control_size::ControlSize;

/// The toolbar.
#[component]
pub fn Toolbar() -> Element {
    let mut axes = use_context::<Signal<Axes>>();
    let now = axes();
    let pages = registry::REGISTRY
        .iter()
        .map(|entry| (entry.page, entry.title.to_string()))
        .collect::<Vec<_>>();
    let themes = Theme::ALL
        .iter()
        .copied()
        .map(|theme| (theme, theme.label().to_string()))
        .collect::<Vec<_>>();
    let typefaces = Typeface::ALL
        .iter()
        .copied()
        .map(|typeface| (typeface, typeface.label().to_string()))
        .collect::<Vec<_>>();
    let accents = Accent::ALL
        .iter()
        .copied()
        .map(|accent| (accent, accent.label().to_string()))
        .collect::<Vec<_>>();
    let levels = MotionLevel::ALL
        .iter()
        .copied()
        .map(|level| (motion_of(level), capitalised(level.slug())))
        .collect::<Vec<_>>();
    let materials = Material::ALL
        .iter()
        .copied()
        .map(|material| (material, material_label(material).to_string()))
        .collect::<Vec<_>>();
    let presets = PresetIndex::all()
        .map(|preset| (preset, preset.label()))
        .collect::<Vec<_>>();
    let blur = match now.blur {
        BlurState::Available => Check::On,
        BlurState::Unavailable => Check::Off,
    };
    rsx! {
        div { class: "g-toolbar",
            Tool { name: "Theme",
                SegmentedControl::<Theme> { label: "Theme", choices: Choice::pairs(themes), tracking: Tracking::SelectOne(now.theme), size: ControlSize::Mini,
                    onchange: move |theme| axes.with_mut(|axes| axes.theme = theme),
                }
            }
            Tool { name: "Type",
                SegmentedControl::<Typeface> { label: "Typeface", choices: Choice::pairs(typefaces), tracking: Tracking::SelectOne(now.typeface), size: ControlSize::Mini,
                    onchange: move |typeface| axes.with_mut(|axes| axes.typeface = typeface),
                }
            }
            Tool { name: "Accent",
                SegmentedControl::<Accent> { label: "Accent", choices: Choice::pairs(accents), tracking: Tracking::SelectOne(now.accent), size: ControlSize::Mini,
                    onchange: move |accent| axes.with_mut(|axes| axes.accent = accent),
                }
            }
            Tool { name: "Motion",
                SegmentedControl::<Motion> { label: "Motion level", choices: Choice::pairs(levels), tracking: Tracking::SelectOne(now.motion), size: ControlSize::Mini,
                    onchange: move |motion| axes.with_mut(|axes| axes.motion = motion),
                }
            }
            Tool { name: "Material",
                ChoiceMenu::<Material> {
                    label: "Material",
                    options: materials,
                    value: now.material,
                    onpick: move |material| axes.with_mut(|axes| axes.material = material),
                }
            }
            Tool { name: "Blur",
                Toggle {
                    label: "Compositor blur (data-blur)",
                    value: blur,
                    onchange: move |next| axes.with_mut(|axes| axes.blur = match next {
                        Check::On => BlurState::Available,
                        Check::Off | Check::Mixed => BlurState::Unavailable,
                    }),
                }
            }
            Tool { name: "Space",
                ChoiceMenu::<PresetIndex> {
                    label: "Space preset",
                    options: presets,
                    value: now.preset,
                    onpick: move |preset| axes.with_mut(|axes| *axes = axes.clone().with_preset(preset)),
                }
            }
        }
        div { class: "g-pages",
            SegmentedControl::<Page> { label: "Page", choices: Choice::pairs(pages), tracking: Tracking::SelectOne(now.page),
                onchange: move |page| axes.with_mut(|axes| axes.page = page),
            }
        }
    }
}

/// One labelled axis.
#[component]
fn Tool(name: String, children: Element) -> Element {
    rsx! {
        div { class: "g-tool",
            span { class: "g-tool-name", "{name}" }
            {children}
        }
    }
}

/// A pop-up button showing the current choice, opening a menu of every choice.
#[component]
fn ChoiceMenu<T: Clone + PartialEq + 'static>(
    label: String,
    options: Vec<(T, String)>,
    value: T,
    onpick: EventHandler<T>,
) -> Element {
    let items = std::iter::once(MenuItem::Header(label.clone()))
        .chain(
            options
                .iter()
                .map(|(option, name)| MenuItem::new(option.clone(), name.clone())),
        )
        .collect::<Vec<_>>();
    rsx! {
        PopUpButton::<T> {
            kind: PopUpKind::PopUp,
            items,
            value: Some(value),
            title: Some(label),
            onpick: move |choice| onpick.call(choice),
        }
    }
}

/// `standard` as `Standard`.
fn capitalised(word: &str) -> String {
    let mut chars = word.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}
