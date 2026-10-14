//! A proposed event: its title, when and where, who is invited, and the day it would land in as
//! a strip of what is already there.

use super::person::PersonView;
use crate::components::companion::proposed_event::model::{DaySlot, MinuteSpan, ProposedEvent};
use dioxus::prelude::*;
use ds_core::word::Word;

/// The most a day has, in minutes.
const DAY: u16 = 1440;
/// The room kept around the busiest hours, in minutes, and the least a strip covers.
const MARGIN: u16 = 60;
const LEAST: u16 = 180;

/// The stretch of the day the strip covers.
fn window(day: &[DaySlot]) -> MinuteSpan {
    let from = day.iter().map(|slot| slot.span.from).min().unwrap_or(540);
    let to = day.iter().map(|slot| slot.span.to).max().unwrap_or(from);
    let from = from.saturating_sub(MARGIN);
    let to = (to + MARGIN).clamp(from + LEAST, DAY.max(from + LEAST));
    MinuteSpan { from, to }
}

/// `slot` as the custom properties the stylesheet places it with, in percent of `window`.
fn placement(slot: &DaySlot, window: MinuteSpan) -> String {
    let whole = f32::from(window.to - window.from);
    let at =
        |minute: u16| f32::from(minute.clamp(window.from, window.to) - window.from) / whole * 100.0;
    let (from, to) = (at(slot.span.from), at(slot.span.to));
    format!("--from:{from:.1}%;--span:{:.1}%", to - from)
}

/// The proposed event as the card body.
#[component]
pub fn EventBody(event: ProposedEvent) -> Element {
    let shown = window(&event.day);
    rsx! {
        div { class: "ds-card-event",
            div { class: "ds-card-event-title", "{event.title}" }
            div { class: "ds-card-event-when", "{event.when}" }
            if let Some(place) = &event.place {
                div { class: "ds-card-event-place", "{place}" }
            }
            for (index , person) in event.people.iter().enumerate() {
                PersonView { key: "{index}", person: person.clone() }
            }
            if !event.day.is_empty() {
                div { class: "ds-day-strip", role: "list", "aria-label": "The day",
                    for (index , slot) in event.day.iter().enumerate() {
                        div {
                            key: "{index}",
                            class: "ds-day-slot",
                            role: "listitem",
                            "data-kind": slot.kind.slug(),
                            style: placement(slot, shown),
                            "{slot.label}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{placement, window};
    use crate::components::companion::proposed_event::model::{BusyKind, DaySlot, MinuteSpan};

    fn slot(from: u16, to: u16) -> DaySlot {
        DaySlot {
            span: MinuteSpan { from, to },
            kind: BusyKind::Proposed,
            label: String::new(),
        }
    }

    #[test]
    fn the_strip_pads_the_busy_hours_and_places_a_slot_in_percent() {
        let day = [slot(600, 660)];
        let shown = window(&day);
        assert_eq!((shown.from, shown.to), (540, 720));
        assert_eq!(placement(&day[0], shown), "--from:33.3%;--span:33.3%");
    }
}
