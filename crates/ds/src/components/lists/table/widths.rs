//! A table's column widths: what each column was given, what a person has dragged it to, and the
//! grid template that lays the header and every row out by them.

use crate::components::lists::table::model::TableColumn;
use ds_core::geometry::units::Px;
use std::ops::Range;

/// A header title's average glyph, in logical pixels: `--fs-help` at weight 600 is about 11 px,
/// and a glyph is a little over half that (the estimate the capsule's readouts use too).
const TITLE_GLYPH: f32 = 6.6;
/// What a header keeps beside its title for the sort indicator and the gap before it.
const SORT_ROOM: f32 = 14.0;
/// A cell's (and a header's) padding on both sides together: `--s-8` each.
const CELL_PADDING: f32 = 16.0;

/// How wide a column needs to be to show its title and every one of `cells` (each cell's content
/// width, padding left out) in full: the widest of them, and the cells' padding. The title's
/// width is estimated from its characters; the cells' are the caller's.
pub(crate) fn fit_width(title: &str, cells: impl IntoIterator<Item = Px>) -> Px {
    let title = title.chars().count() as f32 * TITLE_GLYPH + SORT_ROOM;
    let widest = cells.into_iter().map(|cell| cell.0).fold(title, f32::max);
    Px((widest + CELL_PADDING).ceil())
}

/// The width of each column: a dragged one wins over the column's own.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Widths(Vec<Option<Px>>);

impl Widths {
    /// Column `at` dragged to `width`, held at the column's least.
    pub(crate) fn dragged<C>(mut self, columns: &[TableColumn<C>], at: usize, width: Px) -> Self {
        let Some(column) = columns.get(at) else {
            return self;
        };
        if self.0.len() < columns.len() {
            self.0.resize(columns.len(), None);
        }
        self.0[at] = Some(Px(width.0.max(column.min_width.0)));
        self
    }

    /// Column `at`'s width now.
    pub(crate) fn of<C>(&self, columns: &[TableColumn<C>], at: usize) -> Px {
        self.0
            .get(at)
            .copied()
            .flatten()
            .or_else(|| columns.get(at).map(|column| column.width))
            .unwrap_or(Px(0.0))
    }

    /// How wide all the columns are together, the last at its own width.
    pub(crate) fn total<C>(&self, columns: &[TableColumn<C>]) -> Px {
        self.total_in(columns, 0..columns.len())
    }

    /// How wide the columns `span` are together.
    pub(crate) fn total_in<C>(&self, columns: &[TableColumn<C>], span: Range<usize>) -> Px {
        Px(span.map(|at| self.of(columns, at).0).sum())
    }

    /// `grid-template-columns`: every column its width, the last taking what is left (never
    /// under its own least).
    pub(crate) fn template<C>(&self, columns: &[TableColumn<C>]) -> String {
        self.template_in(columns, 0..columns.len())
    }

    /// `grid-template-columns` for the columns `span` of `columns`: every one its width, except
    /// the last column of the whole table, which takes what is left (never under its own
    /// least). A span that stops short of it is all fixed widths.
    pub(crate) fn template_in<C>(&self, columns: &[TableColumn<C>], span: Range<usize>) -> String {
        let last = columns.len().saturating_sub(1);
        span.map(|at| {
            if at == last {
                let least = columns.get(at).map_or(0.0, |column| column.min_width.0);
                format!("minmax({least}px, 1fr)")
            } else {
                format!("{}px", self.of(columns, at).0)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::{Widths, fit_width};
    use crate::components::lists::table::model::TableColumn;
    use ds_core::geometry::units::Px;

    fn columns() -> Vec<TableColumn<u8>> {
        vec![
            TableColumn::new(0, "Name", Px(160.0)),
            TableColumn::new(1, "Kind", Px(90.0)),
            TableColumn::new(2, "Size", Px(70.0)),
        ]
    }

    #[test]
    fn a_dragged_column_wins_and_stops_at_its_least() {
        let columns = columns();
        let widths = Widths::default().dragged(&columns, 1, Px(120.0));
        assert_eq!(widths.of(&columns, 1), Px(120.0));
        assert_eq!(
            widths.of(&columns, 0),
            Px(160.0),
            "an untouched column keeps its own"
        );
        let narrow = widths.dragged(&columns, 1, Px(10.0));
        assert_eq!(narrow.of(&columns, 1), Px(48.0), "held at the least");
    }

    #[test]
    fn the_columns_together_are_as_wide_as_their_widths_sum() {
        let columns = columns();
        let widths = Widths::default().dragged(&columns, 1, Px(100.0));
        assert_eq!(widths.total(&columns), Px(330.0));
    }

    #[test]
    fn the_template_gives_the_last_column_the_rest() {
        let columns = columns();
        let widths = Widths::default().dragged(&columns, 0, Px(200.0));
        assert_eq!(widths.template(&columns), "200px 90px minmax(48px, 1fr)");
    }

    #[test]
    fn a_span_of_columns_has_its_own_template_and_total() {
        let columns = columns();
        let widths = Widths::default().dragged(&columns, 0, Px(200.0));
        // (span, template, total)
        let cases = [
            (0..1, "200px", 200.0),
            (0..2, "200px 90px", 290.0),
            (1..3, "90px minmax(48px, 1fr)", 160.0),
            (2..3, "minmax(48px, 1fr)", 70.0),
            (0..0, "", 0.0),
        ];
        for (span, template, total) in cases {
            assert_eq!(
                widths.template_in(&columns, span.clone()),
                template,
                "{span:?}"
            );
            assert_eq!(
                widths.total_in(&columns, span.clone()),
                Px(total),
                "{span:?}"
            );
        }
    }

    #[test]
    fn a_column_fits_its_widest_cell_or_its_title_and_the_padding() {
        // (title, cell contents, wanted)
        let cases: [(&str, &[f32], f32); 4] = [
            ("Name", &[120.0, 300.0, 80.0], 316.0),
            ("Date Modified", &[40.0, 52.0], 116.0),
            ("Size", &[], 57.0),
            ("", &[0.0], 30.0),
        ];
        for (title, cells, want) in cases {
            let got = fit_width(title, cells.iter().map(|cell| Px(*cell)));
            assert_eq!(got, Px(want), "{title:?} {cells:?}");
        }
    }

    #[test]
    fn a_fit_narrower_than_the_least_is_held_at_the_least() {
        let columns = columns();
        let widths = Widths::default().dragged(&columns, 2, fit_width("", [Px(0.0)]));
        assert_eq!(widths.of(&columns, 2), Px(48.0));
    }
}
