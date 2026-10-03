//! What the traffic lights show (design/04-COMPONENTS.md "Window frame"), as a pure function of
//! two facts: whether the window is the one the person works in, and whether the pointer is over
//! the group. The window frame's stylesheet draws it; a compositor that draws the lights itself
//! asks here.

use ds_core::vocab::{Activity, Shown};

/// Whether the pointer is over any of the three lights.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GroupHover {
    /// Over the group: any light under the pointer colours all three.
    Over,
    /// Elsewhere.
    #[default]
    Away,
}

/// How a light's disc is filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LightFill {
    /// Its own colour: red, amber or green.
    Colour,
    /// The frame's pill grey with a hairline.
    Grey,
}

/// What one light shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LightState {
    /// The disc.
    pub fill: LightFill,
    /// Whether the mark (x, minus, arrows) is on the disc.
    pub marks: Shown,
}

/// The lights in the window `activity` describes, with the pointer as `hover` says: coloured in
/// the key window and whenever the pointer is over the group, grey otherwise; the marks show only
/// over the group.
pub fn light_state(activity: Activity, hover: GroupHover) -> LightState {
    let fill = match (activity, hover) {
        (Activity::Active, _) | (Activity::Inactive, GroupHover::Over) => LightFill::Colour,
        (Activity::Inactive, GroupHover::Away) => LightFill::Grey,
    };
    let marks = match hover {
        GroupHover::Over => Shown::Visible,
        GroupHover::Away => Shown::Hidden,
    };
    LightState { fill, marks }
}

#[cfg(test)]
mod tests {
    use super::{GroupHover, LightFill, LightState, light_state};
    use ds_core::vocab::{Activity, Shown};

    const CASES: &[(&str, Activity, GroupHover, LightFill, Shown)] = &[
        (
            "key window at rest",
            Activity::Active,
            GroupHover::Away,
            LightFill::Colour,
            Shown::Hidden,
        ),
        (
            "key window under the pointer",
            Activity::Active,
            GroupHover::Over,
            LightFill::Colour,
            Shown::Visible,
        ),
        (
            "inactive window at rest",
            Activity::Inactive,
            GroupHover::Away,
            LightFill::Grey,
            Shown::Hidden,
        ),
        (
            "inactive window under the pointer",
            Activity::Inactive,
            GroupHover::Over,
            LightFill::Colour,
            Shown::Visible,
        ),
    ];

    #[test]
    fn the_lights_follow_activity_and_group_hover() {
        for (name, activity, hover, fill, marks) in CASES {
            assert_eq!(
                light_state(*activity, *hover),
                LightState {
                    fill: *fill,
                    marks: *marks
                },
                "{name}"
            );
        }
    }
}
