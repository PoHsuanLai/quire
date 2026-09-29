//! The band's gates, swept: mailo's `every_pick_is_legible` for the card accent.

use super::floors;
use super::*;
use crate::appearance::{accent::Accent, theme::Scheme};
use crate::core::colour::{oklab::Oklab, srgb::Srgb};
use crate::tokens::hex::{Alpha, Hex};

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
        ("text on wash", measured.text_on_wash, floors::TEXT),
        (
            "material text on card",
            measured.material_text_on_card,
            floors::TEXT,
        ),
        ("text on material", measured.text_on_material, floors::TEXT),
        (
            "text on washed material",
            measured.text_on_washed_material,
            floors::TEXT,
        ),
        ("ink on wash", measured.ink_on_wash, floors::TEXT),
        ("wash shows", measured.wash_shows, floors::WASH_SHOWS),
        ("focus ring", measured.ring, floors::RING),
    ];
    gates
        .into_iter()
        .filter(|(_, got, floor)| got < floor)
        .map(|(gate, got, floor)| {
            format!(
                "{case}: {gate} {got:.2} < {floor} (fill {}, ink {}, text {}, material text {}, wash {}, ring {})",
                roles.fill.css(),
                roles.ink.css(),
                roles.text.css(),
                roles.text_material.css(),
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
                let apart =
                    Oklab::from(Srgb::from(first)).distance(Oklab::from(Srgb::from(second)));
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

/// The numbers design/03-COLOR.md section 20 quotes for Postmark's hue: fill, ink, the card's
/// text, the material's text, wash and ring, light then dark.
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
        light.text_material.css(),
        light.wash_colour().css(),
        light.ring_colour().css(),
        dark.fill.css(),
        dark.ink.css(),
        dark.text.css(),
        dark.text_material.css(),
        dark.wash_colour().css(),
        dark.ring_colour().css(),
    ];
    let want = [
        "#94c0fe",
        "#111b28",
        "#396198",
        "#295086",
        "rgba(148,192,254,.32)",
        "rgba(57,97,152,.75)",
        "#8ebaf7",
        "#111b28",
        "#8ebaf7",
        "#d5e6fe",
        "rgba(142,186,247,.2)",
        "rgba(142,186,247,.55)",
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

/// Every ground the material's text is measured on is kept but one: the dark Popover's wash over
/// a white backdrop, where the card's own ink reaches only 4.1:1 (a material shortfall, not the
/// accent's; design/03-COLOR.md section 20.6). A new ground the ink misses fails here first.
#[test]
fn the_only_ground_the_ink_misses_is_the_dark_popover_wash() {
    use crate::material::material::Material;
    use crate::material::recipe::tint;
    let all = 4 + 4 + 2 * TEXT_MATERIALS.len() * 2;
    for scheme in Scheme::ALL {
        for hue in (0..360).step_by(5) {
            for tenth in 0..=10 {
                let roles = roles(hue, Weight(tenth * 100), scheme);
                let kept = text_grounds(TextOn::Material, scheme, roles.fill, roles.wash);
                let missed = match scheme {
                    Scheme::Light => 0,
                    Scheme::Dark => 1,
                };
                assert_eq!(kept.len(), all - missed, "{scheme:?} h={hue} w=.{tenth}");
                assert_eq!(
                    text_grounds(TextOn::Card, scheme, roles.fill, roles.wash).len(),
                    8,
                    "{scheme:?} h={hue} w=.{tenth}"
                );
            }
        }
    }
    let roles = roles(257, Weight::FULL, Scheme::Dark);
    let (hex, alpha) = tint(Material::Popover, Scheme::Dark).expect("a tint");
    let ground = over(roles.fill, roles.wash, over(hex, alpha, BACKDROPS[1]));
    let kept = text_grounds(TextOn::Material, Scheme::Dark, roles.fill, roles.wash);
    assert!(
        kept.iter().all(|kept| kept.hex != ground),
        "{}",
        ground.css()
    );
}

/// Only the Popover, the Sheet and the Toast point `--accent-text` at the material's text.
#[test]
fn the_text_carrying_materials_take_the_material_text() {
    use crate::material::material::Material;
    for material in Material::ALL {
        let want = match material {
            Material::Popover | Material::Sheet | Material::Toast => TextOn::Material,
            Material::Window
            | Material::Bar
            | Material::Dock
            | Material::Osd
            | Material::Widget => TextOn::Card,
        };
        assert_eq!(text_on(material), want, "{material:?}");
    }
}
