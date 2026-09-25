//! Where one page ends and the next begins, as a pure function of the laid-out flow.
//!
//! Blitz cannot parse CSS fragmentation (stylo gates `break-before`/`break-inside` as
//! gecko-only and ignores `@page`), so the rules are quire's, read from markers
//! (`crate::pdf::flow`):
//!
//! - a forced break (`data-break-before="page"`) ends the page at the marked box's top;
//! - otherwise a page ends where it is full, moved up to the top of any keep-together span the
//!   cut would split: a line box, a replaced element (an image), a table row, a box marked
//!   `data-break-inside="avoid"`, and a text block's first two and last two lines (orphans and
//!   widows, 2 each, as CSS's initial values);
//! - a span taller than a page is not kept: it is cut where the page is full (its own lines
//!   still move whole), since moving it could never make it fit;
//! - a span that already starts at the page's top is never moved, so every page makes progress.

/// A vertical stretch of the laid-out document, in CSS pixels from its top.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Span {
    pub(crate) top: f32,
    pub(crate) bottom: f32,
}

impl Span {
    pub(crate) fn height(self) -> f32 {
        self.bottom - self.top
    }
}

/// What pagination needs to know about a laid-out document.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Flow {
    /// The document's full height.
    pub(crate) height: f32,
    /// Spans a page should not end inside.
    pub(crate) keeps: Vec<Span>,
    /// Where a page must end (the top of a box marked to start a page).
    pub(crate) breaks: Vec<f32>,
}

/// Closer than this is the same position: layout rounds to whole pixels, so a keep whose top
/// is the cut is not split by it.
const SAME: f32 = 0.01;

/// The pages of `flow`, each at most `page` tall, top to bottom; always at least one, so an
/// empty document prints one blank page.
pub(crate) fn paginate(flow: &Flow, page: f32) -> Vec<Span> {
    let mut pages = Vec::new();
    let mut top = 0.0;
    while top < flow.height - SAME || pages.is_empty() {
        let bottom = cut(flow, top, page).max(top);
        pages.push(Span { top, bottom });
        if bottom <= top + SAME {
            break;
        }
        top = bottom;
    }
    pages
}

/// Where the page starting at `top` ends.
fn cut(flow: &Flow, top: f32, page: f32) -> f32 {
    let full = top + page;
    let forced = flow
        .breaks
        .iter()
        .copied()
        .filter(|&at| at > top + SAME && at <= full)
        .reduce(f32::min);
    match forced {
        Some(at) => at,
        None if full >= flow.height => flow.height,
        None => settle(&flow.keeps, top, full, page),
    }
}

/// `full` moved up past every keep it would split, until it splits none.
fn settle(keeps: &[Span], top: f32, full: f32, page: f32) -> f32 {
    let mut cut = full;
    loop {
        let moved = keeps
            .iter()
            .filter(|keep| keep.top > top + SAME && keep.height() <= page)
            .filter(|keep| keep.top < cut - SAME && keep.bottom > cut + SAME)
            .map(|keep| keep.top)
            .fold(cut, f32::min);
        if moved >= cut {
            return cut;
        }
        cut = moved;
    }
}

#[cfg(test)]
mod tests {
    use super::{Flow, Span, paginate};

    const fn span(top: f32, bottom: f32) -> Span {
        Span { top, bottom }
    }

    /// Lines of `height` from `top`, `count` of them.
    fn lines(top: f32, height: f32, count: u16) -> Vec<Span> {
        (0..count)
            .map(|n| span(top + f32::from(n) * height, top + f32::from(n + 1) * height))
            .collect()
    }

    struct Case {
        name: &'static str,
        flow: Flow,
        pages: Vec<(f32, f32)>,
    }

    fn cases() -> Vec<Case> {
        vec![
            Case {
                name: "empty document is one blank page",
                flow: Flow::default(),
                pages: vec![(0.0, 0.0)],
            },
            Case {
                name: "plain flow cuts where full",
                flow: Flow {
                    height: 250.0,
                    ..Flow::default()
                },
                pages: vec![(0.0, 100.0), (100.0, 200.0), (200.0, 250.0)],
            },
            Case {
                name: "a straddling line moves down",
                flow: Flow {
                    height: 150.0,
                    keeps: vec![span(95.0, 105.0)],
                    breaks: vec![],
                },
                pages: vec![(0.0, 95.0), (95.0, 150.0)],
            },
            Case {
                name: "a forced break ends the page early",
                flow: Flow {
                    height: 150.0,
                    keeps: vec![],
                    breaks: vec![40.0],
                },
                pages: vec![(0.0, 40.0), (40.0, 140.0), (140.0, 150.0)],
            },
            Case {
                name: "a break at the very top makes no blank page",
                flow: Flow {
                    height: 50.0,
                    keeps: vec![],
                    breaks: vec![0.0],
                },
                pages: vec![(0.0, 50.0)],
            },
            Case {
                name: "a keep-together block moves whole",
                flow: Flow {
                    height: 200.0,
                    keeps: vec![span(80.0, 150.0), span(80.0, 90.0), span(90.0, 100.0)],
                    breaks: vec![],
                },
                pages: vec![(0.0, 80.0), (80.0, 180.0), (180.0, 200.0)],
            },
            Case {
                name: "a block taller than a page splits between its lines",
                flow: Flow {
                    height: 200.0,
                    keeps: [vec![span(50.0, 170.0)], lines(50.0, 20.0, 6)].concat(),
                    breaks: vec![],
                },
                pages: vec![(0.0, 90.0), (90.0, 190.0), (190.0, 200.0)],
            },
            Case {
                name: "a lone first line (orphan) goes with the rest",
                flow: Flow {
                    height: 200.0,
                    keeps: [
                        lines(90.0, 10.0, 6),
                        vec![span(90.0, 110.0), span(130.0, 150.0)],
                    ]
                    .concat(),
                    breaks: vec![],
                },
                pages: vec![(0.0, 90.0), (90.0, 190.0), (190.0, 200.0)],
            },
            Case {
                name: "a lone last line (widow) takes one more with it",
                flow: Flow {
                    height: 200.0,
                    keeps: [
                        lines(50.0, 10.0, 6),
                        vec![span(50.0, 70.0), span(90.0, 110.0)],
                    ]
                    .concat(),
                    breaks: vec![],
                },
                pages: vec![(0.0, 90.0), (90.0, 190.0), (190.0, 200.0)],
            },
            Case {
                name: "a keep at the page top is never moved",
                flow: Flow {
                    height: 300.0,
                    keeps: vec![span(0.0, 150.0)],
                    breaks: vec![],
                },
                pages: vec![(0.0, 100.0), (100.0, 200.0), (200.0, 300.0)],
            },
        ]
    }

    #[test]
    fn pages_end_where_the_rules_say() {
        for case in cases() {
            let got: Vec<(f32, f32)> = paginate(&case.flow, 100.0)
                .into_iter()
                .map(|page| (page.top, page.bottom))
                .collect();
            assert_eq!(got, case.pages, "{}", case.name);
        }
    }
}
