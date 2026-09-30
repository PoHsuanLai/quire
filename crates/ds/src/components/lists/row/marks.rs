//! A title's matched characters drawn as `mark` runs: what a row shows where a palette's query
//! matched it (design/06-INTERACTIONS.md section 11.2).

use dioxus::prelude::*;

/// `text` with the characters at `marks` wrapped in `mark` runs.
pub(crate) fn marked(text: &str, marks: &[usize]) -> Element {
    let runs = runs(text, marks);
    rsx! {
        for (index , (run , hit)) in runs.into_iter().enumerate() {
            if hit == Hit::Marked {
                mark { key: "{index}", "{run}" }
            } else {
                Fragment { key: "{index}", "{run}" }
            }
        }
    }
}

/// Whether a run of a title matched the query.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Hit {
    Marked,
    Plain,
}

/// `text` split into maximal runs of marked and unmarked characters.
pub(crate) fn runs(text: &str, marks: &[usize]) -> Vec<(String, Hit)> {
    let mut out: Vec<(String, Hit)> = Vec::new();
    for (index, c) in text.chars().enumerate() {
        let hit = if marks.contains(&index) {
            Hit::Marked
        } else {
            Hit::Plain
        };
        match out.last_mut() {
            Some((run, last)) if *last == hit => run.push(c),
            _ => out.push((c.to_string(), hit)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Hit, runs};

    #[test]
    fn marks_split_a_title_into_runs() {
        assert_eq!(
            runs("Re: UIDL", &[4, 5, 6, 7]),
            vec![
                ("Re: ".to_string(), Hit::Plain),
                ("UIDL".to_string(), Hit::Marked)
            ]
        );
    }
}
