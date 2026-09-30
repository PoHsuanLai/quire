//! A module tile's disc: the glyph on its paper or accent disc and the ring while the module is
//! busy (design/26-DETAILS.md 5.2.2, 5.2.5).

use crate::control_center::module_tile_kind::ModuleState;
use dioxus::prelude::*;
use ds::components::content::icon_source::IconSource;
use ds::components::content::icon_view::IconView;
use ds::components::controls::spinner::Spinner;
use ds_motion::detail::{
    detailed::Detailed, moment::Moment, touch::Touch, use_detail::use_detail,
    use_operation::use_operation,
};
use ds_style::icon::render::IconSize;

/// The disc's own reading of the module's state: what the disc plays is its Pending ring.
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

/// `span.ds-module-disc`: `glyph` on the disc, with the pending ring while `state` is busy.
#[component]
pub(crate) fn ModuleDisc(glyph: IconSource, state: ModuleState) -> Element {
    let detail = use_detail(Lighting(state), Touch::Remote);
    // Busy is an operation the tile's own state starts.
    let operation = use_operation(detail.cue());
    rsx! {
        span { class: "ds-module-disc",
            IconView { source: glyph, size: IconSize::Base }
            if state == ModuleState::Busy {
                Spinner { operation }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Lighting;
    use crate::control_center::module_tile_kind::ModuleState::{Busy, Off, On};
    use ds_motion::detail::{
        detailed::{first_table, moment_table},
        moment::Moment,
    };

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
}
