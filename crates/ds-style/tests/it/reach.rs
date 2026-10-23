//! The reach rule (customization design section 7.6): a sheet's `url()` and `@import` may name
//! `data:` or a file inside the sheet's own folder, and the rewrite cuts the rest.

use ds_style::kit::reach::within;
use ds_style::kit::{CustomStyles, LookStyle, Slot, StyleHomes, Styles, UserStyle};
use std::path::{Path, PathBuf};

const HOME: &str = "/home/ada/.config/quire";

struct Case {
    name: &'static str,
    css: &'static str,
    kept: bool,
}

const CASES: &[Case] = &[
    Case {
        name: "data",
        css: "a { background: url(data:image/png;base64,AAAA) }",
        kept: true,
    },
    Case {
        name: "data in quotes, any case",
        css: "a { background: URL( \"DATA:image/png;base64,AAAA\" ) }",
        kept: true,
    },
    Case {
        name: "relative file",
        css: "a { background: url(paper/grain.png) }",
        kept: true,
    },
    Case {
        name: "quoted relative file",
        css: "a { background: url('grain.png') }",
        kept: true,
    },
    Case {
        name: "absolute file inside",
        css: "a { background: url(file:///home/ada/.config/quire/grain.png) }",
        kept: true,
    },
    Case {
        name: "in-document reference",
        css: "a { filter: url(#blur) }",
        kept: true,
    },
    Case {
        name: "https",
        css: "a { background: url(https://example.org/x.png) }",
        kept: false,
    },
    Case {
        name: "http, quoted",
        css: "a { background: url(\"http://example.org/x.png\") }",
        kept: false,
    },
    Case {
        name: "protocol-relative",
        css: "a { background: url(//example.org/x.png) }",
        kept: false,
    },
    Case {
        name: "file elsewhere",
        css: "a { background: url(file:///etc/passwd) }",
        kept: false,
    },
    Case {
        name: "absolute path elsewhere",
        css: "a { background: url(/etc/passwd) }",
        kept: false,
    },
    Case {
        name: "way out by dots",
        css: "a { background: url(../other/x.png) }",
        kept: false,
    },
    Case {
        name: "way out inside a path",
        css: "a { background: url(file:///home/ada/.config/quire/../x.png) }",
        kept: false,
    },
    Case {
        name: "way out by encoding",
        css: "a { background: url(%2e%2e/x.png) }",
        kept: false,
    },
    Case {
        name: "font face",
        css: "@font-face { font-family: X; src: url(https://example.org/x.woff2) }",
        kept: false,
    },
];

#[test]
fn a_reference_stays_only_when_it_is_data_or_inside_the_folder() {
    for case in CASES {
        let (out, cuts) = within(case.css, Some(Path::new(HOME)));
        assert_eq!(out == case.css, case.kept, "{}: {out}", case.name);
        assert_eq!(cuts.is_empty(), case.kept, "{}: {cuts:?}", case.name);
    }
}

#[test]
fn without_a_folder_only_data_stays() {
    let (out, cuts) = within("a { b: url(x.png) } c { d: url(data:,) }", None);
    assert_eq!(out, "a { b: url(\"data:,\") } c { d: url(data:,) }");
    assert_eq!(cuts.len(), 1);
}

#[test]
fn a_cut_url_becomes_an_empty_data_url_and_is_recorded_with_its_line() {
    let css = "a { color: red }\nb {\n  background: url(\"https://example.org/x.png\");\n}\n";
    let (out, cuts) = within(css, Some(Path::new(HOME)));
    assert_eq!(
        out,
        "a { color: red }\nb {\n  background: url(\"data:,\");\n}\n"
    );
    assert_eq!(cuts.len(), 1);
    assert_eq!(cuts[0].line, 3);
    assert_eq!(cuts[0].target, "https://example.org/x.png");
}

#[test]
fn an_import_that_reaches_out_is_removed_and_a_local_one_stays() {
    let css = "@import \"https://example.org/a.css\";\n@import \"more.css\";\na { color: red }";
    let (out, cuts) = within(css, Some(Path::new(HOME)));
    assert_eq!(out, "\n@import \"more.css\";\na { color: red }");
    assert_eq!(cuts.len(), 1);
    assert_eq!(cuts[0].target, "https://example.org/a.css");
}

#[test]
fn text_in_comments_and_strings_is_not_a_reference() {
    let css =
        "/* url(https://example.org/x) */ a::after { content: \"url(https://example.org/x)\" }";
    let (out, cuts) = within(css, Some(Path::new(HOME)));
    assert_eq!(out, css);
    assert!(cuts.is_empty());
}

#[test]
fn a_name_that_ends_in_url_is_not_a_reference() {
    let css = "a { b: curl(https://example.org/x) }";
    assert_eq!(within(css, None).0, css);
}

fn styles() -> Styles {
    Styles {
        look: LookStyle(".look { color: red }".to_owned()),
        desktop: UserStyle(".desktop { color: green }".to_owned()),
        app: UserStyle(".app { background: url(https://example.org/x.png) }".to_owned()),
        homes: StyleHomes {
            desktop: Some(PathBuf::from(HOME)),
            ..StyleHomes::default()
        },
    }
}

#[test]
fn the_sheets_are_drawn_look_then_desktop_then_app_with_the_look_in_its_layer() {
    let drawn = styles().drawn(CustomStyles::Use);
    let slots: Vec<Slot> = drawn.iter().map(|sheet| sheet.slot).collect();
    assert_eq!(slots, [Slot::Look, Slot::Desktop, Slot::App]);
    assert!(
        drawn[0]
            .text
            .starts_with("@layer ds, app, look;\n@layer look {")
    );
    assert_eq!(drawn[1].text, ".desktop { color: green }");
    assert!(
        drawn[2].text.contains("url(\"data:,\")"),
        "{}",
        drawn[2].text
    );
}

#[test]
fn off_draws_nothing_and_a_blank_sheet_draws_no_element() {
    assert!(styles().drawn(CustomStyles::Off).is_empty());
    let only_app = Styles {
        app: UserStyle(".a{}".to_owned()),
        ..Styles::default()
    };
    assert_eq!(only_app.drawn(CustomStyles::Use).len(), 1);
    assert!(Styles::default().drawn(CustomStyles::Use).is_empty());
}

#[test]
fn what_was_cut_is_listed_by_sheet() {
    let dropped = styles().dropped();
    assert_eq!(dropped.len(), 1);
    assert_eq!(dropped[0].slot, Slot::App);
}

#[test]
fn the_fingerprint_follows_every_sheet_and_the_order_of_the_texts() {
    let base = styles();
    let mut moved = base.clone();
    std::mem::swap(&mut moved.desktop, &mut moved.app);
    let mut edited = base.clone();
    edited.look.0.push(' ');
    assert_eq!(base.fingerprint(), styles().fingerprint());
    assert_ne!(base.fingerprint(), moved.fingerprint());
    assert_ne!(base.fingerprint(), edited.fingerprint());
}
