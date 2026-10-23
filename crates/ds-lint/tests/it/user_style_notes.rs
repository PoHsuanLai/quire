//! The cost notes and the app tables of `user_stylesheet` (DESIGN.md sections 7.3 and 7.5): each
//! row is a stylesheet and the `(line, plain words)` of every note it earns.

use ds::selectors::{AppSelectors, ComponentSelectors, PartRename, SURFACE_ATTRIBUTE};
use ds_lint::{UserStyleNoteKind, limits, user_stylesheet, user_stylesheet_with};

struct Case {
    name: &'static str,
    css: &'static str,
    notes: &'static [(u32, &'static str)],
}

const HEAVY: &str = "Heavy blur may slow the desktop";
const SHADOW: &str = "Large shadows may slow the desktop";
const ENDLESS: &str = "This animation never stops";
const ALL: &str = "Animates everything";
const NO_EFFECT: &str = "Has no effect on this desktop";

const CASES: &[Case] = &[
    Case {
        name: "a small blur is quiet",
        css: ".ds-row { filter: blur(4px) }",
        notes: &[],
    },
    Case {
        name: "a blur over the radius is heavy",
        css: ".ds-row { filter: blur(40px) }",
        notes: &[(1, HEAVY)],
    },
    Case {
        name: "the limit itself is quiet, in em too",
        css: ".ds-row { filter: blur(24px) }\n.ds-row { filter: blur(1.5em) }",
        notes: &[],
    },
    Case {
        name: "any blur on a surface root is heavy",
        css: "[data-surface=bar] { filter: blur(2px) }",
        notes: &[(1, HEAVY)],
    },
    Case {
        name: "a blur on a part inside a surface is not on the root",
        css: "[data-surface=bar] .ds-row { filter: blur(2px) }",
        notes: &[],
    },
    Case {
        name: "backdrop-filter is not drawn, so it has no effect and is never also heavy",
        css: ".ds-row { backdrop-filter: blur(40px) }\n.ds-row { backdrop-filter: blur(60px) }\n[data-surface=bar] { backdrop-filter: blur(2px) }",
        notes: &[(1, NO_EFFECT), (2, NO_EFFECT), (3, NO_EFFECT)],
    },
    Case {
        name: "a filter without blur is quiet",
        css: "[data-surface=bar] { filter: saturate(2) }",
        notes: &[],
    },
    Case {
        name: "one big shadow is large",
        css: ".ds-row { box-shadow: 0 10px 90px rgba(0, 0, 0, 0.5) }",
        notes: &[(1, SHADOW)],
    },
    Case {
        name: "stacked shadows add up",
        css: ".ds-row { box-shadow: 0 2px 40px red, 0 4px 30px blue, inset 0 0 20px #000 }",
        notes: &[(1, SHADOW)],
    },
    Case {
        name: "a modest shadow is quiet",
        css: ".ds-row { box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3), 0 8px 24px #0004 }",
        notes: &[],
    },
    Case {
        name: "an endless animation is noted",
        css: ".ds-row { animation: pulse 1s infinite }\n.ds-row { animation-iteration-count: infinite }",
        notes: &[(1, ENDLESS), (2, ENDLESS)],
    },
    Case {
        name: "an endless animation is fine on a spinner",
        css: ".ds-progress-indicator { animation: spin 1s linear infinite }",
        notes: &[],
    },
    Case {
        name: "a counted animation is quiet",
        css: ".ds-row { animation: pulse 1s 3 }",
        notes: &[],
    },
    Case {
        name: "transition all is noted, named properties are not",
        css: ".ds-row { transition: all 200ms }\n.ds-row { transition: opacity 200ms, color 90ms }",
        notes: &[(1, ALL)],
    },
    Case {
        name: "a property the renderer does not draw has no effect",
        css: ".ds-row { text-shadow: 0 0 2px red; mix-blend-mode: multiply }",
        notes: &[(1, NO_EFFECT), (1, NO_EFFECT)],
    },
    Case {
        name: "a parse fault is could not be read",
        css: ".ds { color: red }\n}",
        notes: &[(2, "Could not be read")],
    },
];

#[test]
fn each_stylesheet_earns_exactly_its_cost_notes() {
    for case in CASES {
        let got: Vec<(u32, &str)> = user_stylesheet(case.css, &ds::kits())
            .iter()
            .map(|note| (note.line, note.kind.plain_words()))
            .collect();
        assert_eq!(got, case.notes, "{}", case.name);
    }
}

#[test]
fn every_kind_has_words_a_person_can_read() {
    let kinds = [
        UserStyleNoteKind::Important,
        UserStyleNoteKind::LargeShadow,
        UserStyleNoteKind::EndlessAnimation,
        UserStyleNoteKind::AnimatesEverything,
    ];
    for kind in kinds {
        let words = kind.plain_words();
        assert!(!words.is_empty() && !words.contains(['`', ':']), "{words}");
    }
}

#[test]
fn the_surface_attribute_is_the_one_the_table_names() {
    assert_eq!(limits::SURFACE_ATTRIBUTE, SURFACE_ATTRIBUTE);
}

const MAILO: AppSelectors = AppSelectors {
    app: "mailo",
    components: &[
        ComponentSelectors {
            component: "Thread",
            root: "mailo-thread",
            parts: &["subject", "snippet"],
        },
        ComponentSelectors {
            component: "Stolen",
            root: "ds-button",
            parts: &[],
        },
        ComponentSelectors {
            component: "Lookalike",
            root: "mailolike",
            parts: &[],
        },
    ],
    renames: &[PartRename {
        root: "mailo-thread",
        part: "subject",
        renamed_from: "title",
    }],
};

#[test]
fn an_app_part_listed_in_its_table_is_not_internal() {
    let tables = [MAILO.classes()];
    let css = ".mailo-thread-subject, .mailo-thread-title, .mailo-thread { color: red }";
    assert_eq!(user_stylesheet_with(css, &ds::kits(), &tables), vec![]);
}

#[test]
fn an_app_part_nobody_listed_is_still_internal() {
    let tables = [MAILO.classes()];
    let notes = user_stylesheet_with(".mailo-thread-body { color: red }", &ds::kits(), &tables);
    assert_eq!(notes.len(), 1, "{notes:?}");
    let notes = user_stylesheet(".mailo-thread-subject { color: red }", &ds::kits());
    assert_eq!(notes.len(), 1, "{notes:?}");
}

#[test]
fn an_app_table_cannot_claim_a_name_outside_its_prefix() {
    assert_eq!(MAILO.misnamed_roots(), vec!["ds-button", "mailolike"]);
    let classes = MAILO.classes();
    assert!(!classes.iter().any(|c| c == "mailolike" || c == "ds-button"));
}
