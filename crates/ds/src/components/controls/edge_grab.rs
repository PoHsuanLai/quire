//! A pointer holding the edge of a column or the divider of a split view (design/30 sections 2.6
//! and 2.7): where it went down and the size it started from, so the size follows the pointer 1:1.

use ds_core::geometry::units::Px;

/// One grab of an edge.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct EdgeGrab {
    from: Px,
    size: Px,
}

impl EdgeGrab {
    /// The pointer went down at `from` on an edge whose neighbour was `size` wide.
    pub(crate) fn new(from: Px, size: Px) -> Self {
        EdgeGrab { from, size }
    }

    /// The size the neighbour has with the pointer at `at`.
    pub(crate) fn size_at(self, at: Px) -> Px {
        Px(self.size.0 + at.0 - self.from.0)
    }
}

#[cfg(test)]
mod tests {
    use super::EdgeGrab;
    use ds_core::geometry::units::Px;

    #[test]
    fn the_size_follows_the_pointer_from_where_it_went_down() {
        const CASES: &[(&str, f32, f32)] = &[
            ("at the press", 300.0, 200.0),
            ("to the right", 340.0, 240.0),
            ("to the left", 250.0, 150.0),
        ];
        let grab = EdgeGrab::new(Px(300.0), Px(200.0));
        for &(name, at, want) in CASES {
            assert_eq!(grab.size_at(Px(at)), Px(want), "{name}");
        }
    }
}
