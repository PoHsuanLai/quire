//! What an accent measures against the gates in [`super::floors`]: the numbers the sweep test
//! asserts and the proposal quotes.

use super::grounds::{card_grounds, card_ink, contrast, least, over};
use super::roles::AccentRoles;
use crate::appearance::Scheme;

/// The least ratio each role reaches over the card's grounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Legibility {
    /// The ink on the fill (gate 4.5).
    pub ink_on_fill: f64,
    /// The text accent on the worst ground (gate 4.5).
    pub text_on_card: f64,
    /// The card's ink on the wash over the worst ground (gate 4.5).
    pub ink_on_wash: f64,
    /// The wash against the ground it lies on (gate 1.15).
    pub wash_shows: f64,
    /// The focus ring against its ground (gate 3.0).
    pub ring: f64,
    /// The fill against the ground, reported, not gated: WCAG 1.4.11 asks 3:1 of a control
    /// whose colour alone shows its state; a toggle's knob and a disc's shape also show it.
    pub fill_on_card: f64,
}

/// `roles` measured over the card's grounds in `scheme`.
pub fn legibility(roles: &AccentRoles, scheme: Scheme) -> Legibility {
    let grounds = card_grounds(scheme);
    let ink = card_ink(scheme);
    let ink_on_wash = grounds
        .iter()
        .map(|&ground| contrast(ink, over(roles.fill, roles.wash, ground)))
        .fold(f64::INFINITY, f64::min);
    Legibility {
        ink_on_fill: contrast(roles.ink, roles.fill),
        text_on_card: least(|_| roles.text, &grounds),
        ink_on_wash,
        wash_shows: least(|ground| over(roles.fill, roles.wash, ground), &grounds),
        ring: least(|ground| over(roles.text, roles.ring, ground), &grounds),
        fill_on_card: least(|_| roles.fill, &grounds),
    }
}
