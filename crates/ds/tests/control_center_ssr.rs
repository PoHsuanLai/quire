//! The control center's parts as markup (sill FINDINGS Q78-Q81): every control glyph renders and
//! lints clean; the goldens are `tests/snapshots/control_center/<name>.html`, each linted and
//! every `ds-` class in it styled by the stylesheet.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test control_center_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;
#[path = "control_center/panes.rs"]
mod panes;
#[path = "control_center/rows.rs"]
mod rows;
#[path = "control_center/tiles.rs"]
mod tiles;

use dioxus::prelude::*;
use ds::icon::render::GlyphProps;
use ds::lint::{LintConfig, Profile, markup};
use ds::{Glyph, Icon, IconSize};

/// `icon` at the bar's 22 px, rendered alone.
fn glyph(icon: Icon) -> String {
    let mut dom = VirtualDom::new_with_props(
        Glyph,
        GlyphProps {
            icon,
            size: IconSize::Bar,
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_control_glyph_renders_and_lints_clean() {
    let strict = LintConfig {
        profile: Profile::Strict,
        ..LintConfig::default()
    };
    let mut failures = Vec::new();
    for icon in Icon::CONTROL.iter().copied() {
        let html = glyph(icon);
        let children = html.matches("<path").count()
            + html.matches("<rect").count()
            + html.matches("<circle").count();
        if children != icon.shapes().len() || !html.contains("class=\"ds-ic\"") {
            failures.push(format!("{icon:?} drew {children} children: {html}"));
        }
        for offence in markup(&html, ds::stylesheet(), &strict) {
            failures.push(format!("{icon:?}: {:?} {}", offence.rule, offence.text));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A rendered specimen: its golden name and markup.
type Rendered = (String, String);

fn tile_markup(case: tiles::TileCase) -> String {
    let mut dom = VirtualDom::new_with_props(tiles::tile, case);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Every specimen, rendered.
fn specimens() -> Vec<Rendered> {
    let tiles = tiles::grid()
        .into_iter()
        .chain(tiles::chevrons())
        .map(|(name, case)| (name, tile_markup(case)));
    let rows = rows::CASES
        .into_iter()
        .map(|(case, name)| (name.to_owned(), row_markup(case)));
    let panes = panes::CASES
        .into_iter()
        .map(|(shown, name)| (name.to_owned(), pane_markup(shown)));
    tiles.chain(rows).chain(panes).collect()
}

fn pane_markup(shown: ds::Pane) -> String {
    let mut dom = VirtualDom::new_with_props(panes::panes, panes::PaneProps { shown });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn row_markup(case: rows::RowCase) -> String {
    let mut dom = VirtualDom::new_with_props(rows::row, rows::RowProps { case });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = specimens()
        .iter()
        .filter_map(|(name, html)| {
            golden::check(&format!("control_center/{name}.html"), html).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, html) in specimens() {
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

/// A tile says what it is in its attributes: the state as `data-state` and `aria-pressed`
/// (mixed while busy), the span, and the chevron's own name and `aria-expanded`.
#[test]
fn a_tile_writes_its_state_span_and_chevron() {
    for (name, case) in tiles::grid() {
        let html = tile_markup(case.clone());
        let (state, pressed) = match case.state {
            ds::ModuleState::Off => ("off", "false"),
            ds::ModuleState::On => ("on", "true"),
            ds::ModuleState::Busy => ("busy", "mixed"),
        };
        assert!(html.contains(&format!("data-state=\"{state}\"")), "{name}");
        assert!(
            html.contains(&format!("aria-pressed=\"{pressed}\"")),
            "{name}"
        );
        assert_eq!(
            html.contains("ds-spinner"),
            state == "busy",
            "{name}: breathe only while busy"
        );
        assert!(html.contains("aria-label=\"Wi-Fi details\""), "{name}");
        assert!(html.contains("aria-expanded=\"false\""), "{name}");
    }
}

/// A row writes its trailing kind, a check row whether it is the chosen one, and a toggle row
/// names its switch by the row's title.
#[test]
fn a_row_writes_its_trailing_mark() {
    let cases = [
        (
            rows::RowCase::CheckOn,
            "data-trailing=\"check\"",
            Some("aria-pressed=\"true\""),
        ),
        (
            rows::RowCase::CheckOff,
            "data-trailing=\"check\"",
            Some("aria-pressed=\"false\""),
        ),
        (
            rows::RowCase::Toggle,
            "data-trailing=\"toggle\"",
            Some("aria-label=\"Headphones\""),
        ),
        (rows::RowCase::Chevron, "data-trailing=\"chevron\"", None),
        (rows::RowCase::Value, "84%</span>", None),
        (rows::RowCase::Disabled, "aria-disabled=\"true\"", None),
    ];
    for (case, want, also) in cases {
        let html = row_markup(case);
        assert!(html.contains(want), "{case:?}: {html}");
        if let Some(also) = also {
            assert!(html.contains(also), "{case:?}: {html}");
        }
    }
    assert!(!row_markup(rows::RowCase::None).contains("data-trailing"));
}

/// A switcher mounted on a pane draws that pane alone, at rest, and moves nothing.
#[test]
fn a_switcher_at_rest_draws_one_pane() {
    for (shown, name) in panes::CASES {
        let html = pane_markup(shown);
        assert_eq!(html.matches("class=\"ds-pane\"").count(), 1, "{name}");
        assert!(
            html.contains(&format!("data-pane=\"{}\"", shown.slug())),
            "{name}"
        );
        assert!(html.contains("data-presence=\"present\""), "{name}");
        assert!(!html.contains("data-moving"), "{name}");
    }
}
