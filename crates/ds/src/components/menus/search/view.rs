//! SearchField: a search `TextField` that owns its suggestions panel (`NSSearchField`'s
//! suggestions menu, Spotlight's results). The keyboard never leaves the field: the panel is a
//! `Menu` whose cursor the field drives, hung from the field so a press on the field is inside
//! it. `step` holds what each key does; this file wires it to the field and the menu.

use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_focus::FieldFocus;
use crate::components::fields::text_field_model::FieldKind;
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::choices::{Act as Do, choices, liveness};
use crate::components::menus::menu::cursor::MenuCursor;
use crate::components::menus::menu::hung::{Hung, PanelWidth};
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::components::menus::search::model::{SuggestionSection, flatten};
use crate::components::menus::search::step::{Act, SuggestKey, Suggesting, Text, press};
use crate::host::measure::{Anchor, MountedRef};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_style::tokens::control_size::ControlSize;

/// The key a suggestions panel takes, if `event` is one: a plain Up, Down, Enter or Escape.
fn suggest_key(event: &KeyboardEvent) -> Option<SuggestKey> {
    if !event.modifiers().is_empty() {
        return None;
    }
    match event.key() {
        Key::ArrowUp => Some(SuggestKey::Up),
        Key::ArrowDown => Some(SuggestKey::Down),
        Key::Escape => Some(SuggestKey::Escape),
        _ => None,
    }
}

/// Whether the field holds text.
fn text_of(value: &str) -> Text {
    if value.is_empty() {
        Text::Empty
    } else {
        Text::Filled
    }
}

/// A search field with a suggestions panel under it. `suggestions` are the sections to list: with
/// none, it is a search `TextField`. The panel opens as the caret arrives or the text changes,
/// is as wide as `width` says and flips above the field when there is no room below.
///
/// The field keeps the keyboard throughout. Up and Down move the highlight (wrapping, skipping
/// disabled rows; Down reopens a closed panel), Enter picks the highlighted row through `onpick`,
/// or, with none highlighted, submits the text through `onsubmit`. Escape closes the panel first
/// and clears the field (through `oninput`) second. Tab is the field's own: the caret leaves and
/// the panel closes. A press on the field is inside the panel and leaves it open; a press
/// anywhere else closes it. The pointer over a row highlights it and a click picks it.
///
/// `value` and `oninput` are the controlled pair of a `TextField`; `onpick` hears the picked
/// row's value. A caller that wants the field to act on a pick sets `value` from it.
#[component]
pub fn SearchField<T: Clone + PartialEq + 'static>(
    label: String,
    value: String,
    #[props(default)] placeholder: String,
    #[props(default)] size: ControlSize,
    #[props(default)] tokens: Vec<String>,
    #[props(default)] suggestions: Vec<SuggestionSection<T>>,
    #[props(default = PanelWidth::SameAsField)] width: PanelWidth,
    oninput: EventHandler<String>,
    onpick: EventHandler<T>,
    #[props(default)] onsubmit: EventHandler<String>,
    #[props(default)] focus: FieldFocus,
    #[props(default)] onfocus: EventHandler<()>,
    #[props(default)] onblur: EventHandler<()>,
    #[props(default)] common: Common,
) -> Element {
    let mut state = use_signal(Suggesting::closed);
    let mut field = use_signal(|| None::<MountedRef>);
    let rows: Vec<MenuItem<T>> = flatten(&suggestions);
    let picks = choices(&rows);
    let live = liveness(&picks);
    let open = state().is_open(live.len());
    let mounted = common.mounted;
    let keys_live = live.clone();
    let enter_live = live.clone();
    let typed = value.clone();
    let enter_value = value.clone();
    let enter_picks = picks.clone();
    let anchor = field().map(Anchor::Mounted);
    rsx! {
        TextField {
            label,
            value,
            placeholder,
            size,
            tokens,
            kind: FieldKind::Search,
            focus,
            oninput: move |next: String| {
                state.set(Suggesting::open());
                oninput.call(next);
            },
            onfocus: move |()| {
                state.set(Suggesting::open());
                onfocus.call(());
            },
            onblur: move |()| {
                state.set(Suggesting::closed());
                onblur.call(());
            },
            onkey: move |event: KeyboardEvent| {
                let Some(key) = suggest_key(&event) else {
                    return;
                };
                let (next, act) = press(state(), key, &keys_live, text_of(&typed));
                state.set(next);
                match act {
                    Act::Pass => {}
                    Act::Clear => {
                        event.stop_propagation();
                        event.prevent_default();
                        oninput.call(String::new());
                    }
                    Act::Swallow | Act::Pick(_) | Act::Submit => {
                        event.stop_propagation();
                        event.prevent_default();
                    }
                }
            },
            onsubmit: move |_| {
                let (next, act) = press(state(), SuggestKey::Enter, &enter_live, text_of(&enter_value));
                state.set(next);
                match act {
                    Act::Pick(at) => {
                        if let Some(Do::Pick(value, _)) = enter_picks.get(at).map(|choice| &choice.act) {
                            onpick.call(value.clone());
                        }
                    }
                    Act::Submit => onsubmit.call(enter_value.clone()),
                    Act::Pass | Act::Swallow | Act::Clear => {}
                }
            },
            common: Common {
                mounted: Some(EventHandler::new(move |event: MountedEvent| {
                    field.set(Some(MountedRef(event.data())));
                    if let Some(mounted) = &mounted {
                        mounted.call(event);
                    }
                })),
                ..common
            },
        }
        if open && let Some(anchor) = anchor {
            Menu::<T> {
                placement: MenuPlacement::Popup,
                anchor,
                items: rows,
                hung: Hung::FromField(width),
                active: MenuCursor::Controlled(state().cursor),
                on_active: move |to: Option<usize>| {
                    if let Some(to) = to {
                        state.with_mut(|state| state.cursor = Some(to));
                    }
                },
                onpick: move |value: T| onpick.call(value),
                onclose: move |()| state.set(Suggesting::closed()),
            }
        }
    }
}
