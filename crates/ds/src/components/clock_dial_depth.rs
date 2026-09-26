//! The depth dials' vectors (design/23-WIDGETS.md section 4.2): the indices of a `Bezel` dial
//! (a minute track and twelve hour bars) or a `Sky` dial (twelve dots, the quarters as bars),
//! and the tapered hands, drawn once as themselves and once as their shadow. Every paint is an
//! attribute on `currentColor` (spike S6); the shadow copy takes its colour and offset from its
//! own element.

use crate::components::clock_angles::{Hands, Tenths};
use crate::components::widget_looks::DialLook;
use dioxus::prelude::*;

/// `rotate(a 50 50)`: a turn about the dial's centre in the 100-unit box.
fn turn(angle: Tenths) -> String {
    format!("rotate({} 50 50)", angle.css())
}

/// One index: a round-capped bar from `from` to `to` units below the top, `width` wide.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Bar {
    from: &'static str,
    to: &'static str,
    width: &'static str,
    opacity: &'static str,
}

/// The bar at minute `minute` (0 to 59) of `dial`, or none where the dial draws a dot.
fn index(dial: DialLook, minute: u16) -> Option<Bar> {
    let hour = minute.is_multiple_of(5);
    let quarter = minute.is_multiple_of(15);
    match (dial, hour, quarter) {
        (_, true, true) => Some(Bar {
            from: "10",
            to: "18",
            width: "3.4",
            opacity: ".85",
        }),
        (DialLook::Sky, true, false) => None,
        (_, true, false) => Some(Bar {
            from: "10",
            to: "15.5",
            width: "2.4",
            opacity: ".7",
        }),
        (DialLook::Sky, false, _) => None,
        (_, false, _) => Some(Bar {
            from: "10",
            to: "12",
            width: "1",
            opacity: ".3",
        }),
    }
}

/// The dial's indices, as `class`: `ds-clock-ticks` for the indices, `ds-clock-ticks-lit` for
/// the light catching their lower edge (they are pressed into the face).
pub(crate) fn ticks_svg(class: &'static str, dial: DialLook) -> Element {
    let bars = (0..60u16).filter_map(|minute| index(dial, minute).map(|bar| (minute, bar)));
    let dots = (0..12u16).filter(|hour| dial == DialLook::Sky && !hour.is_multiple_of(3));
    rsx! {
        svg { class, "data-ds-svg": "clock", view_box: "0 0 100 100", "aria-hidden": "true",
            for (minute , bar) in bars {
                line {
                    key: "m{minute}",
                    x1: "50",
                    y1: bar.from,
                    x2: "50",
                    y2: bar.to,
                    stroke: "currentColor",
                    "stroke-width": bar.width,
                    "stroke-linecap": "round",
                    opacity: bar.opacity,
                    transform: turn(Tenths(minute * 60)),
                }
            }
            for hour in dots {
                circle {
                    key: "h{hour}",
                    cx: "50",
                    cy: "12.5",
                    r: "2",
                    fill: "currentColor",
                    opacity: ".55",
                    transform: turn(Tenths(hour * 300)),
                }
            }
        }
    }
}

/// A tapered hand: a thin neck from behind the hub, then a broad blade to `tip`.
fn blade(angle: Tenths, neck: &'static str, tip: &'static str, width: &'static str) -> Element {
    rsx! {
        g { transform: turn(angle),
            line { x1: "50", y1: "57", x2: "50", y2: neck, stroke: "currentColor", "stroke-width": "2.2", "stroke-linecap": "round" }
            line { x1: "50", y1: neck, x2: "50", y2: tip, stroke: "currentColor", "stroke-width": width, "stroke-linecap": "round" }
        }
    }
}

/// The hour and minute hands, as `class`: `ds-clock-hands` for the hands themselves,
/// `ds-clock-shade` for their shadow (which also carries the second hand's, when `second`).
pub(crate) fn tapered_svg(class: &'static str, at: Hands, second: bool) -> Element {
    rsx! {
        svg { class, "data-ds-svg": "clock", view_box: "0 0 100 100", "aria-hidden": "true",
            {blade(at.hour, "42", "27", "5.6")}
            {blade(at.minute, "40", "13", "3.8")}
            if second {
                {second_hand(at.second)}
            }
        }
    }
}

/// The second hand with its counterweight, on its own vector so its colour can differ.
pub(crate) fn second_svg(angle: Tenths) -> Element {
    rsx! {
        svg { class: "ds-clock-second", "data-ds-svg": "clock", view_box: "0 0 100 100", "aria-hidden": "true",
            {second_hand(angle)}
        }
    }
}

fn second_hand(angle: Tenths) -> Element {
    rsx! {
        g { transform: turn(angle),
            line { x1: "50", y1: "64", x2: "50", y2: "9", stroke: "currentColor", "stroke-width": "1.3", "stroke-linecap": "round" }
            line { x1: "50", y1: "57", x2: "50", y2: "66", stroke: "currentColor", "stroke-width": "3.2", "stroke-linecap": "round" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::index;
    use crate::components::widget_looks::DialLook;

    #[test]
    fn a_bezel_draws_every_minute_and_a_sky_only_the_quarters() {
        let bezel = (0..60)
            .filter(|m| index(DialLook::Bezel, *m).is_some())
            .count();
        let sky = (0..60)
            .filter(|m| index(DialLook::Sky, *m).is_some())
            .count();
        assert_eq!((bezel, sky), (60, 4));
    }
}
