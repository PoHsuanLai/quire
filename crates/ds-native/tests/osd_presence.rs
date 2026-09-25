//! The OSD card's presence on a real Blitz document (sill FINDINGS Q76): shown, it enters
//! (`data-presence="entering"`, `osd-in`) and comes to rest; hidden, it leaves (`osd-out`) and
//! `on_hidden` runs when `settle(OsdOut)` is up and not before, with the card no longer drawn;
//! shown again while it fades, the hide is taken back and `on_hidden` never runs for it.

use dioxus::prelude::*;
use ds::{
    Anim, Appearance, Ds, Fraction, Level, LevelGlyph, Material, MotionLevel, Muting, Osd,
    RootChrome, Shown, StaggerIndex, settle,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

static SHOWN: GlobalSignal<Shown> = Signal::global(|| Shown::Visible);
static HIDDEN: GlobalSignal<u32> = Signal::global(|| 0);

const VIEW: Viewport = Viewport {
    width: 400,
    height: 160,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Card() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Osd, chrome: Some(RootChrome::Transparent),
            Osd {
                shown: SHOWN(),
                label: "Sound",
                level: Level { value: Fraction(500), glyph: LevelGlyph::Volume(Muting::Audible) },
                on_hidden: move |()| *HIDDEN.write() += 1,
            }
        }
    }
}

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn presence(harness: &Harness) -> Option<String> {
    harness.attr(".ds-osd", "data-presence")
}

fn hidden(harness: &mut Harness) -> u32 {
    harness.within(|| *HIDDEN.peek())
}

fn show(harness: &mut Harness, shown: Shown) {
    harness.within(|| *SHOWN.write() = shown);
    harness.advance(ms(1));
}

/// A margin either side of a settle: the harness's timers run on the wall clock.
const MARGIN: u64 = 40;

#[test]
fn hidden_it_fades_and_on_hidden_runs_at_settle_and_not_before() {
    let mut harness = Harness::new(Card, VIEW);
    assert_eq!(presence(&harness).as_deref(), Some("entering"));
    harness.advance(ms(300));
    assert_eq!(presence(&harness).as_deref(), Some("present"));
    let out = settle(Anim::OsdOut, MotionLevel::Standard, StaggerIndex::default());
    let out = u64::try_from(out.as_millis()).unwrap_or(u64::MAX);
    show(&mut harness, Shown::Hidden);
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
    harness.advance(ms(out - MARGIN));
    assert_eq!(
        hidden(&mut harness),
        0,
        "not before settle(OsdOut) = {out} ms"
    );
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
    harness.advance(ms(2 * MARGIN));
    assert_eq!(hidden(&mut harness), 1, "at settle(OsdOut)");
    assert_eq!(
        harness.attr(".ds-osd", "data-shown").as_deref(),
        Some("hidden")
    );
    assert_eq!(presence(&harness), None);
    // Shown again: it enters from its first frame.
    show(&mut harness, Shown::Visible);
    assert_eq!(presence(&harness).as_deref(), Some("entering"));
    assert_eq!(harness.attr(".ds-osd", "data-pulse").as_deref(), Some("a"));
}

#[test]
fn a_show_while_it_fades_takes_the_hide_back() {
    let mut harness = Harness::new(Card, VIEW);
    harness.advance(ms(300));
    show(&mut harness, Shown::Hidden);
    harness.advance(ms(100));
    assert_eq!(presence(&harness).as_deref(), Some("leaving"));
    show(&mut harness, Shown::Visible);
    assert_eq!(presence(&harness).as_deref(), Some("present"));
    harness.advance(ms(500));
    assert_eq!(hidden(&mut harness), 0, "the hide was taken back");
    assert_eq!(presence(&harness).as_deref(), Some("present"));
    assert_eq!(
        harness.attr(".ds-osd", "data-shown").as_deref(),
        Some("visible")
    );
}
