//! PickList: a popover with a search field over a list of rows, for choosing one thing or toggling
//! several out of a list too long to scan (a label to add, a folder to move to). It is the
//! command palette's field, rows, keys and marking hung from a control as an `NSPopover` is,
//! rather than centred over the window (design/30 section 2.4).
//!
//! The caller owns the query and the rows, as the palette's caller does: it narrows `groups` by
//! what `oninput` says, and a "Create" row is one more row it adds when nothing matches. Up and
//! Down move the selection, Return picks it, Escape and a press outside close the popover. A
//! row's [`AfterPick`] says whether its pick closes the list: a row that toggles a label keeps it
//! open and the caller redraws the row with its new check, a row that moves the thread closes it.

use crate::components::fields::text_field::TextField;
use crate::components::fields::text_field_focus::FieldFocus;
use crate::components::fields::text_field_model::FieldKind;
use crate::components::lists::row::size::RowSize;
use crate::components::menus::alive::use_alive;
use crate::components::menus::item::item::AfterPick;
use crate::components::menus::palette::palette_body::{StopEvents, draw_groups};
use crate::components::menus::palette::palette_group::PaletteGroups;
use crate::components::menus::palette::palette_lines::{PaletteKey, palette_key};
use crate::components::menus::palette::palette_motion::ListMotion;
use crate::components::menus::palette::palette_reveal::use_reveal;
use crate::components::menus::palette::palette_rows::use_revision;
use crate::components::menus::palette::palette_select::use_palette_selection;
use crate::components::menus::palette::palette_stops::{
    Run, Travel, run_of, shown_groups, stops, travel,
};
use crate::components::overlays::popover::{Arrow, Popover};
use crate::focus::request::use_focus_request;
use crate::host::caret::InitialCaret;
use crate::host::measure::{Anchor, MountedRef};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::geometry::{
    placement::{Align, Placement, Side},
    units::Px,
};
use ds_core::vocab::Availability;

/// Below the control, its start edge level with the control's.
fn below() -> Placement {
    Placement::new(Side::Bottom, Align::Start)
}

/// A filterable list in a popover under `anchor`.
///
/// `label` names the popover and its field; `placeholder` is the field's hint and `empty` what
/// the list says when `groups` is empty. `oninput` hears the query as it is typed; `onpick` the
/// value of a picked row; `onclose` Escape, a press outside, and a pick whose row closes the
/// list (before `onpick`, as the palette does). `arrow` points the popover at its control.
#[component]
pub fn PickList<T: Clone + PartialEq + 'static>(
    anchor: Anchor,
    label: String,
    placeholder: String,
    query: String,
    #[props(into)] groups: PaletteGroups<T>,
    empty: String,
    oninput: EventHandler<String>,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
    #[props(default = below())] placement: Placement,
    #[props(default)] arrow: Arrow,
    #[props(default)] common: Common,
) -> Element {
    let alive = use_alive();
    // A list opened on a query puts the caret after it, as the palette does.
    let focus = use_focus_request().with_caret(InitialCaret::End);
    let selection = use_palette_selection(&query, None, None);
    let in_view = use_reveal();
    let revision = use_revision(&groups.key());
    let shown = shown_groups(&groups.0, &query);
    let all = stops(&shown);
    let live: Vec<Availability> = all.iter().map(|stop| stop.availability()).collect();
    let current = selection.current(all.len());
    in_view.follow((!all.is_empty()).then_some(current), revision);
    let run = {
        let all = all.clone();
        move |index: usize| match run_of(all.get(index)) {
            Run::Pick(value, after) => {
                if after == AfterPick::Close {
                    onclose.call(());
                }
                onpick.call(value);
            }
            Run::Action(action) => action.call(()),
            Run::Nothing => {}
        }
    };
    let onkey = {
        let (run, live, selection) = (run.clone(), live.clone(), selection.clone());
        move |event: KeyboardEvent| match palette_key(&event.key()) {
            Some(PaletteKey::Move(step)) => {
                if let Some(next) = travel(current, Travel::Vertical(step), &live, &[]) {
                    event.prevent_default();
                    selection.select(next);
                }
            }
            Some(PaletteKey::Run) => run(current),
            Some(PaletteKey::Side(_) | PaletteKey::Close) | None => {}
        }
    };
    let body = if all.is_empty() {
        rsx! {
            div { class: "ds-pick-list-empty", "{empty}" }
        }
    } else {
        let point = {
            let (live, selection) = (live.clone(), selection.clone());
            move |index: usize| {
                let enabled = live.get(index) == Some(&Availability::Enabled);
                if enabled && index != current {
                    in_view.pointed(index, revision);
                    selection.select(index);
                }
            }
        };
        let events = StopEvents {
            alive,
            pick: EventHandler::new(run),
            point: EventHandler::new(point),
            mounted: EventHandler::new(move |(index, event): (usize, MountedEvent)| {
                in_view.stop_mounted(index, MountedRef(event.data()));
            }),
            action_mounted: EventHandler::new(move |(index, event): (usize, MountedEvent)| {
                in_view.stop_mounted(index, MountedRef(event.data()));
            }),
            action_ran: EventHandler::new(|()| {}),
            size: RowSize::Compact,
        };
        draw_groups(&shown, current, events, &ListMotion { group: None })
    };
    rsx! {
        Popover { anchor, placement, gap: Px(4.0), arrow, onclose, common,
            div { class: "ds-pick-list", role: "dialog", "aria-label": "{label}",
                TextField {
                    label: label.clone(),
                    value: query.clone(),
                    placeholder,
                    kind: FieldKind::Search,
                    focus: FieldFocus::Controlled(focus),
                    oninput,
                    onkey,
                }
                div {
                    class: "ds-pick-list-rows",
                    role: "listbox",
                    onmousedown: move |event| event.prevent_default(),
                    onmounted: move |event| in_view.list_mounted(MountedRef(event.data())),
                    {body}
                }
            }
        }
    }
}
