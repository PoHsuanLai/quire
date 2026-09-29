//! Matching a menu title against the typed query: the command menu's fuzzy ranker
//! (design/06-INTERACTIONS.md section 11.2) and the `mark` runs it draws.

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
enum Hit {
    Marked,
    Plain,
}

/// `text` split into maximal runs of marked and unmarked characters.
fn runs(text: &str, marks: &[usize]) -> Vec<(String, Hit)> {
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

/// A fuzzy match: its score and the matched characters of the text, by char index.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Match {
    /// Higher is better.
    pub score: f32,
    /// The matched characters.
    pub marks: Vec<usize>,
}

/// The command menu's fuzzy score (design/06-INTERACTIONS.md section 11.2), case-insensitive:
/// a substring scores 100 plus 40 at a word start (20 elsewhere) less half its position; else
/// every query character in order scores `2 + 2 x run` plus 6 at a word start, less .3 per
/// character of spread. `None` when the query is not a subsequence of the text.
pub(crate) fn fuzzy(query: &str, text: &str) -> Option<Match> {
    let q: Vec<char> = query.chars().map(lower).collect();
    let t: Vec<char> = text.chars().map(lower).collect();
    if q.is_empty() {
        return Some(Match {
            score: 0.0,
            marks: Vec::new(),
        });
    }
    if let Some(at) = t.windows(q.len()).position(|window| window == q.as_slice()) {
        let bonus = if word_start(&t, at) { 40.0 } else { 20.0 };
        return Some(Match {
            score: 100.0 + bonus - 0.5 * at as f32,
            marks: (at..at + q.len()).collect(),
        });
    }
    subsequence(&q, &t)
}

/// Every query character in order, scored by runs and word starts.
fn subsequence(q: &[char], t: &[char]) -> Option<Match> {
    let (mut next, mut run, mut score) = (0usize, 0u32, 0.0f32);
    let mut marks = Vec::new();
    for (index, c) in t.iter().enumerate() {
        if q.get(next) == Some(c) {
            run += 1;
            let start = if word_start(t, index) { 6.0 } else { 0.0 };
            score += 2.0 + 2.0 * run as f32 + start;
            marks.push(index);
            next += 1;
        } else {
            run = 0;
        }
    }
    if next < q.len() {
        return None;
    }
    let spread = match (marks.first(), marks.last()) {
        (Some(first), Some(last)) => (last - first) as f32,
        _ => 0.0,
    };
    Some(Match {
        score: score - 0.3 * spread,
        marks,
    })
}

/// One character lower-cased to one character, so indices stay aligned with the text.
fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Index 0, or after whitespace or one of `- _ . / @`.
fn word_start(t: &[char], index: usize) -> bool {
    index == 0
        || t.get(index - 1)
            .is_some_and(|c| c.is_whitespace() || matches!(c, '-' | '_' | '.' | '/' | '@'))
}

#[cfg(test)]
mod tests {
    use super::{Hit, fuzzy, runs};

    /// A match's score and marks, or no match.
    type Want = Option<(f32, &'static [usize])>;

    #[test]
    fn the_fuzzy_score_follows_the_prototype() {
        // (query, text, score, marks)
        #[rustfmt::skip]
        let cases: &[(&str, &str, Want)] = &[
            ("", "Tomorrow", Some((0.0, &[]))),
            ("tom", "Tomorrow", Some((140.0, &[0, 1, 2]))),
            ("row", "Tomorrow", Some((117.5, &[5, 6, 7]))),
            ("week", "Next week", Some((137.5, &[5, 6, 7, 8]))),
            // t(0, start) 2+2+6, m(2) 2+2, w(7) 2+2: 18, less .3 x 7.
            ("tmw", "Tomorrow", Some((15.9, &[0, 2, 7]))),
            ("zz", "Tomorrow", None),
            ("TOM", "tomorrow", Some((140.0, &[0, 1, 2]))),
        ];
        for (query, text, want) in cases {
            let got = fuzzy(query, text).map(|m| (m.score, m.marks));
            let want = want.map(|(score, marks)| (score, marks.to_vec()));
            match (&got, &want) {
                (Some((a, am)), Some((b, bm))) => {
                    assert!(
                        (a - b).abs() < 1e-3 && am == bm,
                        "{query} in {text}: {got:?}"
                    );
                }
                (None, None) => {}
                _ => panic!("{query} in {text}: {got:?}, want {want:?}"),
            }
        }
    }

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
