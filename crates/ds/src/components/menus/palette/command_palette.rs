//! CommandPalette: "the same menu, just bigger and centred" (design/04-COMPONENTS.md section 25).
//!
//! A SearchField over an embedded Rich menu. Keys come from the search field
//! (design/06-INTERACTIONS.md section 2.3): Up and Down move the selection CLAMPED, Enter closes
//! and then runs the selection, Escape closes the topmost layer only; every key then reaches
//! the caller's `onkey` as the event itself, so the caller can `prevent_default` a key it takes. Ranking and grouping are the consumer's
//! (section 11); the palette marks the query in each title with the same fuzzy matcher. The
//! palette lists what it is given flat: a submenu parent is drawn with its chevron but runs
//! nothing, and a disabled item is drawn and skipped as in a menu.
//!
//! Groups are rows or an emoji grid, each under its `SectionHeader`, whose
//! trailing action ("Show More") is a stop of the cursor after the group's last row
//! (`palette_stops`). A caller may claim a key before anything else reads it, knowing where the
//! caret is (`claim`), and may set a pane beside the results (`aside`).
//!
//! It is hosted two ways ([`CommandPaletteHost`]): over the window on a scrim that closes on a
//! pointer down outside the card, or embedded in a surface of its own (a shell launcher's
//! panel), where it draws no scrim and its card fills its container. A
//! caller may keep it mounted and say whether it is `shown` (`palette_shown`).

use crate::components::menus::palette::palette_body::{StopEvents, draw_groups};
use crate::components::menus::palette::palette_claim::{Claim, FieldKey};
use crate::components::menus::palette::palette_group::PaletteGroups;
use crate::components::menus::palette::palette_lines::{PaletteKey, palette_key};
use crate::components::menus::palette::palette_motion::{Book, use_action_book, use_list_motion};
use crate::components::menus::palette::palette_reveal::{Reveal, use_reveal};
use crate::components::menus::palette::palette_rows::{SelectedLine, use_revision, use_row_rects};
use crate::components::menus::palette::palette_select::{PaletteSelection, use_palette_selection};
use crate::components::menus::palette::palette_shown::{
    Change, Retain, Seen, Showing, use_showing,
};
use crate::components::menus::palette::palette_stops::{
    Run, Travel, grid_spans, run_of, shown_groups, stops, travel,
};

use crate::components::fields::search_field::SearchField;
use crate::components::fields::text_input_focus::FieldFocus;
use crate::components::menus::palette::palette_host::{card_corner, hosted};
use crate::components::menus::palette::{
    palette_host::{CommandPaletteHost, PaletteEntrance},
    palette_motion::PaletteHandle,
};
use crate::components::overlays::popover::{Dismiss, Float, Stacking, use_float};
use crate::components::overlays::tooltip::Shown;
use crate::core::geometry::units::{Px, Rect};
use crate::core::vocab::Availability;
use crate::focus::caret::{Caret, HostCaret, InitialCaret};
use crate::focus::field::{FieldHandle, use_field_handle};
use crate::focus::request::{FocusRequest, use_focus_request};
use crate::host::measure::MountedRef;
use crate::motion::detail::reveal::RevealCue;
use crate::style::tokens::{layer::ZLayer, shape::Corner};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// The launcher's preview pane width (design/13 section 13.3.9, proposed): what `aside_width`
/// is when not given.
pub const ASIDE_WIDTH: Px = Px(360.0);

/// Search and commands. `host` says where it draws and `entrance` how its card enters; `id`
/// goes on the card (a shell's blur region names it). `focus` hands the field the keyboard
/// again after something else took it (a menu that closed); the field always takes it as it
/// mounts.
///
/// `groups` are [`PaletteGroups`]: a `Vec<PaletteGroup<T>>` (rows or an emoji grid, each with an
/// optional header action), or the older `Vec<(String, Vec<MenuEntry<T>>)>`. The cursor rests
/// on *stops*: each row, each emoji cell, and each header action, numbered in order with a
/// group's action after its last row or cell (`palette_stops`).
///
/// The selection is the palette's own unless `selected` is given, in which case it shows that
/// stop (clamped) and Up, Down and the pointer only ask for another through `on_select`.
/// Uncontrolled, `on_select` hears every change of the selection, a new query's reset to the
/// first stop included. `on_select_rect` hears the selected row's or cell's rect, in client
/// coordinates, whenever the selection or the element under it changes: what an actions menu
/// anchors to; it is never an empty rect (a row read before layout is read again), and a header
/// action reports none.
///
/// `claim` hears every key the field gets first, with the caret's place ([`FieldKey`]):
/// answering [`Claim::Take`] keeps the key from the palette, the field (its default is
/// prevented: a Space is not typed, a Right does not move the caret) and `onkey`. `onkey`
/// hears every key not claimed, after the palette has read it, as the event itself: call
/// `prevent_default` on a key the caller takes (Tab, Ctrl+K) so the document does not also act
/// on it (Tab would move the focus off the field).
///
/// `aside` is drawn beside the results, under the field, past a hairline divider, `aside_width`
/// wide: over the window the card widens by that much; in a surface the card still fills its
/// container, so the host widens the surface (sill's launcher panel) and the results column
/// narrows by it otherwise. Use it for a [`PreviewPane`](crate::PreviewPane).
///
/// `shown` keeps the palette mounted while hidden: `Some(Shown::Hidden)` lays out nothing and
/// leaves the layer stack, and each change to `Some(Shown::Visible)` replays the entrance,
/// empties the query (through `oninput`) and goes back to the first stop unless `retain` is
/// `Retain::Query`, and gives the field the keyboard. `None` is always shown.
///
/// `initial_caret` is where the field's caret goes each time the palette gives it the keyboard
/// (as it mounts, as it is shown again, and when `focus` asks): after the query by default, as
/// Spotlight does, so a palette opened on a query reads Right as [`Caret::AtEnd`] at once.
/// A `focus` request that places the caret itself (`with_select_all`, `with_caret`) keeps
/// its own placement.
///
/// The list keeps the selected stop in view whoever moved it: it scrolls the least
/// that shows it, aligning it with the nearer edge, never centring it, and does not animate the
/// scroll. A stop the pointer selected is left where it is.
///
/// `reveal` plays the rows' first-show rise (design/26 R13): each row, header and
/// grid `rise`s at `--t-move --e-out`, `--stagger` after the one before (capped at 12), on each
/// opening with `FirstShow::Animate`, or on each new Appear of the caller's own `use_detail` cue
/// (the first result set after an opening); a Change (a later result set) replaces in place with
/// no stagger. The default, `FirstShow::Still`, never rises. A group's own action (Show More,
/// Show Less) plays section 5.4's group expand whoever passes a cue: the rows it adds
/// rise in downward; the rows it removes go and what follows heals up by their height.
///
/// `corner` gives the card a squircle corner (`Corner::Squircle`, the launcher's) or another
/// radius; absent, it keeps `--r-panel`.
///
/// `handle` ([`use_palette_handle`]) lets a caller that runs a group's action itself (`sill debug
/// launcher-key enter`, a demo's own button — never the palette's own Enter or
/// click) still get Show More's rise or Show Less's heal: call its `mark_group_action(group)`
/// before making the change that follows. Absent, or the change turns out not to be that
/// group's, the change plays nothing, same as any other new result set.
#[component]
pub fn CommandPalette<T: Clone + PartialEq + 'static>(
    label: String,
    placeholder: String,
    query: String,
    tokens: Vec<String>,
    #[props(into)] groups: PaletteGroups<T>,
    empty: String,
    oninput: EventHandler<String>,
    onpick: EventHandler<T>,
    onclose: EventHandler<()>,
    #[props(default)] host: CommandPaletteHost,
    #[props(default)] entrance: PaletteEntrance,
    #[props(default)] id: Option<String>,
    #[props(default)] focus: Option<FocusRequest>,
    #[props(default)] selected: Option<usize>,
    #[props(default)] on_select: Option<EventHandler<usize>>,
    #[props(default)] on_select_rect: Option<EventHandler<Rect>>,
    #[props(default)] onkey: Option<EventHandler<KeyboardEvent>>,
    #[props(default)] claim: Option<Callback<FieldKey, Claim>>,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] retain: Retain,
    #[props(default)] corner: Option<Corner>,
    #[props(default)] aside: Option<Element>,
    #[props(default = ASIDE_WIDTH)] aside_width: Px,
    #[props(default)] initial_caret: InitialCaret,
    #[props(into, default)] reveal: RevealCue,
    #[props(default)] handle: Option<PaletteHandle>,
) -> Element {
    let float = use_float(ZLayer::Palette, Stacking::Layer(Dismiss::EscOnly));
    let showing = use_showing(shown, entrance.anim());
    let selection = use_palette_selection(&query, selected, on_select);
    let rects = use_row_rects(on_select_rect);
    let in_view = use_reveal();
    let own_actions = use_action_book();
    let actions = handle.map(PaletteHandle::book).unwrap_or(own_actions);
    let key = groups.key();
    let revision = use_revision(&(tokens.clone(), key.clone()));
    let own_focus = use_focus_request();
    let handle = use_field_handle();
    let host_caret = try_use_context::<HostCaret>();
    let request = landing(focus.unwrap_or(own_focus), initial_caret);
    let shown_groups = shown_groups(&groups.0, &query);
    let motion = use_list_motion(
        reveal,
        showing.change,
        &key,
        &shown_groups,
        Book {
            actions,
            stops: in_view,
        },
    );
    let all = stops(&shown_groups);
    let grids = grid_spans(&shown_groups);
    let live: Vec<Availability> = all.iter().map(|stop| stop.availability()).collect();
    let count = all.len();
    let current = selection.current(count);
    let at_line = (count > 0).then_some(SelectedLine {
        choice: current,
        line: current,
    });
    selection.report(current, count);
    rects.follow(at_line, revision);
    in_view.follow(at_line.map(|at| at.choice), revision);
    follow_showing(
        showing,
        Turn {
            float,
            selection: selection.clone(),
            rects,
            reveal: in_view,
            retain,
            oninput,
            request,
        },
    );
    let run = {
        let all = all.clone();
        move |index: usize| match run_of(all.get(index)) {
            Run::Pick(value) => {
                onclose.call(());
                onpick.call(value);
            }
            Run::Action(action) => {
                actions.ran();
                action.call(());
            }
            Run::Nothing => {}
        }
    };
    let field_key = {
        let run = run.clone();
        let live = live.clone();
        let selection = selection.clone();
        move |event: KeyboardEvent| {
            if let Some(claim) = claim {
                let caret = caret_of(host_caret, handle);
                let key = FieldKey {
                    event: event.clone(),
                    caret,
                };
                if claim.call(key) == Claim::Take {
                    event.prevent_default();
                    return;
                }
            }
            match palette_key(&event.key()) {
                Some(PaletteKey::Move(step)) => {
                    if let Some(next) = travel(current, Travel::Vertical(step), &live, &grids) {
                        selection.select(next);
                    }
                }
                Some(PaletteKey::Side(step)) => {
                    if let Some(next) = travel(current, Travel::Side(step), &live, &grids) {
                        event.prevent_default();
                        selection.select(next);
                    }
                }
                Some(PaletteKey::Run) => run(current),
                Some(PaletteKey::Close) if float.takes_escape() => onclose.call(()),
                Some(PaletteKey::Close) | None => {}
            }
            if let Some(onkey) = onkey {
                onkey.call(event);
            }
        }
    };
    let body = if count == 0 {
        rsx! {
            div { class: "ds-menu-empty", "{empty}" }
        }
    } else {
        draw_groups(
            &shown_groups,
            current,
            StopEvents {
                pick: EventHandler::new(run),
                point: EventHandler::new({
                    let live = live.clone();
                    let selection = selection.clone();
                    move |index: usize| {
                        let enabled = live.get(index) == Some(&Availability::Enabled);
                        if enabled && index != current {
                            in_view.pointed(index, revision);
                            selection.select(index);
                        }
                    }
                }),
                mounted: onmounted_stop(rects, in_view, at_line, revision),
                action_mounted: EventHandler::new(move |(index, event): (usize, MountedEvent)| {
                    in_view.stop_mounted(index, MountedRef(event.data()));
                }),
                action_ran: EventHandler::new(move |()| actions.ran()),
            },
            &motion,
        )
    };
    let field = match showing.seen {
        Seen::Yes => FieldFocus::Controlled(request),
        Seen::Not => FieldFocus::Manual,
    };
    let side = aside.map(|pane| {
        rsx! {
            div {
                class: "ds-palette-aside",
                style: "width:{aside_width.0}px",
                {pane}
            }
        }
    });
    let widened = side.as_ref().map(|_| "shown");
    let card_style = card_style(corner, side.as_ref().map(|_| aside_width));
    let card = rsx! {
        div {
            class: "ds-palette",
            id,
            role: "dialog",
            "aria-label": "{label}",
            "data-host": host.slug(),
            "data-entrance": entrance.slug(),
            "data-presence": showing.presence().slug(),
            "data-shown": showing.shown.slug(),
            "data-pulse": showing.alias(),
            "data-corner": corner.and_then(Corner::attribute),
            "data-aside": widened,
            style: card_style,
            onpointerdown: move |event| event.stop_propagation(),
            SearchField {
                label: label.clone(),
                value: query.clone(),
                placeholder,
                tokens,
                oninput,
                onkey: field_key,
                focus: field,
                handle: Some(handle),
            }
            div {
                class: "ds-menu",
                "data-kind": "rich",
                "data-embed": "palette",
                role: "listbox",
                onmousedown: move |event| event.prevent_default(),
                "data-reveal": motion.reveal_slug(),
                style: motion.heal_style(),
                onmounted: move |event| in_view.list_mounted(MountedRef(event.data())),
                {body}
            }
            {side}
        }
    };
    hosted(host, entrance, float, card, showing.shown, onclose)
}

/// A stop's element mounted: book it, report it if it is the selected one, and keep it in view.
fn onmounted_stop(
    rects: crate::components::menus::palette::palette_rows::RowRects,
    reveal: Reveal,
    at_line: Option<SelectedLine>,
    revision: crate::components::menus::palette::palette_rows::Revision,
) -> EventHandler<(usize, MountedEvent)> {
    EventHandler::new(move |(index, event): (usize, MountedEvent)| {
        reveal.stop_mounted(index, MountedRef(event.data()));
        rects.mounted(index, MountedRef(event.data()), at_line, revision);
    })
}

/// The card's inline style: its corner's, and the pane's width for the card to widen by.
fn card_style(corner: Option<Corner>, aside: Option<Px>) -> Option<String> {
    let corner = corner.map(card_corner);
    let aside = aside.map(|width| format!("--aside:{}px;", width.0));
    match (corner, aside) {
        (None, None) => None,
        (corner, aside) => Some(format!(
            "{}{}",
            corner.unwrap_or_default(),
            aside.unwrap_or_default()
        )),
    }
}

/// Where the field's caret is, through the host's read; `Unknown` without a host or a field.
fn caret_of(host: Option<HostCaret>, handle: FieldHandle) -> Caret {
    match (host, handle.element()) {
        (Some(HostCaret(read)), Some(element)) => read(&element),
        (None, _) | (_, None) => Caret::Unknown,
    }
}

/// The field's focus request with the caret placed at `caret`, unless the caller's request already
/// places it (`with_select_all`, `with_caret`).
fn landing(request: FocusRequest, caret: InitialCaret) -> FocusRequest {
    match request.caret() {
        Some(_) => request,
        None => request.with_caret(caret),
    }
}

/// What a change of `shown` acts on.
struct Turn {
    float: Float,
    selection: PaletteSelection,
    rects: crate::components::menus::palette::palette_rows::RowRects,
    reveal: Reveal,
    retain: Retain,
    oninput: EventHandler<String>,
    request: FocusRequest,
}

/// After a render that showed or hid the palette: shown again, it rejoins the layer stack,
/// replays its entrance, reports its row afresh, starts over unless it retains the query, and
/// takes the keyboard; hidden, it leaves the stack and drops a row read still waiting.
fn follow_showing(showing: Showing, turn: Turn) {
    match showing.change {
        Change::Stay => {}
        Change::Hide => queue_effect(move || {
            turn.float.withdraw();
            turn.rects.forget();
            turn.reveal.forget();
        }),
        Change::Show => queue_effect(move || {
            turn.float.rejoin();
            showing.replay();
            turn.rects.forget();
            turn.reveal.forget();
            if turn.retain == Retain::Nothing {
                turn.selection.reset();
                turn.oninput.call(String::new());
            }
            turn.request.request();
        }),
    }
}
