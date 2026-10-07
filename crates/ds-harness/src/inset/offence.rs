//! What the check reports: one [`Offence`] per box side whose content sits too close.

use crate::inset::scene::Paint;
use ds::prelude::Px;
use std::collections::BTreeMap;
use std::fmt;

/// Which edge of the box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    /// The left edge.
    Left,
    /// The right edge.
    Right,
}

/// A box whose content is closer to an edge than the design allows.
#[derive(Debug, Clone, PartialEq)]
pub struct Offence {
    /// The box, from the document root down.
    pub path: String,
    /// What sits too close: the element under the box, and a text run's words.
    pub content: String,
    /// The edge.
    pub side: Side,
    /// The distance from the box's edge to the nearest content, in logical pixels.
    pub gap: Px,
    /// The least that box is held to.
    pub min: Px,
    /// What paints the edge.
    pub paints: Vec<Paint>,
}

impl Offence {
    /// The last four elements of the box's path: enough to find it.
    pub fn tail(&self) -> String {
        let parts: Vec<&str> = self.path.split(" > ").collect();
        let from = parts.len().saturating_sub(4);
        let lead = if from > 0 { "... > " } else { "" };
        format!("{lead}{}", parts[from..].join(" > "))
    }
}

impl fmt::Display for Offence {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            out,
            "{:?} gap {:.1}px (min {:.1}px) {:?}: {} :: {}",
            self.side,
            self.gap.0,
            self.min.0,
            self.paints,
            self.tail(),
            self.content
        )
    }
}

/// An allowance that excused a gap, so a test can see which entries still do something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Excused {
    /// The allowance that excused it.
    pub when: &'static [&'static str],
}

/// Everything a check found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Findings {
    /// Gaps below the minimum.
    pub offences: Vec<Offence>,
    /// One entry for every gap an [`Allow`] kept from being an offence.
    pub excused: Vec<Excused>,
}

impl Findings {
    /// The findings of `self` and `other`.
    pub fn merge(&mut self, other: Findings) {
        self.offences.extend(other.offences);
        self.excused.extend(other.excused);
    }

    /// The offences as a readable block: grouped by box, tightest gap first.
    pub fn text(&self) -> String {
        let mut by_box: BTreeMap<&str, Vec<&Offence>> = BTreeMap::new();
        for offence in &self.offences {
            by_box.entry(&offence.path).or_default().push(offence);
        }
        let mut text = format!(
            "{} content inset offence(s): content closer to a painted box edge than its minimum\n",
            self.offences.len()
        );
        for (path, group) in by_box {
            text.push_str(&format!("  {path}\n"));
            for offence in group {
                text.push_str(&format!(
                    "    {:?} {:.1}px < {:.1}px: {}\n",
                    offence.side, offence.gap.0, offence.min.0, offence.content
                ));
            }
        }
        text
    }
}
