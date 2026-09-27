//! The band's gates, swept: mailo's `every_pick_is_legible` for the card accent.

use super::floors;
use super::*;
use crate::appearance::{Accent, Scheme};
use crate::tokens::{Alpha, Hex};

fn roles(hue: u16, weight: Weight, scheme: Scheme) -> AccentRoles {
    accent_roles(
        &BAND,
        AccentPick {
            hue: Hue(hue),
            weight,
        },
        scheme,
    )
}

/// Every 5 degrees of hue, weights 0 to 1 in tenths, both schemes: each gate holds, and a
/// failure names the exact colour.
#[test]
fn every_hue_in_the_band_is_legible() {
    let mut failures = Vec::new();
    let mut cases = 0_u32;
    for scheme in Scheme::ALL {
        for hue in (0..360).step_by(5) {
            for tenth in 0..=10 {
                cases += 1;
                let roles = roles(hue, Weight(tenth * 100), scheme);
                let case = format!("{scheme:?} h={hue} w=.{tenth}");
                failures.extend(gate_failures(&case, &roles, scheme));
            }
        }
    }
    assert_eq!(cases, 2 * 72 * 11);
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

/// The wash is translucent at every hue: the material beneath shows through.
#[test]
fn the_wash_stays_translucent() {
    for scheme in Scheme::ALL {
        for hue in (0..360).step_by(5) {
            let roles = roles(hue, Weight::FULL, scheme);
            assert!(
                roles.wash <= floors::WASH_MOST,
                "{scheme:?} h={hue}: wash {:?}",
                roles.wash
            );
        }
    }
}

/// Band B puts a deep ink of the hue on every fill, never white.
#[test]
fn the_ink_is_deep() {
    for scheme in Scheme::ALL {
        for hue in (0..360).step_by(5) {
            let roles = roles(hue, Weight::FULL, scheme);
            assert_ne!(roles.ink, Hex([0xFF, 0xFF, 0xFF]), "{scheme:?} h={hue}");
            assert_eq!(BAND.scheme(scheme).ink, InkRule::Deep);
        }
    }
}

/// The six swatches in the picker are six colours: every pair of built-in fills stands at least
/// 0.06 apart in OKLab in both schemes (the narrowest gap, Violet to Postmark, is 0.064).
#[test]
fn every_built_in_swatch_is_distinct() {
    for scheme in Scheme::ALL {
        let fills: Vec<(Accent, Hex)> = Accent::ALL
            .into_iter()
            .map(|accent| {
                let pick = AccentPick {
                    hue: hue_of(accent),
                    weight: Weight::FULL,
                };
                (accent, accent_roles(&BAND, pick, scheme).fill)
            })
            .collect();
        for (index, &(one, first)) in fills.iter().enumerate() {
            for &(other, second) in &fills[index + 1..] {
                let apart = distance(first, second);
                assert!(
                    apart >= 0.06,
                    "{scheme:?}: {one:?} {} and {other:?} {} are {apart:.3} apart",
                    first.css(),
                    second.css()
                );
            }
        }
    }
}

/// The numbers design/03-COLOR.md section 20 quotes for Postmark's hue.
#[test]
fn postmark_is_the_settled_airy_blue() {
    let pick = AccentPick {
        hue: hue_of(Accent::Postmark),
        weight: Weight::FULL,
    };
    let light = accent_roles(&BAND, pick, Scheme::Light);
    let dark = accent_roles(&BAND, pick, Scheme::Dark);
    let got = [
        light.fill.css(),
        light.ink.css(),
        light.text.css(),
        light.wash_colour().css(),
        light.ring_colour().css(),
        dark.fill.css(),
        dark.ink.css(),
        dark.text.css(),
        dark.wash_colour().css(),
        dark.ring_colour().css(),
    ];
    let want = [
        "#94c0fe",
        "#111b28",
        "#426aa2",
        "rgba(148,192,254,.32)",
        "rgba(66,106,162,.8)",
        "#8ebaf7",
        "#111b28",
        "#88b3f0",
        "rgba(142,186,247,.2)",
        "rgba(136,179,240,.6)",
    ];
    assert_eq!(got, want.map(str::to_owned));
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
