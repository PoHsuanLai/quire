//! The caret is drawn in the frame that draws the text it follows. An app that draws its own
//! caret from `use_caret_rect` sees the box published between layout and paint, so typing one
//! letter shows the glyph and the caret after it together. Frames are counted, not waited for:
//! the harness runs one in the window's order each `step_frame`.

use dioxus::prelude::*;
use ds::edit::caret::use_caret_rect;
use ds::edit::handle::{EditHandle, use_edit_handle};
use ds::edit::input::EditInput;
use ds::host::position::TextPosition;
use ds::host::probe::Probe;
use ds::prelude::*;
use ds::root::common::Common;
use ds_harness::{
    Clock, Driver, Harness, HarnessConfig, Input, PhaseOrder, Query, Stepped, Viewport,
};
use std::cell::RefCell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 200,
    scale_percent: 100,
};

thread_local! {
    static HANDLE: RefCell<Option<EditHandle>> = const { RefCell::new(None) };
}

/// A surface over one paragraph the app edits, and the caret the app draws at the end of it.
#[allow(non_snake_case)]
fn Editor() -> Element {
    let handle = use_edit_handle();
    HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
    let mut text = use_signal(|| "Hello".to_owned());
    let at = Some(TextPosition::new("p0", text.read().len()));
    let caret = use_caret_rect(handle, at);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "position:relative; padding:20px; width:300px; font-size:16px; line-height:20px",
                EditSurface {
                    common: Common { aria_label: Some("Message".to_string()), ..Common::default() },
                    handle,
                    on_input: move |input: EditInput| {
                        if let EditInput::Text(typed) = input {
                            text.write().push_str(&typed);
                        }
                    },
                    p { id: "line", "data-edit-node": "p0", style: "margin:0", "{text}" }
                }
                if let Some(rect) = caret() {
                    div { id: "caret",
                        style: "position:absolute; width:2px; background:red; left:{20.0 + rect.origin.x.0}px; top:{20.0 + rect.origin.y.0}px; height:{rect.size.height.0}px",
                    }
                }
            }
        }
    }
}

/// What one painted frame shows: the paragraph, and where the caret div is against where the
/// layout of that same frame puts the caret after the text.
#[derive(Debug, Clone, PartialEq)]
struct Shown {
    text: String,
    drawn_x: Option<f32>,
    laid_out_x: f32,
}

impl Shown {
    fn caret_follows_text(&self) -> bool {
        self.drawn_x
            .is_some_and(|x| (x - self.laid_out_x).abs() < 0.5)
    }
}

fn shown(harness: &mut Harness) -> Shown {
    let text = harness.text_of("#line").expect("the paragraph");
    let end = TextPosition::new("p0", text.len());
    let laid_out = harness.within(|| {
        let handle = HANDLE.with(|slot| slot.borrow().expect("the editor rendered"));
        match handle.caret_rect(&end) {
            Probe::Found(rect) => rect,
            other => panic!("no caret rect: {other:?}"),
        }
    });
    Shown {
        text,
        drawn_x: harness.rect("#caret").map(|rect| rect.origin.x.0),
        laid_out_x: laid_out.origin.x.0,
    }
}

/// Type `x` into a focused surface and step every frame that follows, noting what each shows.
fn typing_frames(order: PhaseOrder) -> Vec<Shown> {
    let mut harness = Harness::new(Editor, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    let into = harness.centre("#line").expect("the paragraph");
    harness.send(Input::click(into));
    harness.advance(Duration::from_millis(50));
    harness.set_phase_order(order);
    let before = shown(&mut harness);
    assert!(before.caret_follows_text(), "at rest: {before:?}");
    harness.key_pending(ShortcutKey::Char('x'));
    let mut frames = Vec::new();
    let painted_before = harness.frames();
    while harness.step_frame() == Stepped::Painted {
        frames.push(shown(&mut harness));
        assert!(frames.len() < 8, "frames never ran out: {frames:?}");
    }
    assert_eq!(harness.frames() - painted_before, frames.len() as u64);
    frames
}

#[test]
fn the_caret_is_in_the_frame_that_shows_the_new_glyph() {
    let frames = typing_frames(PhaseOrder::BeforePaint);
    let first = frames.first().expect("typing painted a frame");
    assert!(
        first.text.ends_with('x'),
        "the first frame shows the glyph: {first:?}"
    );
    assert!(
        first.caret_follows_text(),
        "the glyph and the caret after it are one frame: {frames:?}"
    );
    assert_eq!(
        frames.len(),
        1,
        "no frame after it changes anything: {frames:?}"
    );
}

#[test]
fn read_after_paint_the_caret_is_a_frame_behind() {
    let frames = typing_frames(PhaseOrder::AfterPaint);
    let first = frames.first().expect("typing painted a frame");
    assert!(
        first.text.ends_with('x'),
        "the first frame shows the glyph: {first:?}"
    );
    assert!(
        !first.caret_follows_text(),
        "this order draws the caret where the text was: {first:?}"
    );
    assert_eq!(frames.len(), 2, "the caret lands a frame later: {frames:?}");
    assert!(frames[1].caret_follows_text());
}
