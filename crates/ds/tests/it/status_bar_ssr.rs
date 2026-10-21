//! The status bar as markup: its text between the two slots, the slots present when empty so the
//! text stays centred, whatever is put in a slot, and the sheet's classes all styled.

use crate::support::hygiene;

use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::prelude::*;
use ds_lint::LintConfig;

fn root(body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material: Material::Window, stylesheet: Inject::Host,
            {body}
        }
    }
}

fn text_only() -> Element {
    root(rsx! {
        StatusBar { text: item_count_text(12, Some("40 GB")) }
    })
}

fn with_slots() -> Element {
    root(rsx! {
        StatusBar {
            text: item_count_text(1, None),
            leading: rsx! {
                span { id: "before", "Trash" }
            },
            trailing: rsx! {
                span { id: "after", "Zoom" }
            },
        }
    })
}

/// A specimen: its name and how it is made.
type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[("text-only", text_only), ("with-slots", with_slots)];

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Lint offences and unstyled classes of every specimen, swept by `golden_hygiene`.
pub(crate) fn hygiene_failures() -> Vec<String> {
    let sheet = ds::stylesheet();
    let config = LintConfig::new(&ds::kits());
    SPECIMENS
        .iter()
        .flat_map(|(name, make)| hygiene::failures(name, &render(*make), sheet, &config))
        .collect()
}

#[test]
fn the_text_sits_between_two_slots_that_are_there_when_empty() {
    let page = render(text_only);
    assert!(page.contains("role=\"status\""), "{page}");
    assert!(page.contains("12 items, 40 GB available"), "{page}");
    let parts: Vec<usize> = [
        "ds-status-bar-leading",
        "ds-status-bar-text",
        "ds-status-bar-trailing",
    ]
    .iter()
    .map(|class| page.find(class).unwrap_or(usize::MAX))
    .collect();
    assert!(
        parts[0] < parts[1] && parts[1] < parts[2] && parts[2] < usize::MAX,
        "{parts:?} in {page}"
    );
}

#[test]
fn a_slot_holds_what_the_app_puts_in_it() {
    let page = render(with_slots);
    assert!(page.contains("1 item<"), "{page}");
    let before = page.find("id=\"before\"").expect("the leading slot");
    let text = page.find("ds-status-bar-text").expect("the text");
    let after = page.find("id=\"after\"").expect("the trailing slot");
    assert!(before < text && text < after, "{page}");
}
