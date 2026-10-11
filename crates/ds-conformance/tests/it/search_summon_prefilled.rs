//! A field whose Blitz editor is built WITH text (its `value` attribute is non-empty when it
//! first lays out), focused before that layout, and typed into at once: the caret must sit after
//! the text, so "nvoice" typed after a prefilled "i" reads "invoice", not "nvoicei" (mailo #28).
//! The same when the text is replaced from outside while the field has the keyboard, and with
//! idle sibling components next to the field that re-render on their own timers, as mail's
//! panels do.

use dioxus::prelude::*;
use ds::focus::select::Select;
use ds::focus::selector::focus_by_selector;
use ds::prelude::*;
use ds_harness::{
    Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Stepped, Viewport,
};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 600,
    height: 300,
    scale_percent: 100,
};

const FIELD: &str = "#case input";

/// How the field is focused.
const OWN: u8 = 0; // `FieldFocus::OnMount`
const NONE: u8 = 1; // `focus_by_selector(.., Select::None)`
const ALL: u8 = 2; // `focus_by_selector(.., Select::All)`

/// What the field holds when it mounts.
const PREFILLED: u8 = 0; // "i"
const REPLACED: u8 = 1; // "", then "i" set from outside while focused (Ctrl+J)

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// A sibling that re-renders itself on a timer and does nothing else.
#[allow(non_snake_case)]
fn Idle() -> Element {
    let mut ticks = use_signal(|| 0u32);
    use_future(move || async move {
        loop {
            ds::base::time::clock::sleep(ms(5)).await;
            ticks.with_mut(|n| *n += 1);
        }
    });
    rsx! { span { style: "display:none", "{ticks}" } }
}

/// The Summon button shows the field (as mail's ⌘K does); Ctrl+J then sets its text from outside.
#[allow(non_snake_case)]
fn Case<const HOW: u8, const SIBLINGS: usize, const MOUNT: u8>() -> Element {
    let mut query = use_signal(String::new);
    let mut shown = use_signal(|| false);
    let own = match HOW {
        OWN => FieldFocus::OnMount,
        _ => FieldFocus::Manual,
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div {
                style: "position:relative; width:600px; height:300px",
                onkeydown: move |event: KeyboardEvent| {
                    if !event.modifiers().ctrl() {
                        return;
                    }
                    if let Key::Character(c) = event.key()
                        && c == "j"
                    {
                        query.set("i".to_owned());
                    }
                },
                Button { label: "Summon",
                    onclick: move |_| {
                        query.set(if MOUNT == PREFILLED { "i".to_owned() } else { String::new() });
                        shown.set(true);
                        if HOW != OWN {
                            let select = if HOW == ALL { Select::All } else { Select::None };
                            spawn(async move {
                                let _ = focus_by_selector(FIELD, select).await;
                            });
                        }
                    }
                }
                for n in 0..SIBLINGS {
                    Idle { key: "{n}" }
                }
                if shown() {
                    div { id: "case", style: "position:absolute; left:40px; top:60px; width:400px",
                        TextField {
                            label: "Search",
                            value: query(),
                            focus: own,
                            oninput: move |next: String| query.set(next),
                        }
                    }
                }
                p { id: "value", style: "position:absolute; left:500px; top:10px", "{query}" }
            }
        }
    }
}

/// Click Summon without settling, then step frame by frame until the field reads focused.
fn summoned(page: fn() -> Element) -> Harness {
    let mut harness = Harness::new(page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    let at = harness.centre(".ds-button").expect("the summon button");
    harness.send(Input::click(at));
    let mut frames = 0;
    while harness.focus_of(FIELD) != FocusState::Focused {
        if harness.step_frame() == Stepped::Idle {
            harness.advance(ms(1));
        }
        frames += 1;
        assert!(frames < 120, "the field never took the keyboard");
    }
    harness
}

fn type_in(harness: &mut Harness, text: &str) -> String {
    text.chars()
        .for_each(|c| harness.send(Input::key(ShortcutKey::Char(c))));
    harness.advance(ms(300));
    harness.text_of("#value").unwrap_or_default()
}

macro_rules! typed_reads {
    ($name:ident, $page:expr, $first:expr, $then:expr) => {
        #[test]
        fn $name() {
            let mut harness = summoned($page);
            $first(&mut harness);
            assert_eq!(type_in(&mut harness, "nvoice"), $then);
        }
    };
}

fn nothing(_: &mut Harness) {}

/// Replace the text from outside while the field has the keyboard, and let that render.
fn replace_from_outside(harness: &mut Harness) {
    harness.send(Input::chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('j')));
    harness.advance(ms(20));
}

// A prefilled field's editor is built with "i": the next keys land after it.
typed_reads!(
    prefilled_own_focus,
    Case::<OWN, 0, PREFILLED>,
    nothing,
    "invoice"
);
typed_reads!(
    prefilled_own_focus_one_sibling,
    Case::<OWN, 1, PREFILLED>,
    nothing,
    "invoice"
);
typed_reads!(
    prefilled_own_focus_two_siblings,
    Case::<OWN, 2, PREFILLED>,
    nothing,
    "invoice"
);
typed_reads!(
    prefilled_selector_none,
    Case::<NONE, 0, PREFILLED>,
    nothing,
    "invoice"
);
typed_reads!(
    prefilled_selector_none_one_sibling,
    Case::<NONE, 1, PREFILLED>,
    nothing,
    "invoice"
);
typed_reads!(
    prefilled_selector_none_two_siblings,
    Case::<NONE, 2, PREFILLED>,
    nothing,
    "invoice"
);
// Select::All on a prefilled field selects "i", so the first key replaces it.
typed_reads!(
    prefilled_selector_all,
    Case::<ALL, 0, PREFILLED>,
    nothing,
    "nvoice"
);
typed_reads!(
    prefilled_selector_all_two_siblings,
    Case::<ALL, 2, PREFILLED>,
    nothing,
    "nvoice"
);
// Text set from outside while the field is focused: the caret ends up after it.
typed_reads!(
    replaced_own_focus,
    Case::<OWN, 0, REPLACED>,
    replace_from_outside,
    "invoice"
);
typed_reads!(
    replaced_own_focus_two_siblings,
    Case::<OWN, 2, REPLACED>,
    replace_from_outside,
    "invoice"
);
typed_reads!(
    replaced_selector_none,
    Case::<NONE, 0, REPLACED>,
    replace_from_outside,
    "invoice"
);
typed_reads!(
    replaced_selector_none_one_sibling,
    Case::<NONE, 1, REPLACED>,
    replace_from_outside,
    "invoice"
);
