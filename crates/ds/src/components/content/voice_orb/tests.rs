use super::model::{Contrast, OrbColour, OrbColours, OrbMask, OrbMetrics, Turn};
use super::step::turn_after;
use super::view::style_attr;
use ds_core::geometry::units::Px;
use ds_core::vocab::Percent;
use ds_style::tokens::hex::Hex;
use std::time::Duration;

/// One size and every value it must give, worked by hand from the reference's formulas.
struct MetricsCase {
    name: &'static str,
    size: f32,
    blur: f32,
    contrast: f32,
    dot: f32,
    shadow: f32,
    mask: OrbMask,
}

const CASES: &[MetricsCase] = &[
    MetricsCase {
        name: "20 px: every floor, no mask, the lowest contrast",
        size: 20.0,
        blur: 1.0,
        contrast: 1.1,
        dot: 0.08,
        shadow: 0.5,
        mask: OrbMask::Off,
    },
    MetricsCase {
        name: "29.9 px: still under 30",
        size: 29.9,
        blur: 1.0,
        contrast: 1.1,
        dot: 0.1196,
        shadow: 0.5,
        mask: OrbMask::Off,
    },
    MetricsCase {
        name: "30 px: the 5 % mask and the raised contrast begin",
        size: 30.0,
        blur: 1.0,
        contrast: 1.44,
        dot: 0.12,
        shadow: 0.5,
        mask: OrbMask::Radius(Percent(5)),
    },
    MetricsCase {
        name: "49.9 px: the last of the fine values",
        size: 49.9,
        blur: 1.0,
        contrast: 1.44,
        dot: 0.1996,
        shadow: 0.5,
        mask: OrbMask::Radius(Percent(5)),
    },
    MetricsCase {
        name: "50 px: the large values begin, 15 % mask",
        size: 50.0,
        blur: 4.0,
        contrast: 1.5,
        dot: 0.4,
        shadow: 2.0,
        mask: OrbMask::Radius(Percent(15)),
    },
    MetricsCase {
        name: "99.9 px: the last of the 15 % mask",
        size: 99.9,
        blur: 4.0,
        contrast: 1.5,
        dot: 0.7992,
        shadow: 2.0,
        mask: OrbMask::Radius(Percent(15)),
    },
    MetricsCase {
        name: "100 px: the 25 % mask",
        size: 100.0,
        blur: 4.0,
        contrast: 1.5,
        dot: 0.8,
        shadow: 2.0,
        mask: OrbMask::Radius(Percent(25)),
    },
    MetricsCase {
        name: "192 px, the default: the scaled values pass their floors",
        size: 192.0,
        blur: 4.0,
        contrast: 1.536,
        dot: 1.536,
        shadow: 2.0,
        mask: OrbMask::Radius(Percent(25)),
    },
    MetricsCase {
        name: "400 px: every value scales",
        size: 400.0,
        blur: 6.0,
        contrast: 3.2,
        dot: 3.2,
        shadow: 3.2,
        mask: OrbMask::Radius(Percent(25)),
    },
];

fn close(got: f32, want: f32) -> bool {
    (got - want).abs() < 1e-3
}

#[test]
fn every_size_threshold_gives_its_metrics() {
    for case in CASES {
        let got = OrbMetrics::of(Px(case.size));
        let ok = close(got.blur.0, case.blur)
            && close(got.contrast.0, case.contrast)
            && close(got.dot.0, case.dot)
            && close(got.shadow.0, case.shadow)
            && got.mask == case.mask;
        assert!(ok, "{}: got {got:?}", case.name);
    }
}

/// `(name, from, elapsed, period, expected)`.
const TURNS: &[(&str, u32, u64, u64, u32)] = &[
    ("no time has passed", 0, 0, 20_000, 0),
    (
        "a quarter of the period is a quarter turn",
        0,
        5_000,
        20_000,
        250_000,
    ),
    ("a whole period is back at the start", 0, 20_000, 20_000, 0),
    (
        "a period and a half is half a turn",
        0,
        30_000,
        20_000,
        500_000,
    ),
    (
        "it carries on from where it stood",
        750_000,
        5_000,
        20_000,
        0,
    ),
    ("it carries on past a wrap", 900_000, 4_000, 20_000, 100_000),
    ("a period of nothing holds it", 400_000, 5_000, 0, 400_000),
];

#[test]
fn the_turn_advances_a_share_of_a_turn_by_the_share_of_the_period() {
    for (name, from, elapsed, period, want) in TURNS {
        let got = turn_after(
            Turn(*from),
            Duration::from_millis(*elapsed),
            Duration::from_millis(*period),
        );
        assert_eq!(got, Turn(*want), "{name}");
    }
}

#[test]
fn a_turn_reads_as_degrees() {
    assert!(close(Turn(0).degrees(), 0.0));
    assert!(close(Turn(250_000).degrees(), 90.0));
    assert!(close(Turn(500_000).degrees(), 180.0));
}

#[test]
fn the_style_writes_the_size_the_metrics_and_only_the_colours_the_caller_brought() {
    let metrics = OrbMetrics::of(Px(96.0));
    let plain = style_attr(Px(96.0), &metrics, &OrbColours::default(), Turn(250_000));
    assert_eq!(
        plain,
        "--orb-size:96px;--orb-blur:4px;--orb-contrast:1.5;--orb-dot:0.768px;--orb-shadow:2px;--orb-turn:90deg;--orb-mask:15%;"
    );
    let custom = OrbColours {
        c2: OrbColour::Custom(Hex([255, 0, 0])),
        ..OrbColours::default()
    };
    let styled = style_attr(Px(96.0), &metrics, &custom, Turn(0));
    assert!(styled.ends_with("--orb-c2:#ff0000;"), "{styled}");
    assert!(!styled.contains("--orb-bg"), "{styled}");
}

#[test]
fn a_small_orb_writes_no_mask() {
    let metrics = OrbMetrics::of(Px(24.0));
    let style = style_attr(Px(24.0), &metrics, &OrbColours::default(), Turn(0));
    assert!(!style.contains("--orb-mask"), "{style}");
    assert_eq!(metrics.contrast, Contrast(1.1));
}
