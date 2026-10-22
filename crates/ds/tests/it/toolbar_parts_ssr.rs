//! A toolbar's parts as markup: a tooltip that carries its item's key, a title and a centre
//! item standing together, and the sheet's row of buttons.

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::components::chrome::toolbar::model::{ToolbarItem, ToolbarRoom};
use ds::components::chrome::toolbar::view::Toolbar;
use ds::prelude::*;
use ds_style::icon::Icon;

/// The markup of the element that opens with `start`, to the end of the page (the stylesheet
/// before it names every class, so a search must begin at the element).
fn render(app: fn() -> Element, start: &str) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    dom.render_immediate(&mut NoOpMutations);
    let page = dioxus_ssr::render(&dom);
    let from = page.find(start).expect("the element");
    page[from..].to_owned()
}

const TOOLBAR: &str = "<div class=\"ds-toolbar\"";

fn page(content: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, {content} }
    }
}

fn back() -> Vec<ToolbarItem<u8>> {
    vec![
        ToolbarItem::new(0, "Back", Icon::ChevronLeft)
            .with_shortcut(Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('[')])),
    ]
}

#[allow(non_snake_case)]
fn Keyed() -> Element {
    page(rsx! {
        Toolbar::<u8> { leading: back(), room: ToolbarRoom::Fixed(Px(600.0)), onpick: |_| {} }
    })
}

#[allow(non_snake_case)]
fn Together() -> Element {
    page(rsx! {
        Toolbar::<u8> {
            title: Some(TextLine::from("Downloads")),
            center: rsx! { span { class: "view-switch", "Icons" } },
            room: ToolbarRoom::Fixed(Px(600.0)),
            onpick: |_| {},
        }
    })
}

#[allow(non_snake_case)]
fn CentreAlone() -> Element {
    page(rsx! {
        Toolbar::<u8> {
            center: rsx! { span { class: "view-switch", "Icons" } },
            room: ToolbarRoom::Fixed(Px(600.0)),
            onpick: |_| {},
        }
    })
}

#[allow(non_snake_case)]
fn Actions() -> Element {
    page(rsx! {
        SheetActions {
            leading: rsx! { span { class: "t-help", "?" } },
            span { class: "t-cancel", "Cancel" }
            span { class: "t-go", "Go" }
        }
    })
}

#[test]
fn a_toolbar_buttons_tooltip_carries_its_items_key() {
    let html = render(Keyed, TOOLBAR);
    assert!(
        html.contains("aria-description=\"Back  \u{2318}[\""),
        "{html}"
    );
}

#[test]
fn a_title_and_a_centre_item_stand_together_and_a_centre_item_alone_takes_the_titles_place() {
    let both = render(Together, TOOLBAR);
    assert!(both.contains("data-with-center"), "{both}");
    assert!(both.contains("ds-toolbar-heading"), "{both}");
    assert!(
        both.find("ds-toolbar-heading") < both.find("ds-toolbar-center"),
        "{both}"
    );
    assert!(
        both.find("ds-toolbar-center") < both.find("view-switch"),
        "{both}"
    );
    let alone = render(CentreAlone, TOOLBAR);
    assert!(!alone.contains("ds-toolbar-center"), "{alone}");
    assert!(!alone.contains("data-with-center"), "{alone}");
    assert!(alone.contains("view-switch"), "{alone}");
}

#[test]
fn a_sheets_buttons_follow_its_leading_control_in_one_group() {
    let html = render(Actions, "<div class=\"ds-sheet-actions\"");
    assert!(html.contains("role=\"group\""), "{html}");
    assert!(html.find("t-help") < html.find("t-cancel"), "{html}");
    assert!(html.find("t-cancel") < html.find("t-go"), "{html}");
}
