//! Files dragged in from outside the window (attachments dropped onto the
//! composer), through the same `ds::file_drop::board::FileDropBoard` and hit test the window's hook feeds: a target
//! lights with `data-drop="target"` under the pointer and `accepts` elsewhere, the innermost
//! target under a release hears its `ondrop` with the paths, and a URL lights nothing.

use dioxus::prelude::*;
use ds::file_drop::drag::{DropAcceptance, FileDrag, FileDragInput, FileDrop, Offer};
use ds::file_drop::hook::use_file_drop;
use ds::prelude::*;
use ds_harness::{Driver, Harness, Input, Query, Viewport};
use std::path::PathBuf;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 320,
    scale_percent: 100,
};

/// A composer taking files, a chip inside it that takes its own, and plain space outside both.
#[allow(non_snake_case)]
fn Window() -> Element {
    let log = use_signal(Vec::<String>::new);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:10px",
                Composer { log }
                div { class: "outside", style: "height:60px; margin-top:10px", "Elsewhere" }
                p { class: "log", {log().join(";")} }
            }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn Composer(log: Signal<Vec<String>>) -> Element {
    let drop = use_file_drop(move |files: FileDrop| {
        log.with_mut(|log| log.push(format!("composer:{}", names(&files.paths))));
    });
    rsx! {
        div { class: "composer", style: "height:120px; padding:10px",
            "data-drop": drop.drop_attr(),
            onmounted: move |event| drop.mounted(event),
            p { class: "composer-drag", style: "margin:0", {describe(&drop.drag())} }
            Chip { log }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn Chip(log: Signal<Vec<String>>) -> Element {
    let drop = use_file_drop(move |files: FileDrop| {
        log.with_mut(|log| log.push(format!("chip:{}", names(&files.paths))));
    });
    rsx! {
        div { class: "chip", style: "height:40px; width:120px",
            "data-drop": drop.drop_attr(),
            onmounted: move |event| drop.mounted(event),
        }
    }
}

fn names(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .filter_map(|path| path.file_name()?.to_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn describe(drag: &FileDrag) -> String {
    match drag {
        FileDrag::Idle => "idle".to_owned(),
        FileDrag::Over { paths, .. } => format!("over:{}", names(paths)),
        FileDrag::Dropped { paths, .. } => format!("dropped:{}", names(paths)),
    }
}

fn files() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/home/me/report.pdf"),
        PathBuf::from("/home/me/photo.jpg"),
    ]
}

fn harness() -> Harness {
    Harness::new(Window, VIEW)
}

fn at(harness: &Harness, selector: &str) -> Point {
    harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is not in the document:\n{}", harness.html()))
}

/// A point inside the composer but off its chip: its bottom-right corner area.
fn composer_only(harness: &Harness) -> Point {
    let rect = harness.rect(".composer").expect("the composer is drawn");
    Point {
        x: Px(rect.origin.x.0 + rect.size.width.0 - 20.0),
        y: Px(rect.origin.y.0 + rect.size.height.0 - 10.0),
    }
}

fn lights(harness: &Harness) -> (Option<String>, Option<String>) {
    (
        harness.attr(".composer", "data-drop"),
        harness.attr(".chip", "data-drop"),
    )
}

fn log(harness: &Harness) -> String {
    harness.text_of(".log").unwrap_or_default()
}

fn some(value: &str) -> Option<String> {
    Some(value.to_owned())
}

#[test]
fn files_over_a_target_light_it_and_moving_off_dims_it() {
    let mut harness = harness();
    let composer = composer_only(&harness);
    assert_eq!(lights(&harness), (None, None), "nothing lit before a drag");

    harness.send(Input::FileDrag(FileDragInput::Entered {
        point: Some(composer),
    }));
    let entered = harness.drop_answer();
    assert_eq!(entered, DropAcceptance::Copy, "paths not known yet: taken");
    assert_eq!(
        lights(&harness),
        (None, None),
        "nothing lit until files are known"
    );

    harness.send(Input::FileDrag(FileDragInput::Offered(Offer::Files(
        files(),
    ))));
    let offered = harness.drop_answer();
    assert_eq!(offered, DropAcceptance::Copy);
    assert_eq!(lights(&harness), (some("target"), some("accepts")));
    assert_eq!(
        harness.text_of(".composer-drag").as_deref(),
        Some("over:report.pdf,photo.jpg")
    );

    harness.send(Input::FileDrag(FileDragInput::Moved {
        point: at(&harness, ".outside"),
    }));
    let off = harness.drop_answer();
    assert_eq!(off, DropAcceptance::Refuse, "nothing to drop on: refused");
    assert_eq!(lights(&harness), (some("accepts"), some("accepts")));
    assert_eq!(harness.text_of(".composer-drag").as_deref(), Some("idle"));

    harness.send(Input::FileDrag(FileDragInput::Left));
    let left = harness.drop_answer();
    assert_eq!(left, DropAcceptance::Refuse);
    assert_eq!(
        lights(&harness),
        (None, None),
        "a drag that left lights nothing"
    );
    assert_eq!(log(&harness), "", "leaving drops nothing");
}

#[test]
fn a_release_on_a_target_hands_it_the_paths() {
    let mut harness = harness();
    let composer = composer_only(&harness);
    harness.send(Input::FileDrag(FileDragInput::Entered {
        point: Some(at(&harness, ".outside")),
    }));
    harness.send(Input::FileDrag(FileDragInput::Offered(Offer::Files(
        files(),
    ))));
    harness.send(Input::FileDrag(FileDragInput::Moved { point: composer }));
    let before = log(&harness);

    harness.send(Input::FileDrag(FileDragInput::Dropped));

    assert_eq!(before, "", "nothing dropped before the release");
    assert_eq!(log(&harness), "composer:report.pdf,photo.jpg");
    assert_eq!(lights(&harness), (None, None), "the drag is over");
    assert_eq!(
        harness.text_of(".composer-drag").as_deref(),
        Some("dropped:report.pdf,photo.jpg"),
        "the target shows what it took until the next drag"
    );

    harness.send(Input::FileDrag(FileDragInput::Entered { point: None }));
    assert_eq!(harness.text_of(".composer-drag").as_deref(), Some("idle"));
}

#[test]
fn the_innermost_target_takes_the_drop() {
    let mut harness = harness();
    let chip = at(&harness, ".chip");
    harness.send(Input::FileDrag(FileDragInput::Entered {
        point: Some(chip),
    }));
    harness.send(Input::FileDrag(FileDragInput::Offered(Offer::Files(
        files(),
    ))));
    assert_eq!(lights(&harness), (some("accepts"), some("target")));

    harness.send(Input::FileDrag(FileDragInput::Dropped));

    assert_eq!(
        log(&harness),
        "chip:report.pdf,photo.jpg",
        "only the chip hears it"
    );
}

#[test]
fn a_release_before_the_paths_arrive_lands_when_they_do() {
    let mut harness = harness();
    let composer = composer_only(&harness);
    harness.send(Input::FileDrag(FileDragInput::Entered {
        point: Some(composer),
    }));
    harness.send(Input::FileDrag(FileDragInput::Dropped));
    let before = log(&harness);

    harness.send(Input::FileDrag(FileDragInput::Offered(Offer::Files(
        files(),
    ))));

    assert_eq!(before, "");
    assert_eq!(log(&harness), "composer:report.pdf,photo.jpg");
}

#[test]
fn a_dragged_url_lights_nothing_and_drops_nothing() {
    let mut harness = harness();
    let composer = composer_only(&harness);
    harness.send(Input::FileDrag(FileDragInput::Entered {
        point: Some(composer),
    }));
    harness.send(Input::FileDrag(FileDragInput::Offered(Offer::Other)));
    let offered = harness.drop_answer();
    assert_eq!(offered, DropAcceptance::Refuse);
    assert_eq!(lights(&harness), (None, None));

    harness.send(Input::FileDrag(FileDragInput::Dropped));

    assert_eq!(log(&harness), "");
}
