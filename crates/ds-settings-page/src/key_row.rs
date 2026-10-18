//! A key's row: its label and help, and the control its kind gets.

use crate::control::{Control, control_of};
use crate::edit::SettingEdit;
use crate::words::word_label;
use dioxus::prelude::*;
use ds::components::controls::segmented::Tracking;
use ds::components::fields::field_row::FieldRow;
use ds::components::menus::item::item::MenuItem;
use ds::components::menus::pop_up_button::PopUpButton;
use ds::prelude::*;
use ds::root::common::Common;
use ds_settings::schema::KeySpec;

/// A key's row. `value` is the key's current value; a change the person makes reaches `onedit`
/// as the key's path and the new word.
#[component]
pub(crate) fn KeyRow(
    spec: KeySpec,
    value: toml::Value,
    onedit: EventHandler<SettingEdit>,
) -> Element {
    let label = spec.label.0.clone();
    let help = (!spec.help.0.is_empty()).then(|| TextLine::from(spec.help.0.clone()));
    let path = spec.path.clone();
    let current = value.as_str().unwrap_or_default().to_owned();
    let say = move |word: String| {
        onedit.call(SettingEdit {
            path: path.clone(),
            value: toml::Value::String(word),
        })
    };
    let control = match control_of(&spec.kind) {
        Control::Switch(pair) => {
            let on = current == pair.on;
            rsx! {
                Toggle {
                    label: label.clone(),
                    value: if on { Check::On } else { Check::Off },
                    onchange: move |next| {
                        say(match next {
                            Check::On => pair.on.clone(),
                            Check::Off | Check::Mixed => pair.off.clone(),
                        });
                    },
                }
            }
        }
        Control::Segments(words) => {
            let choices = Choice::pairs(
                words
                    .iter()
                    .map(|word| (word.clone(), word_label(&spec, word))),
            );
            rsx! {
                SegmentedControl::<String> {
                    label: label.clone(),
                    choices,
                    tracking: Tracking::SelectOne(current),
                    onchange: move |word: String| say(word),
                }
            }
        }
        Control::PopUp(words) => {
            let items = words
                .iter()
                .map(|word| MenuItem::new(word.clone(), word_label(&spec, word)))
                .collect::<Vec<_>>();
            rsx! {
                PopUpButton::<String> {
                    items,
                    value: Some(current),
                    common: Common { aria_label: Some(label.clone()), ..Common::default() },
                    onpick: move |word: String| say(word),
                }
            }
        }
        Control::Shown => rsx! { Label { text: shown(&value) } },
    };
    rsx! {
        FieldRow { label: TextLine::from(label), help, {control} }
    }
}

/// A value as text for the read-only control: a string as it is, anything else as TOML writes it.
fn shown(value: &toml::Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}
