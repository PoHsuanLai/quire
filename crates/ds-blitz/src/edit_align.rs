//! Where each character of an inline root's DOM text landed in its laid-out text.
//!
//! Blitz lays an inline root's text out as one string (`inline_layout_data.text`): its text
//! nodes concatenated after white-space collapsing, with a `br` as `\n` and generated text (a
//! list marker, `::before`) in between, and no record of which node a byte came from. mailo's
//! positions are offsets into its own text nodes, so both directions need this map. Pure over
//! strings, so the table below is the proof.

/// A character boundary in one text node and the layout byte it sits at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stop {
    /// UTF-8 bytes into the text node.
    pub(crate) dom: usize,
    /// UTF-8 bytes into the layout text.
    pub(crate) layout: usize,
}

/// The stops of each text in `texts` (one per character boundary, the end included), laid out
/// in order as `layout`. A text character missing from the layout (collapsed whitespace, or
/// anything the layout does not contain) sits where the next one does; layout text no DOM text
/// accounts for (a marker, a `br`) is stepped over.
pub(crate) fn align(layout: &str, texts: &[&str]) -> Vec<Vec<Stop>> {
    let mut at = 0;
    texts
        .iter()
        .map(|text| {
            let mut stops: Vec<Stop> = text
                .char_indices()
                .map(|(dom, ch)| {
                    let (layout_at, next) = place(layout, at, ch);
                    at = next;
                    Stop {
                        dom,
                        layout: layout_at,
                    }
                })
                .collect();
            stops.push(Stop {
                dom: text.len(),
                layout: at,
            });
            stops
        })
        .collect()
}

/// Where `ch` lands in `layout` searching from `at`, and where the search goes on from.
fn place(layout: &str, at: usize, ch: char) -> (usize, usize) {
    let rest = &layout[at..];
    match rest.chars().next() {
        Some(first) if same(first, ch) || (first.is_whitespace() && ch.is_whitespace()) => {
            (at, at + first.len_utf8())
        }
        // Collapsed away: it sits where the next character does.
        _ if ch.is_whitespace() => (at, at),
        _ => match rest.char_indices().find(|&(_, found)| same(found, ch)) {
            // Layout-only text before it (a marker, a `br`): step over.
            Some((skip, found)) => (at + skip, at + skip + found.len_utf8()),
            // Not laid out at all: it sits where the next character does.
            None => (at, at),
        },
    }
}

/// The same character, allowing for `text-transform` changing its case.
fn same(laid: char, written: char) -> bool {
    laid == written || laid.to_lowercase().eq(written.to_lowercase())
}

/// The DOM position of layout byte `layout` among `stops` (every text's, in order): the last
/// stop at or before it, so a boundary between two texts reads as the start of the later one,
/// and a caret after collapsed spaces sits after them. `None` for no stops at all.
pub(crate) fn to_dom(stops: &[(usize, Stop)], layout: usize) -> Option<(usize, usize)> {
    stops
        .iter()
        .rev()
        .find(|(_, stop)| stop.layout <= layout)
        .or_else(|| stops.first())
        .map(|&(text, stop)| (text, stop.dom))
}

/// The layout byte of `dom` bytes into a text with `stops`: the stop at it, or the last before.
pub(crate) fn to_layout(stops: &[Stop], dom: usize) -> usize {
    stops
        .iter()
        .rev()
        .find(|stop| stop.dom <= dom)
        .or_else(|| stops.first())
        .map_or(0, |stop| stop.layout)
}

#[cfg(test)]
mod tests {
    use super::{Stop, align, to_dom, to_layout};

    fn layouts(stops: &[Stop]) -> Vec<usize> {
        stops.iter().map(|stop| stop.layout).collect()
    }

    /// (layout, texts, each text's stops' layout bytes)
    type Case = (
        &'static str,
        &'static [&'static str],
        &'static [&'static [usize]],
    );

    const CASES: &[Case] = &[
        // Preserved: one to one.
        ("ab cd", &["ab ", "cd"], &[&[0, 1, 2, 3], &[3, 4, 5]]),
        // Two spaces collapsed to one: the second sits before `b`.
        ("a b", &["a  b"], &[&[0, 1, 2, 2, 3]]),
        // A newline collapsed to a space.
        ("a b", &["a\nb"], &[&[0, 1, 2, 3]]),
        // A list marker before the text, a `br` between two texts.
        ("* hi\nyo", &["hi", "yo"], &[&[2, 3, 4], &[5, 6, 7]]),
        // Upper-cased by text-transform.
        ("AB", &["ab"], &[&[0, 1, 2]]),
        // Multi-byte text.
        ("注音", &["注音"], &[&[0, 3, 6]]),
        // A leading space collapsed at the start of the line.
        ("hi", &[" hi"], &[&[0, 0, 1, 2]]),
    ];

    #[test]
    fn text_nodes_align_to_the_layout_text() {
        for &(layout, texts, expected) in CASES {
            let aligned = align(layout, texts);
            let got: Vec<Vec<usize>> = aligned.iter().map(|stops| layouts(stops)).collect();
            let want: Vec<Vec<usize>> = expected.iter().map(|bytes| bytes.to_vec()).collect();
            assert_eq!(got, want, "{layout:?} from {texts:?}");
        }
    }

    #[test]
    fn a_layout_byte_maps_back_to_its_text_and_offset() {
        let aligned = align("* hi\nyo", &["hi", "yo"]);
        let all: Vec<(usize, Stop)> = aligned
            .iter()
            .enumerate()
            .flat_map(|(text, stops)| stops.iter().map(move |&stop| (text, stop)))
            .collect();
        let cases = [
            (0, (0, 0)),
            (2, (0, 0)),
            (3, (0, 1)),
            (4, (0, 2)),
            (5, (1, 0)),
            (7, (1, 2)),
        ];
        for (layout, expected) in cases {
            assert_eq!(to_dom(&all, layout), Some(expected), "layout byte {layout}");
        }
        assert_eq!(to_dom(&[], 3), None);
    }

    #[test]
    fn a_dom_offset_maps_to_its_layout_byte() {
        let aligned = align("a b", &["a  b"]);
        let cases = [(0, 0), (1, 1), (2, 2), (3, 2), (4, 3), (9, 3)];
        for (dom, expected) in cases {
            assert_eq!(to_layout(&aligned[0], dom), expected, "dom byte {dom}");
        }
    }
}
