//! ModulePanel on a real Blitz document (sill FINDINGS Q100): a `LevelControl` inside a panel
//! on the grid takes a press and a drag (the panel adds no role or handler in its way), and the
//! panel spans both columns inside the grid's padding.

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Fraction, Icon, LevelControl, LevelGlyph, Material, ModuleGrid, ModulePanel,
    ModuleState, ModuleTile, Muting, Point, Px, Rect, TileSpan,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

static VALUE: GlobalSignal<Fraction> = Signal::global(|| Fraction(200));

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

/// A 320 px panel's grid: two tiles, then the Sound module.
#[allow(non_snake_case)]
fn Sound() -> Element {
    let percent = VALUE().0 / 10;
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { style: "width:320px",
                ModuleGrid {
                    ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", state: ModuleState::On, onclick: |_| {} }
                    ModuleTile { glyph: Icon::Bluetooth, title: "Bluetooth", state: ModuleState::Off, onclick: |_| {} }
                    ModulePanel {
                        glyph: Icon::Volume2,
                        title: "Speakers",
                        trailing: rsx! { "{percent}%" },
                        LevelControl {
                            label: "Volume",
                            value: VALUE(),
                            glyph: LevelGlyph::Volume(Muting::Audible),
                            onchange: move |next| *VALUE.write() = next,
                        }
                    }
                }
            }
        }
    }
}

const TICK: Duration = Duration::from_millis(50);

fn rect(harness: &Harness, selector: &str) -> Rect {
    harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

fn level(harness: &mut Harness) -> u16 {
    harness.within(|| VALUE.peek().0)
}

#[test]
fn a_level_inside_a_panel_on_the_grid_takes_a_press_and_a_drag() {
    let mut harness = Harness::new(Sound, VIEW);
    harness.within(|| *VALUE.write() = Fraction(200));
    harness.advance(Duration::from_millis(400));
    let panel = rect(&harness, ".ds-module-panel");
    assert_eq!(
        (panel.origin.x.0, panel.size.width.0),
        (12.0, 296.0),
        "the panel spans both columns inside the grid's padding"
    );
    let rail = rect(&harness, ".ds-level-rail");
    let at = |share: f32| Point {
        x: Px(rail.origin.x.0 + rail.size.width.0 * share),
        y: Px(rail.origin.y.0 + rail.size.height.0 / 2.0),
    };
    harness.pointer_down(at(0.3));
    harness.advance(TICK);
    let pressed = level(&mut harness);
    harness.pointer_move(at(0.8));
    harness.advance(TICK);
    let dragged = level(&mut harness);
    harness.pointer_up(at(0.8));
    harness.advance(TICK);
    assert!(pressed.abs_diff(300) <= 20, "the press lands: {pressed}");
    assert!(dragged.abs_diff(800) <= 20, "the drag follows: {dragged}");
    assert_eq!(
        harness.text_of(".ds-module-panel-trailing").as_deref(),
        Some(format!("{}%", dragged / 10).as_str()),
        "the header's trailing slot follows the level"
    );
}
