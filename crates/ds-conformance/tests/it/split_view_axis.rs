//! SplitView on a real Blitz document (design/30 section 2.7): panes in a column, panes sized as
//! shares of the view, a split inside a split, and the dim of the panes without the focus.

use dioxus::prelude::*;
use ds::components::chrome::split_view::model::{
    PaneSize, PaneSpec, Share, SplitAxis, SplitPane, UnfocusedPanes,
};
use ds::components::chrome::split_view::view::SplitView;
use ds::prelude::*;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 600,
    scale_percent: 100,
};

/// How the page under test lays its panes out.
#[derive(Clone, Copy)]
enum Layout {
    /// Two panes in a row, the first a quarter of the view, the second a half of what is left.
    RowOfShares,
    /// Two panes in a column, a half and a rest.
    ColumnOfShares,
    /// A row of a fixed pane and a rest holding a column of a share pane and a rest.
    ColumnInRow,
    /// A column of a share pane and a rest holding a row of a share pane and a rest.
    RowInColumn,
    /// Two panes that dim the one without the focus.
    Dimming,
}

thread_local! {
    static LAYOUT: Cell<Layout> = const { Cell::new(Layout::RowOfShares) };
}

fn share(part: f32) -> PaneSpec {
    PaneSpec::sharing(Share(part))
}

fn pane(part: f32, class: &'static str) -> SplitPane {
    SplitPane::new(share(part), rsx! { div { class: "{class}", "{class}" } })
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let body = match LAYOUT.with(Cell::get) {
        Layout::RowOfShares => rsx! {
            SplitView { label: "Row", panes: vec![pane(0.25, "first")],
                SplitView { label: "Inner", panes: vec![pane(0.5, "second")], div { class: "third", "third" } }
            }
        },
        Layout::ColumnOfShares => rsx! {
            SplitView { label: "Column", axis: SplitAxis::Column, panes: vec![pane(0.5, "first")],
                div { class: "second", "second" }
            }
        },
        Layout::ColumnInRow => rsx! {
            SplitView {
                label: "Outer",
                panes: vec![SplitPane::new(PaneSpec::SIDEBAR, rsx! { div { class: "side", "side" } })],
                SplitView { label: "Inner", axis: SplitAxis::Column, panes: vec![pane(0.5, "top")],
                    div { class: "bottom", "bottom" }
                }
            }
        },
        Layout::RowInColumn => rsx! {
            SplitView {
                label: "Outer",
                axis: SplitAxis::Column,
                panes: vec![pane(0.25, "top")],
                SplitView { label: "Inner", panes: vec![pane(0.5, "left")], div { class: "right", "right" } }
            }
        },
        Layout::Dimming => rsx! {
            SplitView {
                label: "Dim",
                unfocused: UnfocusedPanes::Dim,
                panes: vec![SplitPane::new(share(0.5), rsx! { input { class: "one" } })],
                input { class: "two" }
            }
        },
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:600px", {body} }
        }
    }
}

fn start(layout: Layout) -> Harness {
    LAYOUT.with(|cell| cell.set(layout));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(900));
    harness
}

fn size(harness: &Harness, selector: &str) -> (f32, f32) {
    harness
        .rect(selector)
        .map_or((-1.0, -1.0), |rect| (rect.size.width.0, rect.size.height.0))
}

fn near(got: f32, want: f32, what: &str) {
    assert!((got - want).abs() < 1.5, "{what}: {got}, wanted {want}");
}

#[test]
fn share_panes_are_parts_of_the_view_and_the_rest_takes_what_is_left() {
    let harness = start(Layout::RowOfShares);
    near(size(&harness, ".first").0, 200.0, "a quarter of 800");
    let (inner, _) = size(&harness, ".ds-split-rest");
    near(inner, 600.0, "the outer rest");
    near(
        size(&harness, ".second").0,
        300.0,
        "half of the nested view",
    );
    near(size(&harness, ".third").0, 300.0, "the nested rest");
}

#[test]
fn a_column_stacks_its_panes_and_sizes_them_by_height() {
    let harness = start(Layout::ColumnOfShares);
    let (width, height) = size(&harness, ".ds-split-pane");
    near(height, 300.0, "half of 600");
    near(width, 800.0, "as wide as the view");
    let first = harness.rect(".first").expect("first");
    let second = harness.rect(".second").expect("second");
    assert!(
        second.origin.y.0 >= first.origin.y.0 + first.size.height.0 - 1.0,
        "the second pane lies below the first"
    );
    let divider = size(&harness, ".ds-split-divider");
    near(divider.1, 6.0, "the drag zone is 6 px tall");
    assert_eq!(
        harness
            .attr(".ds-split-divider", "aria-orientation")
            .as_deref(),
        Some("horizontal")
    );
}

#[test]
fn dragging_a_column_divider_resizes_by_height_and_keeps_the_share() {
    let mut harness = start(Layout::ColumnOfShares);
    let from = harness.centre(".ds-split-divider").expect("divider");
    harness.send(Input::drag(
        from,
        Point {
            x: from.x,
            y: Px(from.y.0 + 90.0),
        },
        6,
    ));
    harness.advance(Duration::from_millis(600));
    near(
        size(&harness, ".ds-split-pane").1,
        390.0,
        "dragged 90 px down",
    );
    let held = PaneSize::Share(Share(0.65)).resolve(Px(600.0));
    near(held.0, 390.0, "the 390 px is 0.65 of the view");
    let from = harness.centre(".ds-split-divider").expect("divider");
    harness.send(Input::drag(
        from,
        Point {
            x: from.x,
            y: Px(from.y.0 + 400.0),
        },
        6,
    ));
    harness.advance(Duration::from_millis(600));
    near(
        size(&harness, ".ds-split-pane").1,
        540.0,
        "held to nine tenths",
    );
}

#[test]
fn a_double_click_on_a_share_divider_returns_to_the_preferred_share() {
    let mut harness = start(Layout::ColumnOfShares);
    let from = harness.centre(".ds-split-divider").expect("divider");
    harness.send(Input::drag(
        from,
        Point {
            x: from.x,
            y: Px(from.y.0 + 90.0),
        },
        6,
    ));
    harness.advance(Duration::from_millis(600));
    let at = harness.centre(".ds-split-divider").expect("divider");
    harness.send(Input::click(at));
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(900));
    near(size(&harness, ".ds-split-pane").1, 300.0, "back to half");
}

#[test]
fn a_column_split_inside_a_row_split_lays_out_and_drags_on_its_own_axis() {
    let mut harness = start(Layout::ColumnInRow);
    near(size(&harness, ".side").0, 240.0, "the sidebar");
    let top = harness.rect(".top").expect("top");
    let bottom = harness.rect(".bottom").expect("bottom");
    near(
        top.size.width.0,
        560.0,
        "the inner pane spans the outer rest",
    );
    near(
        size(&harness, ".ds-split-rest .ds-split-pane").1,
        300.0,
        "half of the inner view",
    );
    assert!(
        bottom.origin.y.0 > top.origin.y.0 + 290.0,
        "bottom is below top"
    );
    let dividers = harness.count(".ds-split-divider");
    assert_eq!(dividers, 2, "one divider for each split");
    // The inner divider is the second in document order: drag it down.
    let inner = harness
        .centre(".ds-split-rest .ds-split-divider")
        .expect("inner divider");
    harness.send(Input::drag(
        inner,
        Point {
            x: inner.x,
            y: Px(inner.y.0 + 60.0),
        },
        6,
    ));
    harness.advance(Duration::from_millis(600));
    near(
        size(&harness, ".ds-split-rest .ds-split-pane").1,
        360.0,
        "the inner pane grew by the drag",
    );
    near(
        size(&harness, ".side").0,
        240.0,
        "the outer pane did not move",
    );
}

#[test]
fn a_row_split_inside_a_column_split_shares_the_pane_below() {
    let harness = start(Layout::RowInColumn);
    near(
        size(&harness, ".ds-split-pane").1,
        150.0,
        "a quarter of 600",
    );
    near(
        size(&harness, ".left").0,
        400.0,
        "half of the full-width inner view",
    );
    near(size(&harness, ".right").0, 400.0, "the inner rest");
}

#[test]
fn the_pane_without_the_focus_dims_and_the_one_with_it_does_not() {
    let mut harness = start(Layout::Dimming);
    let at = harness.centre(".two").expect("button");
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(600));
    let dim = harness.opacity_of(".ds-split-pane").expect("pane");
    let kept = harness.opacity_of(".ds-split-rest").expect("rest");
    near(dim * 100.0, 60.0, "the first pane at --pane-dim");
    near(kept * 100.0, 100.0, "the pane with the focus");
}
