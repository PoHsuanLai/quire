//! design/26's lint on quire's own sheets, one sheet at a time: no stylesheet loops
//! (`Rule::InfiniteLoop`) and every component times its motion with the grammar's tokens
//! (`Rule::OffGrammarTiming`, `Profile::Details`), except the sheets listed here, each with its
//! reason. A sheet on the list that no longer offends is a failure too, so the list only shrinks.

use ds::lint::{LintConfig, Profile, Rule, stylesheet};

/// Sheets allowed to break the details rules, and why. Each is mail's (mailo pins quire by tag,
/// so these change when mailo decides, design/05 section 12 item 4), or the keyframe table
/// itself.
const ALLOWED: &[(&str, Rule, &str)] = &[
    (
        "motion",
        Rule::InfiniteLoop,
        "the keyframe table's own pulse classes: `Anim::{Dest, Breathe, Spin, Busy}` loop by \
         their recipe (mail's drop target, halo and busy word)",
    ),
    (
        "motion",
        Rule::OffGrammarTiming,
        "the keyframe table's own pulse classes, each at its recipe's token (an animated \
         emoji's awake window, the loops above, the send ring and the chip flash)",
    ),
    (
        "sync_halo",
        Rule::InfiniteLoop,
        "mail's account halo: breathes while idle and spins while syncing (design/05 section 12 \
         item 4: mailo decides)",
    ),
    (
        "sync_halo",
        Rule::OffGrammarTiming,
        "the halo's loops run at `--t-ambient` and `--t-spin`",
    ),
    (
        "send_pill",
        Rule::InfiniteLoop,
        "mail's send pill spins its ring while the outbox has no answer",
    ),
    (
        "send_pill",
        Rule::OffGrammarTiming,
        "the send pill's ring spins at `--t-spin` and drains over `--t-send-ring`",
    ),
    (
        "sidebar_item",
        Rule::InfiniteLoop,
        "mail's drop destination pulses (`dest`) while a drag hovers it",
    ),
    (
        "sidebar_item",
        Rule::OffGrammarTiming,
        "the drop destination's pulse period is `--t-float`",
    ),
];

fn sheets() -> Vec<(&'static str, String)> {
    let mut all: Vec<(&str, String)> = ds::components::CSS
        .iter()
        .chain(ds::detail::CSS)
        .map(|(name, css)| (*name, (*css).to_owned()))
        .collect();
    all.push(("motion", ds::css::motion_css::motion_css()));
    all
}

fn details() -> LintConfig {
    LintConfig {
        profile: Profile::Details,
        ..LintConfig::default()
    }
}

#[test]
fn no_sheet_loops_or_times_off_the_grammar_but_the_listed_ones() {
    let mut failures = Vec::new();
    let mut used = Vec::new();
    for (name, css) in sheets() {
        for offence in stylesheet(&css, &details()) {
            if !matches!(offence.rule, Rule::InfiniteLoop | Rule::OffGrammarTiming) {
                continue;
            }
            match ALLOWED
                .iter()
                .find(|(sheet, rule, _)| *sheet == name && *rule == offence.rule)
            {
                Some(allowed) => used.push((allowed.0, allowed.1)),
                None => failures.push(format!("{name}: {:?} {}", offence.rule, offence.text)),
            }
        }
    }
    for (sheet, rule, _) in ALLOWED {
        if !used.contains(&(*sheet, *rule)) {
            failures.push(format!(
                "{sheet}: {rule:?} is allowed but no longer offends"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_details_primitives_and_the_spinner_never_loop() {
    for (name, css) in sheets() {
        if !(name.starts_with("detail_") || name == "spinner") {
            continue;
        }
        let offences: Vec<_> = stylesheet(&css, &details())
            .into_iter()
            .filter(|offence| matches!(offence.rule, Rule::InfiniteLoop | Rule::OffGrammarTiming))
            .collect();
        assert!(
            offences.is_empty(),
            "{name}: {:?}",
            offences.iter().map(|o| &o.text).collect::<Vec<_>>()
        );
    }
}
