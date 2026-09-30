//! A table's column widths: what each column was given, what a person has dragged it to, and the
//! grid template that lays the header and every row out by them.

use crate::components::lists::table::model::TableColumn;
use ds_core::geometry::units::Px;

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

    /// `grid-template-columns`: every column its width, the last taking what is left (never
    /// under its own least).
    pub(crate) fn template<C>(&self, columns: &[TableColumn<C>]) -> String {
        let last = columns.len().saturating_sub(1);
        (0..columns.len())
            .map(|at| {
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
    use super::Widths;
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
    fn the_template_gives_the_last_column_the_rest() {
        let columns = columns();
        let widths = Widths::default().dragged(&columns, 0, Px(200.0));
        assert_eq!(widths.template(&columns), "200px 90px minmax(48px, 1fr)");
    }
}
