//! The band's gates, swept: mailo's `every_pick_is_legible` for the card accent.

use super::floors;
use super::*;
use crate::appearance::{Accent, Scheme};
use crate::space::ratio;
use crate::tokens::{Alpha, Hex, quad};

/// Every candidate, every 5 degrees of hue, weights 0 to 1 in tenths, both schemes: each gate
/// holds, and a failure names the exact colour.
#[test]
fn every_hue_in_every_band_is_legible() {
    let mut failures = Vec::new();
    let mut cases = 0_u32;
    for candidate in Candidate::ALL {
        for scheme in Scheme::ALL {
            for hue in (0..360).step_by(5) {
                for tenth in 0..=10 {
                    cases += 1;
                    let pick = AccentPick {
                        hue: Hue(hue),
                        weight: Weight(tenth * 100),
                    };
                    let roles = accent_roles(&candidate.band(), pick, scheme);
                    let case = format!("{candidate:?} {scheme:?} h={hue} w=.{tenth}");
                    failures.extend(gate_failures(&case, &roles, scheme));
                }
            }
        }
    }
    assert_eq!(cases, 3 * 2 * 72 * 11);
    assert!(
        failures.is_empty(),
        "{} of {cases}: {failures:#?}",
        failures.len()
    );
}

fn gate_failures(case: &str, roles: &AccentRoles, scheme: Scheme) -> Vec<String> {
    let measured = legibility(roles, scheme);
    let gates = [
        ("ink on fill", measured.ink_on_fill, floors::TEXT),
        ("text on card", measured.text_on_card, floors::TEXT),
        ("ink on wash", measured.ink_on_wash, floors::TEXT),
        ("wash shows", measured.wash_shows, floors::WASH_SHOWS),
        ("focus ring", measured.ring, floors::RING),
    ];
    gates
        .into_iter()
        .filter(|(_, got, floor)| got < floor)
        .map(|(gate, got, floor)| {
            format!(
                "{case}: {gate} {got:.2} < {floor} (fill {}, ink {}, text {}, wash {}, ring {})",
                roles.fill.css(),
                roles.ink.css(),
                roles.text.css(),
                roles.wash_colour().css(),
                roles.ring_colour().css()
            )
        })
        .collect()
}

/// The wash is translucent in every band: the material beneath shows through.
#[test]
fn the_wash_stays_translucent() {
    for candidate in Candidate::ALL {
        for scheme in Scheme::ALL {
            for hue in (0..360).step_by(5) {
                let pick = AccentPick {
                    hue: Hue(hue),
                    weight: Weight::FULL,
                };
                let roles = accent_roles(&candidate.band(), pick, scheme);
                assert!(
                    roles.wash <= floors::WASH_MOST,
                    "{candidate:?} {scheme:?} h={hue}: wash {:?}",
                    roles.wash
                );
            }
        }
    }
}

fn luminance_ratio(fore: Hex, back: Hex) -> f64 {
    ratio(&fore.css(), &back.css()).unwrap_or(1.0)
}

/// The complaint the bands answer: every candidate's light fill at Postmark's hue is lighter
/// than Postmark's navy (white stands off it less), and none is darker in dark.
#[test]
fn every_candidate_is_lighter_than_postmark() {
    let white = Hex([0xFF, 0xFF, 0xFF]);
    let postmark = quad(Accent::Postmark, Scheme::Light).accent;
    for candidate in Candidate::ALL {
        let pick = AccentPick {
            hue: hue_of(Accent::Postmark),
            weight: Weight::FULL,
        };
        let fill = accent_roles(&candidate.band(), pick, Scheme::Light).fill;
        assert!(
            luminance_ratio(white, fill) < luminance_ratio(white, postmark),
            "{candidate:?}: {} is not lighter than {}",
            fill.css(),
            postmark.css()
        );
    }
}

/// The ink a band names is the ink it gets, and white ink never lands on a pale fill.
#[test]
fn the_ink_follows_the_rule() {
    for candidate in Candidate::ALL {
        for scheme in Scheme::ALL {
            let band = candidate.band();
            let roles = accent_roles(
                &band,
                AccentPick {
                    hue: Hue(90),
                    weight: Weight::FULL,
                },
                scheme,
            );
            let white = roles.ink == Hex([0xFF, 0xFF, 0xFF]);
            match band.scheme(scheme).ink {
                InkRule::White => assert!(white, "{candidate:?} {scheme:?}"),
                InkRule::Deep => assert!(!white, "{candidate:?} {scheme:?}"),
            }
        }
    }
}

/// Compositing is per channel and exact at the ends.
#[test]
fn over_blends_per_channel() {
    let red = Hex([255, 0, 0]);
    let white = Hex([255, 255, 255]);
    assert_eq!(over(red, Alpha(1000), white), red);
    assert_eq!(over(red, Alpha(0), white), white);
    assert_eq!(over(red, Alpha(500), white), Hex([255, 128, 128]));
}
