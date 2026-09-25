//! The widget parts as markup (sill FINDINGS Q182, Q183; design/04-COMPONENTS.md "Widgets"):
//! the frame in each size on the desktop and as a tile, the clock face in both looks and
//! phases, and the level ring at three levels and charging. Each golden is
//! `tests/snapshots/widgets/<name>.html`; each lints clean and every `ds-` class in it is
//! styled by the stylesheet.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test widgets_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    Appearance, ClockFace, ClockLook, ClockTime, DayPhase, Ds, Fraction, Glyph, Icon, IconSize,
    Inject, LevelRing, Material, RingMark, RootChrome, Seconds, Theme, WidgetFrame, WidgetHost,
    WidgetMetrics, WidgetSize, WidgetTitle,
};

#[derive(Props, Clone)]
struct HostProps {
    make: fn() -> Element,
}

impl PartialEq for HostProps {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

fn host(props: HostProps) -> Element {
    (props.make)()
}

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// `body` on the desktop layer: a transparent Widget root with the grid unit written, as sill's
/// desktop surface draws it.
fn desktop(theme: Theme, body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Widget, chrome: Some(RootChrome::Transparent), stylesheet: Inject::Host,
            div { style: WidgetMetrics::default().style_attr(), {body} }
        }
    }
}

/// `body` in the notification center: a Popover root.
fn center(body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, stylesheet: Inject::Host,
            div { style: WidgetMetrics::default().style_attr(), {body} }
        }
    }
}

fn batteries() -> Option<WidgetTitle> {
    Some(WidgetTitle::new(Icon::BatteryFull, "Batteries"))
}

const TEN_TO_TEN: ClockTime = ClockTime {
    hour: 9,
    minute: 50,
    second: Seconds::Hidden,
};

const LATE: ClockTime = ClockTime {
    hour: 23,
    minute: 5,
    second: Seconds::Shown(42),
};

type Case = (&'static str, fn() -> Element);

const CASES: &[Case] = &[
    ("frame-small-desktop", || {
        desktop(
            Theme::Light,
            rsx! { WidgetFrame { size: WidgetSize::Small, "84%" } },
        )
    }),
    ("frame-medium-desktop-titled", || {
        desktop(
            Theme::Light,
            rsx! { WidgetFrame { size: WidgetSize::Medium, title: batteries(), id: "widget-battery", "84%" } },
        )
    }),
    ("frame-large-desktop-dark", || {
        desktop(
            Theme::Dark,
            rsx! { WidgetFrame { size: WidgetSize::Large, "Up next" } },
        )
    }),
    ("frame-small-tile", || {
        center(rsx! { WidgetFrame { size: WidgetSize::Small, host: WidgetHost::Tile, "84%" } })
    }),
    ("frame-medium-tile-titled", || {
        center(
            rsx! { WidgetFrame { size: WidgetSize::Medium, host: WidgetHost::Tile, title: batteries(), "84%" } },
        )
    }),
    ("frame-large-tile", || {
        center(rsx! { WidgetFrame { size: WidgetSize::Large, host: WidgetHost::Tile, "Up next" } })
    }),
    ("clock-analog-day", || {
        desktop(
            Theme::Light,
            rsx! { ClockFace { time: TEN_TO_TEN, label: "Taipei" } },
        )
    }),
    ("clock-analog-night-seconds", || {
        desktop(
            Theme::Light,
            rsx! { ClockFace { time: LATE, phase: DayPhase::Night, label: "London" } },
        )
    }),
    ("clock-digital-day", || {
        desktop(
            Theme::Dark,
            rsx! { ClockFace { time: TEN_TO_TEN, look: ClockLook::Digital, label: "Taipei" } },
        )
    }),
    ("clock-digital-night-seconds", || {
        desktop(
            Theme::Light,
            rsx! { ClockFace { time: LATE, phase: DayPhase::Night, look: ClockLook::Digital, label: "London" } },
        )
    }),
    ("ring-8", || {
        desktop(
            Theme::Light,
            rsx! { LevelRing { level: Fraction(80), label: "Mouse" } },
        )
    }),
    ("ring-45", || {
        desktop(
            Theme::Light,
            rsx! { LevelRing { level: Fraction(450), label: "Headphones", Glyph { icon: Icon::Headphones, size: IconSize::Base } } },
        )
    }),
    ("ring-100", || {
        desktop(
            Theme::Dark,
            rsx! { LevelRing { level: Fraction(1000), label: "Keyboard" } },
        )
    }),
    ("ring-charging-15", || {
        desktop(
            Theme::Light,
            rsx! { LevelRing { level: Fraction(150), mark: RingMark::Charging, label: "This computer" } },
        )
    }),
];

fn html(name: &str) -> String {
    let (_, make) = CASES
        .iter()
        .find(|(case, _)| *case == name)
        .unwrap_or_else(|| panic!("no case {name}"));
    render(*make)
}

#[test]
fn every_widget_part_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|(name, make)| {
            golden::check(&format!("widgets/{name}.html"), &render(*make)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_part_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, make) in CASES {
        let html = render(*make);
        for offence in markup(&html, sheet, &LintConfig::default()) {
            failures.push(format!("{name}: {:?} {}", offence.rule, offence.text));
        }
        for class in html
            .split("class=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .filter(|class| class.starts_with("ds-"))
        {
            let needle = format!(".{class}");
            let styled = sheet.match_indices(&needle).any(|(at, _)| {
                !sheet[at + needle.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            });
            if !styled {
                failures.push(format!("{name}: .{class} is not styled"));
            }
        }
    }
    failures.dedup();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The desktop card is in a Widget scope and the tile in none of its own; both say their size
/// and host, and the title row is drawn only when asked for.
#[test]
fn a_frame_says_its_size_and_host_and_only_the_desktop_has_a_material() {
    let desktop = html("frame-medium-desktop-titled");
    assert!(desktop.contains("data-size=\"medium\""), "{desktop}");
    assert!(desktop.contains("data-host=\"desktop\""), "{desktop}");
    assert!(desktop.contains("id=\"widget-battery\""), "{desktop}");
    assert!(desktop.contains("ds-widget-title"), "{desktop}");
    assert_eq!(
        desktop.matches("data-material=\"widget\"").count(),
        2,
        "root and card scope"
    );
    let tile = html("frame-medium-tile-titled");
    assert!(tile.contains("data-host=\"tile\""), "{tile}");
    assert!(!tile.contains("data-material=\"widget\""), "{tile}");
    assert_eq!(
        tile.matches("data-material=").count(),
        1,
        "the popover root only"
    );
    assert!(!html("frame-small-tile").contains("ds-widget-title"));
}

/// Day forces the dial's scope to the light scheme and night to the dark, whatever the root's;
/// the second hand is drawn only when the seconds are shown.
#[test]
fn the_dial_takes_its_phases_scheme_and_its_second_hand_only_when_shown() {
    let day = html("clock-analog-day");
    assert!(day.contains("data-theme=\"light\""), "{day}");
    assert!(!day.contains("ds-clock-second"), "{day}");
    assert!(
        day.contains("rotate(295 50 50)"),
        "the hour hand at ten to ten: {day}"
    );
    let night = html("clock-analog-night-seconds");
    assert!(night.contains("data-theme=\"dark\""), "{night}");
    assert!(night.contains("ds-clock-second"), "{night}");
    assert!(
        night.contains("rotate(252 50 50)"),
        "the second hand at 42 s: {night}"
    );
    assert!(html("clock-digital-night-seconds").contains(">23:05:42<"));
    assert!(html("clock-digital-day").contains(">09:50<"));
}

/// The ring's arc is the level in percent, its tone follows the level unless charging, and it
/// reports its value as a progress bar.
#[test]
fn the_ring_draws_its_level_and_tone() {
    let cases = [
        ("ring-8", "8 100", "critical", "8"),
        ("ring-45", "45 100", "ok", "45"),
        ("ring-100", "100 100", "ok", "100"),
        ("ring-charging-15", "15 100", "ok", "15"),
    ];
    for (name, dash, tone, now) in cases {
        let html = html(name);
        assert!(
            html.contains(&format!("stroke-dasharray=\"{dash}\"")),
            "{name}: {html}"
        );
        assert!(
            html.contains(&format!("data-tone=\"{tone}\"")),
            "{name}: {html}"
        );
        assert!(
            html.contains(&format!("aria-valuenow=\"{now}\"")),
            "{name}: {html}"
        );
        assert!(!html.contains("a-bump"), "{name}: nothing bumps on mount");
    }
    assert!(html("ring-charging-15").contains("ds-ring-bolt"));
    assert!(!html("ring-45").contains("ds-ring-bolt"));
}
