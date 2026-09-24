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
//! It is hosted two ways ([`CommandPaletteHost`]): over the window on a scrim that closes on a
//! pointer down outside the card, or embedded in a surface of its own (a shell launcher's
//! panel, sill FINDINGS Q40), where it draws no scrim and its card fills its container. A
//! caller may keep it mounted and say whether it is `shown` (`palette_shown`, sill Q63).

use crate::components::menu::MenuKind;
use crate::components::menu_entry::MenuEntry;
use crate::components::menu_lines::{Act, Choice, Nav, choices, liveness, moved_live};
use crate::components::menu_rows::{Drawn, render_lines};
use crate::components::palette_lines::{PaletteKey, choice_lines, headed, marked, palette_key};
use crate::components::palette_rows::{SelectedLine, use_revision, use_row_rects};
use crate::components::palette_select::{PaletteSelection, use_palette_selection};
use crate::components::palette_shown::{Change, Retain, Seen, Showing, use_showing};
use crate::components::popover::{Dismiss, Float, Stacking, use_float};
use crate::components::search_field::SearchField;
use crate::components::text_input::Focus;
use crate::components::tooltip::Shown;
use crate::components::vocab::Availability;
use crate::focus::request::{FocusRequest, use_focus_request};
use crate::geometry::{MountedRef, Point, Rect};
use crate::motion::anim::Anim;
use crate::tokens::{Corner, ZLayer};
use dioxus::core::queue_effect;
use dioxus::prelude::*;

/// Where the palette draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CommandPaletteHost {
    /// Over the window, on the palette layer: a scrim, and the card 11 % down.
    #[default]
    Overlay,
    /// In place, filling its container: no scrim, no layer of its own to draw on (a shell
    /// surface that is the palette). Its card paints the enclosing material.
    Surface,
}

impl CommandPaletteHost {
    fn slug(self) -> &'static str {
        match self {
            CommandPaletteHost::Overlay => "overlay",
            CommandPaletteHost::Surface => "surface",
        }
    }
}

/// How the palette's card enters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PaletteEntrance {
    /// `peek-in`, S's palette.
    #[default]
    PeekIn,
    /// `cmdk-in`, C's command menu.
    CmdkIn,
}

impl PaletteEntrance {
    fn anim(self) -> Anim {
        match self {
            PaletteEntrance::PeekIn => Anim::PeekIn,
            PaletteEntrance::CmdkIn => Anim::CmdkIn,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            PaletteEntrance::PeekIn => "peek-in",
            PaletteEntrance::CmdkIn => "cmdk-in",
        }
    }
}

/// Search and commands. `host` says where it draws and `entrance` how its card enters; `id`
/// goes on the card (a shell's blur region names it). `focus` hands the field the keyboard
/// again after something else took it (a menu that closed); the field always takes it as it
/// mounts.
///
/// The selection is the palette's own unless `selected` is given, in which case it shows that
/// choice (clamped) and Up, Down and the pointer only ask for another through `on_select`.
/// Uncontrolled, `on_select` hears every change of the selection, a new query's reset to the
/// first choice included. `on_select_rect` hears the selected row's rect, in client
/// coordinates, whenever the selection or the row under it changes: what an actions menu
/// anchors to; it is never an empty rect (a row read before layout is read again). `onkey`
/// hears every key the field gets, after the palette has read it, as the event itself: call
/// `prevent_default` on a key the caller takes (Tab, Ctrl+K) so the document does not also act
/// on it (Tab would move the focus off the field).
///
/// `shown` keeps the palette mounted while hidden: `Some(Shown::Hidden)` lays out nothing and
/// leaves the layer stack, and each change to `Some(Shown::Visible)` replays the entrance,
/// empties the query (through `oninput`) and goes back to the first choice unless `retain` is
/// `Retain::Query`, and gives the field the keyboard. `None` is always shown.
///
/// `corner` gives the card a squircle corner (`Corner::Squircle`, the launcher's) or another
/// radius; absent, it keeps `--r-panel`.
#[component]
pub fn CommandPalette<T: Clone + PartialEq + 'static>(
    label: String,
    placeholder: String,
    query: String,
    tokens: Vec<String>,
    groups: Vec<(String, Vec<MenuEntry<T>>)>,
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
    #[props(default)] shown: Option<Shown>,
    #[props(default)] retain: Retain,
    #[props(default)] corner: Option<Corner>,
) -> Element {
    let float = use_float(ZLayer::Palette, Stacking::Layer(Dismiss::EscOnly));
    let showing = use_showing(shown, entrance.anim());
    let selection = use_palette_selection(&query, selected, on_select);
    let rects = use_row_rects(on_select_rect);
    let revision = use_revision(&(tokens.clone(), groups.clone()));
    let own_focus = use_focus_request();
    let request = focus.unwrap_or(own_focus);
    let entries = headed(&groups);
    let shown = marked(&entries, &query);
    let picks = choices(&shown);
    let live = liveness(&picks);
    let lines = choice_lines(&entries);
    let count = picks.len();
    let current = selection.current(count);
    let at_line = lines.get(current).map(|&line| SelectedLine {
        choice: current,
        line,
    });
    selection.report(current, count);
    rects.follow(at_line, revision);
    follow_showing(
        showing,
        Turn {
            float,
            selection: selection.clone(),
            rects,
            retain,
            oninput,
            request,
        },
    );
    let run = {
        let picks = picks.clone();
        move |index: usize| {
            if let Some(Choice {
                act: Act::Pick(value),
                availability: Availability::Enabled,
            }) = picks.get(index)
            {
                onclose.call(());
                onpick.call(value.clone());
            }
        }
    };
    let field_key = {
        let run = run.clone();
        let live = live.clone();
        let selection = selection.clone();
        move |event: KeyboardEvent| {
            match palette_key(&event.key()) {
                Some(PaletteKey::Move(step)) => {
                    selection.select(moved_live(Nav::Clamp, current, &live, step))
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
        render_lines(
            &shown,
            MenuKind::Rich.row(),
            Drawn {
                selected: current,
                open: None,
                onpick: EventHandler::new(run),
                onpoint: EventHandler::new({
                    let live = live.clone();
                    let selection = selection.clone();
                    move |(index, _): (usize, Point)| {
                        let enabled = live.get(index) == Some(&Availability::Enabled);
                        if enabled && index != current {
                            selection.select(index);
                        }
                    }
                }),
                onmounted: EventHandler::new(move |(index, event): (usize, MountedEvent)| {
                    if let Some(&line) = lines.get(index) {
                        rects.mounted(line, MountedRef(event.data()), at_line, revision);
                    }
                }),
                onrelease: None,
            },
        )
    };
    let field = match showing.seen {
        Seen::Yes => Focus::Controlled(request),
        Seen::Not => Focus::Manual,
    };
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
            style: corner.map(card_corner),
            onpointerdown: move |event| event.stop_propagation(),
            SearchField {
                label: label.clone(),
                value: query.clone(),
                placeholder,
                tokens,
                oninput,
                onkey: field_key,
                focus: field,
            }
            div {
                class: "ds-menu",
                "data-kind": "rich",
                "data-embed": "palette",
                role: "listbox",
                onmousedown: move |event| event.prevent_default(),
                {body}
            }
        }
    };
    hosted(host, float, card, showing.shown, onclose)
}

/// What a change of `shown` acts on.
struct Turn {
    float: Float,
    selection: PaletteSelection,
    rects: crate::components::palette_rows::RowRects,
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
        }),
        Change::Show => queue_effect(move || {
            turn.float.rejoin();
            showing.replay();
            turn.rects.forget();
            if turn.retain == Retain::Nothing {
                turn.selection.reset();
                turn.oninput.call(String::new());
            }
            turn.request.request();
        }),
    }
}

/// The card's own corner: its radius, and a squircle's extent and shadow circle.
fn card_corner(corner: Corner) -> String {
    match corner {
        Corner::Squircle(_) => corner.squircle_style(),
        Corner::Token(_) | Corner::Px(_) => format!("border-radius:{};", corner.css()),
    }
}

/// The card where `host` draws it: in place, or on the palette layer over its scrim, which
/// closes the palette on a pointer down outside the card while it is the topmost layer.
fn hosted(
    host: CommandPaletteHost,
    float: Float,
    card: Element,
    shown: Shown,
    onclose: EventHandler<()>,
) -> Element {
    match host {
        CommandPaletteHost::Surface => card,
        CommandPaletteHost::Overlay => {
            float.show(
                rsx! {
                    div {
                        class: "ds-palette-wrap",
                        "data-shown": shown.slug(),
                        onpointerdown: move |_| {
                            if float.is_top() {
                                onclose.call(());
                            }
                        },
                        {card}
                    }
                },
                onclose,
            );
            rsx! {}
        }
    }
}
