//! What an accent measures against the gates in [`super::floors`]: the numbers the sweep test
//! asserts and the proposal quotes.

use super::grounds::{card_grounds, card_ink, contrast, least, over};
use super::roles::AccentRoles;
use super::text_grounds::{GroundKind, TextOn, least_on, text_grounds};
use crate::appearance::theme::Scheme;

/// The least ratio each role reaches over the card's grounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Legibility {
    /// The ink on the fill (gate 4.5).
    pub ink_on_fill: f64,
    /// The card's text accent on the worst card ground (gate 4.5).
    pub text_on_card: f64,
    /// The card's text accent on the wash over the worst card ground: a menu's match highlight
    /// on the selected row (gate 4.5).
    pub text_on_wash: f64,
    /// The material's text accent on the worst card ground or wash over one (gate 4.5).
    pub material_text_on_card: f64,
    /// The material's text accent on the worst text-carrying material over a black or white
    /// backdrop: PolkitPrompt's "Details" on its Sheet (gate 4.5).
    pub text_on_material: f64,
    /// The material's text accent on the wash over such a material: the launcher's match
    /// highlight on its selected row (gate 4.5).
    pub text_on_washed_material: f64,
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
    let card_text = text_grounds(TextOn::Card, scheme, roles.fill, roles.wash);
    let material_text = text_grounds(TextOn::Material, scheme, roles.fill, roles.wash);
    let on = |kind| least_on(roles.text, kind, &card_text);
    let material_on = |kind| least_on(roles.text_material, kind, &material_text);
    let ink = card_ink(scheme);
    let ink_on_wash = grounds
        .iter()
        .map(|&ground| contrast(ink, over(roles.fill, roles.wash, ground)))
        .fold(f64::INFINITY, f64::min);
    Legibility {
        ink_on_fill: contrast(roles.ink, roles.fill),
        text_on_card: on(GroundKind::Card),
        text_on_wash: on(GroundKind::WashOnCard),
        material_text_on_card: material_on(GroundKind::Card)
            .min(material_on(GroundKind::WashOnCard)),
        text_on_material: material_on(GroundKind::Material),
        text_on_washed_material: material_on(GroundKind::WashOnMaterial),
        ink_on_wash,
        wash_shows: least(|ground| over(roles.fill, roles.wash, ground), &grounds),
        ring: least(|ground| over(roles.text, roles.ring, ground), &grounds),
        fill_on_card: least(|_| roles.fill, &grounds),
    }
}
