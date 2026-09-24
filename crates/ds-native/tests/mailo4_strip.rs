//! mailo gaps 4, the strip's press, on a real Blitz document: a strip button's `on_press` fires
//! inside the click, before any measurement, so a host that cannot measure (no layout) still
//! acts; where the rect resolves, the measured `onclick` follows it.

use dioxus::prelude::*;
use ds::components::vocab::{Emphasis, PulseKey, Selection, StaggerIndex};
use ds::{
    ActionId, Anim, Appearance, Ds, HostMeasure, HoverStrip, Icon, ListRow, Material, Measured,
    Point, Presence, Px, Shown, StripAction,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 720,
    height: 240,
    scale_percent: 100,
};

/// Whether the page's subtree can measure an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Layout {
    /// The harness's own measurer: rects resolve.
    Measured,
    /// A measurer that answers `Unknown` for every element, as a host without layout does.
    Absent,
}

/// A measurer with no layout to read.
fn unknown(_: &MountedData) -> Measured {
    Measured::Unknown
}

/// One row whose strip the caller shows, logging the press and the measured click.
#[component]
fn Page(layout: Layout) -> Element {
    if layout == Layout::Absent {
        use_context_provider(|| HostMeasure(unknown));
    }
    let mut log = use_signal(Vec::<String>::new);
    let actions = vec![StripAction {
        id: ActionId("archive".to_string()),
        icon: Icon::Archive,
        label: "Archive".to_string(),
        fly: "Archive → out of Inbox".to_string(),
        onhover: None,
        onclick: EventHandler::new(move |_| log.with_mut(|log| log.push("rect".to_string()))),
    }];
    rsx! {
        ul { class: "list", style: "width:600px; padding:20px; margin:0",
            ListRow {
                selection: Selection::Unselected,
                emphasis: Emphasis::Strong,
                index: StaggerIndex::new(0),
                presence: Presence::Present,
                name: "Dana Okafor",
                via: None,
                subject: "Re: UIDL stability",
                snippet: None,
                time: "",
                tags: rsx! {},
                star: None,
                star_pulse: PulseKey::rest(Anim::StarPop),
                strip: rsx! {
                    HoverStrip {
                        actions,
                        shown: Shown::Visible,
                        on_press: move |id: ActionId| log.with_mut(|log| log.push(format!("press:{}", id.0))),
                    }
                },
                onclick: move |_| log.with_mut(|log| log.push("open".to_string())),
            }
        }
        p { class: "log", "{log.read().join(\",\")}" }
    }
}

#[allow(non_snake_case)]
fn Measuring() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, Page { layout: Layout::Measured } }
    }
}

#[allow(non_snake_case)]
fn Unmeasured() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, Page { layout: Layout::Absent } }
    }
}

/// Click the strip's button once the pop-in has played, and let the measurement run.
fn press(app: fn() -> Element) -> String {
    let mut harness = Harness::new(app, VIEW);
    harness.advance(Duration::from_millis(600));
    // The strip is centred on its row by `translateY(-50%)`, which the layout rect leaves out
    // and hit testing applies (as in `mailo_lists.rs`).
    let lift = harness
        .rect(".ds-strip")
        .map_or(0.0, |strip| strip.size.height.0 / 2.0);
    let at = harness
        .centre(".ds-strip .ds-icon-button")
        .unwrap_or_else(|| panic!("no strip button:\n{}", harness.html()));
    harness.click(Point {
        x: at.x,
        y: Px(at.y.0 - lift),
    });
    harness.advance(Duration::from_millis(200));
    harness.text_of(".log").unwrap_or_default()
}

#[test]
fn with_no_layout_the_press_still_acts() {
    assert_eq!(press(Unmeasured), "press:archive");
}

#[test]
fn with_layout_the_press_comes_first_and_the_rect_follows() {
    assert_eq!(press(Measuring), "press:archive,rect");
}
