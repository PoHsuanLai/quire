//! The rows of a settings pane on a real Blitz document: a FieldRow leads with a tile, its label
//! keeps its words' width when the window narrows, and a row inside a FormSection keeps its
//! height and vertical padding; a source list's tiles follow the sidebar's size; a source list's
//! glyphs take the ink of a selection that moves; RowMore's press reaches the host first.

use dioxus::prelude::*;
use ds::base::geometry::units::{Point, Px};
use ds::base::vocab::RowState;
use ds::components::app::row_more::RowMore;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::sidebar_model::SidebarSection;
use ds::components::fields::field_row::FieldRow;
use ds::components::forms::icon_tile::TileFace;
use ds::components::overlays::alert_model::{AlertButton, AlertRole};
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::control_size::{ControlSize, SidebarSize};
use ds::style::tokens::hex::Hex;
use ds_blitz::Extent;
use ds_harness::{Backdrop, Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const WIDE: Viewport = Viewport {
    width: 600,
    height: 300,
    scale_percent: 100,
};
const NARROW: Viewport = Viewport {
    width: 320,
    height: 300,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Pane() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            FormSection { title: "General".to_string(),
                FieldRow { label: TextLine::from("Default"), leading: Some(TileFace::Glyph(Icon::Globe, Hex([0x30, 0xb0, 0x50]))),
                    Button { label: "A control wider than the space left beside the label", size: ControlSize::ExtraLarge, onclick: |_| {} }
                }
                FieldRow { label: TextLine::from("Quiet"),
                    span { "On" }
                }
            }
        }
    }
}

/// `app` at the wide window, on the virtual clock.
fn on_virtual(app: fn() -> Element) -> Harness {
    Harness::new(app, HarnessConfig::new(WIDE).with_clock(Clock::Virtual))
}

fn height(harness: &Harness, selector: &str) -> f32 {
    harness.rect(selector).expect(selector).size.height.0
}

fn width(harness: &Harness, selector: &str) -> f32 {
    harness.rect(selector).expect(selector).size.width.0
}

#[test]
fn a_leading_tile_is_the_row_icon_tile() {
    let harness = on_virtual(Pane);
    assert_eq!(width(&harness, ".ds-field-row-leading .ds-icon-tile"), 28.0);
}

#[test]
fn the_label_keeps_its_width_when_the_window_narrows() {
    let mut harness = on_virtual(Pane);
    harness.resize_window(Extent::new(NARROW.width, NARROW.height));
    // "Default" is about 45 px of one line; squeezed, it broke into "De" and "fault".
    assert!(
        width(&harness, ".ds-field-row-title") >= 44.0,
        "the label shrank"
    );
    assert!(
        height(&harness, ".ds-field-row-title") < 20.0,
        "the label broke onto lines"
    );
}

#[test]
fn a_grouped_row_keeps_its_height_and_vertical_padding() {
    let harness = on_virtual(Pane);
    let rows = ".ds-form-section-group > .ds-field-row";
    assert_eq!(height(&harness, &format!("{rows}:nth-child(2)")), 48.0);
    // 40 for the extra-large control and 6 above and below it.
    assert_eq!(height(&harness, &format!("{rows}:nth-child(1)")), 52.0);
}

#[derive(Props, Clone, PartialEq)]
struct SideProps {
    size: SidebarSize,
}

fn side_app(props: SideProps) -> Element {
    let mut here = use_signal(|| "a");
    let items: Vec<ListItem<&'static str>> = [
        ("a", "Inbox", RowLeading::Icon(Icon::Inbox)),
        ("b", "Starred", RowLeading::Icon(Icon::Star)),
        (
            "c",
            "Trash",
            RowLeading::Tile(TileFace::Glyph(Icon::Trash, Hex([0xff, 0x3b, 0x30]))),
        ),
    ]
    .into_iter()
    .map(|(key, title, leading)| {
        ListItem::row(
            key,
            title,
            rsx! {
                Row { title: TextLine::from(title), leading,
                    state: RowState { selection: Selection::of(&here(), &key), ..RowState::default() },
                    onclick: move |_| here.set(key) }
            },
        )
    })
    .collect();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "height:280px;display:flex",
                Sidebar::<&'static str> { label: "Mail", size: props.size, cursor: Some(here()),
                    sections: vec![SidebarSection::List(items)], onselect: move |key| here.set(key) }
            }
        }
    }
}

fn small() -> Element {
    side_app(SideProps {
        size: SidebarSize::Small,
    })
}
fn medium() -> Element {
    side_app(SideProps {
        size: SidebarSize::Medium,
    })
}
fn large() -> Element {
    side_app(SideProps {
        size: SidebarSize::Large,
    })
}

#[test]
fn a_sidebar_tile_follows_the_sidebar_size() {
    type App = fn() -> Element;
    const CASES: &[(&str, App, f32)] = &[
        ("small", small, 20.0),
        ("medium", medium, 24.0),
        ("large", large, 28.0),
    ];
    for &(name, app, side) in CASES {
        let harness = on_virtual(app);
        let tile = harness
            .rect(".ds-row-leading .ds-icon-tile")
            .expect(name)
            .size;
        assert_eq!((tile.width.0, tile.height.0), (side, side), "{name}");
    }
}

/// The darkness of the row at `top`'s glyph column.
fn ink_at(harness: &mut Harness, top: u32) -> u64 {
    let picture = harness.render_over(Backdrop::Scheme).expect("a picture");
    (top..top + 32)
        .flat_map(|y| (18..40).map(move |x| (x, y)))
        .map(|(x, y)| u64::from(255 - picture.get_pixel(x, y)[1]))
        .sum()
}

#[test]
fn a_source_list_glyph_takes_the_ink_of_a_selection_that_moves() {
    let mut harness = on_virtual(medium);
    let row = harness
        .rect(".ds-list-item:nth-child(2)")
        .expect("the second row");
    let at = Point {
        x: Px(row.origin.x.0 + 30.0),
        y: Px(row.origin.y.0 + 10.0),
    };
    let selected_before = ink_at(&mut harness, 12);
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(400));
    // The row that lost the selection draws its glyph dark on the white ground, where a baked
    // white would leave nothing but its words; the row that won draws it white on the accent.
    let (lost, won) = (ink_at(&mut harness, 12), ink_at(&mut harness, 44));
    assert!(
        lost > 5_000,
        "the deselected glyph is white on white: {lost}"
    );
    assert!(
        won > selected_before / 2,
        "the selected row lost its fill: {won}"
    );
    let picture = harness.render_over(Backdrop::Scheme).expect("a picture");
    let glyph = |x: u32, y: u32| picture.get_pixel(x, y)[0];
    let brightest = (44..76)
        .flat_map(|y| (20..38).map(move |x| (x, y)))
        .map(|(x, y)| glyph(x, y))
        .max();
    assert_eq!(
        brightest,
        Some(255),
        "the selected glyph stays dark on the accent"
    );
}

#[allow(non_snake_case)]
fn More() -> Element {
    let mut log = use_signal(String::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            RowMore {
                shown: Some(Shown::Visible),
                on_press: move |()| log.write().push_str("press;"),
                onclick: move |_| log.write().push_str("rect;"),
            }
            p { class: "log", {log()} }
        }
    }
}

#[test]
fn row_more_reports_the_press_before_it_has_measured_itself() {
    let mut harness = Harness::new(More, HarnessConfig::new(WIDE).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(400));
    let at = harness.centre(".ds-row-more").expect("the button");
    harness.send(Input::pointer_move(at));
    harness.send(Input::click(at));
    harness.advance(Duration::from_millis(100));
    let log = harness.text_of(".log").unwrap_or_default();
    assert!(log.starts_with("press;"), "the press comes first: {log}");
    assert!(log.ends_with("rect;"), "the rect follows: {log}");
}

#[allow(non_snake_case)]
fn Asking() -> Element {
    let named = |label: &str, role, id: &str| {
        AlertButton::new(label, role, EventHandler::new(|()| {})).with_common(Common {
            id: Some(id.to_owned()),
            ..Common::default()
        })
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Alert { title: "Turn Bluetooth off?",
                buttons: vec![named("Turn Off", AlertRole::Normal, "turn-off"), named("Cancel", AlertRole::Cancel, "cancel")] }
        }
    }
}

#[test]
fn an_alert_button_carries_the_id_its_host_gave_it() {
    let harness = on_virtual(Asking);
    assert_eq!(harness.text_of("#turn-off").as_deref(), Some("Turn Off"));
    assert_eq!(harness.text_of("#cancel").as_deref(), Some("Cancel"));
}

#[allow(non_snake_case)]
fn WideControl() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            FormSection { title: "Apps".to_string(),
                FieldRow { label: TextLine::from("Apps it may use"),
                    Button { label: "Add an app", onclick: |_| {} }
                    Button { label: "Remove the selected app", onclick: |_| {} }
                    Button { label: "Reset to the defaults", onclick: |_| {} }
                }
            }
        }
    }
}

/// A control wider than the room beside the label shrinks and wraps its parts inside the row
/// (System Settings' behaviour), rather than pushing them past the window's edge.
#[test]
fn a_wide_control_wraps_inside_the_row_instead_of_overflowing() {
    let mut harness = on_virtual(WideControl);
    harness.resize_window(Extent::new(NARROW.width, NARROW.height));
    let control = harness.rect(".ds-field-row-control").expect("control");
    let right = control.origin.x.0 + control.size.width.0;
    assert!(
        right <= NARROW.width as f32,
        "the control ends at {right}, past the window"
    );
    assert!(
        width(&harness, ".ds-field-row-title") >= 44.0,
        "the label shrank"
    );
}

#[allow(non_snake_case)]
fn ShortLabelToggle() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            FormSection { title: "General".to_string(),
                FieldRow { label: TextLine::from("Wi-Fi"),
                    Toggle { label: "Wi-Fi", value: Check::On, onchange: |_| {} }
                }
            }
        }
    }
}

/// A short label does not leave the control floating mid-row: in a setting row the control sits
/// at the trailing edge, inside the row's padding (14 px in a FormSection).
#[test]
fn a_setting_rows_control_sits_at_the_trailing_edge() {
    let mut harness = on_virtual(ShortLabelToggle);
    harness.resize_window(Extent::new(WIDE.width, WIDE.height));
    let row = harness.rect(".ds-field-row").expect("row");
    let control = harness.rect(".ds-field-row-control").expect("control");
    let row_right = row.origin.x.0 + row.size.width.0;
    let control_right = control.origin.x.0 + control.size.width.0;
    assert!(
        (control_right - (row_right - 14.0)).abs() < 0.5,
        "the control ends at {control_right}, the row's padded edge is {}",
        row_right - 14.0
    );
}
