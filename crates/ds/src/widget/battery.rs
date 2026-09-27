//! The Batteries widget on the widget contract (design/23-WIDGETS.md sections 4.1 and 9.6): this
//! computer's battery and each device's, as the reference composes them. Small with one
//! device: the ring at the top left, the percentage as the card's hero figure. Small with
//! several: a 2 x 2 grid of rings, no numbers, an empty place a bare track. Medium: a row of up
//! to four rings, the percentage under each. Each ring holds the device's filled glyph
//! ([`crate::DeviceGlyph`]) and fills on the host's wake stamp.

use crate::components::battery_figure::BatteryFigure;
use crate::components::battery_level::{BatteryLevel, RingMark};
use crate::components::device_glyph::{Device, DeviceGlyph};
use crate::components::text_runs::Text;
use crate::components::vocab::Fraction;
use crate::components::widget_kind::WidgetSize;
use crate::icon::render::IconSize;
use crate::motion::WakeStamp;
use crate::widget::contract::{NoIntent, Widget, WidgetContext, WidgetKind};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

/// The Batteries widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BatteryWidget;

/// One battery.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BatteryCell {
    /// The device's name, what its ring is labelled for assistive technology.
    pub name: String,
    /// Its glyph.
    pub device: Device,
    /// Its charge, in thousandths.
    pub level: Fraction,
    /// Charging or not.
    pub mark: RingMark,
}

/// One moment of the Batteries widget.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatteryEntry {
    /// No reading yet: bare tracks, no numbers.
    #[default]
    Waiting,
    /// The batteries, this computer's first; the first four are drawn.
    Devices(Vec<BatteryCell>),
    /// Nothing reports a battery: a quiet line in the provider's words ("No batteries").
    Absent(String),
}

/// How many rings a grid or a row holds (the reference's widget shows up to four).
pub const MAX_RINGS: usize = 4;

/// How the widget lays out its rings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum BatteryLayout {
    /// Small, one battery: the ring and the hero figure.
    Solo,
    /// Small, several: a 2 x 2 grid of rings, no numbers.
    Grid,
    /// Medium: a row of rings, a figure under each.
    Row,
}

impl BatteryLayout {
    /// The layout for `size` with `count` batteries.
    pub(crate) fn of(size: WidgetSize, count: usize) -> Self {
        match (size, count) {
            (WidgetSize::Small, 1) => BatteryLayout::Solo,
            (WidgetSize::Small, _) => BatteryLayout::Grid,
            (WidgetSize::Medium | WidgetSize::Large, _) => BatteryLayout::Row,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            BatteryLayout::Solo => "solo",
            BatteryLayout::Grid => "grid",
            BatteryLayout::Row => "row",
        }
    }
}

impl Widget for BatteryWidget {
    type Entry = BatteryEntry;
    type Intent = NoIntent;

    fn kind() -> WidgetKind {
        WidgetKind::fixed("quire.battery")
    }

    fn name() -> Text {
        Text::from("Batteries")
    }

    fn sizes() -> &'static [WidgetSize] {
        &[WidgetSize::Small, WidgetSize::Medium]
    }

    fn description() -> Text {
        Text::from("See the charge of this computer and your devices.")
    }

    fn placeholder(_size: WidgetSize) -> BatteryEntry {
        BatteryEntry::Waiting
    }

    fn preview(size: WidgetSize) -> BatteryEntry {
        let cell = |name: &str, device, level| BatteryCell {
            name: name.to_owned(),
            device,
            level: Fraction(level),
            mark: RingMark::Plain,
        };
        let laptop = cell("Laptop", Device::Laptop, 820);
        match size {
            WidgetSize::Small => BatteryEntry::Devices(vec![laptop]),
            WidgetSize::Medium | WidgetSize::Large => BatteryEntry::Devices(vec![
                laptop,
                cell("Phone", Device::Phone, 640),
                cell("Headphones", Device::Headphones, 450),
                cell("Mouse", Device::Mouse, 900),
            ]),
        }
    }

    fn view(entry: &BatteryEntry, cx: WidgetContext<NoIntent>) -> Element {
        let wake = cx.wake;
        match entry {
            BatteryEntry::Waiting => waiting(BatteryLayout::of(cx.size, MAX_RINGS)),
            BatteryEntry::Absent(words) => rsx! {
                div { class: "ds-batteries", "data-layout": "absent",
                    span { class: "ds-batteries-quiet", "{words}" }
                }
            },
            BatteryEntry::Devices(cells) => {
                let shown = &cells[..cells.len().min(MAX_RINGS)];
                match BatteryLayout::of(cx.size, shown.len()) {
                    BatteryLayout::Solo => solo(&shown[0], wake),
                    BatteryLayout::Grid => grid(shown, wake),
                    BatteryLayout::Row => row(shown, wake),
                }
            }
        }
    }
}

/// One battery's ring, its device's glyph inside.
fn ring(cell: &BatteryCell, wake: WakeStamp) -> Element {
    rsx! {
        BatteryLevel { key: "{cell.name}", level: cell.level, mark: cell.mark, label: cell.name.clone(), wake,
            DeviceGlyph { device: cell.device, size: IconSize::Base }
        }
    }
}

/// An empty place: the bare track.
fn empty(key: usize) -> Element {
    rsx! {
        BatteryLevel { key: "empty-{key}", level: Fraction(0), label: "" }
    }
}

fn solo(cell: &BatteryCell, wake: WakeStamp) -> Element {
    rsx! {
        div { class: "ds-batteries", "data-layout": BatteryLayout::Solo.slug(),
            {ring(cell, wake)}
            span { class: "ds-batteries-hero", BatteryFigure { level: cell.level, wake } }
        }
    }
}

fn grid(cells: &[BatteryCell], wake: WakeStamp) -> Element {
    rsx! {
        div { class: "ds-batteries", "data-layout": BatteryLayout::Grid.slug(),
            for cell in cells {
                {ring(cell, wake)}
            }
            for place in cells.len()..MAX_RINGS {
                {empty(place)}
            }
        }
    }
}

fn row(cells: &[BatteryCell], wake: WakeStamp) -> Element {
    rsx! {
        div { class: "ds-batteries", "data-layout": BatteryLayout::Row.slug(),
            for cell in cells {
                div { key: "{cell.name}", class: "ds-batteries-cell",
                    {ring(cell, wake)}
                    span { class: "ds-batteries-figure", BatteryFigure { level: cell.level, wake } }
                }
            }
        }
    }
}

/// No reading yet: four bare tracks in the size's layout, no numbers.
fn waiting(layout: BatteryLayout) -> Element {
    rsx! {
        div { class: "ds-batteries", "data-layout": layout.slug(), "data-waiting": "",
            for place in 0..MAX_RINGS {
                {empty(place)}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BatteryLayout;
    use crate::components::widget_kind::WidgetSize;

    #[test]
    fn the_layout_follows_the_size_and_the_count() {
        let cases = [
            (WidgetSize::Small, 1, BatteryLayout::Solo),
            (WidgetSize::Small, 2, BatteryLayout::Grid),
            (WidgetSize::Small, 4, BatteryLayout::Grid),
            (WidgetSize::Medium, 1, BatteryLayout::Row),
            (WidgetSize::Medium, 4, BatteryLayout::Row),
        ];
        for (size, count, want) in cases {
            assert_eq!(BatteryLayout::of(size, count), want, "{size:?} {count}");
        }
    }
}
