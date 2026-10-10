//! What a result row says beyond its title: the characters the query matched, drawn emphasised,
//! and a second line under the title. Data and the pure rules that follow from it; `view` draws
//! them.

use std::ops::Range;

/// Characters of a title drawn emphasised, as ranges over its characters (not its bytes), sorted
/// and never touching one another. A typed value, so a row never carries markup.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Marks(Vec<Range<usize>>);

/// Whether a run of a title is emphasised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Emphasis {
    /// Drawn as the rest of the title is.
    Plain,
    /// Matched the query.
    Matched,
}

/// A stretch of a title drawn one way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Run {
    pub text: String,
    pub emphasis: Emphasis,
}

impl Marks {
    /// The marks over `ranges` of characters: empty ranges are dropped, and ranges that overlap
    /// or touch become one.
    pub fn over(ranges: impl IntoIterator<Item = Range<usize>>) -> Self {
        let mut sorted: Vec<Range<usize>> = ranges
            .into_iter()
            .filter(|range| range.start < range.end)
            .collect();
        sorted.sort_by_key(|range| range.start);
        let mut merged: Vec<Range<usize>> = Vec::with_capacity(sorted.len());
        for range in sorted {
            match merged.last_mut() {
                Some(last) if range.start <= last.end => last.end = last.end.max(range.end),
                _ => merged.push(range),
            }
        }
        Marks(merged)
    }

    /// Every place a word of `query` occurs in `title`, ignoring case: what a search that matched
    /// by substring marks. A query of no words marks nothing.
    pub fn of_query(title: &str, query: &str) -> Self {
        let text: Vec<char> = title.chars().map(lower).collect();
        let found = query.split_whitespace().flat_map(|word| {
            let word: Vec<char> = word.chars().map(lower).collect();
            let at: Vec<usize> = match word.len() {
                0 => Vec::new(),
                len => (0..text.len().saturating_sub(len - 1))
                    .filter(|&start| text[start..start + len] == word[..])
                    .collect(),
            };
            let len = word.len();
            at.into_iter().map(move |start| start..start + len)
        });
        Marks::over(found)
    }

    /// Whether nothing is marked.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// `title` cut into plain and matched runs, in order; the marks past its end are ignored.
    pub(crate) fn runs(&self, title: &str) -> Vec<Run> {
        let chars: Vec<char> = title.chars().collect();
        let run = |range: Range<usize>, emphasis| Run {
            text: chars[range].iter().collect(),
            emphasis,
        };
        let mut runs = Vec::new();
        let mut at = 0;
        for range in &self.0 {
            let (start, end) = (range.start.min(chars.len()), range.end.min(chars.len()));
            if start < end {
                if at < start {
                    runs.push(run(at..start, Emphasis::Plain));
                }
                runs.push(run(start..end, Emphasis::Matched));
                at = end;
            }
        }
        if at < chars.len() {
            runs.push(run(at..chars.len(), Emphasis::Plain));
        }
        runs
    }
}

/// One character lower-cased to one character, so indices stay aligned with the title.
fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// What an item says beside its title.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemText {
    /// The characters of the title the query matched.
    pub marks: Marks,
    /// A second, fainter line under the title: an address, a folder, a snippet.
    pub subtitle: Option<String>,
    /// A tooltip for the row, shown at once on hover, also while the row is disabled: why it
    /// cannot be picked ("Wait for the agent to finish"). The row's key equivalent, when it
    /// has one, follows it as `Name  ⌘K`.
    pub tip: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{Emphasis, Marks};

    /// Character ranges as `(start, end)` pairs.
    type Spans = &'static [(usize, usize)];

    fn marks(spans: Spans) -> Marks {
        Marks::over(spans.iter().map(|&(start, end)| start..end))
    }

    #[test]
    fn marks_are_sorted_and_merged() {
        // (given, want)
        const CASES: &[(Spans, Spans)] = &[
            (&[], &[]),
            (&[(2, 2)], &[]),
            (&[(4, 6), (0, 2)], &[(0, 2), (4, 6)]),
            (&[(0, 3), (2, 5)], &[(0, 5)]),
            (&[(0, 2), (2, 4)], &[(0, 4)]),
        ];
        for &(given, want) in CASES {
            let merged = marks(given);
            assert_eq!(merged, marks(want), "{given:?}");
            assert_eq!(merged.is_empty(), want.is_empty(), "{given:?}");
        }
    }

    #[test]
    fn a_query_marks_each_word_wherever_it_occurs() {
        // (title, query, marked characters)
        const CASES: &[(&str, &str, Spans)] = &[
            ("Invoice from Dana", "dana", &[(13, 17)]),
            ("Invoice from Dana", "inv dana", &[(0, 3), (13, 17)]),
            ("banana", "an", &[(1, 5)]),
            ("Invoice", "", &[]),
            ("Invoice", "zzz", &[]),
            ("Invoice", "invoice and more", &[(0, 7)]),
        ];
        for &(title, query, want) in CASES {
            assert_eq!(
                Marks::of_query(title, query),
                marks(want),
                "{title:?} {query:?}"
            );
        }
    }

    #[test]
    fn a_title_is_cut_where_the_marks_are() {
        let marks = Marks::of_query("Invoice from Dana", "from");
        let runs: Vec<(String, Emphasis)> = marks
            .runs("Invoice from Dana")
            .into_iter()
            .map(|run| (run.text, run.emphasis))
            .collect();
        assert_eq!(
            runs,
            vec![
                ("Invoice ".to_owned(), Emphasis::Plain),
                ("from".to_owned(), Emphasis::Matched),
                (" Dana".to_owned(), Emphasis::Plain),
            ]
        );
        assert!(Marks::default().runs("").is_empty());
    }
}
