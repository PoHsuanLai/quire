//! SearchField: a search `TextField` that owns its suggestions panel (`NSSearchField`'s
//! suggestions menu, Spotlight's results). The keyboard never leaves the field: the panel is a
//! `Menu` whose cursor the field drives, hung from the field so a press on the field is inside
//! it, or, as a card, drawn inline under the field in one surface with it. `step` holds what each
//! key does and `highlight` which row is lit; this file wires them to the field and the menu.

use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_focus::FieldFocus;
use crate::components::fields::text_field_model::{FieldBezel, FieldKind};
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu::choices::Choice;
use crate::components::menus::menu::choices::{Act as Do, choices, liveness};
use crate::components::menus::menu::cursor::MenuCursor;
use crate::components::menus::menu::hung::{Hung, PanelWidth};
use crate::components::menus::menu::menu::Menu;
use crate::components::menus::menu::placement::MenuPlacement;
use crate::components::menus::search::card::card;
use crate::components::menus::search::highlight::{initial, shown_at, value_at};
use crate::components::menus::search::model::{
    CardPlace, EscapeOrder, InitialHighlight, SearchCursor, SuggestionSection, SuggestionsPresent,
    flatten,
};
use crate::components::menus::search::step::{Act, SuggestKey, Suggesting, Text, press};
use crate::components::overlays::flow::Flow;
use crate::host::measure::{Anchor, MountedRef};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::units::Point;
use ds_core::vocab::Shown;
use ds_style::tokens::control_size::ControlSize;
use std::rc::Rc;

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

/// What a move of the highlight tells the caller, and where it is kept.
#[derive(Clone)]
struct Moves<T: 'static> {
    picks: Rc<Vec<Choice<T>>>,
    /// The highlighted choice when the move was asked for.
    from: Option<usize>,
    state: Signal<Suggesting>,
    told: EventHandler<Option<T>>,
}

impl<T: Clone + 'static> Moves<T> {
    /// The field is now in `next`; the caller hears of a new highlight.
    fn to(&self, next: Suggesting) {
        if next.cursor != self.from {
            self.told.call(value_at(&self.picks, next.cursor));
        }
        let mut state = self.state;
        state.set(next);
    }
}

/// A search field with a suggestions panel under it. `suggestions` are the sections to list: with
/// none, it is a search `TextField`. The panel opens as the caret arrives or the text changes,
/// is as wide as `width` says and flips above the field when there is no room below.
///
/// The field keeps the keyboard throughout. Up and Down move the highlight (wrapping, skipping
/// disabled rows; Down reopens a closed panel), Enter picks the highlighted row through `onpick`,
/// or, with none highlighted, submits the text through `onsubmit`. Escape takes one step a press,
/// in the order `escape` says (the panel, the text, then the window; or the text, then the
/// window). Tab is the field's own unless `onkey` takes it: the caret leaves and the panel
/// closes. A press on the field is inside the panel and leaves it open; a press anywhere else
/// closes it. The pointer over a row highlights it and a click picks it.
///
/// `highlight` says whether new results light their first row (`TopHit`), so Return picks it.
/// The highlighted row is the field's own, or the caller's: `cursor: SearchCursor::Is(value)`
/// lights the row picking `value`, and `on_highlight` hears each move the keys, the pointer or
/// `highlight` ask for, so a caller can complete the text of the row that is lit on Tab (read in
/// `onkey`, which hears every key the field leaves alone).
///
/// `present` is `Popup` (a menu under the field) or `Card` (the field and its results as one
/// surface, with `place` floating it at the window level and `ondismiss` hearing a press outside
/// it or an Escape that nothing else took; with no `place` it is drawn where it stands).
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
    #[props(default)] highlight: InitialHighlight,
    #[props(default)] cursor: SearchCursor<T>,
    #[props(default)] on_highlight: EventHandler<Option<T>>,
    #[props(default)] escape: EscapeOrder,
    #[props(default)] present: SuggestionsPresent,
    #[props(default)] place: Option<CardPlace>,
    #[props(default)] ondismiss: EventHandler<()>,
    oninput: EventHandler<String>,
    onpick: EventHandler<T>,
    #[props(default)] onsubmit: EventHandler<String>,
    #[props(default)] onkey: EventHandler<KeyboardEvent>,
    #[props(default)] focus: FieldFocus,
    #[props(default)] onfocus: EventHandler<()>,
    #[props(default)] onblur: EventHandler<()>,
    #[props(default)] common: Common,
) -> Element {
    let mut state = use_signal(Suggesting::closed);
    let mut field = use_signal(|| None::<MountedRef>);
    let rows: Vec<MenuItem<T>> = flatten(&suggestions);
    // Results that change light their top hit again, whatever moved the highlight before.
    use_effect(use_reactive!(|rows| {
        if highlight == InitialHighlight::TopHit {
            let picks = choices(&rows);
            let at = initial(highlight, &liveness(&picks));
            on_highlight.call(value_at(&picks, at));
            state.with_mut(|state| state.cursor = at);
        }
    }));
    let picks = Rc::new(choices(&rows));
    let live = liveness(&picks);
    let lit = shown_at(&cursor, &picks, state().cursor);
    let now = Suggesting {
        shown: state().shown,
        cursor: lit,
    };
    let open = now.is_open(live.len());
    let moves = Moves {
        picks: picks.clone(),
        from: lit,
        state,
        told: on_highlight,
    };
    let fresh = Suggesting {
        shown: Shown::Visible,
        cursor: initial(highlight, &live),
    };
    let mounted = common.mounted;
    let keys_live = live.clone();
    let enter_live = live.clone();
    let typed = value.clone();
    let enter_value = value.clone();
    let enter_picks = picks.clone();
    let (type_moves, key_moves, enter_moves, focus_moves) =
        (moves.clone(), moves.clone(), moves.clone(), moves.clone());
    let anchor = field().map(Anchor::Mounted);
    let bezel = match present {
        SuggestionsPresent::Popup => FieldBezel::Bezeled,
        SuggestionsPresent::Card => FieldBezel::Plain,
    };
    let text_field = rsx! {
        TextField {
            label,
            value,
            placeholder,
            size,
            tokens,
            bezel,
            kind: FieldKind::Search,
            focus,
            oninput: move |next: String| {
                type_moves.to(fresh);
                oninput.call(next);
            },
            onfocus: move |()| {
                focus_moves.to(fresh);
                onfocus.call(());
            },
            onblur: move |()| {
                state.set(Suggesting::closed());
                onblur.call(());
            },
            onkey: move |event: KeyboardEvent| {
                let Some(key) = suggest_key(&event) else {
                    return onkey.call(event);
                };
                let (next, act) = press(now, key, &keys_live, text_of(&typed), escape);
                key_moves.to(next);
                match act {
                    Act::Pass => onkey.call(event),
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
                let (next, act) = press(now, SuggestKey::Enter, &enter_live, text_of(&enter_value), escape);
                enter_moves.to(next);
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
    };
    let on_active = move |to: Option<usize>| {
        if let Some(to) = to {
            moves.to(Suggesting {
                shown: Shown::Visible,
                cursor: Some(to),
            });
        }
    };
    match present {
        SuggestionsPresent::Popup => rsx! {
            {text_field}
            if open && let Some(anchor) = anchor {
                Menu::<T> {
                    placement: MenuPlacement::Popup,
                    anchor,
                    items: rows,
                    hung: Hung::FromField(width),
                    active: MenuCursor::Controlled(lit),
                    on_active,
                    onpick: move |value: T| onpick.call(value),
                    onclose: move |()| state.set(Suggesting::closed()),
                }
            }
        },
        SuggestionsPresent::Card => {
            let results = open.then(|| {
                rsx! {
                    Menu::<T> {
                        placement: MenuPlacement::Popup,
                        anchor: Anchor::Point(Point::default()),
                        flow: Flow::Inline,
                        items: rows,
                        active: MenuCursor::Controlled(lit),
                        on_active,
                        onpick: move |value: T| {
                            state.set(Suggesting::closed());
                            onpick.call(value);
                        },
                        onclose: move |()| {},
                    }
                }
            });
            card(text_field, results, place, ondismiss)
        }
    }
}
