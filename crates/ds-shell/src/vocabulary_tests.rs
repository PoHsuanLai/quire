//! The vocabulary the kits give the linter, checked against the stylesheet they generate.

use crate::{kits, stylesheet};
use ds_lint::{tokenize, walk};

/// Every custom property the generated sections declare on a `.ds` root block (the reset, token,
/// accent, material, shape and ground sections), asked of the generated CSS itself.
fn root_declarations() -> Vec<String> {
    let sheet = stylesheet();
    let end = sheet
        .find("/* == motion == */")
        .expect("the stylesheet has a motion section");
    let (rules, _) = walk::walk(&tokenize::tokens(&sheet[..end]));
    rules
        .into_iter()
        .filter(|rule| {
            rule.selector
                .strip_prefix(".ds")
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('['))
        })
        .flat_map(|rule| rule.declarations)
        .map(|decl| decl.property.text)
        .filter(|name| name.starts_with("--"))
        .collect()
}

#[test]
fn every_root_declaration_is_known() {
    let known = kits().vocabulary().vars;
    let missing: Vec<String> = root_declarations()
        .into_iter()
        .filter(|name| !known.contains(name))
        .collect();
    assert!(missing.is_empty(), "declared but not known: {missing:?}");
}

#[test]
fn the_kits_reach_the_vocabulary() {
    const CASES: &[&str] = &[
        "--paper",
        "--handle-ring",
        "--t-quick",
        "--c-violet-soft",
        "--swatch-postmark",
        "--f-grad",
        "--m-tint",
        "--m-tint-alpha",
        "--font-data",
        "--s-1",
        "--s-36",
        "--ctl-h-m",
        "--switch-knob-s",
        "--cc-panel-r",
        "--bar-status-w",
        "--shell-menu-font",
        "--dock-tile-px",
        "--scale-hair",
        "--icons-glyph-share",
    ];
    let known = kits().vocabulary().vars;
    for name in CASES {
        assert!(known.contains(*name), "{name}");
    }
    for gone in ["--t-tap", "--d-heal", "--d-fly", "--e-spring", "--squish"] {
        assert!(!known.contains(gone), "{gone}");
    }
}

#[test]
fn a_keyframes_name_and_its_alias_are_known() {
    let known = kits().vocabulary().keyframes;
    for name in ["fade", "fade--b", "menu-out"] {
        assert!(known.contains(name), "{name}");
    }
    for name in ["wobble", "fade--c"] {
        assert!(!known.contains(name), "{name}");
    }
}
