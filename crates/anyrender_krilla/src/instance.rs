//! Which instance of a variable font a run was shaped at, in the units krilla takes.
//!
//! anyrender passes a run's variation as normalized coordinates (F2Dot14, one per `fvar` axis,
//! after `avar`), which is what a rasteriser needs. krilla takes user-space axis values
//! (`wght` 700) and instantiates the subset at them itself, applying `avar` on the way. So the
//! painter undoes both steps: `avar` inverted (its segment maps are monotonic, so the inverse is
//! the same piecewise-linear map read the other way), then the `fvar` normalization.

use skrifa::raw::TableProvider as _;
use skrifa::{FontRef, MetadataProvider as _};

/// One axis at one user-space value: the tag's four bytes and the value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AxisValue {
    pub(crate) tag: [u8; 4],
    pub(crate) value: f32,
}

/// The user-space values of `coords` (normalized, F2Dot14) in `font`; empty for a static font
/// or a run at the default instance (all zero), which krilla embeds as it is.
pub(crate) fn user_values(font: &FontRef<'_>, coords: &[i16]) -> Vec<AxisValue> {
    if coords.iter().all(|&coord| coord == 0) {
        return Vec::new();
    }
    let maps = segment_maps(font);
    font.axes()
        .iter()
        .zip(coords)
        .enumerate()
        .map(|(index, (axis, &coord))| {
            let normalized = f32::from(coord) / 16384.0;
            let before_avar = maps
                .get(index)
                .map_or(normalized, |map| invert(map, normalized));
            AxisValue {
                tag: axis.tag().to_be_bytes(),
                value: denormalize(
                    before_avar,
                    axis.min_value(),
                    axis.default_value(),
                    axis.max_value(),
                ),
            }
        })
        .collect()
}

/// Each axis's `avar` segment map as (from, to) pairs; empty without an `avar`.
fn segment_maps(font: &FontRef<'_>) -> Vec<Vec<(f32, f32)>> {
    let Ok(avar) = font.avar() else {
        return Vec::new();
    };
    avar.axis_segment_maps()
        .iter()
        .map(|maps| {
            maps.map(|maps| {
                maps.axis_value_maps()
                    .iter()
                    .map(|pair| {
                        (
                            pair.from_coordinate().to_f32(),
                            pair.to_coordinate().to_f32(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
        })
        .collect()
}

/// The coordinate `map` sends to `to`: the segment map read backwards. A map with fewer than
/// two pairs is the identity (as `avar` defines it).
fn invert(map: &[(f32, f32)], to: f32) -> f32 {
    map.windows(2)
        .find(|pair| pair[0].1 <= to && to <= pair[1].1)
        .map_or(to, |pair| {
            let ((from_a, to_a), (from_b, to_b)) = (pair[0], pair[1]);
            if (to_b - to_a).abs() < f32::EPSILON {
                from_a
            } else {
                from_a + (to - to_a) * (from_b - from_a) / (to_b - to_a)
            }
        })
}

/// The user value a normalized `value` (-1..=1) stands for on an axis `min..=max` around
/// `default`.
fn denormalize(value: f32, min: f32, default: f32, max: f32) -> f32 {
    if value < 0.0 {
        default + value * (default - min)
    } else {
        default + value * (max - default)
    }
}

#[cfg(test)]
mod tests {
    use super::{denormalize, invert};

    #[test]
    fn denormalizing_follows_each_side_of_the_default() {
        const CASES: &[(f32, f32)] = &[(-1.0, 100.0), (-0.5, 250.0), (0.0, 400.0), (1.0, 900.0)];
        for &(normalized, want) in CASES {
            assert_eq!(
                denormalize(normalized, 100.0, 400.0, 900.0),
                want,
                "{normalized}"
            );
        }
    }

    #[test]
    fn avar_is_read_backwards() {
        let map = [(-1.0, -1.0), (0.0, 0.0), (0.5, 0.8), (1.0, 1.0)];
        assert_eq!(invert(&map, 0.8), 0.5);
        assert!((invert(&map, 0.4) - 0.25).abs() < 1e-6);
        assert!((invert(&map, 0.9) - 0.75).abs() < 1e-6);
        assert_eq!(invert(&[], 0.3), 0.3);
    }
}
