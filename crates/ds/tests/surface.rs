//! `Surface` as markup: a nested scope re-stamps the attributes it overrides and inherits the
//! rest, and the `Env` its children read agrees with what it stamped. One golden per override.

#[path = "support/golden.rs"]
#[allow(dead_code)] // Only `check` is used here.
mod golden;

use dioxus::core::VirtualDom;
use dioxus::prelude::*;
use ds::{Accent, Appearance, BlurState, Ds, Material, Scheme, Surface, use_env};

#[derive(Props, Clone, PartialEq)]
struct Setup {
    theme: Option<Scheme>,
    accent: Option<Accent>,
    blur: Option<BlurState>,
}

/// What the scope says to a component inside it.
#[component]
fn Reads() -> Element {
    let env = use_env();
    rsx! {
        p {
            "{env.scheme.slug()} {env.resolved.accent.slug()} {env.material.slug()} {env.blur.slug()}"
        }
    }
}

#[allow(non_snake_case)]
fn Root(setup: Setup) -> Element {
    rsx! {
        Ds {
            appearance: Appearance { accent: Accent::Postmark, ..Appearance::default() },
            material: Material::Popover,
            blur: BlurState::Unavailable,
            Surface {
                material: Material::Sheet,
                theme: setup.theme,
                accent: setup.accent,
                blur: setup.blur,
                Reads {}
            }
        }
    }
}

/// The nested scope alone, from its opening tag to its close (`Reads` draws no `div`): the
/// root's own attributes and the overlays after it are not what this test is about.
fn nested(setup: Setup) -> String {
    let mut dom = VirtualDom::new_with_props(Root, setup);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    let start = html
        .match_indices("<div class=\"ds\"")
        .nth(1)
        .unwrap_or_else(|| panic!("no nested .ds in {html}"))
        .0;
    let end = start + html[start..].find("</div>").expect("the scope closes") + "</div>".len();
    html[start..end].to_owned()
}

#[test]
fn each_override_is_stamped_and_read() {
    let cases = [
        (
            "inherits",
            Setup {
                theme: None,
                accent: None,
                blur: None,
            },
            "light postmark sheet off",
        ),
        (
            "accent",
            Setup {
                theme: None,
                accent: Some(Accent::Violet),
                blur: None,
            },
            "light violet sheet off",
        ),
        (
            "blur",
            Setup {
                theme: None,
                accent: None,
                blur: Some(BlurState::Available),
            },
            "light postmark sheet on",
        ),
        (
            "all",
            Setup {
                theme: Some(Scheme::Dark),
                accent: Some(Accent::Violet),
                blur: Some(BlurState::Available),
            },
            "dark violet sheet on",
        ),
    ];
    let mut failures = Vec::new();
    for (name, setup, reads) in cases {
        let html = nested(setup);
        if !html.contains(&format!("<p>{reads}</p>")) {
            failures.push(format!("{name}: the children read something else: {html}"));
        }
        if let Err(diff) = golden::check(&format!("root/surface/{name}.html"), &html) {
            failures.push(diff);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
