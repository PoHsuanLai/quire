//! The bar gaps' menu controls on a real Blitz document (FINDINGS "Bar gaps", sill Q11): the
//! menu reports the choice under the pointer and every release over it; a press that began
//! outside and is released on an item picks it; a pick calls `onpick` before `onclose`; Escape
//! and an outside click fade the menu out before `onclose`; a bar menu opens with no entrance;
//! a status line is drawn and never selected; a press reports where it happened.

use dioxus::prelude::*;
use ds::{
    Anchor, Anim, Appearance, Availability, Ds, Icon, IconButton, IconButtonVariant, Key, Material,
    Menu, MenuEntrance, MenuEntry, MenuKind, MotionLevel, Point, PointerButton, Press, Px,
    StaggerIndex, Trail, settle,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

/// A document under test.
type App = fn() -> Element;

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn item(value: u8, title: &str, availability: Availability) -> MenuEntry<u8> {
    MenuEntry::Item {
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
        availability,
    }
}

/// A status line, then three choices: Open (0), Pause (1, disabled), Quit (2).
fn entries() -> Vec<MenuEntry<u8>> {
    vec![
        MenuEntry::Info {
            title: "Wired".to_string(),
            detail: Some("192.168.1.4".to_string()),
        },
        item(1, "Open", Availability::Enabled),
        item(2, "Pause", Availability::Disabled),
        item(3, "Quit", Availability::Enabled),
    ]
}

/// Which way the menu under test starts and appears.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Setup {
    open: bool,
    entrance: MenuEntrance,
}

/// A page with an opener that opens the menu on the press (as a bar title does), the menu, and
/// a log of what the menu reported, in order.
#[component]
fn MenuPage(setup: Setup) -> Element {
    let mut open = use_signal(|| setup.open);
    let mut log = use_signal(Vec::<String>::new);
    let mut note = move |line: String| log.with_mut(|log| log.push(line));
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "display:flex; flex-direction:column; height:340px",
                div {
                    class: "opener",
                    style: "width:80px; height:24px",
                    onmousedown: move |_| open.set(true),
                }
                p { class: "log", {log().join(",")} }
            }
            if open() {
                Menu::<u8> {
                    kind: MenuKind::Slim,
                    anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                    entries: entries(),
                    entrance: setup.entrance,
                    on_hover: move |index: Option<usize>| {
                        note(format!("hover:{}", index.map_or("none".to_string(), |i| i.to_string())));
                    },
                    on_release: move |press: Press| note(format!("release:{:?}", press.button)),
                    onpick: move |value: u8| note(format!("pick:{value}")),
                    onclose: move |()| {
                        note("close".to_string());
                        open.set(false);
                    },
                }
            }
        }
    }
}

#[allow(non_snake_case)]
fn OpenMenu() -> Element {
    rsx! { MenuPage { setup: Setup { open: true, entrance: MenuEntrance::Animated } } }
}

#[allow(non_snake_case)]
fn ClosedMenu() -> Element {
    rsx! { MenuPage { setup: Setup { open: false, entrance: MenuEntrance::Animated } } }
}

#[allow(non_snake_case)]
fn InstantMenu() -> Element {
    rsx! { MenuPage { setup: Setup { open: true, entrance: MenuEntrance::Instant } } }
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

/// The centre of the row titled `title`.
fn row(harness: &Harness, title: &str) -> Point {
    let index = ["Open", "Pause", "Quit"]
        .iter()
        .position(|t| *t == title)
        .unwrap_or_else(|| panic!("no row {title}"));
    harness
        // The status line is the menu's first child.
        .centre(&format!(".ds-menu > :nth-child({})", index + 2))
        .unwrap_or_else(|| panic!("{title} is not drawn:\n{}", harness.html()))
}

/// Let the menu mount, measure and place itself.
fn settle_in(harness: &mut Harness) {
    harness.advance(ms(80));
}

#[test]
fn the_menu_reports_the_choice_under_the_pointer() {
    let mut harness = Harness::new(OpenMenu, VIEW);
    settle_in(&mut harness);
    harness.pointer_move(row(&harness, "Open"));
    harness.pointer_move(row(&harness, "Quit"));
    let info = harness.centre(".ds-menu-info").expect("the status line");
    harness.pointer_move(info);
    assert_eq!(log(&harness), "hover:0,hover:2,hover:none");
}

#[test]
fn a_status_line_is_drawn_and_never_selected() {
    let mut harness = Harness::new(OpenMenu, VIEW);
    settle_in(&mut harness);
    assert_eq!(
        harness.text_of(".ds-menu-info-title").as_deref(),
        Some("Wired")
    );
    assert_eq!(
        harness.text_of(".ds-menu-info-detail").as_deref(),
        Some("192.168.1.4")
    );
    assert_eq!(
        harness.count(".ds-menu-item"),
        3,
        "a status line is no item"
    );
    // Up from the first choice wraps to the last, past the status line.
    harness.key(Key::Up);
    assert_eq!(
        harness
            .text_of(".ds-menu-item[*|aria-selected=true] .ds-menu-title")
            .as_deref(),
        Some("Quit")
    );
    harness.key(Key::Down);
    assert_eq!(
        harness
            .text_of(".ds-menu-item[*|aria-selected=true] .ds-menu-title")
            .as_deref(),
        Some("Open")
    );
}

/// design/13 section 13.3.2: while the button is held, release on an enabled item picks it.
#[test]
fn a_press_dragged_onto_an_item_and_released_picks_it() {
    let mut harness = Harness::new(ClosedMenu, VIEW);
    let opener = harness.centre(".opener").expect("the opener");
    harness.pointer_move(opener);
    harness.pointer_down(opener);
    settle_in(&mut harness);
    assert_eq!(harness.count(".ds-menu"), 1, "the press opened the menu");
    let quit = row(&harness, "Quit");
    harness.pointer_move(quit);
    harness.pointer_up(quit);
    harness.advance(ms(20));
    assert_eq!(log(&harness), "hover:2,release:Primary,pick:3,close");
    assert_eq!(harness.count(".ds-menu"), 0, "the pick closed it");
}

/// A drag released on a disabled item picks nothing and closes, through the exit fade.
#[test]
fn a_drag_released_on_a_disabled_item_closes_picking_nothing() {
    let mut harness = Harness::new(ClosedMenu, VIEW);
    let opener = harness.centre(".opener").expect("the opener");
    harness.pointer_move(opener);
    harness.pointer_down(opener);
    settle_in(&mut harness);
    let pause = row(&harness, "Pause");
    harness.pointer_move(pause);
    harness.pointer_up(pause);
    harness.advance(fade() + ms(40));
    assert_eq!(log(&harness), "hover:1,release:Primary,close");
}

/// A click that starts and ends on an item is a click: it picks, and a release after a press
/// inside the menu picks nothing by itself. `onpick` runs before `onclose` (sill Q11).
#[test]
fn a_click_picks_before_it_closes() {
    let mut harness = Harness::new(OpenMenu, VIEW);
    settle_in(&mut harness);
    harness.click(row(&harness, "Open"));
    harness.advance(ms(20));
    assert_eq!(log(&harness), "hover:0,release:Primary,pick:1,close");
}

/// How long the exit fade runs, asked of the motion table.
fn fade() -> Duration {
    settle(
        Anim::MenuOut,
        MotionLevel::Standard,
        StaggerIndex::default(),
    )
}

/// Escape fades the menu out (`data-presence="leaving"`) and calls `onclose` when the fade has
/// settled, not before.
#[test]
fn escape_fades_the_menu_out_before_it_closes() {
    let mut harness = Harness::new(OpenMenu, VIEW);
    settle_in(&mut harness);
    harness.key(Key::Escape);
    assert_eq!(
        harness.attr(".ds-menu", "data-presence").as_deref(),
        Some("leaving")
    );
    harness.advance(fade() - ms(40));
    assert_eq!(log(&harness), "", "still fading");
    assert_eq!(harness.count(".ds-menu"), 1);
    harness.advance(ms(80));
    assert_eq!(log(&harness), "close");
    assert_eq!(harness.count(".ds-menu"), 0);
}

/// An outside click fades the menu out too.
#[test]
fn an_outside_click_fades_the_menu_out_before_it_closes() {
    let mut harness = Harness::new(OpenMenu, VIEW);
    settle_in(&mut harness);
    harness.click(Point {
        x: Px(440.0),
        y: Px(320.0),
    });
    assert_eq!(
        harness.attr(".ds-menu", "data-presence").as_deref(),
        Some("leaving")
    );
    assert_eq!(log(&harness), "");
    harness.advance(fade() + ms(40));
    assert_eq!(log(&harness), "close");
}

/// A bar menu opens at rest, with no entrance; the default plays one.
#[test]
fn a_bar_menu_opens_with_no_entrance() {
    // (app, name, presence on the first frame, data-entrance)
    let cases: [(App, &str, &str, Option<&str>); 2] = [
        (InstantMenu, "instant", "present", Some("instant")),
        (OpenMenu, "animated", "entering", None),
    ];
    for (app, name, presence, entrance) in cases {
        let harness = Harness::new(app, VIEW);
        assert_eq!(
            harness.attr(".ds-menu", "data-presence").as_deref(),
            Some(presence),
            "{name}"
        );
        assert_eq!(
            harness.attr(".ds-menu", "data-entrance").as_deref(),
            entrance,
            "{name}"
        );
    }
}

// ---- Where a press happened ---------------------------------------------------------------

#[allow(non_snake_case)]
fn PressAt() -> Element {
    let mut seen = use_signal(|| "none".to_string());
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Bar,
            div { style: "display:flex; padding:20px 0 0 100px; height:60px",
                IconButton {
                    variant: IconButtonVariant::Status,
                    icon: Icon::Wifi,
                    label: "Network",
                    onclick: move |press: Press| {
                        seen.set(format!("{:?} {} {}", press.button, press.at.x.0, press.at.y.0));
                    },
                }
            }
            p { class: "seen", "{seen}" }
        }
    }
}

/// A press carries its surface-local point, for SNI's `Activate(x, y)` and `ContextMenu(x, y)`.
#[test]
fn a_press_reports_where_it_happened() {
    let mut harness = Harness::new(PressAt, VIEW);
    // (point, button)
    const CASES: &[(f32, f32, PointerButton)] = &[
        (105.0, 25.0, PointerButton::Primary),
        (115.0, 35.0, PointerButton::Secondary),
    ];
    for &(x, y, button) in CASES {
        harness.press(Point { x: Px(x), y: Px(y) }, button);
        assert_eq!(
            harness.text_of(".seen").as_deref(),
            Some(format!("{button:?} {x} {y}").as_str())
        );
    }
}

// ---- The exported measurer ----------------------------------------------------------------

#[allow(non_snake_case)]
fn MeasuredRoot() -> Element {
    // What a shell-host surface's root does, where `ds_native::launch` did not provide it.
    ds_native::measure::provide();
    let probe = ds::use_rect();
    let width = probe.rect().map_or(0.0, |rect| rect.size.width.0);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Bar,
            div { style: "display:flex",
                div { style: "width:123px; height:10px", onmounted: move |event| probe.on_mounted(event) }
            }
            p { class: "width", "{width}" }
        }
    }
}

/// `ds_native::measure::provide()` gives a root the Blitz rect read `use_rect` goes through.
#[test]
fn the_exported_measurer_reads_a_rect() {
    let mut harness = Harness::new(MeasuredRoot, VIEW);
    harness.advance(ms(80));
    assert_eq!(harness.text_of(".width").as_deref(), Some("123"));
}
