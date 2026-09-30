//! A linear slider with tick marks on a real Blitz document (design/30 section 2.1): a press or a
//! drag reports the nearest mark, never a value between two, the arrows move one mark, and the
//! marks are drawn where the value would stop.

use dioxus::prelude::*;
use ds::{Appearance, Ds, Fraction, Material, Point, Px, ShortcutKey, Slider, Ticks};
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

static VALUE: GlobalSignal<Fraction> = Signal::global(|| Fraction(0));

const VIEW: Viewport = Viewport {
    width: 320,
    height: 80,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "margin:20px; width:200px",
                Slider {
                    label: "Level",
                    value: VALUE(),
                    ticks: Ticks::Every(Fraction(250)),
                    onchange: move |next| *VALUE.write() = next,
                }
            }
        }
    }
}

fn level(harness: &mut Harness) -> u16 {
    harness.within(|| VALUE.peek().0)
}

#[test]
fn a_press_and_the_arrows_stop_on_the_marks() {
    let mut harness =
        Harness::with_config(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(100));
    assert_eq!(
        harness.count(".ds-slider-tick"),
        5,
        "a mark at each quarter, ends included"
    );
    let track = harness.rect(".ds-slider-track").expect("the track");
    let at = |share: f32| Point {
        x: Px(track.origin.x.0 + track.size.width.0 * share),
        y: Px(track.origin.y.0 + track.size.height.0 / 2.0),
    };
    harness.pointer_move(at(0.62));
    harness.pointer_down(at(0.62));
    harness.advance(ms(80));
    harness.pointer_up(at(0.62));
    harness.advance(ms(80));
    assert_eq!(
        level(&mut harness),
        500,
        "62 % of the track is the middle mark"
    );
    harness.key(ShortcutKey::Right);
    assert_eq!(level(&mut harness), 750, "an arrow is one mark");
    harness.key(ShortcutKey::Right);
    harness.key(ShortcutKey::Right);
    assert_eq!(level(&mut harness), 1000, "and stops at the end");
    harness.key(ShortcutKey::Left);
    assert_eq!(level(&mut harness), 750);
}
