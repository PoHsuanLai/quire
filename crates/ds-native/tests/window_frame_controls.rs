//! The window frame's controls on a real Blitz document, against a stub `HostWindow` that logs
//! what it is asked (FINDINGS "Window frame"): a titlebar drag moves once past the threshold and a
//! plain click does not; a double-click zooms; the green light zooms; a long press on it opens the
//! tiling menu, whose Fill maximizes and whose placements the host cannot make are unavailable; a
//! press on a light never starts a move; an edge resizes; a maximized window neither moves nor
//! resizes and its green light restores; the lights are reached by Tab, ArrowDown opens the menu
//! and Escape closes it.

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, HostWindow, Key, Material, Maximized, Point, Px, ResizeEdge, Support,
    TileError, TrafficLights, WindowFrame, WindowState, WindowTile, Zoom, use_window_host_provider,
};
use ds_native::{Harness, Viewport};
use std::rc::Rc;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

/// A host that logs every request and can only maximize.
struct Stub {
    log: Signal<Vec<String>>,
    maximized: Maximized,
}

impl Stub {
    fn note(&self, line: String) {
        let mut log = self.log;
        log.with_mut(|log| log.push(line));
    }
}

impl HostWindow for Stub {
    fn begin_move(&self) {
        self.note("move".to_owned());
    }
    fn begin_resize(&self, edge: ResizeEdge) {
        self.note(format!("resize:{}", edge.slug()));
    }
    fn zoom(&self, zoom: Zoom) {
        self.note(format!("zoom:{zoom:?}"));
    }
    fn minimize(&self) {
        self.note("minimize".to_owned());
    }
    fn close(&self) {
        self.note("close".to_owned());
    }
    fn tile(&self, tile: WindowTile) -> Result<(), TileError> {
        self.note(format!("tile:{tile:?}"));
        match self.supports(tile) {
            Support::Yes => Ok(()),
            Support::No => Err(TileError::Unsupported),
        }
    }
    fn supports(&self, tile: WindowTile) -> Support {
        match tile {
            WindowTile::Fill => Support::Yes,
            WindowTile::LeftHalf | WindowTile::RightHalf | WindowTile::Centre => Support::No,
        }
    }
    fn state(&self) -> WindowState {
        WindowState {
            maximized: self.maximized,
            ..WindowState::default()
        }
    }
}

/// A framed window whose host is the stub, in `maximized`.
fn framed(maximized: Maximized) -> Element {
    let log = use_signal(Vec::<String>::new);
    use_window_host_provider(move || Rc::new(Stub { log, maximized }));
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Window,
            window: WindowFrame::titlebar("Inbox", TrafficLights::Shown),
            p { class: "log", {log().join(",")} }
        }
    }
}

#[allow(non_snake_case)]
fn Normal() -> Element {
    framed(Maximized::Off)
}

#[allow(non_snake_case)]
fn Zoomed() -> Element {
    framed(Maximized::On)
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn at(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

fn start(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(ms(50));
    harness
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

/// The titlebar's empty middle, clear of the lights, the title and the top edge.
const BAR: Point = Point {
    x: Px(400.0),
    y: Px(16.0),
};

/// Press at `from`, move through `path`, release at the last point.
fn drag(harness: &mut Harness, from: Point, path: &[Point]) {
    harness.pointer_move(from);
    harness.pointer_down(from);
    for &point in path {
        harness.pointer_move(point);
    }
    harness.pointer_up(path.last().copied().unwrap_or(from));
    harness.advance(ms(30));
}

#[test]
fn a_titlebar_drag_moves_once_past_the_threshold() {
    let mut harness = start(Normal);
    let path = [
        at(402.0, 16.0),
        at(403.0, 18.0),
        at(410.0, 16.0),
        at(430.0, 22.0),
    ];
    drag(&mut harness, BAR, &path);
    assert_eq!(log(&harness), "move");
}

#[test]
fn a_plain_click_on_the_titlebar_does_not_move() {
    let mut harness = start(Normal);
    harness.click(BAR);
    harness.advance(ms(600));
    drag(&mut harness, BAR, &[at(403.0, 17.0), at(404.0, 13.0)]);
    assert_eq!(
        log(&harness),
        "",
        "a click, then a press that stays within 4 px"
    );
}

#[test]
fn a_double_click_on_the_titlebar_zooms() {
    let mut harness = start(Normal);
    harness.click(BAR);
    harness.advance(ms(60));
    harness.click(BAR);
    harness.advance(ms(60));
    assert_eq!(log(&harness), "zoom:Toggle");
}

#[test]
fn the_green_light_zooms_and_the_others_close_and_minimize() {
    let mut harness = start(Normal);
    for light in ["zoom", "minimize", "close"] {
        let point = centre(&harness, &format!(".ds-light[*|data-light={light}]"));
        harness.click(point);
        harness.advance(ms(60));
    }
    assert_eq!(log(&harness), "zoom:Toggle,minimize,close");
}

#[test]
fn a_press_on_a_light_does_not_begin_a_move() {
    let mut harness = start(Normal);
    for light in ["close", "minimize", "zoom"] {
        let from = centre(&harness, &format!(".ds-light[*|data-light={light}]"));
        let path = [
            at(from.x.0 + 6.0, from.y.0),
            at(from.x.0 + 40.0, from.y.0 + 3.0),
        ];
        drag(&mut harness, from, &path);
    }
    assert!(!log(&harness).contains("move"), "{}", log(&harness));
}

#[test]
fn a_double_click_on_a_light_does_not_zoom_the_titlebar_way() {
    let mut harness = start(Normal);
    let close = centre(&harness, ".ds-light[*|data-light=close]");
    harness.click(close);
    harness.advance(ms(60));
    harness.click(close);
    harness.advance(ms(60));
    assert_eq!(log(&harness), "close,close");
}

#[test]
fn a_long_press_on_the_green_light_opens_the_menu_and_fill_maximizes() {
    let mut harness = start(Normal);
    let green = centre(&harness, ".ds-light[*|data-light=zoom]");
    harness.pointer_move(green);
    harness.pointer_down(green);
    harness.advance(ms(600));
    assert_eq!(harness.count(".ds-menu"), 1, "{}", harness.html());
    assert_eq!(
        harness
            .attr(".ds-light[*|data-light=zoom]", "aria-expanded")
            .as_deref(),
        Some("true")
    );
    harness.pointer_up(green);
    harness.advance(ms(60));
    assert_eq!(
        log(&harness),
        "",
        "the release that ends the hold does not zoom"
    );
    let fill = centre(&harness, ".ds-menu .ds-menu-item");
    harness.click(fill);
    harness.advance(ms(300));
    assert_eq!(log(&harness), "zoom:Maximize");
    assert_eq!(harness.count(".ds-menu"), 0);
}

#[test]
fn a_placement_the_host_cannot_make_is_unavailable() {
    let mut harness = start(Normal);
    let green = centre(&harness, ".ds-light[*|data-light=zoom]");
    harness.press(green, ds::PointerButton::Secondary);
    harness.advance(ms(60));
    // The header is the menu's first child; the rows follow it.
    let rows: Vec<(String, Option<String>)> = (2..=5)
        .map(|n| {
            let row = format!(".ds-menu > :nth-child({n})");
            let title = harness
                .text_of(&format!("{row} .ds-menu-title"))
                .unwrap_or_default();
            (title, harness.attr(&row, "aria-disabled"))
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("Fill".to_owned(), None),
            ("Left half".to_owned(), Some("true".to_owned())),
            ("Right half".to_owned(), Some("true".to_owned())),
            ("Centre".to_owned(), Some("true".to_owned())),
        ],
        "{}",
        harness.html()
    );
    let left = centre(&harness, ".ds-menu > :nth-child(3)");
    harness.click(left);
    harness.advance(ms(300));
    assert_eq!(log(&harness), "", "an unavailable row places nothing");
}

#[test]
fn resting_on_the_green_light_opens_the_menu_and_passing_over_it_does_not() {
    let mut harness = start(Normal);
    let green = centre(&harness, ".ds-light[*|data-light=zoom]");
    harness.pointer_move(green);
    harness.advance(ms(300));
    harness.pointer_move(BAR);
    harness.advance(ms(700));
    assert_eq!(harness.count(".ds-menu"), 0, "passed over");
    harness.pointer_move(green);
    harness.advance(ms(900));
    assert_eq!(harness.count(".ds-menu"), 1, "rested on");
}

#[test]
fn an_edge_resizes_from_that_edge() {
    let mut harness = start(Normal);
    for (point, _) in [(at(478.0, 120.0), "right"), (at(2.0, 238.0), "bottom-left")] {
        harness.pointer_move(point);
        harness.pointer_down(point);
        harness.pointer_up(point);
        harness.advance(ms(30));
    }
    assert_eq!(log(&harness), "resize:right,resize:bottom-left");
}

#[test]
fn a_maximized_window_neither_moves_nor_resizes_and_its_green_light_restores() {
    let mut harness = start(Zoomed);
    drag(&mut harness, BAR, &[at(420.0, 16.0), at(440.0, 30.0)]);
    assert_eq!(harness.count(".ds-resize-edge"), 0);
    assert_eq!(
        harness
            .attr(".ds-light[*|data-light=zoom]", "aria-label")
            .as_deref(),
        Some("Restore")
    );
    assert_eq!(
        harness.attr(".ds-titlebar", "data-window").as_deref(),
        Some("maximized")
    );
    let green = centre(&harness, ".ds-light[*|data-light=zoom]");
    harness.click(green);
    harness.advance(ms(60));
    assert_eq!(log(&harness), "zoom:Toggle");
}

#[test]
fn the_lights_take_tab_arrow_down_opens_the_menu_and_escape_closes_it() {
    let mut harness = start(Normal);
    harness.key(Key::Tab);
    assert!(
        harness.is_focused(".ds-light[*|data-light=close]"),
        "{}",
        harness.html()
    );
    harness.key(Key::Tab);
    harness.key(Key::Tab);
    assert!(harness.is_focused(".ds-light[*|data-light=zoom]"));
    harness.key(Key::Down);
    harness.advance(ms(60));
    assert_eq!(harness.count(".ds-menu"), 1);
    harness.key(Key::Escape);
    harness.advance(ms(400));
    assert_eq!(harness.count(".ds-menu"), 0);
    assert_eq!(log(&harness), "");
}
