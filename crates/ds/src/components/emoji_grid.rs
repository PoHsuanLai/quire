//! EmojiGrid: emoji as a grid of cells, moved through with the arrow keys in two dimensions
//! (sill Q291; design/04-COMPONENTS.md section 46). On its own it takes the keyboard itself; in
//! a command palette it is one group ([`GroupEntries::Grid`](crate::GroupEntries)) and the
//! palette's field keeps the keyboard, moving through it with the palette's cursor.
//!
//! The glyphs paint in colour through `.ds-emoji-text` (the `--font-emoji` stack). A cell's name
//! is its accessible label and its Fly tooltip, never a caption under the cell.

use crate::components::emoji_grid_nav::{GridMove, GridStep, grid_step};
use crate::components::tooltip::{Tooltip, TooltipKind};
use crate::components::vocab::Selection;
use crate::geometry::Px;
use dioxus::prelude::*;

/// One emoji: what picking it yields, the characters drawn, and its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmojiCell<T> {
    /// What picking it yields.
    pub value: T,
    /// The emoji itself (a skin tone already applied).
    pub glyph: String,
    /// Its name ("grinning face"): the cell's label and tooltip.
    pub name: String,
}

/// A grid's cells and its shape: what a palette's grid group holds.
#[derive(Debug, Clone, PartialEq)]
pub struct EmojiCells<T> {
    /// The cells, in reading order.
    pub cells: Vec<EmojiCell<T>>,
    /// Cells per row (0 reads as 1).
    pub columns: u8,
    /// A cell's side.
    pub cell: Px,
}

/// The launcher's cell side (design/13 section 13.3.9: eight 56 px cells in the results column).
pub const EMOJI_CELL: Px = Px(56.0);

/// The launcher's columns.
pub const EMOJI_COLUMNS: u8 = 8;

/// What a grid's cells report, by cell number.
#[derive(Clone, Copy)]
pub(crate) struct CellEvents {
    /// A click on a cell.
    pub pick: EventHandler<usize>,
    /// The pointer moved over a cell.
    pub point: EventHandler<usize>,
    /// A cell's element mounted.
    pub mounted: Option<EventHandler<(usize, MountedEvent)>>,
}

/// The grid's inline style: its columns and rows at the cell's side.
pub(crate) fn grid_style(columns: u8, cell: Px) -> String {
    let side = cell.0;
    format!(
        "grid-template-columns:repeat({},{side}px);grid-auto-rows:{side}px",
        columns.max(1)
    )
}

/// The cells, `selected` highlighted.
pub(crate) fn draw_cells<T>(
    cells: &[EmojiCell<T>],
    selected: Option<usize>,
    events: CellEvents,
) -> Element {
    rsx! {
        for (index , cell) in cells.iter().enumerate() {
            div {
                key: "{index}",
                class: "ds-emoji-cell",
                role: "gridcell",
                "aria-selected": Selection::of(&Some(index), &selected).aria(),
                "aria-label": "{cell.name}",
                onmousemove: move |_| events.point.call(index),
                onclick: move |_| events.pick.call(index),
                onmounted: move |event| {
                    if let Some(mounted) = events.mounted {
                        mounted.call((index, event));
                    }
                },
                Tooltip { kind: TooltipKind::Fly, text: cell.name.clone(),
                    span { class: "ds-emoji-glyph", "aria-hidden": "true", "{cell.glyph}" }
                }
            }
        }
    }
}

/// A grid of emoji on its own, taking the keyboard when focused: the arrows move the selection
/// (Left and Right wrap rows, Up and Down stay in the grid at its edges), Enter or Space picks
/// it. The selection is the caller's: `selected` is drawn and a move asks for another through
/// `on_select` (from no selection, the first key asks for cell 0). A click picks; the pointer
/// over a cell asks to select it. `label` names the grid for a screen reader.
#[component]
pub fn EmojiGrid<T: Clone + PartialEq + 'static>(
    cells: Vec<EmojiCell<T>>,
    onpick: EventHandler<T>,
    #[props(default = EMOJI_COLUMNS)] columns: u8,
    #[props(default = EMOJI_CELL)] cell: Px,
    #[props(default)] selected: Option<usize>,
    #[props(default)] on_select: Option<EventHandler<usize>>,
    #[props(default = "Emoji".to_string())] label: String,
) -> Element {
    let count = cells.len();
    let ask = move |next: usize| {
        if let Some(on_select) = on_select
            && selected != Some(next)
        {
            on_select.call(next);
        }
    };
    let values: Vec<T> = cells.iter().map(|cell| cell.value.clone()).collect();
    let pick = {
        let values = values.clone();
        move |index: usize| {
            if let Some(value) = values.get(index) {
                onpick.call(value.clone());
            }
        }
    };
    let keys = {
        let pick = pick.clone();
        move |event: KeyboardEvent| {
            let step = match event.key() {
                Key::ArrowUp => Some(GridStep::Up),
                Key::ArrowDown => Some(GridStep::Down),
                Key::ArrowLeft => Some(GridStep::Left),
                Key::ArrowRight => Some(GridStep::Right),
                key if is_pick(&key) => {
                    event.prevent_default();
                    if let Some(at) = selected {
                        pick(at);
                    }
                    None
                }
                _ => None,
            };
            let Some(step) = step else {
                return;
            };
            event.prevent_default();
            let next = match (
                selected,
                grid_step(selected.unwrap_or(0), step, count, columns),
            ) {
                (None, _) => 0,
                (Some(_), GridMove::To(next)) => next,
                (Some(at), GridMove::Out(_)) => at,
            };
            if count > 0 {
                ask(next);
            }
        }
    };
    let body = draw_cells(
        &cells,
        selected,
        CellEvents {
            pick: EventHandler::new(pick),
            point: EventHandler::new(ask),
            mounted: None,
        },
    );
    rsx! {
        div {
            class: "ds-emoji-grid ds-emoji-text",
            role: "grid",
            "aria-label": "{label}",
            tabindex: "0",
            style: grid_style(columns, cell),
            onkeydown: keys,
            {body}
        }
    }
}

/// Enter and Space pick the selected cell.
fn is_pick(key: &Key) -> bool {
    match key {
        Key::Enter => true,
        Key::Character(text) => text == " ",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{grid_style, is_pick};
    use crate::geometry::Px;
    use dioxus::prelude::Key;

    #[test]
    fn the_grid_lays_its_columns_at_the_cell_side() {
        assert_eq!(
            grid_style(8, Px(56.0)),
            "grid-template-columns:repeat(8,56px);grid-auto-rows:56px"
        );
        assert_eq!(
            grid_style(0, Px(40.0)),
            "grid-template-columns:repeat(1,40px);grid-auto-rows:40px"
        );
    }

    #[test]
    fn enter_and_space_pick() {
        assert!(is_pick(&Key::Enter));
        assert!(is_pick(&Key::Character(" ".to_string())));
        assert!(!is_pick(&Key::Character("a".to_string())));
        assert!(!is_pick(&Key::Tab));
    }
}
