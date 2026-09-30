//! The order of a pinned-tile grid after one tile is dragged to another's place (design/30
//! section 2.11): pure, so the drop's arithmetic is tested without a pointer.

/// `order` with `key` taken out and put where the tile at index `to` was: the tiles between shift
/// one place toward the gap it left. A key not in `order`, or a `to` past the end, leaves it as
/// it was.
pub fn moved<K: Clone + PartialEq>(order: &[K], key: &K, to: usize) -> Vec<K> {
    let Some(from) = order.iter().position(|held| held == key) else {
        return order.to_vec();
    };
    if to >= order.len() {
        return order.to_vec();
    }
    let mut next = order.to_vec();
    let taken = next.remove(from);
    next.insert(to, taken);
    next
}

#[cfg(test)]
mod tests {
    use super::moved;

    #[test]
    fn a_tile_takes_the_place_it_is_dropped_on() {
        // (order, dragged, dropped on, result)
        const CASES: &[(&[u8], u8, usize, &[u8])] = &[
            (&[1, 2, 3, 4], 1, 2, &[2, 3, 1, 4]),
            (&[1, 2, 3, 4], 4, 0, &[4, 1, 2, 3]),
            (&[1, 2, 3, 4], 2, 2, &[1, 3, 2, 4]),
            (&[1, 2, 3, 4], 3, 2, &[1, 2, 3, 4]),
            (&[1, 2, 3, 4], 9, 1, &[1, 2, 3, 4]),
            (&[1, 2, 3, 4], 1, 7, &[1, 2, 3, 4]),
            (&[], 1, 0, &[]),
        ];
        for &(order, key, to, want) in CASES {
            assert_eq!(moved(order, &key, to), want, "{order:?} {key} to {to}");
        }
    }
}
