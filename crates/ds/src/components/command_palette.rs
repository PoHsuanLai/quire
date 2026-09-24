//! CommandPalette: "the same menu, just bigger and centred" (design/04-COMPONENTS.md section 25).
//!
//! A SearchField over an embedded Rich menu. Keys come from the search field
//! (design/06-INTERACTIONS.md section 2.3): Up and Down move the selection CLAMPED, Enter closes
//! and then runs the selection, Escape closes the topmost layer only; every other key reaches
//! the caller's `onkey` after the palette's own. Ranking and grouping are the consumer's
//! (section 11); the palette marks the query in each title with the same fuzzy matcher. The
//! palette lists what it is given flat: a submenu parent is drawn with its chevron but runs
//! nothing, and a disabled item is drawn and skipped as in a menu.
//!
//! It is hosted two ways ([`CommandPaletteHost`]): over the window on a scrim that closes on a
//! pointer down outside the card, or embedded in a surface of its own (a shell launcher's
//! panel, sill FINDINGS Q40), where it draws no scrim and its card fills its container.

use crate::components::menu::MenuKind;
use crate::components::menu_entry::MenuEntry;
use crate::components::menu_lines::{Act, Choice, Nav, choices, liveness, moved_live};
use crate::components::menu_rows::{Drawn, render_lines};
use crate::components::palette_lines::{PaletteKey, choice_lines, headed, marked, palette_key};
use crate::components::palette_rows::{SelectedLine, use_row_rects};
use crate::components::palette_select::use_palette_selection;
use crate::components::popover::{Dismiss, Float, Stacking, use_entrance, use_float};
use crate::components::search_field::SearchField;
use crate::components::text_input::Focus;
use crate::components::vocab::Availability;
use crate::focus::request::FocusRequest;
use crate::geometry::{MountedRef, Point, Rect};
use crate::motion::anim::Anim;
use crate::tokens::{Corner, ZLayer};
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
/// anchors to. `onkey` hears every key the field gets, after the palette has read it.
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
    #[props(default)] onkey: Option<EventHandler<KeyboardData>>,
    #[props(default)] corner: Option<Corner>,
) -> Element {
    let float = use_float(ZLayer::Palette, Stacking::Layer(Dismiss::EscOnly));
    let presence = use_entrance(entrance.anim());
    let selection = use_palette_selection(&query, selected, on_select);
    let rects = use_row_rects(on_select_rect);
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
    rects.follow(at_line);
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
        move |event: KeyboardData| {
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
                        rects.mounted(line, MountedRef(event.data()), at_line);
                    }
                }),
                onrelease: None,
            },
        )
    };
    let field = match focus {
        Some(request) => Focus::Controlled(request),
        None => Focus::OnMount,
    };
    let card = rsx! {
        div {
            class: "ds-palette",
            id,
            role: "dialog",
            "aria-label": "{label}",
            "data-host": host.slug(),
            "data-entrance": entrance.slug(),
            "data-presence": presence.slug(),
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
    hosted(host, float, card, onclose)
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
    onclose: EventHandler<()>,
) -> Element {
    match host {
        CommandPaletteHost::Surface => card,
        CommandPaletteHost::Overlay => {
            float.show(
                rsx! {
                    div {
                        class: "ds-palette-wrap",
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
