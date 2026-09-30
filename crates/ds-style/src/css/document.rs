//! How the stylesheet is laid out as text: a header, each section under `/* == name == */`, each
//! component sheet under `/* -- name -- */`. The assembly decides which sections and sheets go
//! in, and in what order.

use crate::css::UTILITIES;
use crate::css::grain::grain_uri;
use crate::emit::{property, rule};
use crate::kit::Sheet;

/// The whole stylesheet: the header, then each `(section, body)` in the order given.
pub(crate) fn document(sections: &[(&str, String)]) -> String {
    let mut css =
        String::from("/* ds::stylesheet(), generated from the token table. Do not edit. */\n");
    for (name, body) in sections {
        css.push_str(&format!("/* == {name} == */\n{}\n", body.trim_end()));
    }
    css
}

/// `own`, in its order, with each of `extra` placed right after the sheet it names, the sheets
/// that name it after it in turn. A sheet whose anchor is in neither list goes last, so it is
/// still styled, and `unplaced` names it for a test to catch.
pub fn placed(
    own: &[(&'static str, &'static str)],
    extra: &[Sheet],
) -> Vec<(&'static str, &'static str)> {
    fn follow(name: &str, extra: &[Sheet], out: &mut Vec<(&'static str, &'static str)>) {
        for sheet in extra.iter().filter(|sheet| sheet.after == name) {
            out.push((sheet.name, sheet.css));
            follow(sheet.name, extra, out);
        }
    }
    let mut out = Vec::with_capacity(own.len() + extra.len());
    for (name, css) in own {
        out.push((*name, *css));
        follow(name, extra, &mut out);
    }
    out.extend(unplaced(own, extra).map(|sheet| (sheet.name, sheet.css)));
    out
}

/// The sheets of `extra` whose anchor is neither in `own` nor another of `extra`.
pub fn unplaced<'a>(
    own: &'a [(&'static str, &'static str)],
    extra: &'a [Sheet],
) -> impl Iterator<Item = Sheet> + 'a {
    extra.iter().copied().filter(move |sheet| {
        !own.iter().any(|(name, _)| *name == sheet.after)
            && !extra.iter().any(|other| other.name == sheet.after)
    })
}

/// Component sheets, each under its marker, in the order given.
pub fn sheets(sheets: &[(&str, &str)]) -> String {
    sheets
        .iter()
        .map(|(name, css)| format!("/* -- {name} -- */\n{}\n", css.trim_end()))
        .collect()
}

/// `.ds-truncate` and the other utilities, then the grain tile on `.ds-grain`.
pub fn utilities_css() -> String {
    let grain = rule(
        ".ds-grain",
        &[property(
            "background-image",
            &format!("url(\"{}\")", grain_uri()),
        )],
    );
    format!("{}\n{grain}", UTILITIES.trim_end())
}

#[cfg(test)]
mod tests {
    use super::{placed, unplaced};
    use crate::kit::Sheet;

    const OWN: [(&str, &str); 3] = [("a", ".a{}"), ("b", ".b{}"), ("c", ".c{}")];
    const EXTRA: [Sheet; 3] = [
        Sheet {
            name: "x",
            css: ".x{}",
            after: "a",
        },
        Sheet {
            name: "y",
            css: ".y{}",
            after: "x",
        },
        Sheet {
            name: "z",
            css: ".z{}",
            after: "b",
        },
    ];

    fn names(list: &[(&str, &str)]) -> Vec<String> {
        list.iter().map(|(name, _)| (*name).to_owned()).collect()
    }

    #[test]
    fn a_contributed_sheet_follows_its_anchor_and_the_sheets_that_name_it() {
        assert_eq!(names(&placed(&OWN, &EXTRA)), ["a", "x", "y", "b", "z", "c"]);
    }

    #[test]
    fn a_sheet_with_no_anchor_goes_last_and_is_reported() {
        let lost = [Sheet {
            name: "q",
            css: ".q{}",
            after: "nowhere",
        }];
        assert_eq!(names(&placed(&OWN, &lost)), ["a", "b", "c", "q"]);
        assert_eq!(unplaced(&OWN, &lost).count(), 1);
        assert_eq!(unplaced(&OWN, &EXTRA).count(), 0);
    }
}
