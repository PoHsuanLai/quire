//! The candidate looks of the widget faces (design/23-WIDGETS.md section 4): how a battery's
//! level is drawn, how a clock's dial is built, and how a desktop widget's card is finished.
//! Each defaults to the look the widgets shipped with, and writes its attribute only when it is
//! not the default, so a caller that names none draws exactly what it drew before.

/// How a `LevelRing` draws its level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatteryLook {
    /// A stroked ring on a faint track (the first widgets).
    #[default]
    Ring,
    /// The ring lies in a groove pressed into the plate, round a raised boss that holds the
    /// glyph; the level is a glossy liquid in the groove.
    Well,
    /// A horizontal cell: a recessed capsule bed holding a glossy liquid as wide as the level,
    /// with a terminal nub.
    Cell,
}

impl BatteryLook {
    /// Every look, the default first.
    pub const ALL: [BatteryLook; 3] = [BatteryLook::Ring, BatteryLook::Well, BatteryLook::Cell];

    /// The look's word.
    pub fn slug(self) -> &'static str {
        match self {
            BatteryLook::Ring => "ring",
            BatteryLook::Well => "well",
            BatteryLook::Cell => "cell",
        }
    }

    /// `data-look`: written only for a look other than the default.
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != BatteryLook::default()).then(|| self.slug())
    }
}

/// How an analog `ClockFace` builds its dial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DialLook {
    /// A flat paper disc with twelve ticks (the first widgets).
    #[default]
    Paper,
    /// A raised bezel round a recessed sky face, a minute track, tapered hands with shadows.
    Bezel,
    /// The whole dial is a well in the plate with a sky ground, dot indices, tapered hands
    /// with shadows.
    Sky,
}

impl DialLook {
    /// Every look, the default first.
    pub const ALL: [DialLook; 3] = [DialLook::Paper, DialLook::Bezel, DialLook::Sky];

    /// The look's word.
    pub fn slug(self) -> &'static str {
        match self {
            DialLook::Paper => "paper",
            DialLook::Bezel => "bezel",
            DialLook::Sky => "sky",
        }
    }

    /// `data-dial`: written only for a look other than the default.
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != DialLook::default()).then(|| self.slug())
    }
}

/// How a desktop `WidgetFrame`'s card is finished.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FrameFinish {
    /// The material's plate as it is.
    #[default]
    Plain,
    /// The plate with a bevel: a brighter top highlight, a sheen down its top, a shaded foot.
    Lit,
}

impl FrameFinish {
    /// Every finish, the default first.
    pub const ALL: [FrameFinish; 2] = [FrameFinish::Plain, FrameFinish::Lit];

    /// The finish's word.
    pub fn slug(self) -> &'static str {
        match self {
            FrameFinish::Plain => "plain",
            FrameFinish::Lit => "lit",
        }
    }

    /// `data-finish`: written only for a finish other than the default.
    pub(crate) fn attr(self) -> Option<&'static str> {
        (self != FrameFinish::default()).then(|| self.slug())
    }
}

#[cfg(test)]
mod tests {
    use super::{BatteryLook, DialLook, FrameFinish};

    #[test]
    fn only_a_look_other_than_the_default_writes_its_attribute() {
        assert_eq!(BatteryLook::Ring.attr(), None);
        assert_eq!(BatteryLook::Cell.attr(), Some("cell"));
        assert_eq!(DialLook::Paper.attr(), None);
        assert_eq!(DialLook::Sky.attr(), Some("sky"));
        assert_eq!(FrameFinish::Plain.attr(), None);
        assert_eq!(FrameFinish::Lit.attr(), Some("lit"));
    }
}
