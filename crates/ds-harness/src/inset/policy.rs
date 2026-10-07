//! How much room content must keep from its box's edge, and where the design wants less.

use ds::prelude::Px;
use ds::style::tokens::spacing::SpacingToken;

/// The text inset of a menu item, a row, a card: `--s-8`, the least any content sits from the
/// painted edge of its box unless an [`Allow`] says why not (design/34-MODERN-LOOK.md,
/// 04-COMPONENTS.md sections 3 and 5).
pub fn text_inset() -> Px {
    Px(f32::from(SpacingToken::S8.tenths()) / 10.0)
}

/// A box class for which the design asks for a smaller inset than the default, with the reason.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Allow {
    /// What the box carries, all of it: its class (`ds-button`) and attributes
    /// (`[data-size=mini]`).
    pub when: &'static [&'static str],
    /// The least inset that class is held to.
    pub min: Px,
    /// Why the design wants this: the token or design section it follows.
    pub reason: &'static str,
}

/// The rule a check measures against.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// The least inset of any box not in `allow`.
    pub min: Px,
    /// Boxes held to a smaller inset, each with its reason.
    pub allow: Vec<Allow>,
    /// How much a gap may fall short of its minimum before it is an offence: layout snaps to a
    /// fraction of a pixel.
    pub slack: Px,
}

impl Policy {
    /// The default inset and no allowances: what an app starts from, adding its own
    /// [`Policy::allowing`].
    pub fn new() -> Self {
        Policy {
            min: text_inset(),
            allow: Vec::new(),
            slack: Px(0.5),
        }
    }

    /// quire's own policy: the default inset, with [`QUIRE_ALLOW`].
    pub fn quire() -> Self {
        Policy::new().allowing(QUIRE_ALLOW)
    }

    /// This policy with `entries` allowed.
    pub fn allowing(mut self, entries: &[Allow]) -> Self {
        self.allow.extend_from_slice(entries);
        self
    }

    /// The least inset a box with these classes is held to, and the allowance that lowered it.
    pub fn min_for(&self, tokens: &[String]) -> (Px, Option<&Allow>) {
        let lowest = self
            .allow
            .iter()
            .filter(|entry| {
                entry
                    .when
                    .iter()
                    .all(|need| tokens.iter().any(|token| token == need))
            })
            .min_by(|a, b| a.min.0.total_cmp(&b.min.0));
        match lowest {
            Some(entry) => (entry.min, Some(entry)),
            None => (self.min, None),
        }
    }
}

impl Default for Policy {
    fn default() -> Self {
        Policy::new()
    }
}

/// The quire components the design holds to a smaller inset than `--s-8`, each with its reason.
/// Every value is the component's own padding token plus any hairline, read from its stylesheet;
/// a regular-size control keeps 8 or more (`--ctl-pad-m` 10, `--tf-pad` 8, `--s-8`).
pub const QUIRE_ALLOW: &[Allow] = &[
    Allow {
        when: &["ds-button", "[data-size=mini]"],
        min: Px(6.0),
        reason: "a Mini control's label inset is `--ctl-pad-xs` 6 (ControlSize::Mini pad_x, design/29-SIZING.md section 3, on the 4 px grid, R7)",
    },
    Allow {
        when: &["ds-segmented", "[data-size=mini]"],
        min: Px(7.0),
        reason: "a Mini segmented control: the track's `--knob-inset` 1 plus the segment's `--sg-pad` = `--ctl-pad-xs` 6",
    },
    Allow {
        when: &["ds-segmented-segment"],
        min: Px(6.0),
        reason: "a segment's own label inset is its size's `--sg-pad`, 6 at Mini (the selected thumb is the box)",
    },
    Allow {
        when: &["ds-text-field-frame", "^[data-size=mini]"],
        min: Px(6.0),
        reason: "a Mini field's `--tf-pad` is `--s-6` (text_field.css)",
    },
    Allow {
        when: &["ds-text-field-frame", "^[data-size=small]"],
        min: Px(6.0),
        reason: "a Small field's `--tf-pad` is `--s-6` (text_field.css)",
    },
    Allow {
        when: &["ds-key-equivalent-key"],
        min: Px(6.0),
        reason: "a key cap is a capsule: `padding: --s-1 --s-5` plus its 1 px hairline (key_equivalent.css, 04-COMPONENTS.md section 14)",
    },
    Allow {
        when: &["ds-badge", "[data-size=mini]"],
        min: Px(3.0),
        reason: "a Mini count badge: `padding: 0 --s-3` in a 12 px capsule (badge.css, design/30 section 2.9)",
    },
    Allow {
        when: &["ds-badge"],
        min: Px(4.0),
        reason: "a count badge: `padding: 0 --s-4` in a capsule at least its height wide (badge.css, design/30 section 2.9)",
    },
    Allow {
        when: &["ds-chip"],
        min: Px(7.0),
        reason: "a chip's capsule padding is `--s-7` (04-COMPONENTS.md section 10: `padding:1.5px 7px`)",
    },
    Allow {
        when: &["ds-notification-count"],
        min: Px(5.0),
        reason: "the notification count capsule: `padding: 0 --s-5` (card.css)",
    },
    Allow {
        when: &["ds-scrubber-tooltip"],
        min: Px(6.0),
        reason: "the scrubber's time tooltip: `padding: --s-2 --s-6` (scrubber.css, 04-COMPONENTS.md section 22 tooltip)",
    },
    Allow {
        when: &["ds-stop"],
        min: Px(6.0),
        reason: "a Space gradient stop pill: `padding: --s-3 --s-5 --s-3 --s-4` plus its hairline, around a swatch and a remove glyph (space_editor.css)",
    },
    Allow {
        when: &["ds-send-pill"],
        min: Px(6.0),
        reason: "the send pill's trailing edge is `--s-6`: the consumer's PillAction button sits inside it (send_pill.css; 12 leading)",
    },
    Allow {
        when: &["ds-module-tile"],
        min: Px(7.0),
        reason: "a Control Center tile ends in an 18 px chevron icon button: `padding-right: --s-4` plus the hairline, the 14 px glyph centred in the button (module_tile.css)",
    },
    Allow {
        when: &["ds-menu-item"],
        min: Px(6.0),
        reason: "an item with a state column starts at `--s-2` and its 14 px check glyph is centred in the 22 px column (design/27 section 13.3: 22 px check column); an item without one is held to 8 (`data-state-column=absent`, item.css)",
    },
];
