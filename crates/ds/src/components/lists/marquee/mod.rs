//! Marquee: the rubber band a drag from empty space draws over a grid of tiles or a table of rows,
//! and the selection it makes. One behaviour for `CollectionView` and `VirtualTable`: the pure
//! geometry is in `model`, the pointer's drag and the scroll at the edges in `hook`, the drawn
//! band in `band`.

pub(crate) mod band;
pub(crate) mod hook;
pub(crate) mod model;
