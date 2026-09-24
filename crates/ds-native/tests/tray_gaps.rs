//! The tray gaps (FINDINGS "Tray gaps", sill Q6-Q8) on a real Blitz document: an external
//! symbolic icon takes the text colour and an external image keeps its own; a submenu opens on
//! a rest after the delay and closes on Left; a disabled item is skipped by Down; a right-click
//! on an icon button reports a secondary press.

#[path = "support/probe.rs"]
mod probe;

use dioxus::prelude::*;
use ds::{
    Anchor, Appearance, Availability, Button, ButtonVariant, Ds, ExternalIcon, IconButton,
    IconButtonVariant, IconSize, IconSource, IconUrl, IconView, Key, Material, Menu, MenuEntry,
    MenuKind, Point, PointerButton, Press, Px, Theme, Trail,
};
use ds_native::{Harness, Viewport};
use image::{ImageFormat, Rgba, RgbaImage};
use probe::{distance, keep, modal, pixels, rect};
use std::io::Cursor;
use std::time::Duration;

/// Wide enough for a menu and its submenu side by side.
const VIEW: Viewport = Viewport {
    width: 640,
    height: 360,
    scale_percent: 100,
};

/// The root every quire surface draws inside.
#[component]
fn Root(children: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, {children} }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn centre(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

// ---- External icons ------------------------------------------------------------------------

/// `image` as PNG bytes.
fn png(image: &RgbaImage) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    image
        .write_to(&mut bytes, ImageFormat::Png)
        .expect("a PNG encodes");
    bytes.into_inner()
}

/// A 16 x 16 one-bit mask: the middle 8 x 8 opaque (black, which the dark ink is not),
/// the ring around it fully transparent.
fn mask_png() -> Vec<u8> {
    png(&RgbaImage::from_fn(16, 16, |x, y| {
        let inside = (4..12).contains(&x) && (4..12).contains(&y);
        Rgba([0, 0, 0, if inside { 255 } else { 0 }])
    }))
}

/// A 16 x 16 opaque RGB picture: red.
const PICTURE_RED: [u8; 3] = [220, 30, 40];

fn picture_png() -> Vec<u8> {
    let [r, g, b] = PICTURE_RED;
    png(&RgbaImage::from_fn(16, 16, |_, _| Rgba([r, g, b, 255])))
}

/// Test-only paint: `.ink-swatch` fills with the ink token, `.ink-host` sets the text colour
/// to it, so the icon inside must come out the swatch's colour.
const PROBE_CSS: &str = ".ink-host{color:var(--ink);display:flex;gap:16px;padding:16px}\
    .ink-swatch{width:32px;height:32px;background:var(--ink)}";

#[allow(non_snake_case)]
fn ExternalApp() -> Element {
    let symbolic = IconSource::Symbolic(ExternalIcon {
        url: IconUrl::png(&mask_png()),
        size: IconSize::Large,
    });
    let image = IconSource::Image(ExternalIcon {
        url: IconUrl::png(&picture_png()),
        size: IconSize::Large,
    });
    // Dark, so the ink is near-white and the mask's own black cannot pass for it.
    let dark = Appearance {
        theme: Theme::Dark,
        ..Appearance::default()
    };
    rsx! {
        Ds { appearance: dark, material: Material::Window,
            style { {PROBE_CSS} }
            div { class: "ink-host",
                span { class: "ink-swatch" }
                span { class: "probe-symbolic", IconView { source: symbolic } }
                span { class: "probe-image", IconView { source: image } }
            }
        }
    }
}

#[test]
fn a_symbolic_mask_takes_the_ink_and_an_image_keeps_its_colour() {
    let mut harness = Harness::new(ExternalApp, VIEW);
    // A data: image lands one resolve late (spike S7).
    harness.advance(ms(100));
    let frame = harness.render().expect("renders");
    keep(&frame, "tray-external-icons");
    let ink = modal(&pixels(&frame, rect(&harness, ".ink-swatch"), 4.0));
    let symbolic = rect(&harness, ".probe-symbolic .ds-ext-icon");
    assert_eq!(symbolic.size.width, Px(18.0), "drawn at its own size");
    // The mask's opaque middle (4..12 of 16, so 4.5..13.5 of 18) is the ink; its transparent
    // ring shows the ground.
    let middle = ds::Rect {
        origin: Point {
            x: symbolic.origin.x + Px(6.0),
            y: symbolic.origin.y + Px(6.0),
        },
        size: ds::Size {
            width: Px(6.0),
            height: Px(6.0),
        },
    };
    let painted = modal(&pixels(&frame, middle, 0.0));
    assert!(
        distance(painted, ink) <= 12,
        "the mask's middle is {painted:?}, the ink is {ink:?}"
    );
    let ground = modal(&pixels(&frame, rect(&harness, ".ink-host"), 1.0));
    let corner = frame
        .get_pixel(
            symbolic.origin.x.0 as u32 + 1,
            symbolic.origin.y.0 as u32 + 1,
        )
        .0;
    assert!(
        distance(corner, ground) <= 12,
        "the mask's transparent ring is {corner:?}, the ground is {ground:?}"
    );
    // The image keeps its own red inside the same ink-coloured host.
    let image = rect(&harness, ".probe-image .ds-ext-icon");
    let red = modal(&pixels(&frame, image, 3.0));
    let [r, g, b] = PICTURE_RED;
    assert!(
        distance(red, [r, g, b, 255]) <= 12,
        "the image is {red:?}, not its own red"
    );
    assert!(distance(red, ink) > 60, "the image took the ink");
}

// ---- Presses ----------------------------------------------------------------------------

#[allow(non_snake_case)]
fn PressApp() -> Element {
    let mut seen = use_signal(|| "none".to_string());
    let record = move |press: Press| seen.set(format!("{:?}", press.button));
    rsx! {
        Root {
            div { style: "display:flex; gap:12px; padding:20px",
                IconButton {
                    variant: IconButtonVariant::Tool,
                    icon: ds::Icon::Star,
                    label: "Tray item",
                    id: "tray-0",
                    onclick: record,
                }
                Button { variant: ButtonVariant::Mini, label: "Mini", id: "mini", onclick: record }
            }
            p { class: "seen", "{seen}" }
        }
    }
}

#[test]
fn a_right_click_reports_a_secondary_press() {
    let mut harness = Harness::new(PressApp, VIEW);
    let seen = |harness: &Harness| harness.text_of(".seen").unwrap_or_default();
    assert_eq!(
        harness.attr(".ds-icon-button", "id").as_deref(),
        Some("tray-0")
    );
    let icon = centre(&harness, "#tray-0");
    // (button, what the handler heard)
    const CASES: &[(PointerButton, &str)] = &[
        (PointerButton::Secondary, "Secondary"),
        (PointerButton::Primary, "Primary"),
        (PointerButton::Middle, "Middle"),
    ];
    for &(button, want) in CASES {
        harness.press(icon, button);
        assert_eq!(seen(&harness), want, "{button:?} on the icon button");
    }
    let mini = centre(&harness, "#mini");
    harness.press(mini, PointerButton::Secondary);
    assert_eq!(seen(&harness), "Secondary", "a right-click on a Button");
}

// ---- Submenus and disabled items ---------------------------------------------------------

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

fn entries() -> Vec<MenuEntry<u8>> {
    vec![
        item(1, "Open", Availability::Enabled),
        item(2, "Pause", Availability::Disabled),
        item(3, "Quit", Availability::Enabled),
        MenuEntry::Submenu {
            title: "More".to_string(),
            tile: None,
            availability: Availability::Enabled,
            children: vec![
                item(10, "About", Availability::Enabled),
                item(11, "Help", Availability::Enabled),
            ],
        },
    ]
}

#[allow(non_snake_case)]
fn MenuApp() -> Element {
    let mut picked = use_signal(|| 0u8);
    let mut open = use_signal(|| true);
    rsx! {
        Root {
            // The overlay bounds are the root's content box: make it the viewport's height, so
            // the pointer can reach a menu anywhere on it (FINDINGS "Gallery fixes B").
            div { style: "height:340px",
                p { class: "picked", "{picked}" }
            }
            if open() {
                Menu::<u8> {
                    kind: MenuKind::Context,
                    anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                    entries: entries(),
                    onpick: move |value| picked.set(value),
                    onclose: move |()| open.set(false),
                }
            }
        }
    }
}

/// The selected row's title.
fn selected(harness: &Harness) -> String {
    harness
        .text_of(".ds-menu-item[*|aria-selected=true]")
        .unwrap_or_default()
}

const SUBMENU: &str = ".ds-menu[*|data-depth]";
const PARENT: &str = ".ds-menu-item[*|aria-haspopup=true]";

#[test]
fn a_disabled_item_is_skipped_by_down_and_ignores_a_click() {
    let mut harness = Harness::new(MenuApp, VIEW);
    harness.advance(ms(80));
    assert_eq!(
        harness
            .attr(".ds-menu-item[*|aria-disabled=true]", "aria-disabled")
            .as_deref(),
        Some("true")
    );
    assert_eq!(selected(&harness), "Open");
    harness.key(Key::Down);
    assert_eq!(
        selected(&harness),
        "Quit",
        "Down skipped the disabled Pause"
    );
    harness.key(Key::Up);
    assert_eq!(selected(&harness), "Open", "Up skipped it too");
    let disabled = centre(&harness, ".ds-menu-item[*|aria-disabled=true]");
    harness.click(disabled);
    assert_eq!(
        harness.text_of(".picked").as_deref(),
        Some("0"),
        "nothing picked"
    );
    assert_eq!(harness.count(".ds-menu"), 1, "the menu stayed open");
}

#[test]
fn a_rest_opens_the_submenu_after_the_delay_and_left_closes_it() {
    let mut harness = Harness::new(MenuApp, VIEW);
    harness.advance(ms(80));
    let parent = centre(&harness, PARENT);
    harness.pointer_move(parent);
    assert_eq!(
        selected(&harness),
        "More",
        "the pointer highlights the parent"
    );
    harness.advance(ms(120));
    assert_eq!(
        harness.count(SUBMENU),
        0,
        "not open before the 200 ms delay"
    );
    harness.advance(ms(250));
    assert_eq!(
        harness.count(SUBMENU),
        1,
        "open after the delay:\n{}",
        harness.html()
    );
    assert_eq!(
        harness.attr(PARENT, "aria-expanded").as_deref(),
        Some("true")
    );
    // Beside the menu, its first row level with the parent row.
    let (menu, sub, row) = (
        rect(&harness, ".ds-menu:not([*|data-depth])"),
        rect(&harness, SUBMENU),
        rect(&harness, PARENT),
    );
    assert!(
        (sub.origin.x.0 - (menu.origin.x.0 + menu.size.width.0 + 2.0)).abs() <= 1.0,
        "the submenu is 2 px right of the menu: {sub:?} beside {menu:?}"
    );
    assert!(
        (sub.origin.y.0 - (row.origin.y.0 - 5.0)).abs() <= 1.0,
        "the submenu's top is the parent row's top less 5: {sub:?} for {row:?}"
    );
    harness.key(Key::Left);
    assert_eq!(harness.count(SUBMENU), 0, "Left closed the submenu");
    assert_eq!(harness.count(".ds-menu"), 1, "and only the submenu");
}

#[test]
fn right_opens_the_submenu_at_once_and_its_item_picks() {
    let mut harness = Harness::new(MenuApp, VIEW);
    harness.advance(ms(80));
    for _ in 0..2 {
        harness.key(Key::Down);
    }
    assert_eq!(selected(&harness), "More");
    harness.key(Key::Right);
    harness.advance(ms(120));
    assert_eq!(
        harness.count(SUBMENU),
        1,
        "Right opened it without the delay"
    );
    // The keyboard opened it, so it has the focus: Down moves inside it, Enter picks.
    harness.key(Key::Down);
    harness.key(Key::Enter);
    assert_eq!(harness.text_of(".picked").as_deref(), Some("11"));
    assert_eq!(
        harness.count(".ds-menu"),
        0,
        "picking closed the whole menu"
    );
}

#[test]
fn escape_in_a_keyboard_submenu_closes_one_level() {
    let mut harness = Harness::new(MenuApp, VIEW);
    harness.advance(ms(80));
    for _ in 0..2 {
        harness.key(Key::Down);
    }
    harness.key(Key::Enter);
    harness.advance(ms(120));
    assert_eq!(
        harness.count(SUBMENU),
        1,
        "Enter on a parent opens its submenu"
    );
    harness.key(Key::Escape);
    assert_eq!(harness.count(SUBMENU), 0, "Escape closed the submenu");
    assert_eq!(harness.count(".ds-menu"), 1, "the menu stays");
    // The menu takes the focus back a frame later.
    harness.advance(ms(80));
    harness.key(Key::Escape);
    // It fades out first (`Anim::MenuOut`, bar gaps), then closes.
    assert_eq!(
        harness.attr(".ds-menu", "data-presence").as_deref(),
        Some("leaving"),
        "a second Escape starts the menu's exit"
    );
    harness.advance(
        ds::settle(
            ds::Anim::MenuOut,
            ds::MotionLevel::Standard,
            ds::StaggerIndex::default(),
        ) + ms(40),
    );
    assert_eq!(
        harness.count(".ds-menu"),
        0,
        "a second Escape closes the menu"
    );
}
