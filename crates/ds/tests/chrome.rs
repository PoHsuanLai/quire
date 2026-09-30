//! The root chrome as markup (bar gaps): what each material's root stamps
//! and draws (the window's loose layers and grain; shell chrome's `.ds-frame` group; nothing on
//! a transparent root), the explicit overrides, and a `Surface` on the frame ground. One golden
//! per material and per override, under `snapshots/root/chrome/`.

#[path = "support/golden.rs"]
#[allow(dead_code)] // Only `check` is used here.
mod golden;

use dioxus::core::VirtualDom;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::prelude::*;
use ds::root::chrome::Ground;
use ds_style::appearance::blur::BlurState;
use ds_style::space::presets::PRESETS;
use ds_style::tokens::shape::Corner;

#[derive(Props, Clone, PartialEq)]
struct Setup {
    material: Material,
    blur: BlurState,
    chrome: Option<RootChrome>,
    ground: Option<Ground>,
    surface: Option<Ground>,
    radius: Option<Corner>,
}

#[allow(non_snake_case)]
fn Root(setup: Setup) -> Element {
    let look = SpaceLook {
        dots: PRESETS[0].dots.to_vec(),
        ..SpaceLook::default()
    };
    rsx! {
        Ds {
            appearance: Appearance { theme: Theme::Light, ..Appearance::default() },
            look,
            material: setup.material,
            blur: setup.blur,
            stylesheet: Inject::Host,
            chrome: setup.chrome,
            ground: setup.ground,
            radius: setup.radius,
            if let Some(on) = setup.surface {
                Surface { material: Material::Widget, on: Some(on), p { "inside" } }
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

fn setup(material: Material) -> Setup {
    Setup {
        material,
        blur: BlurState::Available,
        chrome: None,
        ground: None,
        surface: None,
        radius: None,
    }
}

#[test]
fn each_materials_root_matches_its_golden() {
    let mut failures = Vec::new();
    for material in Material::ALL.iter().copied() {
        let name = format!("root/chrome/{}.html", material.slug());
        if let Err(failure) = golden::check(&name, &render(setup(material))) {
            failures.push(failure);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_overrides_match_their_goldens() {
    let cases = [
        (
            "painted-popover",
            Setup {
                chrome: Some(RootChrome::Painted),
                ..setup(Material::Popover)
            },
        ),
        (
            "transparent-bar",
            Setup {
                chrome: Some(RootChrome::Transparent),
                ..setup(Material::Bar)
            },
        ),
        (
            "paper-bar",
            Setup {
                ground: Some(Ground::Paper),
                ..setup(Material::Bar)
            },
        ),
        (
            "frame-surface-in-a-window",
            Setup {
                surface: Some(Ground::Frame),
                ..setup(Material::Window)
            },
        ),
        (
            "dock-radius",
            Setup {
                radius: Some(Corner::Px(Px(12.0))),
                ..setup(Material::Dock)
            },
        ),
        (
            "paper-surface-in-a-bar",
            Setup {
                surface: Some(Ground::Paper),
                ..setup(Material::Bar)
            },
        ),
    ];
    let mut failures = Vec::new();
    for (name, case) in cases {
        if let Err(failure) = golden::check(&format!("root/chrome/{name}.html"), &render(case)) {
            failures.push(failure);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
