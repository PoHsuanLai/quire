//! ModuleTile's specimens: every state in each span, in both schemes, and the chevron's three
//! forms (live, inert, absent).

use dioxus::prelude::*;
use ds::{
    Appearance, Chevron, Ds, Expanded, Icon, Inject, Material, ModuleState, ModuleTile, Theme,
    TileSpan,
};

/// One tile specimen.
#[derive(Props, Clone, PartialEq)]
pub struct TileCase {
    pub theme: Theme,
    pub state: ModuleState,
    pub span: TileSpan,
    pub chevron: Chevron,
    pub detail: Expanded,
    pub live: Live,
}

/// Whether the specimen's chevron has a handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Live {
    Handled,
    Unhandled,
}

pub const STATES: [(ModuleState, &str); 3] = [
    (ModuleState::Off, "off"),
    (ModuleState::On, "on"),
    (ModuleState::Busy, "busy"),
];

pub const SPANS: [(TileSpan, &str); 2] = [(TileSpan::Half, "half"), (TileSpan::Full, "full")];

pub const THEMES: [(Theme, &str); 2] = [(Theme::Light, "light"), (Theme::Dark, "dark")];

/// Every state in each span and scheme, with a live chevron: `tile-<state>-<span>-<scheme>`.
pub fn grid() -> Vec<(String, TileCase)> {
    let mut cases = Vec::new();
    for (theme, scheme) in THEMES {
        for (state, word) in STATES {
            for (span, width) in SPANS {
                cases.push((
                    format!("tile-{word}-{width}-{scheme}"),
                    TileCase {
                        theme,
                        state,
                        span,
                        chevron: Chevron::Detail,
                        detail: Expanded::Closed,
                        live: Live::Handled,
                    },
                ));
            }
        }
    }
    cases
}

/// The chevron's other forms, in light: none, inert, and open.
pub fn chevrons() -> Vec<(String, TileCase)> {
    let base = TileCase {
        theme: Theme::Light,
        state: ModuleState::Off,
        span: TileSpan::Half,
        chevron: Chevron::Detail,
        detail: Expanded::Closed,
        live: Live::Handled,
    };
    vec![
        (
            "tile-no-chevron".to_owned(),
            TileCase {
                chevron: Chevron::None,
                ..base.clone()
            },
        ),
        (
            "tile-inert-chevron".to_owned(),
            TileCase {
                live: Live::Unhandled,
                ..base.clone()
            },
        ),
        (
            "tile-open-chevron".to_owned(),
            TileCase {
                detail: Expanded::Open,
                ..base
            },
        ),
    ]
}

/// A tile in a Popover root of its scheme.
pub fn tile(case: TileCase) -> Element {
    let on_detail = match case.live {
        Live::Handled => Some(EventHandler::new(|_| {})),
        Live::Unhandled => None,
    };
    rsx! {
        Ds {
            appearance: Appearance { theme: case.theme, ..Appearance::default() },
            material: Material::Popover,
            stylesheet: Inject::Host,
            ModuleTile {
                glyph: Icon::Wifi,
                title: "Wi-Fi",
                status: "Home",
                state: case.state,
                chevron: case.chevron,
                span: case.span,
                onclick: |_| {},
                on_detail,
                expanded: case.detail,
            }
        }
    }
}
