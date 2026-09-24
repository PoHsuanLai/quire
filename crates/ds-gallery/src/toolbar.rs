//! The toolbar: every axis the gallery sweeps, built from quire's own controls: segmented
//! controls for theme, accent and motion level, menus for the material and the Space, a toggle
//! for blur, tabs for the page.

use crate::axes::{Axes, PresetIndex, material_label, motion_of};
use crate::page::Page;
use crate::registry;
use dioxus::prelude::*;
use ds::{
    Accent, Anchor, Availability, BlurState, Button, ButtonVariant, Check, Icon, Material, Menu,
    MenuEntry, MenuKind, MotionLevel, MountedRef, SegSize, SegmentedControl, Switch, Tabs, Theme,
    Toggle, Trail,
};

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
        .into_iter()
        .map(|theme| (theme, theme.label().to_string()))
        .collect::<Vec<_>>();
    let accents = Accent::ALL
        .into_iter()
        .map(|accent| (accent, accent.label().to_string()))
        .collect::<Vec<_>>();
    let levels = MotionLevel::ALL
        .into_iter()
        .map(|level| (motion_of(level), capitalised(level.slug())))
        .collect::<Vec<_>>();
    let materials = Material::ALL
        .into_iter()
        .map(|material| (material, material_label(material).to_string()))
        .collect::<Vec<_>>();
    let presets = PresetIndex::all()
        .map(|preset| (preset, preset.label()))
        .collect::<Vec<_>>();
    let blur = match now.blur {
        BlurState::Available => Switch::On,
        BlurState::Unavailable => Switch::Off,
    };
    rsx! {
        div { class: "g-toolbar",
            Tool { name: "Theme",
                SegmentedControl::<Theme> { label: "Theme", options: themes, value: now.theme, size: SegSize::Small,
                    onchange: move |theme| axes.with_mut(|axes| axes.theme = theme),
                }
            }
            Tool { name: "Accent",
                SegmentedControl::<Accent> { label: "Accent", options: accents, value: now.accent, size: SegSize::Small,
                    onchange: move |accent| axes.with_mut(|axes| axes.accent = accent),
                }
            }
            Tool { name: "Motion",
                SegmentedControl::<ds::Motion> { label: "Motion level", options: levels, value: now.motion, size: SegSize::Small,
                    onchange: move |motion| axes.with_mut(|axes| axes.motion = motion),
                }
            }
            Tool { name: "Material",
                Choice::<Material> {
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
                        Switch::On => BlurState::Available,
                        Switch::Off => BlurState::Unavailable,
                    }),
                }
            }
            Tool { name: "Space",
                Choice::<PresetIndex> {
                    label: "Space preset",
                    options: presets,
                    value: now.preset,
                    onpick: move |preset| axes.with_mut(|axes| *axes = axes.clone().with_preset(preset)),
                }
            }
        }
        div { class: "g-pages",
            Tabs::<Page> { label: "Page", tabs: pages, value: now.page,
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

/// A button showing the current choice, opening a dropdown menu of every choice.
#[component]
fn Choice<T: Clone + PartialEq + 'static>(
    label: String,
    options: Vec<(T, String)>,
    value: T,
    onpick: EventHandler<T>,
) -> Element {
    let mut open = use_signal(|| Switch::Off);
    let mut anchor = use_signal(|| None::<MountedRef>);
    let current = options
        .iter()
        .find(|(option, _)| *option == value)
        .map_or(String::new(), |(_, name)| name.clone());
    let items = options
        .iter()
        .map(|(option, name)| MenuEntry::Item {
            availability: Availability::Enabled,
            value: option.clone(),
            title: name.clone(),
            detail: None,
            tile: None,
            trail: Trail::None,
            check: Some(if *option == value {
                Check::Checked
            } else {
                Check::Unchecked
            }),
        })
        .collect::<Vec<_>>();
    let entries = std::iter::once(MenuEntry::Header(label))
        .chain(items)
        .collect::<Vec<_>>();
    rsx! {
        span { class: "g-anchor", onmounted: move |event| anchor.set(Some(MountedRef(event.data()))),
            Button {
                variant: ButtonVariant::Secondary,
                label: current,
                icon: Some(Icon::ChevronDown),
                pressed: Some(open()),
                onclick: move |_| open.set(Switch::On),
            }
        }
        if let (Switch::On, Some(mounted)) = (open(), anchor()) {
            Menu::<T> {
                kind: MenuKind::Dropdown,
                anchor: Anchor::Mounted(mounted),
                entries,
                onpick: move |choice| onpick.call(choice),
                onclose: move |_| open.set(Switch::Off),
            }
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
