//! A Space's colours handed to one of quire's own dots as custom properties, so the stylesheet
//! paints them (mailo gaps 3). Writing `background:linear-gradient(…#hex…)` inline was a literal
//! colour past the stylesheet, which quire's own markup lint flags (`Rule::HexColour`); a
//! custom property on a `ds-*` element is how quire hands a per-instance value to its sheet,
//! as `FrameVars` does for `--f-*`.
//!
//! The stops are `--dot-c1`, `--dot-c2`, `--dot-c3`, and `data-stops` says how many there are,
//! so the sheet draws the same `135deg` gradient [`super::gradient`] writes: one colour flat,
//! two at 0 % and 100 %, three at 0 %, 50 % and 100 %.

use crate::tokens::VarName;

/// The custom properties a dot's paint writes, in order.
pub(crate) const DOT_VARS: [VarName; 3] = [
    VarName("--dot-c1"),
    VarName("--dot-c2"),
    VarName("--dot-c3"),
];

/// How many stops a dot paints, and which colours.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Stops {
    One(String),
    Two(String, String),
    Three(String, String, String),
}

/// One dot's colours, ready for its `style` and `data-stops` attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DotPaint(Stops);

impl DotPaint {
    /// The gradient over `stops`, one colour per Space dot. A Space holds at most three dots
    /// (the editor's limit); a longer list, which only a hand-edited file could hold, keeps its
    /// first, middle and last stop.
    pub(crate) fn gradient(stops: &[String]) -> Self {
        let stop = |at: usize| stops.get(at).cloned().unwrap_or_default();
        DotPaint(match stops.len() {
            0 | 1 => Stops::One(stop(0)),
            2 => Stops::Two(stop(0), stop(1)),
            n => Stops::Three(stop(0), stop(n / 2), stop(n - 1)),
        })
    }

    /// One flat colour.
    pub(crate) fn solid(colour: &str) -> Self {
        DotPaint(Stops::One(colour.to_owned()))
    }

    /// `data-stops`: `1`, `2` or `3`.
    pub(crate) fn count(&self) -> &'static str {
        match self.0 {
            Stops::One(_) => "1",
            Stops::Two(..) => "2",
            Stops::Three(..) => "3",
        }
    }

    /// The custom properties, `--dot-c1:#…;…`, one per stop.
    pub(crate) fn style_attr(&self) -> String {
        let colours: Vec<&str> = match &self.0 {
            Stops::One(one) => vec![one],
            Stops::Two(first, last) => vec![first, last],
            Stops::Three(first, middle, last) => vec![first, middle, last],
        };
        DOT_VARS
            .iter()
            .zip(colours)
            .map(|(var, colour)| format!("{}:{colour};", var.as_str()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::DotPaint;

    fn owned(stops: &[&str]) -> Vec<String> {
        stops.iter().map(|stop| (*stop).to_owned()).collect()
    }

    #[test]
    fn each_stop_count_writes_its_properties() {
        const CASES: &[(&[&str], &str, &str)] = &[
            (&["#111111"], "1", "--dot-c1:#111111;"),
            (
                &["#111111", "#222222"],
                "2",
                "--dot-c1:#111111;--dot-c2:#222222;",
            ),
            (
                &["#111111", "#222222", "#333333"],
                "3",
                "--dot-c1:#111111;--dot-c2:#222222;--dot-c3:#333333;",
            ),
            (
                &["#111111", "#222222", "#333333", "#444444", "#555555"],
                "3",
                "--dot-c1:#111111;--dot-c2:#333333;--dot-c3:#555555;",
            ),
            (&[], "1", "--dot-c1:;"),
        ];
        for (stops, count, style) in CASES {
            let paint = DotPaint::gradient(&owned(stops));
            assert_eq!(paint.count(), *count, "{stops:?}");
            assert_eq!(paint.style_attr(), *style, "{stops:?}");
        }
        assert_eq!(DotPaint::solid("#abcdef").style_attr(), "--dot-c1:#abcdef;");
    }
}
