//! Column on a real Blitz document: a zero gap lets children touch, a fill column is as tall as
//! its parent, and a fixed column is as tall as a control of its size.

use dioxus::prelude::*;
use ds::components::chrome::column::model::{ColumnExtent, ColumnGap};
use ds::components::chrome::column::view::Column;
use ds::prelude::*;
use ds::style::tokens::control_size::ControlSize;
use ds::style::tokens::spacing::SpacingToken;
use ds_harness::{Clock, Harness, HarnessConfig, Query, Viewport};
use std::cell::Cell;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 300,
    scale_percent: 100,
};

thread_local! {
    static LAYOUT: Cell<Layout> = const { Cell::new(Layout::Gapless) };
}

#[derive(Clone, Copy)]
enum Layout {
    Gapless,
    Stepped,
    Filling,
    Fixed,
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let body = match LAYOUT.with(Cell::get) {
        Layout::Gapless => rsx! {
            Column { gap: ColumnGap::None, div { class: "a", style: "height:20px" } div { class: "b", style: "height:20px" } }
        },
        Layout::Stepped => rsx! {
            Column { gap: SpacingToken::S20, div { class: "a", style: "height:20px" } div { class: "b", style: "height:20px" } }
        },
        Layout::Filling => rsx! {
            Column { extent: ColumnExtent::Fill, gap: ColumnGap::None, div { class: "a", style: "height:20px" } }
        },
        Layout::Fixed => rsx! {
            Column { extent: ColumnExtent::Fixed(ControlSize::Large), div { class: "a" } }
        },
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { class: "parent", style: "height:200px;width:300px", {body} }
        }
    }
}

fn start(layout: Layout) -> Harness {
    LAYOUT.with(|cell| cell.set(layout));
    Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn spacing(harness: &Harness) -> f32 {
    let a = harness.rect(".a").expect("a");
    let b = harness.rect(".b").expect("b");
    b.origin.y.0 - (a.origin.y.0 + a.size.height.0)
}

#[test]
fn a_gap_of_none_lets_children_touch_and_a_step_spaces_them() {
    assert_eq!(spacing(&start(Layout::Gapless)), 0.0);
    assert_eq!(spacing(&start(Layout::Stepped)), 20.0);
}

#[test]
fn a_fill_column_is_as_tall_as_its_parent() {
    let harness = start(Layout::Filling);
    let column = harness.rect(".ds-column").expect("column");
    assert_eq!(column.size.height.0, 200.0);
    assert_eq!(column.size.width.0, 300.0);
}

#[test]
fn a_fixed_column_is_as_tall_as_a_control_of_its_size() {
    let harness = start(Layout::Fixed);
    let column = harness.rect(".ds-column").expect("column");
    assert_eq!(
        column.size.height.0,
        ControlSize::Large.scale().height.0 as f32
    );
}
