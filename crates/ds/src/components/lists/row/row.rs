//! Row: one line of a list, a menu's kin (`NSTableCellView`, design/30 section 2.6). A leading
//! element, a title, a detail, a trailing accessory and the row's state are one anatomy for
//! settings rows, sidebar rows, palette results and mail rows: the last hands its own `content`
//! in place of the words.

use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::disclosure::Collapsing;
use crate::components::controls::key_equivalent::KeyStyle;
use crate::components::controls::press::PressListeners;
use crate::components::controls::progress::busy::{busy_class, use_busy_seen};
use crate::components::lists::row::accessory::{self, Accessory};
use crate::components::lists::row::action::RowAction;
use crate::components::lists::row::action::trailing as action_button;
use crate::components::lists::row::chord::{RowChord, shown_chord};
use crate::components::lists::row::confirm::{self, RowConfirm};
use crate::components::lists::row::disclosure_button::RowDisclosure;
use crate::components::lists::row::leading::{self, RowLeading};
use crate::components::lists::row::marks::marked;
use crate::components::lists::row::motion::RowMotion;
use crate::components::lists::row::shape::RowShape;
use crate::components::lists::row::shape_view;
use crate::components::lists::row::size::RowSize;
use crate::focus::click::kept_click;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::Availability;
use ds_core::vocab::{Emphasis, FocusStyle, RowState, Shown};
use ds_core::word::Word;

/// Whether a row is part of an outline, and whether it opens one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Outline {
    /// Not part of an outline.
    #[default]
    None,
    /// A row with nothing under it: no triangle, its space kept so its title lines up with its
    /// siblings'.
    Leaf,
    /// A row that opens the `children` under it.
    Branch(Shown),
}

/// A hook a row calls when its element mounts, held apart from the scope that made it: a parent
/// that lists its rows and is itself gone by the time the renderer reports the mount (a palette
/// unmounted a frame after it mounted) is not called, where an `EventHandler` it owns would panic.
#[derive(Clone)]
pub struct RowMounted(std::rc::Rc<dyn Fn(MountedEvent)>);

impl RowMounted {
    /// A hook running `hook`.
    pub fn new(hook: impl Fn(MountedEvent) + 'static) -> Self {
        RowMounted(std::rc::Rc::new(hook))
    }

    fn call(&self, event: MountedEvent) {
        (self.0)(event);
    }
}

impl std::fmt::Debug for RowMounted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RowMounted")
    }
}

/// Equal only to itself: a hook is made anew each render.
impl PartialEq for RowMounted {
    fn eq(&self, other: &Self) -> bool {
        std::rc::Rc::ptr_eq(&self.0, &other.0)
    }
}

/// Call `handler` with `event`, if there is one.
pub(crate) fn relay(handler: Option<EventHandler<PointerEvent>>) -> impl FnMut(PointerEvent) {
    move |event| {
        if let Some(handler) = handler {
            handler.call(event);
        }
    }
}

/// The words: the title marked where a query matched it, or its runs as given.
fn title_words(title: &TextLine, marks: &[usize]) -> Element {
    match title {
        TextLine::Plain(plain) if !marks.is_empty() => marked(plain, marks),
        TextLine::Plain(_) | TextLine::Runs(_) => text(title),
    }
}

/// One row. `onclick` hears a press on the row (a toggle's own press is the toggle's).
///
/// `state` is the row's [`RowState`]: `selection` (`aria-selected`), `emphasis` (an unread row
/// is `Strong`), `availability` (`Busy` swaps the accessory for the small spinner and takes no
/// press) and `drop` (its part in a drag). `marks` are the title's matched characters, by index.
/// `content` replaces the title and detail with the caller's own (a mail row's name, subject and
/// tags). `edit` is the same place held by a field the person types in (a folder renamed where it
/// stands: a `TextField` with `FieldBezel::Plain` takes the row's own face): the row keeps its
/// leading, accessory and action, takes no press or context menu while it is there
/// (`data-editing`), and the keys, presses and drags inside the field stay the field's, so
/// the list around it neither moves its cursor, jumps by typeahead nor picks on them. `action` is a button at the end that acts without picking the row; `chord` is the keys
/// of the row's first action, shown while the row is selected.
///
/// `confirm` makes the row ask a question in its own line: the words give way to it, the accessory,
/// chord and action to a Cancel and a confirming button, and the row is not picked meanwhile
/// ([`RowConfirm`]).
///
/// `outline` makes the row a branch that opens its `children`, or a leaf that keeps the
/// triangle's space; `on_toggle` hears the state a press on the triangle asks for. The pointer
/// hooks hand the row's own pointer events to the caller (a drag over the sidebar). `motion` is
/// the row's part in a motion of its list's rows (a palette group's Show More or Less).
/// `onmounted` hears the row's element as it mounts (see [`RowMounted`]).
#[component]
pub fn Row(
    #[props(default)] leading: RowLeading,
    #[props(into)] title: TextLine,
    #[props(default)] marks: Vec<usize>,
    #[props(default)] detail: Option<TextLine>,
    #[props(default)] content: Option<Element>,
    #[props(default)] edit: Option<Element>,
    #[props(default)] accessory: Accessory,
    #[props(default)] action: Option<RowAction>,
    #[props(default)] chord: RowChord,
    #[props(default)] confirm: Option<RowConfirm>,
    #[props(default)] shape: RowShape,
    #[props(default)] state: RowState,
    #[props(default)] size: RowSize,
    #[props(default)] outline: Outline,
    #[props(default)] motion: RowMotion,
    #[props(default)] onmounted: Option<RowMounted>,
    #[props(default)] on_toggle: Option<EventHandler<Shown>>,
    #[props(default)] onclick: Option<EventHandler<Press>>,
    #[props(default)] onpointerenter: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerleave: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerdown: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointermove: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerup: Option<EventHandler<PointerEvent>>,
    #[props(default)] children: Element,
    #[props(default)] common: Common,
) -> Element {
    let RowState {
        selection,
        emphasis,
        availability,
        drop,
    } = state;
    let busy_seen = use_busy_seen(availability);
    let asking = confirm.is_some();
    let editing = edit.is_some();
    let live = availability == Availability::Enabled && !asking && !editing;
    let listen = onclick.map(PressListeners::new);
    let words = match (&confirm, edit, content) {
        (Some(asked), _, _) => confirm::question(asked),
        (None, Some(field), _) => rsx! {
            span {
                class: "ds-row-words ds-row-edit",
                onkeydown: |event| event.stop_propagation(),
                onkeyup: |event| event.stop_propagation(),
                onclick: |event| {
                    event.stop_propagation();
                    kept_click(&event);
                },
                oncontextmenu: |event| event.stop_propagation(),
                onmousedown: |event| event.stop_propagation(),
                onpointerdown: |event| event.stop_propagation(),
                {field}
            }
        },
        (None, None, Some(content)) => rsx! {
            span { class: "ds-row-words", {content} }
        },
        (None, None, None) => shape_view::words(
            &shape,
            title_words(&title, &marks),
            detail.as_ref().map(text),
        ),
    };
    let when = shape_view::when(&shape).filter(|_| !asking);
    let keys = shown_chord(&chord, selection).cloned().filter(|_| !asking);
    let answer = confirm.as_ref().map(confirm::buttons);
    let expanded = match outline {
        Outline::Branch(shown) => Some(shown.aria()),
        Outline::None | Outline::Leaf => None,
    };
    let triangle = match outline {
        Outline::None => rsx! {},
        Outline::Leaf => rsx! {
            span { class: "ds-row-disclosure", "data-outline": "leaf" }
        },
        Outline::Branch(shown) => rsx! {
            RowDisclosure { shown, on_toggle }
        },
    };
    let name = title.plain_text();
    let data = common.data_attributes();
    let mounted = common.clone();
    let row = rsx! {
        div {
            class: busy_class(common.class("ds-row"), busy_seen),
            id: common.id.clone(),
            role: "option",
            "aria-label": common.aria_label.clone(),
            "aria-selected": selection.aria(),
            "aria-checked": accessory.checked(),
            "aria-expanded": expanded,
            "aria-disabled": availability.aria_disabled(),
            "aria-busy": availability.aria_busy(),
            "data-density": size.slug(),
            "data-leading": leading::row_slug(&leading, &shape),
            "data-emphasis": emphasis_slug(emphasis),
            "data-focus": FocusStyle::Highlight.slug(),
            "data-drop": drop.drop_attr(),
            "data-drag": drop.drag_attr(),
            "data-shape": shape_view::slug(&shape),
            "data-trailing": accessory.slug(),
            "data-action": action.as_ref().map(|_| "true"),
            "data-confirm": confirm.as_ref().map(|_| "true"),
            "data-editing": editing.then_some("true"),
            "data-row-motion": motion.attr(),
            onpointerenter: relay(onpointerenter),
            onpointerleave: relay(onpointerleave),
            onpointerdown: relay(onpointerdown),
            onpointermove: relay(onpointermove),
            onpointerup: relay(onpointerup),
            onclick: move |event| {
                if let (true, Some(listen)) = (live, listen) {
                    listen.click(&event);
                }
            },
            oncontextmenu: move |event| {
                if let (true, Some(listen)) = (live, listen) {
                    listen.context_menu(&event);
                }
            },
            onmounted: move |event| {
                mounted.mounted(event.clone());
                if let Some(hook) = &onmounted {
                    hook.call(event);
                }
            },
            ..data,
            {triangle}
            {leading::draw(&leading, &shape)}
            {words}
            {when}
            if !asking {
                {accessory::draw(&accessory, &name, availability)}
            }
            if let Some(keys) = keys {
                crate::components::controls::key_equivalent::KeyEquivalent { shortcut: keys, style: KeyStyle::Text }
            }
            if let (Some(action), false) = (action.as_ref(), asking) {
                {action_button(action)}
            }
            {answer}
        }
    };
    match outline {
        Outline::Branch(shown) => rsx! {
            div { class: "ds-row-branch", role: "none",
                {row}
                Collapsing { shown, {children} }
            }
        },
        Outline::None | Outline::Leaf => row,
    }
}

/// The `data-emphasis` word.
fn emphasis_slug(emphasis: Emphasis) -> &'static str {
    emphasis.slug()
}
