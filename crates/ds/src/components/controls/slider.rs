//! Slider: `NSSlider` (design/30 section 2.1): a value in a range, set by a drag, a click or the
//! arrows, in the linear look of a settings row or the level look of the control center.
//! The look picks the drawing and the gesture that goes with it (`slider_linear`,
//! `slider_bezel`); what a caller sees is one component and one `onchange`.
//! Markup: `div.ds-slider[role=slider][data-look][data-size]`, the level as `--f`; parts `track`,
//! `fill`, `thumb`, `icon`, and a `tick` for each mark.

use crate::components::content::level_glyph::vocab::LevelSource;
use crate::components::controls::slider_bezel::BezelSlider;
use crate::components::controls::slider_linear::LinearSlider;
use crate::components::controls::slider_model::SliderLook;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::{Availability, Fraction};
use ds_style::tokens::control_size::ControlSize;

/// A value in a range, `value` in thousandths.
///
/// `look: Linear` is the thin track under a round knob (`NSSlider`, the default); `ticks` (a mark at every multiple of
/// that share of the range) puts marks on it and snaps the value to them, and `step` is a key's step without them (zero is a
/// twentieth). `look: Level` is the Control Center's level slider: a 22 high capsule under a round
/// knob that carries the glyph. The level look carries `glyph`,
/// a `LevelGlyph` that follows `value` or a `VolumeState` (both convert): the speaker then draws
/// that state's waves and slash, as the bar's volume item does; it steps a sixteenth a key, a
/// sixty-fourth with Shift, and takes neither `ticks` nor `step`.
/// `Disabled` and `Busy` take no press, drag or key, and `Disabled` leaves the tab order.
#[component]
pub fn Slider(
    label: String,
    value: Fraction,
    #[props(default)] look: SliderLook,
    #[props(default)] step: Fraction,
    #[props(default)] ticks: Option<Fraction>,
    #[props(default)] glyph: Option<LevelSource>,
    #[props(default)] size: ControlSize,
    #[props(default)] availability: Availability,
    #[props(default)] onchange: EventHandler<Fraction>,
    #[props(default)] common: Common,
) -> Element {
    match look {
        SliderLook::Linear => rsx! {
            LinearSlider { label, value, step, ticks, size, availability, onchange, common }
        },
        SliderLook::Level => rsx! {
            BezelSlider { label, value, look, glyph, size, availability, onchange, common }
        },
    }
}
