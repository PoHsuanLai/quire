//! MonthGrid's density as markup (design/04-COMPONENTS.md section 39; sill Q190): forced
//! compact, and `Auto` inside a small desktop `WidgetFrame` (compact) and a medium one
//! (regular). Each golden is `tests/snapshots/month_grid/<name>.html`, linted and every `ds-`
//! class in it styled by the stylesheet. The month is August 2026 from Monday, six weeks.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test month_grid_density_ssr` rewrites them.

#[path = "support/golden.rs"]
mod golden;
#[path = "support/month_sample.rs"]
mod month_sample;

use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    Appearance, Ds, Inject, Material, MonthDensity, MonthGrid, RootChrome, Step, WeekNumbers,
    WidgetFrame, WidgetMetrics, WidgetSize,
};
use month_sample::{AUGUST, First, sample};

/// One specimen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    /// `MonthDensity::Compact` on a Popover, outside any frame, asking for week numbers.
    Compact,
    /// `Auto` in a small desktop frame.
    SmallFrame,
    /// `Auto` in a medium desktop frame.
    MediumFrame,
}

const CASES: [(Case, &str); 3] = [
    (Case::Compact, "compact"),
    (Case::SmallFrame, "auto-small-frame"),
    (Case::MediumFrame, "auto-medium-frame"),
];

#[derive(Props, Clone, PartialEq)]
struct CaseProps {
    case: Case,
}

/// The grid with the step buttons and week numbers asked for.
fn grid(density: MonthDensity) -> Element {
    rsx! {
        MonthGrid {
            data: sample(AUGUST, First::Monday),
            weeks: WeekNumbers::Show,
            density,
            onstep: EventHandler::new(|_: Step| {}),
        }
    }
}

/// `size`'s frame on the desktop layer, as sill's desktop surface draws it.
fn framed(size: WidgetSize) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Widget, chrome: Some(RootChrome::Transparent), stylesheet: Inject::Host,
            div { style: WidgetMetrics::default().style_attr(),
                WidgetFrame { size, {grid(MonthDensity::Auto)} }
            }
        }
    }
}

#[allow(non_snake_case)]
fn Specimen(props: CaseProps) -> Element {
    match props.case {
        Case::Compact => rsx! {
            Ds { appearance: Appearance::default(), material: Material::Popover, stylesheet: Inject::Host,
                {grid(MonthDensity::Compact)}
            }
        },
        Case::SmallFrame => framed(WidgetSize::Small),
        Case::MediumFrame => framed(WidgetSize::Medium),
    }
}

fn render(case: Case) -> String {
    let mut dom = VirtualDom::new_with_props(Specimen, CaseProps { case });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .filter_map(|&(case, name)| {
            golden::check(&format!("month_grid/{name}.html"), &render(case)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (case, name) in CASES {
        let html = render(case);
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

/// Compact wherever it resolves so: `data-density=compact`, no week column though the caller
/// asked for one, and the plain step buttons in place of the Tool buttons; the medium frame
/// keeps the regular grid, its week numbers and its Tool buttons.
#[test]
fn the_density_resolves_and_says_so() {
    for case in [Case::Compact, Case::SmallFrame] {
        let html = render(case);
        assert!(html.contains("data-density=\"compact\""), "{case:?}");
        assert!(html.contains("data-weeks=\"hide\""), "{case:?}");
        assert_eq!(
            html.matches("class=\"ds-month-week\"").count(),
            0,
            "{case:?}"
        );
        assert_eq!(
            html.matches("class=\"ds-month-step\"").count(),
            2,
            "{case:?}"
        );
        assert!(!html.contains("ds-icon-button"), "{case:?}");
        assert_eq!(
            html.matches("class=\"ds-month-day\"").count(),
            42,
            "{case:?}"
        );
    }
    let medium = render(Case::MediumFrame);
    assert!(medium.contains("data-density=\"regular\""));
    assert!(medium.contains("data-weeks=\"show\""));
    assert_eq!(medium.matches("class=\"ds-month-week\"").count(), 1 + 6);
    assert_eq!(medium.matches("data-variant=\"tool\"").count(), 2);
    assert!(!medium.contains("ds-month-step"));
}
