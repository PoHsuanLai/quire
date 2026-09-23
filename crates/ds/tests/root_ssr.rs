//! The `Ds` root as markup: exactly the attributes it stamps, the stylesheet only when inlined,
//! the frame layers only on a Window, and never the word "system" (design/03-COLOR.md section
//! 17.1, design/05-MOTION.md section 9 rule 11, spike S12).

use dioxus::core::VirtualDom;
use dioxus::prelude::*;
use ds::{
    Accent, Appearance, BlurState, Ds, FrameVars, HostModality, Inject, InputModality, Material,
    Motion, ReducedMotion, Scheme, SpaceLook, Surface, SystemPrefs, Theme,
};
use std::collections::BTreeMap;

#[derive(Props, Clone, PartialEq)]
struct Setup {
    appearance: Appearance,
    system: SystemPrefs,
    look: SpaceLook,
    material: Material,
    blur: BlurState,
    stylesheet: Inject,
    modality: Option<InputModality>,
    surface: Option<(Material, Option<Scheme>)>,
}

impl Default for Setup {
    fn default() -> Self {
        Setup {
            appearance: Appearance::default(),
            system: SystemPrefs::default(),
            look: SpaceLook::default(),
            material: Material::Popover,
            blur: BlurState::default(),
            stylesheet: Inject::Host,
            modality: None,
            surface: None,
        }
    }
}

#[allow(non_snake_case)]
fn Root(setup: Setup) -> Element {
    if let Some(modality) = setup.modality {
        use_context_provider(|| HostModality(Signal::new(modality)));
    }
    rsx! {
        Ds {
            appearance: setup.appearance,
            system: setup.system,
            look: setup.look.clone(),
            material: setup.material,
            blur: setup.blur,
            stylesheet: setup.stylesheet,
            if let Some((material, theme)) = setup.surface {
                Surface { material, theme,
                    p { "inside" }
                }
            } else {
                p { "inside" }
            }
        }
    }
}

fn render(setup: Setup) -> String {
    let mut dom = VirtualDom::new_with_props(Root, setup);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// The attributes of the first tag that opens with `open` (`<div class="ds"`).
fn attributes_of(markup: &str, nth: usize) -> BTreeMap<String, String> {
    let start = markup
        .match_indices("<div class=\"ds\"")
        .nth(nth)
        .unwrap_or_else(|| panic!("no .ds number {nth} in {markup}"))
        .0;
    let tag = &markup[start + "<div".len()..];
    let tag = &tag[..tag.find('>').expect("the tag closes")];
    let mut attributes = BTreeMap::new();
    let mut rest = tag.trim_start();
    while let Some(eq) = rest.find("=\"") {
        let name = rest[..eq].trim().to_owned();
        let value_start = eq + 2;
        let value_end = value_start + rest[value_start..].find('"').expect("the value closes");
        attributes.insert(name, rest[value_start..value_end].to_owned());
        rest = rest[value_end + 1..].trim_start();
    }
    attributes
}

fn expected(pairs: &[(&str, String)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

#[test]
fn the_root_stamps_exactly_its_attributes() {
    let dark_reduced = SystemPrefs {
        scheme: Scheme::Dark,
        motion: ReducedMotion::Reduce,
        ..SystemPrefs::default()
    };
    struct Case {
        name: &'static str,
        setup: Setup,
        theme: &'static str,
        accent: &'static str,
        motion: &'static str,
        material: &'static str,
        blur: &'static str,
        modality: &'static str,
        scheme: Scheme,
    }
    let cases = [
        Case {
            name: "system everything resolves against a dark, reduced desktop",
            setup: Setup {
                appearance: Appearance {
                    theme: Theme::System,
                    accent: Accent::Violet,
                    motion: Motion::System,
                },
                system: dark_reduced,
                blur: BlurState::Available,
                ..Setup::default()
            },
            theme: "dark",
            accent: "violet",
            motion: "reduced",
            material: "popover",
            blur: "on",
            modality: "pointer",
            scheme: Scheme::Dark,
        },
        Case {
            name: "an explicit light theme and calm motion win over the desktop",
            setup: Setup {
                appearance: Appearance {
                    theme: Theme::Light,
                    accent: Accent::Postmark,
                    motion: Motion::Calm,
                },
                system: dark_reduced,
                material: Material::Bar,
                ..Setup::default()
            },
            theme: "light",
            accent: "postmark",
            motion: "calm",
            material: "bar",
            blur: "off",
            modality: "pointer",
            scheme: Scheme::Light,
        },
        Case {
            name: "the Space's own dark theme answers a system app theme",
            setup: Setup {
                look: SpaceLook {
                    theme: Theme::Dark,
                    ..SpaceLook::default()
                },
                modality: Some(InputModality::Keyboard),
                ..Setup::default()
            },
            theme: "dark",
            accent: "postmark",
            motion: "standard",
            material: "popover",
            blur: "off",
            modality: "keyboard",
            scheme: Scheme::Dark,
        },
    ];
    for case in cases {
        let look = case.setup.look.clone();
        let markup = render(case.setup);
        let style = FrameVars::of(&look, case.scheme).style_attr();
        let want = expected(&[
            ("class", "ds".to_owned()),
            ("data-theme", case.theme.to_owned()),
            ("data-accent", case.accent.to_owned()),
            ("data-motion", case.motion.to_owned()),
            ("data-material", case.material.to_owned()),
            ("data-blur", case.blur.to_owned()),
            ("data-modality", case.modality.to_owned()),
            ("data-hover", "cold".to_owned()),
            ("style", style),
        ]);
        assert_eq!(attributes_of(&markup, 0), want, "{}", case.name);
        assert!(
            !markup.to_lowercase().contains("system"),
            "{}: \"system\" reached the markup: {markup}",
            case.name
        );
    }
}

#[test]
fn the_stylesheet_is_inlined_only_when_asked() {
    let inline = render(Setup {
        stylesheet: Inject::Inline,
        ..Setup::default()
    });
    assert_eq!(inline.matches("<style").count(), 1, "one <style>: {inline}");
    let host = render(Setup::default());
    assert!(!host.contains("<style"), "no <style> with Inject::Host");
}

#[test]
fn only_a_window_draws_the_frame_layers_and_grain() {
    let window = render(Setup {
        material: Material::Window,
        ..Setup::default()
    });
    for class in [
        "class=\"ds-layer\"",
        "class=\"ds-layer back\"",
        "class=\"ds-grain\"",
    ] {
        assert_eq!(window.matches(class).count(), 1, "{class} in {window}");
    }
    let popover = render(Setup::default());
    assert!(!popover.contains("ds-layer"), "no layers on a popover");
    assert!(!popover.contains("ds-grain"), "no grain on a popover");
}

#[test]
fn a_surface_nests_a_scope_without_a_stylesheet() {
    let markup = render(Setup {
        stylesheet: Inject::Inline,
        material: Material::Window,
        surface: Some((Material::Popover, Some(Scheme::Dark))),
        ..Setup::default()
    });
    let nested = attributes_of(&markup, 1);
    let want = expected(&[
        ("class", "ds".to_owned()),
        ("data-theme", "dark".to_owned()),
        ("data-accent", "postmark".to_owned()),
        ("data-motion", "standard".to_owned()),
        ("data-material", "popover".to_owned()),
        ("data-blur", "off".to_owned()),
    ]);
    assert_eq!(nested, want);
    assert_eq!(markup.matches("<style").count(), 1, "the root's only");
    assert!(markup.contains("inside"), "children render");
}
