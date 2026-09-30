//! The report's notes, one table: each row is a stylesheet and the notes it earns.

use super::note::ParseFault::*;
use super::note::UserStyleNoteKind::{self, *};
use super::user_stylesheet;

fn internal(selector: &str) -> UserStyleNoteKind {
    InternalSelector {
        selector: selector.to_owned(),
    }
}

fn unknown(name: &str) -> UserStyleNoteKind {
    UnknownVariable {
        name: name.to_owned(),
    }
}

fn remote(url: &str) -> UserStyleNoteKind {
    NonLocalUrl {
        url: url.to_owned(),
    }
}

/// A stylesheet, and the `(line, kind)` of every note it earns, in order.
struct Case {
    name: &'static str,
    css: &'static str,
    notes: Vec<(u32, UserStyleNoteKind)>,
}

fn cases() -> Vec<Case> {
    let case = |name, css, notes| Case { name, css, notes };
    vec![
        case("an empty file has nothing to say", "", vec![]),
        case("comments only", "/* nothing yet */\n", vec![]),
        case(
            "token overrides are the recommended form",
            ".ds { --accent: red; --t-quick: 120ms }",
            vec![],
        ),
        case(
            "a surface, a component root, its part and an axis are public",
            "[*|data-surface=bar] .ds-slider-thumb, .ds-button[*|data-variant=primary] { color: red }",
            vec![],
        ),
        case(
            "aria is a public family",
            ".ds-toggle[*|aria-checked=true] { color: red }",
            vec![],
        ),
        case(
            "raw colours, durations and fonts are the point of it",
            ".ds { color: #f00; transition: opacity 90ms ease; font-family: Georgia }",
            vec![],
        ),
        case(
            "a media query nests rules",
            "@media (min-width: 10px) { .ds-menu { color: red } }",
            vec![],
        ),
        case(
            "an unknown variable is noted where it is read",
            ".ds { color: var(--acent) }",
            vec![(1, unknown("--acent"))],
        ),
        case(
            "a variable the file declares itself is known",
            ".ds { --mine: red }\n.ds-row { color: var(--mine) }",
            vec![],
        ),
        case(
            "a design-system token is known",
            ".ds-row { color: var(--accent) }",
            vec![],
        ),
        case(
            "an internal class, an id and an unlisted attribute are noted",
            ".ds-chip-label { }\n#main { }\n[*|data-theme=dark] { }",
            vec![
                (1, internal(".ds-chip-label")),
                (2, internal("#main")),
                (3, internal("[data-theme]")),
            ],
        ),
        case(
            "a public attribute without the namespace never matches on Blitz",
            "[data-surface=bar] { color: red }",
            vec![(
                1,
                UnprefixedAttribute {
                    selector: "[data-surface]".to_owned(),
                },
            )],
        ),
        case(
            "important is noted and the rule still stands",
            ".ds-row {\n  color: red !important;\n}",
            vec![(2, Important)],
        ),
        case(
            "only local urls are quiet",
            ".a { background: url(data:image/png;base64,AAAA) }\n.ds { background: url(\"file:///tmp/x.png\") }",
            vec![(1, internal(".a"))],
        ),
        case(
            "a remote or relative url is noted",
            ".ds { background: url(https://example.com/x.png) }\n.ds { background: url('x.png') }\n@import \"other.css\";",
            vec![
                (1, remote("https://example.com/x.png")),
                (2, remote("x.png")),
                (3, remote("other.css")),
            ],
        ),
        case(
            "a stray closer is a parse error",
            ".ds { color: red }\n}",
            vec![(2, Parse(StrayCloser('}')))],
        ),
        case(
            "a block never closed is a parse error",
            ".ds { color: red",
            vec![(1, Parse(Unclosed('{')))],
        ),
        case(
            "a selector with no block is a parse error",
            ".ds { color: red }\n.ds-row\n",
            vec![(2, Parse(RuleWithoutBlock))],
        ),
        case(
            "a declaration without a colon is a parse error",
            ".ds { color red; margin: 0 }",
            vec![(1, Parse(DeclarationWithoutColon))],
        ),
        case(
            "a string that runs off its line is a parse error",
            ".ds { content: \"open\n}",
            vec![(1, Parse(BadString))],
        ),
    ]
}

#[test]
fn each_stylesheet_earns_exactly_its_notes() {
    for case in cases() {
        let got: Vec<(u32, UserStyleNoteKind)> = user_stylesheet(case.css, &ds::kits())
            .into_iter()
            .map(|note| (note.line, note.kind))
            .collect();
        assert_eq!(got, case.notes, "{}", case.name);
    }
}

#[test]
fn only_parse_errors_are_errors() {
    let notes = user_stylesheet(".x { color: var(--nope) !important }\n}", &ds::kits());
    let errors: Vec<bool> = notes.iter().map(|note| note.is_parse_error()).collect();
    assert_eq!(errors, [false, false, false, true], "{notes:?}");
}

#[test]
fn a_note_reads_with_its_line_and_column() {
    let notes = user_stylesheet("\n.ds { color: var(--nope) }", &ds::kits());
    assert_eq!(
        notes[0].to_string(),
        "2:18: note: `--nope` is not a design-system token or declared here"
    );
}
