//! Every pair design/03-COLOR.md section 6 gates, over both schemes, the six accents and every
//! preset frame; and the materials' text over the two worst backdrops, black and white
//! (design/21-SPACES.md section 7, the plan's `material-legible-over-black-and-white`).

use ds::tokens::Alpha;
use ds::{
    Accent, CardAccent, ColourToken, Dot, FrameVars, Grain, Material, PRESETS, Scheme, SpaceLook,
    Theme, derive, quad, ratio, recipe,
};

/// The alpha the settings key ships with (`appearance.material_tint_alpha = 80`).
const DEFAULT_TINT_ALPHA: Alpha = Alpha(800);

fn measured(fore: &str, back: &str) -> f64 {
    ratio(fore, back).unwrap_or_else(|| panic!("{fore} on {back} is not a hex pair"))
}

fn colour(token: ColourToken, scheme: Scheme) -> String {
    token.value(scheme).css()
}

#[test]
fn every_accent_is_legible_in_both_schemes() {
    let mut failures = Vec::new();
    for scheme in Scheme::ALL {
        let surface = colour(ColourToken::Surface, scheme);
        let ink = colour(ColourToken::Ink, scheme);
        for accent in Accent::ALL {
            let quad = quad(accent, scheme);
            let pairs = [
                (
                    "accent on the card",
                    quad.accent.css(),
                    surface.clone(),
                    3.0,
                ),
                ("ink on the accent tint", ink.clone(), quad.soft.css(), 4.5),
                ("text on the accent", quad.ink.css(), quad.accent.css(), 4.5),
            ];
            for (label, fore, back, floor) in pairs {
                let got = measured(&fore, &back);
                if got < floor {
                    failures.push(format!(
                        "{scheme:?} {accent:?}: {label} {fore} on {back} is {got:.2}, needs {floor}"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Every `X` / `X-ink` pair: text on the accent, on `--ok`, on `--warn` and on `--danger`, in
/// both schemes (the accent here is Postmark; `every_accent_is_legible_in_both_schemes` covers
/// the other five). Dark `--danger-ink` was white on `#E0705A`, 3.17:1 (FINDINGS "mailo gaps").
#[test]
fn every_ink_on_its_colour_is_legible_in_both_schemes() {
    const PAIRS: [(ColourToken, ColourToken); 4] = [
        (ColourToken::AccentInk, ColourToken::Accent),
        (ColourToken::OkInk, ColourToken::Ok),
        (ColourToken::WarnInk, ColourToken::Warn),
        (ColourToken::DangerInk, ColourToken::Danger),
    ];
    let mut failures = Vec::new();
    for scheme in Scheme::ALL {
        for (ink, ground) in PAIRS {
            let (fore, back) = (colour(ink, scheme), colour(ground, scheme));
            let got = measured(&fore, &back);
            if got < 4.5 {
                failures.push(format!(
                    "{scheme:?}: {} {fore} on {} {back} is {got:.2}, needs 4.5",
                    ink.var().as_str(),
                    ground.var().as_str()
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// The four gates for one Space, measured on what the root actually paints: the frame ink from
/// [`FrameVars`], the stops the gradient is made of, and the card's accent (the Space's when it
/// lends one, Postmark otherwise).
fn gates(look: &SpaceLook, scheme: Scheme) -> Vec<String> {
    let vars = FrameVars::of(look, scheme);
    let stops = derive(&look.dots, scheme).stops;
    let surface = colour(ColourToken::Surface, scheme);
    let ink = colour(ColourToken::Ink, scheme);
    let (accent, soft) = match &vars.accent {
        Some([accent, soft, _]) => (accent.clone(), soft.clone()),
        None => (
            colour(ColourToken::Accent, scheme),
            colour(ColourToken::AccentSoft, scheme),
        ),
    };
    let worst = |fore: &str| {
        stops
            .iter()
            .map(|stop| (measured(fore, stop), stop.clone()))
            .fold((f64::INFINITY, String::new()), |a, b| {
                if b.0 < a.0 { b } else { a }
            })
    };
    let (ink_on_stop, ink_stop) = worst(&vars.ink);
    let (faint_on_stop, faint_stop) = worst(&vars.ink_faint);
    let checks = [
        ("sidebar text", vars.ink.clone(), ink_stop, ink_on_stop, 4.5),
        (
            "faint text",
            vars.ink_faint.clone(),
            faint_stop,
            faint_on_stop,
            3.0,
        ),
        (
            "accent on the card",
            accent.clone(),
            surface.clone(),
            measured(&accent, &surface),
            3.0,
        ),
        (
            "ink on the accent tint",
            ink.clone(),
            soft.clone(),
            measured(&ink, &soft),
            4.5,
        ),
    ];
    checks
        .into_iter()
        .filter(|check| check.3 < check.4)
        .map(|(label, fore, back, got, floor)| {
            format!("{label}: {fore} on {back} is {got:.2}, needs {floor}")
        })
        .collect()
}

#[test]
fn every_preset_frame_is_legible() {
    let mut failures = Vec::new();
    for (index, preset) in PRESETS.iter().enumerate() {
        for scheme in Scheme::ALL {
            for card_accent in [CardAccent::Postmark, CardAccent::SpaceHue] {
                let look = SpaceLook {
                    dots: preset.dots.to_vec(),
                    grain: Grain(40),
                    theme: Theme::System,
                    card_accent,
                };
                for failure in gates(&look, scheme) {
                    failures.push(format!(
                        "preset {index} {scheme:?} {card_accent:?}: {failure}"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn every_one_dot_space_on_the_sweep_is_legible() {
    // design/21-SPACES.md section 7 (proposed): hue 0-355 in 5 degree steps, five chromas.
    let mut failures = Vec::new();
    for step in 0..72u16 {
        for chroma in [0.0, 0.25, 0.5, 0.75, 1.0] {
            let hue = f32::from(step * 5);
            let look = SpaceLook {
                dots: vec![Dot { hue, chroma }],
                card_accent: CardAccent::SpaceHue,
                ..SpaceLook::default()
            };
            for scheme in Scheme::ALL {
                for failure in gates(&look, scheme) {
                    failures.push(format!("hue {hue} chroma {chroma} {scheme:?}: {failure}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// `rgba(r,g,b,a)` over an opaque backdrop, rounded to 8 bits as a screen would.
fn over(tint: &str, backdrop: [u8; 3]) -> String {
    let inner = tint
        .strip_prefix("rgba(")
        .and_then(|rest| rest.strip_suffix(')'))
        .unwrap_or_else(|| panic!("{tint} is not rgba()"));
    let parts: Vec<f64> = inner
        .split(',')
        .map(|part| part.parse().unwrap_or_else(|_| panic!("{tint}: {part}")))
        .collect();
    let [r, g, b, a] = parts[..] else {
        panic!("{tint}: four components")
    };
    let mix = |channel: f64, under: u8| (channel * a + f64::from(under) * (1.0 - a)).round();
    format!(
        "#{:02x}{:02x}{:02x}",
        mix(r, backdrop[0]) as u8,
        mix(g, backdrop[1]) as u8,
        mix(b, backdrop[2]) as u8
    )
}

const BLACK: [u8; 3] = [0, 0, 0];
const WHITE: [u8; 3] = [255, 255, 255];

#[test]
fn the_solid_tints_hold_text_over_black_and_white() {
    let mut failures = Vec::new();
    for material in Material::ALL.into_iter().filter(|m| *m != Material::Window) {
        for scheme in Scheme::ALL {
            let ink = colour(ColourToken::Ink, scheme);
            let solid = recipe(material, scheme, DEFAULT_TINT_ALPHA).tint_solid;
            for (name, backdrop) in [("black", BLACK), ("white", WHITE)] {
                let ground = over(&solid, backdrop);
                let got = measured(&ink, &ground);
                if got < 4.5 {
                    failures.push(format!(
                        "{material:?} {scheme:?} over {name}: {ink} on {ground} is {got:.2}"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Section 17.2's translucent tints at the key's default hold the card's ink at 4.5:1 over a
/// pure black and a pure white backdrop, the two worst a blur can show. The contrast gates
/// assume an opaque ground; how they hold over blur is design/03-COLOR.md open decision 11, so
/// this is the floor, measured on what `recipe` paints. Four alphas were raised to meet it in
/// wave 1 (the dark bar and dock .66, the widget .54 light and .65 dark).
#[test]
fn the_translucent_tints_hold_text_over_black_and_white() {
    let mut failures = Vec::new();
    for material in Material::ALL.into_iter().filter(|m| *m != Material::Window) {
        for scheme in Scheme::ALL {
            let ink = colour(ColourToken::Ink, scheme);
            let tint = recipe(material, scheme, DEFAULT_TINT_ALPHA).tint;
            for (name, backdrop) in [("black", BLACK), ("white", WHITE)] {
                let ground = over(&tint, backdrop);
                let got = measured(&ink, &ground);
                if got < 4.5 {
                    failures.push(format!(
                        "{material:?} {scheme:?} over {name}: {ink} on {ground} is {got:.2}"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// A fill as a screen shows it over `stop`: a hex fill is opaque, an `rgba()` one is mixed.
fn fill_over(fill: &str, stop: &str) -> String {
    if fill.starts_with('#') {
        return fill.to_owned();
    }
    let ds::Hex(under) = ds::Hex::parse(stop).unwrap_or_else(|| panic!("{stop} is not hex"));
    over(fill, under)
}

/// A status item and every control on the frame ground (bar gaps): the ink under the pointer,
/// `--f-ink`, on the hover fill `--f-pill-hover`, and on the pressed or open fill `--f-pill`,
/// each laid over every stop of the gradient, clears 4.5 in every preset and both schemes.
#[test]
fn frame_ink_holds_on_the_hover_and_pressed_fills() {
    let mut failures = Vec::new();
    for scheme in Scheme::ALL {
        for (index, preset) in PRESETS.iter().enumerate() {
            let look = SpaceLook {
                dots: preset.dots.to_vec(),
                ..SpaceLook::default()
            };
            let vars = FrameVars::of(&look, scheme);
            for stop in derive(&look.dots, scheme).stops {
                for (name, fill) in [
                    ("--f-pill-hover", &vars.pill_hover),
                    ("--f-pill", &vars.pill),
                ] {
                    let ground = fill_over(fill, &stop);
                    let got = measured(&vars.ink, &ground);
                    if got < 4.5 {
                        failures.push(format!(
                            "{scheme:?} preset {}: --f-ink {} on {name} over {stop} ({ground}) is {got:.2}",
                            index + 1,
                            vars.ink
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

/// The alpha of `material`'s tint over blur at the key's default, read off its recipe.
fn tint_alpha(material: Material, scheme: Scheme) -> f64 {
    let tint = recipe(material, scheme, DEFAULT_TINT_ALPHA).tint;
    tint.trim_end_matches(')')
        .rsplit(',')
        .next()
        .and_then(|alpha| alpha.parse().ok())
        .unwrap_or_else(|| panic!("{tint} has no alpha"))
}

/// The ink each tinted chrome material draws in, over its own gradient.
fn chrome_ink(material: Material, vars: &FrameVars, scheme: Scheme) -> String {
    match ds::Ground::of(material) {
        ds::Ground::Frame => vars.ink.clone(),
        ds::Ground::Paper => colour(ColourToken::Ink, scheme),
    }
}

/// The tinted chrome (bar gaps): the ink a material's ground draws in, on every stop of every
/// preset's gradient laid at the material's alpha over black and over white, in both schemes;
/// `solid` is the blur-off floor .94, the other the tint alpha over blur at the key's default.
fn tinted_chrome_failures(alpha_of: impl Fn(Material, Scheme) -> f64) -> Vec<String> {
    let mut failures = Vec::new();
    let tinted = [
        Material::Bar,
        Material::Dock,
        Material::Popover,
        Material::Osd,
        Material::Widget,
    ];
    for material in tinted {
        for scheme in Scheme::ALL {
            let alpha = alpha_of(material, scheme);
            for (index, preset) in PRESETS.iter().enumerate() {
                let look = SpaceLook {
                    dots: preset.dots.to_vec(),
                    ..SpaceLook::default()
                };
                let vars = FrameVars::of(&look, scheme);
                let ink = chrome_ink(material, &vars, scheme);
                for stop in derive(&look.dots, scheme).stops {
                    let ds::Hex([r, g, b]) = ds::Hex::parse(&stop).expect("hex stop");
                    let tint = format!("rgba({r},{g},{b},{alpha})");
                    for (name, backdrop) in [("black", BLACK), ("white", WHITE)] {
                        let ground = over(&tint, backdrop);
                        let got = measured(&ink, &ground);
                        if got < 4.5 {
                            failures.push(format!(
                                "{material:?} {scheme:?} preset {} stop {stop} at {alpha} over {name}: {ink} on {ground} is {got:.2}",
                                index + 1
                            ));
                        }
                    }
                }
            }
        }
    }
    failures
}

/// Without blur the tinted chrome is its gradient at the solid floor .94, and the ink its
/// ground draws in holds 4.5 on every stop of every preset over black and over white.
#[test]
fn the_tinted_chrome_holds_its_ink_without_blur() {
    let failures = tinted_chrome_failures(|_, _| 0.94);
    assert!(failures.is_empty(), "{failures:#?}");
}

/// Over blur, at the tint alphas design/03-COLOR.md section 17.2 gave the flat tints, six
/// material/scheme pairs fell short over a pure black or white backdrop (worst: the light
/// widget, 3.91 over black; FINDINGS "Bar gaps"). The smallest .02 raises that clear all of
/// them are settled (2026-09-24): the dark bar, both dock schemes, the dark OSD and both
/// widget schemes (`crates/ds/src/material/recipe.rs`). This is now the same gate as the
/// blur-off test above, just over the tint alpha instead of the solid floor: a future retune
/// that drops any material/scheme pair below 4.5 over blur fails it.
#[test]
fn the_tinted_chrome_holds_its_ink_over_blur() {
    let failures = tinted_chrome_failures(tint_alpha);
    assert!(failures.is_empty(), "{failures:#?}");
}
