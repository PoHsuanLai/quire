//! AppSwitcher: the Cmd+Tab panel (design/13-BEHAVIOUR-menus-windows.md section 13.3.5;
//! design/20-SURFACES.md section 1.11; design/04-COMPONENTS.md section 43). One row of app
//! icons on the Osd material with the Space gradient, as the OSD card draws it; the selection a rounded square of `--f-pill` that springs from
//! cell to cell, the selected app's name under it. The component only draws and reports: the
//! show delay, the keys, the modifier's release and the MRU order are the shell's.

use crate::components::icon_view::IconView;
use crate::components::switcher_fit::{SwitcherMetrics, fit};
use crate::components::tooltip::{Shown, Tooltip, TooltipKind};
use crate::components::vocab::{PulseKey, Selection};
use crate::geometry::Px;
use crate::icon::external::IconSource;
use crate::icon::family::PlateFamily;
use crate::icon::render::{IconPx, IconSize};
use crate::motion::Anim;
use dioxus::prelude::*;

/// An application in the switcher, by the shell's own id for it (its app id).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AppKey(pub String);

/// Whether a tile is staying or leaving.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TilePresence {
    /// In the row.
    #[default]
    Present,
    /// Quitting (Q): the tile plays `fold` once (`--t-big --e-exit`, held); the shell drops it
    /// from `apps` at `settle(Anim::Fold)`.
    Leaving,
}

/// One application's tile.
#[derive(Debug, Clone, PartialEq)]
pub struct SwitcherApp {
    /// Which application: what `onhover` and `onactivate` report.
    pub key: AppKey,
    /// Its name, shown under the tile while it is selected.
    pub name: String,
    /// Its icon: a glyph, a symbolic icon or its picture. The switcher sizes a glyph itself; an
    /// external icon is stretched to the cell's icon square.
    pub icon: IconSource,
    /// The app plate a glyph sits on, as the dock draws a placeholder (design/08 section 2);
    /// `None` for an app whose own picture is its plate.
    pub plate: Option<PlateFamily>,
    /// Staying or leaving.
    pub presence: TilePresence,
}

impl SwitcherApp {
    /// A present tile with no plate.
    pub fn new(key: impl Into<String>, name: impl Into<String>, icon: IconSource) -> Self {
        SwitcherApp {
            key: AppKey(key.into()),
            name: name.into(),
            icon,
            plate: None,
            presence: TilePresence::Present,
        }
    }
}

/// The switcher panel. `apps` in the shell's order (most recent first), `selected` the one the
/// selection is on. `output` is the width of the output it is shown on: past `output - 64` the
/// icons shrink to `metrics.min_icon` and then the row scrolls so the selection stays in view
/// (`None`: never shrunk). The pointer resting on a tile calls `onhover` with its key (the
/// shell selects it); a click calls `onactivate`. Place it in a transparent Osd root
/// (`Ds { material: Material::Osd, chrome: Some(RootChrome::Transparent), .. }`); it appears
/// with `fade` at `--t-quick`, with no pop.
#[component]
pub fn AppSwitcher(
    apps: Vec<SwitcherApp>,
    selected: AppKey,
    #[props(default)] output: Option<Px>,
    #[props(default)] metrics: SwitcherMetrics,
    onhover: EventHandler<AppKey>,
    onactivate: EventHandler<AppKey>,
) -> Element {
    let at = apps
        .iter()
        .position(|app| app.key == selected)
        .unwrap_or_default();
    let row = fit(apps.len(), at, metrics, output);
    let style = format!(
        "--switcher-cell:{}px;--switcher-icon:{}px;--switcher-gap:{}px;--switcher-view:{}px;--switcher-shift:{}px;--switcher-at:{at}",
        row.cell.0, row.icon.0, metrics.gap.0, row.view.0, row.shift.0
    );
    let size = IconSize::Px(IconPx(row.icon.0.clamp(0.0, 255.0) as u8));
    rsx! {
        div { class: "ds-switcher", role: "listbox", "aria-label": "Applications", style,
            div { class: "ds-frame",
                div { class: "ds-grain" }
            }
            div { class: "ds-switcher-view",
                div { class: "ds-switcher-row",
                    span { class: "ds-switcher-ring", "aria-hidden": "true" }
                    for app in apps {
                        {tile(app.clone(), Selection::of(&app.key, &selected), size, onhover, onactivate)}
                    }
                }
            }
        }
    }
}

/// One tile: its icon, and its name as a Fly under it, up while it is selected.
fn tile(
    app: SwitcherApp,
    selection: Selection,
    size: IconSize,
    onhover: EventHandler<AppKey>,
    onactivate: EventHandler<AppKey>,
) -> Element {
    let leaving = match app.presence {
        TilePresence::Present => None,
        TilePresence::Leaving => PulseKey::rest(Anim::Fold).fired().attrs(),
    };
    let class = match &leaving {
        Some((anim, _)) => format!("ds-switcher-cell {anim}"),
        None => "ds-switcher-cell".to_owned(),
    };
    let alias = leaving.map(|(_, alias)| alias);
    let shown = match selection {
        Selection::Selected => Shown::Visible,
        Selection::Unselected => Shown::Hidden,
    };
    let hovered = app.key.clone();
    let activated = app.key.clone();
    let name = app.name.clone();
    rsx! {
        div {
            key: "{app.key.0}",
            class,
            role: "option",
            "aria-selected": selection.aria(),
            "aria-label": "{name}",
            "data-pulse": alias,
            "data-presence": match app.presence {
                TilePresence::Present => None,
                TilePresence::Leaving => Some("leaving"),
            },
            onpointerenter: move |_| onhover.call(hovered.clone()),
            onclick: move |_| onactivate.call(activated.clone()),
            Tooltip { kind: TooltipKind::Fly, text: app.name, shown,
                span { class: "ds-switcher-icon",
                    IconView { source: app.icon, size, plate: app.plate }
                }
            }
        }
    }
}
