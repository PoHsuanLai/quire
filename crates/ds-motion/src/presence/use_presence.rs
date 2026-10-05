//! A surface's presence driven by the caller's `shown` (the OSD card, the screenshot thumbnail,
//! the preview pane): entering, present, leaving, then hidden with `on_hidden` at the exit's
//! settle, and a show while it leaves taking the hide back. The rules are the pure [`Life`]
//! machine in `step`, run by `use_machine`; a surface whose life something else decides runs it
//! itself. Present keeps the entrance declared in each surface's CSS, so the entrance's settle
//! timer (wall clock) never cancels an entrance the frame clock is still playing, and a
//! taken-back hide plays `hold` (Blitz at the pin keeps a cancelled animation's last value).

use super::spec::PresenceSpec;
use super::step::change;
use super::{Life, Presence, PresenceOut, PresenceParams};
use crate::machine::use_machine;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_style::scope::use_scope_signal;

pub use super::EntranceAlias;

/// What a surface draws this render: where it is in its life and which entrance name it plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Presented {
    /// Its life: `data-presence` and `data-shown`.
    pub presence: Presence,
    /// Its entrance's name: `data-pulse`.
    pub alias: EntranceAlias,
}

/// A surface's presence and entrance alias for this render, driven by `shown`: a change of
/// `shown` steps the machine at once (so the render draws the new phase), and its settle timer
/// is the machine's wake. `on_hidden` runs when the exit has settled.
pub fn use_presence(
    shown: Shown,
    spec: PresenceSpec,
    on_hidden: Option<EventHandler<()>>,
) -> Presented {
    let scope = use_scope_signal();
    let params = PresenceParams {
        enter: spec.enter,
        exit: spec.exit,
        motion: scope.peek().resolved.motion,
    };
    let machine = use_machine(
        |_| Life::hidden(),
        params,
        || (),
        move |PresenceOut::Gone, _| {
            if let Some(on_hidden) = on_hidden {
                on_hidden.call(());
            }
        },
    );
    let mut last = use_hook(|| CopyValue::new(None::<Shown>));
    let input = change(*last.peek(), shown);
    last.set(Some(shown));
    if let Some(input) = input {
        machine.send_from_render(input);
    }
    let life = *machine.state().read();
    Presented {
        presence: life.presence,
        alias: life.alias,
    }
}
