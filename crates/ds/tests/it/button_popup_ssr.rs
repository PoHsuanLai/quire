//! A button that opens something says so: `popup` is `aria-haspopup`, and a button without it
//! writes no such attribute.

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::prelude::*;

fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let page = dioxus_ssr::render(&dom);
    let from = page.find("<button").expect("a button");
    let to = page[from..].find("</button>").expect("its end");
    page[from..from + to].to_owned()
}

fn page(popup: Option<HasPopup>) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Button { label: "Sort", popup, shown: Shown::Hidden, onclick: |_| {} }
        }
    }
}

#[allow(non_snake_case)]
fn Menu() -> Element {
    page(Some(HasPopup::Menu))
}
#[allow(non_snake_case)]
fn Dialog() -> Element {
    page(Some(HasPopup::Dialog))
}
#[allow(non_snake_case)]
fn Listbox() -> Element {
    page(Some(HasPopup::Listbox))
}
#[allow(non_snake_case)]
fn Plain() -> Element {
    page(None)
}

#[test]
fn a_button_that_opens_something_writes_its_kind_as_aria_haspopup() {
    assert!(render(Menu).contains("aria-haspopup=\"menu\""));
    assert!(render(Dialog).contains("aria-haspopup=\"dialog\""));
    assert!(render(Listbox).contains("aria-haspopup=\"listbox\""));
}

#[test]
fn a_button_that_opens_nothing_writes_no_aria_haspopup() {
    let html = render(Plain);
    assert!(!html.contains("aria-haspopup"), "{html}");
    assert!(html.contains("aria-expanded=\"false\""), "{html}");
}
