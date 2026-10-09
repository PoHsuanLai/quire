//! The component sheets use tokens only: no colour, duration, font or unprefixed attribute
//! selector written by hand. One scan over the sheets the component suites used to scan each for
//! its own (controls, lists, forms, overlays, the Space editor); a new sheet is a name in the
//! table, not a new test.

use crate::css_scan::{STYLES, token_violations};

/// The sheets (by `ds::component_sheets()` name) scanned besides the controls' own.
const SHEETS: &[&str] = &[
    "form",
    "form_section",
    "icon_tile",
    "pane_stack",
    "thread_row",
    "row",
    "list",
    "disclosure",
    "hover_strip",
    "command_pill",
    "pin_tile",
    "provider_mark",
    "drag_ghost",
    "edge_peek",
    "text_runs",
    "command_palette",
    "hover_card",
    "link_pill",
    "menu",
    "menu_item",
    "peek",
    "popover",
    "scrim",
    "send_pill",
    "sheet",
    "toast",
    "tooltip",
    "space_editor",
];

#[test]
fn component_sheets_use_tokens_only() {
    let all = ds::component_sheets();
    let mut failures = Vec::new();
    for name in SHEETS {
        let Some((_, css)) = all.iter().find(|(n, _)| n == name) else {
            failures.push(format!("{name}: not in the component list"));
            continue;
        };
        if css.trim().starts_with("/*") && css.lines().count() < 3 {
            failures.push(format!("{name}.css is still the stub"));
        }
        for problem in token_violations(css) {
            failures.push(format!("{name}.css: {problem}"));
        }
    }
    for (component, sheets) in STYLES {
        for problem in token_violations(sheets[0]) {
            failures.push(format!("{component}.css: {problem}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
