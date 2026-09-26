//! UserPicturePicker: the letter and the 42 animated emoji as a grid of discs, the current
//! choice marked, for Settings' Users page and first run (design/25-EMOJI.md section 7).
//!
//! Not an [`crate::EmojiGrid`]: that grid draws text glyphs in the colour font and yields a
//! caller's value per glyph, while this one draws each choice as the picture itself (the letter
//! disc and `AnimatedEmoji` still frames, 64 px) and behaves as one radio group. It shares the
//! grid's arrow-key rule ([`grid_step`]) and its inline column style.

use super::choice::PictureChoice;
use crate::components::avatar::{AvatarFace, AvatarSize, face};
use crate::components::emoji::{AnimatedEmoji, EmojiId, EmojiPlayback};
use crate::components::emoji_grid::grid_style;
use crate::components::emoji_grid_nav::{GridMove, GridStep, grid_step};
use crate::components::user_picture::PictureSize;
use crate::geometry::Px;
use dioxus::prelude::*;

/// A picker cell's side: the 64 px disc and 6 px around it.
pub const PICTURE_CELL: Px = Px(76.0);

/// The picker's default columns.
pub const PICTURE_COLUMNS: u8 = 8;

/// One cell: what picking it chooses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Letter,
    Emoji(EmojiId),
}

impl Cell {
    fn choice(self) -> PictureChoice {
        match self {
            Cell::Letter => PictureChoice::Letter,
            Cell::Emoji(emoji) => PictureChoice::Emoji(emoji),
        }
    }
}

/// Every cell in order: the letter, then the set in `EmojiId::ALL`'s order.
fn cells() -> Vec<Cell> {
    std::iter::once(Cell::Letter)
        .chain(EmojiId::ALL.into_iter().map(Cell::Emoji))
        .collect()
}

/// The cell `choice` marks: the letter or its emoji; none for `Auto` or `Photo`, which the
/// Settings page offers as its own rows beside the picker.
pub(crate) fn marked(choice: PictureChoice) -> Option<usize> {
    match choice {
        PictureChoice::Letter => Some(0),
        PictureChoice::Emoji(emoji) => EmojiId::ALL
            .iter()
            .position(|each| *each == emoji)
            .map(|at| at + 1),
        PictureChoice::Auto | PictureChoice::Photo => None,
    }
}

/// The key's step, if it is an arrow.
fn step_of(key: &Key) -> Option<GridStep> {
    match key {
        Key::ArrowUp => Some(GridStep::Up),
        Key::ArrowDown => Some(GridStep::Down),
        Key::ArrowLeft => Some(GridStep::Left),
        Key::ArrowRight => Some(GridStep::Right),
        _ => None,
    }
}

/// The picture choices as one radio group: `letter` (the user's initial disc, drawn at 64
/// whatever size it carries) and every emoji of the set as a still frame, 64 px, `columns`
/// across. `choice` is the caller's and is marked (`aria-checked`); a click, or an arrow key
/// when the group has the keyboard (the grid's rule: Left and Right wrap rows, Up and Down stay
/// inside), asks for another through `onpick`. Nothing plays: 43 loops at once is motion nobody
/// asked for (`EmojiPlayback::Still`).
#[component]
pub fn UserPicturePicker(
    letter: AvatarFace,
    choice: PictureChoice,
    onpick: EventHandler<PictureChoice>,
    #[props(default = PICTURE_COLUMNS)] columns: u8,
    #[props(default = "Picture".to_string())] label: String,
) -> Element {
    let all = cells();
    let count = all.len();
    let at = marked(choice);
    let keys = {
        let all = all.clone();
        move |event: KeyboardEvent| {
            let Some(step) = step_of(&event.key()) else {
                return;
            };
            event.prevent_default();
            let next = match (at, grid_step(at.unwrap_or(0), step, count, columns)) {
                (None, _) => 0,
                (Some(_), GridMove::To(next)) => next,
                (Some(here), GridMove::Out(_)) => here,
            };
            if at != Some(next)
                && let Some(cell) = all.get(next)
            {
                onpick.call(cell.choice());
            }
        }
    };
    rsx! {
        div {
            class: "ds-picture-picker",
            role: "radiogroup",
            "aria-label": "{label}",
            tabindex: "0",
            style: grid_style(columns, PICTURE_CELL),
            onkeydown: keys,
            for (index , cell) in all.into_iter().enumerate() {
                div {
                    key: "{index}",
                    class: "ds-picture-cell",
                    role: "radio",
                    "aria-checked": if at == Some(index) { "true" } else { "false" },
                    "aria-label": cell_name(cell, letter.initial),
                    onclick: move |_| onpick.call(cell.choice()),
                    {cell_body(cell, letter)}
                }
            }
        }
    }
}

fn cell_name(cell: Cell, initial: char) -> String {
    match cell {
        Cell::Letter => format!("Letter {initial}"),
        Cell::Emoji(emoji) => emoji.name().to_owned(),
    }
}

fn cell_body(cell: Cell, letter: AvatarFace) -> Element {
    match cell {
        Cell::Letter => face(AvatarFace {
            size: AvatarSize::Size64,
            ..letter
        }),
        Cell::Emoji(emoji) => rsx! {
            AnimatedEmoji { emoji, size: PictureSize::Medium, playback: EmojiPlayback::Still }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{cells, marked};
    use crate::components::emoji::EmojiId;
    use crate::components::user_picture::PictureChoice;

    #[test]
    fn the_letter_comes_first_then_the_whole_set() {
        assert_eq!(cells().len(), EmojiId::ALL.len() + 1);
        let cases = [
            (PictureChoice::Letter, Some(0)),
            (PictureChoice::Emoji(EmojiId::Grinning), Some(1)),
            (PictureChoice::Emoji(EmojiId::Fox), Some(EmojiId::ALL.len())),
            (PictureChoice::Auto, None),
            (PictureChoice::Photo, None),
        ];
        for (choice, want) in cases {
            assert_eq!(marked(choice), want, "{choice:?}");
        }
        for (index, cell) in cells().into_iter().enumerate() {
            assert_eq!(marked(cell.choice()), Some(index), "{cell:?}");
        }
    }
}
