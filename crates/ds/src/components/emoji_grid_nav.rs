//! How the arrow keys move through an emoji grid (sill Q291): pure, beside `emoji_grid`.
//!
//! Left and Right walk the cells in reading order, so they wrap from a row's end to the next
//! row's start and back, as the reference's character viewer does; they stop at the first and
//! the last cell. Up and Down move a whole row in the same column; Down into a short last row
//! lands on its last cell. Past the top row or the bottom row the grid has nowhere to go: a
//! palette takes that as leaving the grid for its neighbouring groups, a grid on its own stays.

/// One arrow key in a grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridStep {
    /// Up a row.
    Up,
    /// Down a row.
    Down,
    /// The previous cell.
    Left,
    /// The next cell.
    Right,
}

/// Where an arrow key goes from a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridMove {
    /// To this cell (perhaps the same one).
    To(usize),
    /// Out past this edge.
    Out(GridEdge),
}

/// A grid's edge that Up or Down crosses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GridEdge {
    /// Above the first row.
    Top,
    /// Below the last row.
    Bottom,
}

/// Where `step` goes from cell `selected` in a grid of `count` cells, `columns` wide (a width of
/// 0 reads as 1). A selection past the end is taken as the last cell.
pub fn grid_step(selected: usize, step: GridStep, count: usize, columns: u8) -> GridMove {
    if count == 0 {
        return GridMove::To(0);
    }
    let columns = usize::from(columns.max(1));
    let last = count - 1;
    let at = selected.min(last);
    match step {
        GridStep::Left => GridMove::To(at.saturating_sub(1)),
        GridStep::Right => GridMove::To((at + 1).min(last)),
        GridStep::Up if at < columns => GridMove::Out(GridEdge::Top),
        GridStep::Up => GridMove::To(at - columns),
        GridStep::Down if at + columns <= last => GridMove::To(at + columns),
        GridStep::Down if at / columns < last / columns => GridMove::To(last),
        GridStep::Down => GridMove::Out(GridEdge::Bottom),
    }
}

#[cfg(test)]
mod tests {
    use super::{GridEdge, GridMove, GridStep, grid_step};

    #[test]
    fn arrows_move_in_two_dimensions_wrapping_rows_and_leaving_at_the_top_and_bottom() {
        use GridMove::{Out, To};
        use GridStep::{Down, Left, Right, Up};
        // 13 cells, 8 wide: row 0 is 0..=7, row 1 is 8..=12.
        #[rustfmt::skip]
        let cases: &[(usize, GridStep, GridMove)] = &[
            (0, Right, To(1)),
            (0, Left, To(0)),
            (7, Right, To(8)),
            (8, Left, To(7)),
            (12, Right, To(12)),
            (0, Down, To(8)),
            (4, Down, To(12)),
            (3, Up, Out(GridEdge::Top)),
            (8, Up, To(0)),
            (12, Up, To(4)),
            (12, Down, Out(GridEdge::Bottom)),
            (9, Down, Out(GridEdge::Bottom)),
            (99, Left, To(11)),
        ];
        for &(from, step, want) in cases {
            assert_eq!(grid_step(from, step, 13, 8), want, "{from} {step:?}");
        }
    }

    #[test]
    fn a_degenerate_grid_stays_put() {
        assert_eq!(grid_step(0, GridStep::Down, 0, 8), GridMove::To(0));
        assert_eq!(grid_step(1, GridStep::Down, 3, 0), GridMove::To(2));
        assert_eq!(
            grid_step(2, GridStep::Down, 3, 0),
            GridMove::Out(GridEdge::Bottom)
        );
    }
}
