//! MenuBar: the model's six menus, two of them drawn open as rows.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::{Anchor, BarCommand, BarMenu, BarSection, Flow, Menu, MenuBarModel, MenuPlacement, Point};

/// One menu of the model as an inline `Menu`.
#[component]
fn Open(menu: BarMenu<BarCommand>) -> Element {
    rsx! {
        div { class: "g-col",
            span { class: "g-name", "{menu.title}" }
            div { class: "g-menu-card",
                Menu::<BarCommand> {
                    placement: MenuPlacement::Bar,
                    anchor: Anchor::Point(Point::default()),
                    items: menu.items,
                    flow: Flow::Inline,
                    onpick: |_| {},
                    onclose: |_| {},
                }
            }
        }
    }
}

/// The MenuBar section.
#[component]
pub fn MenuBarSection() -> Element {
    let model = MenuBarModel::standard("Notes", |command| command);
    let titles: Vec<String> = model
        .menus()
        .iter()
        .map(|menu| menu.title.clone())
        .collect();
    let pick = |section| model.menu(section).cloned();
    rsx! {
        Section { title: "MenuBar", note: "The model of an app's bar: App, File, Edit, View, Window and Help with the standard items and their standard keys, Help led by its search; Command-? opens Help. The bar itself is the shell's; here two of its menus are drawn as the one Menu draws them.",
            p { class: "g-code", "{titles.join(\" | \")}" }
            div { class: "g-row g-row-top",
                if let Some(menu) = pick(BarSection::Edit) {
                    Open { menu }
                }
                if let Some(menu) = pick(BarSection::App) {
                    Open { menu }
                }
                if let Some(menu) = pick(BarSection::Help) {
                    Open { menu }
                }
            }
        }
    }
}
