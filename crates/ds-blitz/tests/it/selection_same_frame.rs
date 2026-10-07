//! A selection is drawn in the frame that draws the text it covers. An app that draws its own
//! selection from `use_selection_rects` sees the boxes published between layout and paint, so
//! extending it with Shift+Right and typing over it each show text and selection together. Frames
//! are counted, not waited for.

use dioxus::prelude::*;
use ds::edit::handle::{EditHandle, use_edit_handle};
use ds::edit::input::EditInput;
use ds::edit::selection::use_selection_rects;
use ds::host::position::{TextPosition, TextRange};
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
    /// The selection the app drew in its latest render.
    static RANGE: RefCell<Option<TextRange>> = const { RefCell::new(None) };
}

/// A paragraph the app edits, with a selection it draws itself: Shift+Right extends it, typing
/// replaces it.
#[allow(non_snake_case)]
fn Editor() -> Element {
    let handle = use_edit_handle();
    HANDLE.with(|slot| *slot.borrow_mut() = Some(handle));
    let mut text = use_signal(|| "Hello".to_owned());
    let mut focus = use_signal(|| 0usize);
    let mut anchor = use_signal(|| 0usize);
    let range = (anchor() != focus()).then(|| TextRange {
        anchor: TextPosition::new("p0", anchor()),
        focus: TextPosition::new("p0", focus()),
    });
    RANGE.with(|slot| *slot.borrow_mut() = range.clone());
    let boxes = use_selection_rects(handle, range);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            div { style: "position:relative; padding:20px; width:300px; font-size:16px; line-height:20px",
                EditSurface {
                    common: Common { aria_label: Some("Message".to_string()), ..Common::default() },
                    handle,
                    on_input: move |input: EditInput| match input {
                        EditInput::Text(typed) => {
                            let (low, high) = (anchor().min(focus()), anchor().max(focus()));
                            text.write().replace_range(low..high, &typed);
                            anchor.set(low + typed.len());
                            focus.set(low + typed.len());
                        }
                        EditInput::Key(key) if key.key == Key::ArrowRight => {
                            focus.set((focus() + 1).min(text.read().len()));
                        }
                        _ => {}
                    },
                    p { id: "line", "data-edit-node": "p0", style: "margin:0", "{text}" }
                }
                for (index , rect) in boxes().into_iter().enumerate() {
                    div { id: "sel{index}",
                        style: "position:absolute; background:red; left:{20.0 + rect.origin.x.0}px; top:{20.0 + rect.origin.y.0}px; width:{rect.size.width.0}px; height:{rect.size.height.0}px",
                    }
                }
            }
        }
    }
}

/// What one painted frame shows: the paragraph, the selection boxes drawn (left, width), and the
/// boxes the layout of that same frame puts under the app's selection.
#[derive(Debug, Clone, PartialEq)]
struct Shown {
    text: String,
    drawn: Vec<(f32, f32)>,
    laid_out: Vec<(f32, f32)>,
}

impl Shown {
    fn selection_follows_text(&self) -> bool {
        self.drawn.len() == self.laid_out.len()
            && self
                .drawn
                .iter()
                .zip(&self.laid_out)
                .all(|(a, b)| (a.0 - b.0).abs() < 0.5 && (a.1 - b.1).abs() < 0.5)
    }
}

fn shown(harness: &mut Harness) -> Shown {
    let text = harness.text_of("#line").expect("the paragraph");
    let range = RANGE.with(|slot| slot.borrow().clone());
    let laid_out = harness.within(|| {
        let handle = HANDLE.with(|slot| slot.borrow().expect("the editor rendered"));
        match range {
            Some(range) => match handle.selection_rects(&range) {
                Probe::Found(rects) => rects,
                other => panic!("no selection rects: {other:?}"),
            },
            None => Vec::new(),
        }
    });
    let drawn = (0..harness.count("[id^=sel]"))
        .map(|index| harness.rect(&format!("#sel{index}")).expect("a box"))
        .map(|rect| (rect.origin.x.0, rect.size.width.0))
        .collect();
    Shown {
        text,
        drawn,
        laid_out: laid_out
            .into_iter()
            .map(|rect| (rect.origin.x.0, rect.size.width.0))
            .collect(),
    }
}

/// Step every frame the input owes, noting what each shows.
fn frames_of(harness: &mut Harness) -> Vec<Shown> {
    let mut frames = Vec::new();
    let before = harness.frames();
    while harness.step_frame() == Stepped::Painted {
        frames.push(shown(harness));
        assert!(frames.len() < 8, "frames never ran out: {frames:?}");
    }
    assert_eq!(harness.frames() - before, frames.len() as u64);
    frames
}

/// A focused surface, three characters selected from the start, and the frames that took.
fn selected(order: PhaseOrder) -> (Harness, Vec<Vec<Shown>>) {
    let mut harness = Harness::new(Editor, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    let into = harness.centre("#line").expect("the paragraph");
    harness.send(Input::click(into));
    harness.advance(Duration::from_millis(50));
    harness.set_phase_order(order);
    let mut each = Vec::new();
    for _ in 0..3 {
        harness.chord_pending(&[ShortcutKey::Shift], ShortcutKey::Right);
        each.push(frames_of(&mut harness));
    }
    (harness, each)
}

#[test]
fn the_selection_grows_in_the_frame_of_each_shift_arrow() {
    let (_harness, extends) = selected(PhaseOrder::BeforePaint);
    for (step, frames) in extends.iter().enumerate() {
        assert_eq!(frames.len(), 1, "step {step}: {frames:?}");
        let shown = &frames[0];
        assert_eq!(shown.drawn.len(), 1, "step {step}: {shown:?}");
        assert!(shown.selection_follows_text(), "step {step}: {shown:?}");
    }
    let widths: Vec<f32> = extends.iter().map(|f| f[0].drawn[0].1).collect();
    assert!(
        widths.windows(2).all(|pair| pair[1] > pair[0]),
        "{widths:?}"
    );
}

#[test]
fn typing_over_a_selection_replaces_the_text_and_drops_the_selection_in_one_frame() {
    let (mut harness, _) = selected(PhaseOrder::BeforePaint);
    harness.key_pending(ShortcutKey::Char('X'));
    let frames = frames_of(&mut harness);
    let first = frames.first().expect("typing painted a frame");
    assert_eq!(first.text, "Xlo", "the first frame shows the new text");
    assert!(
        first.drawn.is_empty() && first.selection_follows_text(),
        "the stale selection is gone in that same frame: {frames:?}"
    );
    assert_eq!(
        frames.len(),
        1,
        "no frame after it changes anything: {frames:?}"
    );
}

#[test]
fn read_after_paint_the_selection_is_a_frame_behind() {
    let (mut harness, extends) = selected(PhaseOrder::AfterPaint);
    assert!(
        extends.iter().all(|frames| frames.len() == 2),
        "each step lands a frame later: {extends:?}"
    );
    assert!(!extends[2][0].selection_follows_text(), "{:?}", extends[2]);
    harness.key_pending(ShortcutKey::Char('X'));
    let frames = frames_of(&mut harness);
    let first = frames.first().expect("typing painted a frame");
    assert_eq!(first.text, "Xlo");
    assert!(
        !first.drawn.is_empty(),
        "the old selection is still drawn: {first:?}"
    );
    assert_eq!(frames.len(), 2, "{frames:?}");
    assert!(frames[1].drawn.is_empty() && frames[1].selection_follows_text());
}
