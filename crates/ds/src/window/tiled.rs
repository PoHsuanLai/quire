//! Which edges of a window the compositor has tiled against a neighbour or the screen (the
//! xdg-shell `tiled_*` states). A tiled window squares the corners and drops the shadow on those
//! sides, so a frame reads it; a host that cannot tell reports none.

use ds_core::word::Word;

/// One side of a window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum TileEdge {
    /// The top side.
    Top,
    /// The bottom side.
    Bottom,
    /// The left side.
    Left,
    /// The right side.
    Right,
}

impl TileEdge {
    fn bit(self) -> u8 {
        match self {
            TileEdge::Top => 1,
            TileEdge::Bottom => 2,
            TileEdge::Left => 4,
            TileEdge::Right => 8,
        }
    }
}

/// The set of tiled edges. The default, [`Tiled::NONE`], is a window that floats free.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Tiled(u8);

impl Tiled {
    /// No edge is tiled.
    pub const NONE: Tiled = Tiled(0);

    /// The same set with `edge` added.
    pub fn with(self, edge: TileEdge) -> Tiled {
        Tiled(self.0 | edge.bit())
    }

    /// Whether `edge` is tiled.
    pub fn contains(self, edge: TileEdge) -> bool {
        self.0 & edge.bit() != 0
    }

    /// Whether any edge is tiled.
    pub fn any(self) -> bool {
        self.0 != 0
    }

    /// The tiled edges, in the order top, bottom, left, right.
    pub fn edges(self) -> impl Iterator<Item = TileEdge> {
        TileEdge::ALL
            .iter()
            .copied()
            .filter(move |e| self.contains(*e))
    }
}

impl FromIterator<TileEdge> for Tiled {
    fn from_iter<I: IntoIterator<Item = TileEdge>>(edges: I) -> Self {
        edges.into_iter().fold(Tiled::NONE, Tiled::with)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_holds_exactly_the_edges_put_in() {
        const CASES: &[(&[TileEdge], &[TileEdge])] = &[
            (&[], &[]),
            (&[TileEdge::Left], &[TileEdge::Left]),
            (
                &[TileEdge::Right, TileEdge::Top, TileEdge::Right],
                &[TileEdge::Top, TileEdge::Right],
            ),
            (
                &[
                    TileEdge::Left,
                    TileEdge::Bottom,
                    TileEdge::Right,
                    TileEdge::Top,
                ],
                &[
                    TileEdge::Top,
                    TileEdge::Bottom,
                    TileEdge::Left,
                    TileEdge::Right,
                ],
            ),
        ];
        for (put, want) in CASES {
            let tiled: Tiled = put.iter().copied().collect();
            assert_eq!(tiled.edges().collect::<Vec<_>>(), *want, "{put:?}");
            assert_eq!(tiled.any(), !want.is_empty(), "{put:?}");
            for edge in TileEdge::ALL {
                assert_eq!(
                    tiled.contains(*edge),
                    want.contains(edge),
                    "{put:?} {edge:?}"
                );
            }
        }
    }

    #[test]
    fn the_default_floats_free() {
        assert_eq!(Tiled::default(), Tiled::NONE);
        assert!(!Tiled::NONE.any());
    }
}
