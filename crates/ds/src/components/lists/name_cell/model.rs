//! How many characters of a name fit in the room a Name cell has, pure.

use ds_core::geometry::units::Px;

/// The icon's side: `IconSize::Base`.
const ICON: f32 = 16.0;
/// The gap between the icon and the name: `--s-6`.
const GAP: f32 = 6.0;
/// The body type's average advance in logical pixels. Blitz has no `text-overflow` and quire no
/// text measure for text it has not laid out, so a cut name is a budget of characters, not a
/// measurement: a name of narrow letters is cut a little early, one of wide letters a little late.
const ADVANCE: f32 = 7.0;

/// How many characters of a name fit in a cell `room` wide, the icon and the gap taken out; at
/// least one. A column's width less the cell's padding (and the outline's indent and triangle,
/// when it is one) is the room.
pub fn name_chars(room: Px) -> usize {
    match room.0.is_finite() {
        true => (((room.0 - ICON - GAP) / ADVANCE).floor().max(1.0)) as usize,
        false => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::name_chars;
    use ds_core::geometry::units::Px;

    #[test]
    fn the_budget_is_the_room_beside_the_icon_over_the_average_advance_and_never_under_one() {
        let cases = [
            (Px(300.0), 39),
            (Px(100.0), 11),
            (Px(29.0), 1),
            (Px(0.0), 1),
            (Px(-20.0), 1),
            (Px(f32::NAN), 1),
        ];
        for (room, want) in cases {
            assert_eq!(name_chars(room), want, "{room:?}");
        }
    }
}
