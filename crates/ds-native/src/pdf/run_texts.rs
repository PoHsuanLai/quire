//! The text behind every glyph run Blitz will paint, read from the same parley layouts it
//! paints from, so the PDF's text is the text that was shaped.
//!
//! blitz-paint hands anyrender a run's glyphs as `glyph_run.positioned_glyphs()` and nothing
//! else. The layouts are public (`inline_layout_data`, list markers, text inputs' editors), and
//! a parley run's glyphs are its visual clusters' glyphs in order, each cluster knowing its
//! byte range in the layout's text. So each glyph run's glyphs are paired with their clusters'
//! text here, before painting, and keyed as the painter will see the run
//! (`anyrender_krilla::RunKey`).

use anyrender_krilla::{GlyphSource, RunKey, RunText, RunTexts};
use blitz_dom::BaseDocument;
use blitz_dom::node::{ListItemLayoutPosition, Marker, TextBrush};
use parley::{Layout, PositionedLayoutItem, Run};
use std::ops::Range;

/// The run texts of every layout in `doc`, in document order.
pub(crate) fn collect(doc: &BaseDocument) -> RunTexts {
    let mut texts = RunTexts::default();
    for (_, node) in doc.tree().iter() {
        let Some(element) = node.element_data() else {
            continue;
        };
        if let Some(inline) = &element.inline_layout_data {
            record(&mut texts, &inline.text, &inline.layout);
        }
        if let Some(list) = &element.list_item_data
            && let ListItemLayoutPosition::Outside(layout) = &list.position
        {
            let marker = match &list.marker {
                Marker::Char(c) => c.to_string(),
                Marker::String(s) => s.clone(),
            };
            record(&mut texts, &marker, layout);
        }
        if let Some(input) = element.text_input_data()
            && let Some(layout) = input.editor.try_layout()
        {
            record(&mut texts, input.editor.raw_text(), layout);
        }
    }
    texts
}

/// Record every glyph run of `layout`, whose clusters index into `text`.
fn record(texts: &mut RunTexts, text: &str, layout: &Layout<TextBrush>) {
    for line in layout.lines() {
        // Every glyph the line draws, in the order its glyph runs draw them, with its
        // cluster's range: a glyph run is a stretch of one parley run's visual clusters.
        let mut clusters = line.runs().flat_map(|run| glyph_clusters(&run));
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                continue;
            };
            let glyphs: Vec<anyrender::Glyph> = glyph_run
                .positioned_glyphs()
                .map(|glyph| anyrender::Glyph {
                    id: glyph.id,
                    x: glyph.x,
                    y: glyph.y,
                })
                .collect();
            let ranges: Vec<_> = clusters.by_ref().take(glyphs.len()).collect();
            let run = glyph_run.run();
            let key = RunKey::new(run.font(), run.font_size(), glyphs);
            let run_text = RunText::from_glyphs(ranges.iter().map(|range| GlyphSource {
                cluster: range.start,
                text: text.get(range.clone()).unwrap_or(""),
            }));
            texts.insert(key, run_text);
        }
    }
}

/// Each glyph of `run`, in drawing order, as its cluster's byte range. A ligature is one glyph
/// in its first cluster and none in the clusters it swallowed (parley's ligature
/// continuations), so their text joins the glyph's: `fi` drawn as one glyph copies as `fi`.
fn glyph_clusters(run: &Run<'_, TextBrush>) -> Vec<Range<usize>> {
    let mut clusters: Vec<(Range<usize>, usize)> = Vec::new();
    let mut swallowed: Option<Range<usize>> = None;
    for cluster in run.visual_clusters() {
        let range = cluster.text_range();
        let glyphs = cluster.glyphs().count();
        if glyphs == 0 && cluster.is_ligature_continuation() {
            // Visual order: after its ligature's glyph left to right, before it right to left.
            match clusters.last_mut() {
                Some((held, _)) if !cluster.is_rtl() => *held = union(held, &range),
                _ => swallowed = Some(swallowed.map_or(range.clone(), |held| union(&held, &range))),
            }
            continue;
        }
        let range = swallowed
            .take()
            .map_or(range.clone(), |held| union(&held, &range));
        clusters.push((range, glyphs));
    }
    clusters
        .into_iter()
        .flat_map(|(range, glyphs)| std::iter::repeat_n(range, glyphs))
        .collect()
}

fn union(a: &Range<usize>, b: &Range<usize>) -> Range<usize> {
    a.start.min(b.start)..a.end.max(b.end)
}
