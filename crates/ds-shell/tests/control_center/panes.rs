//! PaneSwitcher's specimens: at rest on each pane, a grid of tiles and a list of networks.

use dioxus::prelude::*;
use ds::{
    Accessory, Appearance, Check, Ds, Icon, Inject, List, ListItem, Material, Pane, PaneSwitcher,
    Row, RowLeading, RowSize,
};
use ds_shell::{Chevron, ModuleGrid, ModuleState, ModuleTile, TileSpan};

#[derive(Props, Clone, PartialEq)]
pub struct PaneProps {
    pub shown: Pane,
}

pub const CASES: [(Pane, &str); 2] = [(Pane::Root, "panes-root"), (Pane::Detail, "panes-detail")];

/// A switcher at rest on `shown`, mounted there.
pub fn panes(props: PaneProps) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, stylesheet: Inject::Host,
            PaneSwitcher {
                shown: props.shown,
                root: rsx! {
                    ModuleGrid {
                        ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", status: "Home", state: ModuleState::On, chevron: Chevron::Detail, onclick: |_| {}, on_detail: |_| {} }
                        ModuleTile { glyph: Icon::Moon, title: "Focus", state: ModuleState::Off, onclick: |_| {} }
                        ModuleTile { glyph: Icon::Play, title: "Now Playing", state: ModuleState::Off, span: TileSpan::Full, onclick: |_| {} }
                    }
                },
                detail: rsx! {
                    List::<&'static str> {
                        label: "Networks",
                        items: vec![ListItem::row("Home", "Home", rsx! { Row { leading: RowLeading::Icon(Icon::Wifi), title: "Home", accessory: Accessory::Check(Check::On), size: RowSize::Settings, onclick: |_| {} } })],
                    }
                },
            }
        }
    }
}
