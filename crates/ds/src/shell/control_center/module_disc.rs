//! A module tile's disc: the glyph on its paper or accent disc, the bounded ring while the module
//! is busy, and the glyph's answer to the module turning on (design/26-DETAILS.md 5.2.2,
//! 5.2.5): a fill once through its layers, or a morph into its "on" glyph.

use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::components::controls::spinner::{Spinner, SpinnerKind};
use crate::motion::detail::{
    armed::Armed,
    cue::Cue,
    detailed::Detailed,
    first_show::FirstShow,
    layer_glyph::{LayerGlyph, Layering},
    moment::Moment,
    morph::MorphStyle,
    morph_glyph::MorphGlyph,
    pending::PendingLayers,
    settle::{SettleStyle, Settling},
    use_detail::use_detail,
    use_operation::use_operation,
    use_settle::use_settle,
};
use crate::shell::control_center::module_tile_kind::{DiscMotion, ModuleState};
use crate::style::icon::Icon;
use crate::style::icon::render::IconSize;
use dioxus::prelude::*;

/// The disc's own reading of the module's state: coming on is its success, whether it came from
/// off (a press) or from busy (the operation landed). Only the disc's glyph plays it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Lighting(pub(crate) ModuleState);

impl Detailed for Lighting {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from.0, to.0) {
            (ModuleState::Off | ModuleState::On | ModuleState::Busy, ModuleState::Busy) => {
                Moment::Pending
            }
            (ModuleState::Off | ModuleState::Busy, ModuleState::On) => Moment::Success,
            (ModuleState::On, ModuleState::On) => Moment::Rest,
            (ModuleState::Off | ModuleState::On | ModuleState::Busy, ModuleState::Off) => {
                Moment::Change
            }
        }
    }

    fn first(state: &Self) -> Moment {
        match state.0 {
            ModuleState::Busy => Moment::Pending,
            ModuleState::Off | ModuleState::On => Moment::Rest,
        }
    }
}

/// `span.ds-module-disc`: `glyph` on the disc, answering `state` as `motion` says; a change the
/// tile's own press caused (kept in `armed`) springs.
#[component]
pub(crate) fn ModuleDisc(
    glyph: IconSource,
    state: ModuleState,
    motion: DiscMotion,
    first: FirstShow,
    armed: Armed,
) -> Element {
    let detail = use_detail(Lighting(state), FirstShow::Still, armed.touch());
    armed.spend(detail.cue());
    // Busy is an operation the tile's own state starts: its ring is bounded by the cap (R4).
    let operation = use_operation(detail.cue());
    let settling = use_settle(detail.cue(), SettleStyle::Fill(layers(&glyph)));
    let face = face(glyph, state, motion, first, detail.cue(), settling);
    rsx! {
        span { class: "ds-module-disc", "data-motion": motion_slug(motion),
            {face}
            if state == ModuleState::Busy {
                Spinner { kind: SpinnerKind::Breathe, operation }
            }
        }
    }
}

/// How many layers a glyph fills through: one per shape (a Wi-Fi glyph's dot and three arcs).
fn layers(glyph: &IconSource) -> PendingLayers {
    match glyph {
        IconSource::Glyph(icon) => {
            PendingLayers(u8::try_from(icon.shapes().len()).unwrap_or(u8::MAX))
        }
        IconSource::Symbolic(_) | IconSource::Image(_) | IconSource::Status(_) => PendingLayers(1),
    }
}

/// The `data-motion` word: written only for a disc that moves, so a still tile's markup is as
/// before.
fn motion_slug(motion: DiscMotion) -> Option<&'static str> {
    match motion {
        DiscMotion::Still => None,
        DiscMotion::Fill => Some("fill"),
        DiscMotion::Morph(_) => Some("morph"),
    }
}

/// The glyph as `motion` draws it this frame.
fn face(
    glyph: IconSource,
    state: ModuleState,
    motion: DiscMotion,
    first: FirstShow,
    cue: Cue,
    settling: Settling,
) -> Element {
    match (motion, glyph) {
        (DiscMotion::Fill, IconSource::Glyph(icon)) => rsx! {
            LayerGlyph { icon, size: IconSize::Base, layering: filling(settling) }
        },
        (DiscMotion::Morph(on), IconSource::Glyph(off)) => rsx! {
            MorphGlyph { icon: shown(off, on, state), size: IconSize::Base, style: MorphStyle::DownUp, touch: cue.touch() }
        },
        (DiscMotion::Still | DiscMotion::Fill | DiscMotion::Morph(_), source) => rsx! {
            IconView { source, size: IconSize::Base, first }
        },
    }
}

/// The layers a fill has reached: all of them at rest.
fn filling(settling: Settling) -> Layering {
    match settling {
        Settling::Filling(upto) => Layering::Filling(upto),
        Settling::Rest | Settling::Drawing(_) | Settling::Sealing(_) => Layering::Whole,
    }
}

/// The glyph a morphing disc shows: the "on" glyph only when on.
fn shown(off: Icon, on: Icon, state: ModuleState) -> Icon {
    match state {
        ModuleState::On => on,
        ModuleState::Off | ModuleState::Busy => off,
    }
}

#[cfg(test)]
mod tests {
    use super::{Lighting, shown};
    use crate::motion::detail::{
        detailed::{first_table, moment_table},
        moment::Moment,
    };
    use crate::shell::control_center::module_tile_kind::ModuleState::{Busy, Off, On};
    use crate::style::icon::Icon;

    #[test]
    fn the_disc_reads_coming_on_as_its_success() {
        moment_table(&[
            (Lighting(Off), Lighting(On), Moment::Success),
            (Lighting(Busy), Lighting(On), Moment::Success),
            (Lighting(Off), Lighting(Busy), Moment::Pending),
            (Lighting(On), Lighting(Busy), Moment::Pending),
            (Lighting(On), Lighting(Off), Moment::Change),
            (Lighting(Busy), Lighting(Off), Moment::Change),
            (Lighting(On), Lighting(On), Moment::Rest),
        ]);
        first_table(&[
            (Lighting(Busy), Moment::Pending),
            (Lighting(On), Moment::Rest),
            (Lighting(Off), Moment::Rest),
        ]);
    }

    #[test]
    fn a_morphing_disc_shows_its_on_glyph_only_when_on() {
        let cases = [
            (On, Icon::MoonFilled),
            (Off, Icon::Moon),
            (Busy, Icon::Moon),
        ];
        for (state, want) in cases {
            assert_eq!(
                shown(Icon::Moon, Icon::MoonFilled, state),
                want,
                "{state:?}"
            );
        }
    }
}
