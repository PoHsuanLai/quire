//! What an alert says and what its buttons do: the words `Alert` hands to its body, and the one
//! place an input is carried out (`Words::answer`), through the pure rule in `alert_arming`.

use crate::components::content::icon_source::IconSource;
use crate::components::content::text_runs::TextLine;
use crate::components::overlays::alert_arming::{AlertInput, Arming, Hearing, Route, route};
use crate::components::overlays::alert_model::{AlertButton, AlertRole, AlertStyle, Suppression};
use dioxus::prelude::*;
use ds_motion::machine::MachineRef;

/// What an alert says and what its buttons do.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Words {
    pub(super) title: String,
    pub(super) message: Option<TextLine>,
    pub(super) style: AlertStyle,
    pub(super) buttons: Vec<AlertButton>,
    pub(super) icon: Option<IconSource>,
    pub(super) suppression: Option<Suppression>,
    pub(super) help: Option<EventHandler<()>>,
    pub(super) dismiss: Option<EventHandler<()>>,
    pub(super) arming: MachineRef<Arming>,
    pub(super) announce: bool,
}

impl Words {
    pub(super) fn roles(&self) -> Vec<AlertRole> {
        self.buttons.iter().map(|button| button.role).collect()
    }

    /// Whether the caller asked to hear a dismissal.
    fn hearing(&self) -> Hearing {
        match self.dismiss {
            Some(_) => Hearing::Dismissal,
            None => Hearing::Buttons,
        }
    }

    /// Carry out what `input` comes to now (nothing while the alert is not armed): Escape, or the
    /// sheet closing another way, is the caller's dismissal when it asked to hear one, else the
    /// Cancel button's press (none when the alert has no Cancel).
    pub(super) fn answer(&self, input: AlertInput) {
        let arming = *self.arming.state().peek();
        match route(arming, input, &self.roles(), self.hearing()) {
            Route::Press(index) => {
                if let Some(button) = self.buttons.get(index) {
                    button.onpress.call(());
                }
            }
            Route::Dismiss => {
                if let Some(dismissed) = self.dismiss {
                    dismissed.call(());
                }
            }
            Route::Nothing => {}
        }
    }
}
