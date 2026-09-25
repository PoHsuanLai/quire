//! TreeItem: a place in a sidebar tree, a `details`/`summary` row in the sidebar item's chrome
//! (mailo gaps 6; design/04-COMPONENTS.md sections 19 and 34).
//!
//! mailo's folder tree is folders inside folders: each folder a `details` whose `summary` is
//! the row and whose body holds the subfolders. `SidebarItem` is a single `button` and cannot
//! hold children, so mailo drew its own rows, chevron and drop styling beside quire's. This is
//! that row, drawn by quire: the sidebar item's look, a chevron that turns as the folder opens,
//! the same drop states (one shared rule set, `.ds-drop-place`), and a trailing slot for the ⋯
//! button that never toggles the folder.

use crate::components::count::{Count, CountPlace};
use crate::components::press::Press;
use crate::components::row_hooks::relay;
use crate::components::sidebar_item::PlaceId;
use crate::components::text_runs::Text;
use crate::components::tree_item_parts::{
    Row, label_part, leaf_chevron, open_attribute, trailing_slot,
};
use crate::components::vocab::{DropState, Expanded, Here};
use crate::icon::Icon;
use dioxus::prelude::*;

/// Whether a tree item's children are showing. Controlled: the item draws what it is given and
/// reports a toggle through `on_toggle`, so the app's own state (a folder the person closed
/// stays closed across a re-render) is the only state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Disclosure {
    /// The children show; the chevron points down.
    #[default]
    Open,
    /// Only the row shows; the chevron points right.
    Closed,
}

impl Disclosure {
    /// The other state: what a press on the row asks for.
    pub fn flip(self) -> Self {
        match self {
            Disclosure::Open => Disclosure::Closed,
            Disclosure::Closed => Disclosure::Open,
        }
    }

    /// The same fact as a trigger's `aria-expanded`.
    fn expanded(self) -> Expanded {
        match self {
            Disclosure::Open => Expanded::Open,
            Disclosure::Closed => Expanded::Closed,
        }
    }
}

/// Whether an item can hold others. A leaf has no `details` to open: it is drawn as a plain
/// row with the chevron's space kept, so its label lines up with its siblings'.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TreeShape {
    /// It holds children: a `details` with a turning chevron.
    #[default]
    Branch,
    /// It holds nothing: a row, no chevron, `children` ignored.
    Leaf,
}

/// A place in a sidebar tree.
///
/// `open` is the item's state and `on_toggle` hears the state a press on the row asks for
/// (the summary's own toggle is prevented, so nothing changes until the app says so).
/// `label` is a [`Text`]; with `onselect` it is a button of its own that selects the place and
/// does not toggle (mailo's folder name), without it a span, and a press on it toggles like the
/// rest of the row. `glyph` is drawn before the label, `count` after it, then `trailing`: a slot
/// for the ⋯ `IconButton`, fenced so no press in it reaches the summary (give the button
/// `Propagation::Stop` as well; it costs nothing). `here` marks the current place
/// (`aria-current`, the sidebar item's current look).
///
/// `editing` is an in-place rename (mailo gaps 7): given, it is drawn in the label's place (in
/// `span.ds-tree-item-edit[data-slot=editing]`, at the label's metrics, so nothing on the row
/// moves) instead of the label and its select button. Give it a `TextInput` with
/// `variant: FieldFace::Bare` (it takes the row's face) and `focus: Focus::Controlled(request)`
/// from `use_focus_request().with_select_all()` (or `Focus::OnMount`), and handle Enter and
/// Escape in its `onkey`; take the slot away to end the rename. A press in it never toggles or
/// selects the row; keys are the field's (they start there), and bubble on as any key does.
///
/// The trailing slot carries `data-slot="trailing"`: select on that, never on `.ds-*`, to style
/// your own element inside it (`[*|data-slot=trailing] .fold-more`).
///
/// A drag over the tree works as over the sidebar (design/06 section 6.1): `place` is written
/// `data-place`, the pointer hooks hand over the row's pointer events, and `drop` is drawn by
/// the rules `SidebarItem` uses (`.ds-drop-place`): `Accepts` outlined, `Target` lit and grown,
/// `Source` dimmed.
///
/// `children` are the nested items (more `TreeItem`s), drawn in `div.ds-tree-item-children`
/// (`role=group`), indented one `--s-12` step per level.
#[component]
pub fn TreeItem(
    #[props(into)] label: Text,
    open: Disclosure,
    on_toggle: EventHandler<Disclosure>,
    #[props(default)] shape: TreeShape,
    #[props(default)] glyph: Option<Icon>,
    #[props(default)] count: Option<u32>,
    #[props(default)] here: Here,
    #[props(default)] onselect: Option<EventHandler<Press>>,
    #[props(default)] trailing: Option<Element>,
    #[props(default)] editing: Option<Element>,
    #[props(default)] drop: DropState,
    #[props(default)] place: Option<PlaceId>,
    #[props(default)] onpointerenter: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerleave: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointermove: Option<EventHandler<PointerEvent>>,
    #[props(default)] onpointerup: Option<EventHandler<PointerEvent>>,
    #[props(default)] children: Element,
) -> Element {
    let row = Row {
        current: here.aria_current(),
        drop_attr: drop.drop_attr(),
        drag_attr: drop.drag_attr(),
        place: place.map(|PlaceId(name)| name),
    };
    let count = count.map(|value| rsx! { Count { value, place: CountPlace::Item } });
    let body = rsx! {
        if let Some(icon) = glyph {
            crate::icon::render::Glyph { icon, size: crate::icon::render::IconSize::Base }
        }
        {label_part(label, onselect, editing)}
        {count}
        if let Some(element) = trailing {
            {trailing_slot(element)}
        }
    };
    match shape {
        TreeShape::Leaf => rsx! {
            div { class: "ds-tree-item", "data-shape": "leaf",
                div {
                    class: "ds-tree-item-row ds-drop-place",
                    "aria-current": row.current,
                    "data-drop": row.drop_attr,
                    "data-drag": row.drag_attr,
                    "data-place": row.place,
                    onpointerenter: relay(onpointerenter),
                    onpointerleave: relay(onpointerleave),
                    onpointermove: relay(onpointermove),
                    onpointerup: relay(onpointerup),
                    {leaf_chevron()}
                    {body}
                }
            }
        },
        TreeShape::Branch => rsx! {
            details { class: "ds-tree-item", ..open_attribute(open),
                summary {
                    class: "ds-tree-item-row ds-drop-place",
                    "aria-expanded": open.expanded().aria(),
                    "aria-current": row.current,
                    "data-drop": row.drop_attr,
                    "data-drag": row.drag_attr,
                    "data-place": row.place,
                    onpointerenter: relay(onpointerenter),
                    onpointerleave: relay(onpointerleave),
                    onpointermove: relay(onpointermove),
                    onpointerup: relay(onpointerup),
                    onclick: move |event| {
                        // The summary's own toggle would change the `details` behind the app's
                        // back; the app's state decides, through `open`.
                        event.prevent_default();
                        on_toggle.call(open.flip());
                    },
                    span { class: "ds-tree-item-chevron", "aria-hidden": "true",
                        crate::icon::render::Glyph { icon: Icon::ChevronRight, size: crate::icon::render::IconSize::Tiny }
                    }
                    {body}
                }
                div { class: "ds-tree-item-children", role: "group", {children} }
            }
        },
    }
}
