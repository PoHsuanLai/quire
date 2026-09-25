//! The stylesheet and the Rust timers cannot drift: every `animation` in the generated CSS
//! names keyframes an [`Anim`] plays at that `Anim`'s own duration and easing tokens, every
//! `transition` uses table tokens, and the token blocks resolve, level by level, to the values
//! the Rust table gives `settle()` (design/05-MOTION.md section 7.1).

use ds::{
    Anim, DurationToken, EasingToken, MotionLevel, ScalarToken, StaggerIndex, settle, stylesheet,
};
use std::collections::BTreeMap;
use std::time::Duration;

fn strip_comments(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start + 2..].find("*/") {
            Some(end) => rest = &rest[start + 2 + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

fn matching(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, byte) in text.bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

/// A rule outside `@keyframes`: its selector and its `(property, value)` declarations.
type Rule = (String, Vec<(String, String)>);

/// Every rule outside `@keyframes`, and the keyframe names.
fn parse(css: &str) -> (Vec<Rule>, Vec<String>) {
    let css = strip_comments(css);
    let mut rules = Vec::new();
    let mut keyframes = Vec::new();
    let mut rest = css.as_str();
    while let Some(open) = rest.find('{') {
        let selector = rest[..open].trim().to_owned();
        let Some(close) = matching(&rest[open..]) else {
            break;
        };
        let body = &rest[open + 1..open + close];
        if let Some(name) = selector.strip_prefix("@keyframes ") {
            keyframes.push(name.trim().to_owned());
        } else {
            let decls = body
                .split(';')
                .filter_map(|pair| pair.split_once(':'))
                .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
                .collect();
            rules.push((selector, decls));
        }
        rest = &rest[open + close + 1..];
    }
    (rules, keyframes)
}

/// Top-level commas only: `a, b` but not the one inside `cubic-bezier(…)` or `calc(…)`.
fn split_list(value: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0usize, 0usize);
    for (index, byte) in value.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.saturating_sub(1),
            b',' if depth == 0 => {
                parts.push(value[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(value[start..].trim());
    parts
}

#[test]
fn every_animation_is_an_anims_recipe() {
    let (rules, keyframes) = parse(stylesheet());
    let mut checked = 0;
    let mut failures = Vec::new();
    for (selector, decls) in &rules {
        for (property, value) in decls.iter().filter(|(p, _)| p == "animation") {
            for one in split_list(value).into_iter().filter(|one| *one != "none") {
                checked += 1;
                let words: Vec<&str> = one.split_whitespace().collect();
                let [name, duration, easing, ..] = words[..] else {
                    failures.push(format!("{selector} {property}: {one}"));
                    continue;
                };
                if !keyframes.iter().any(|k| k == name) {
                    failures.push(format!("{selector}: no @keyframes {name}"));
                }
                let base = name.strip_suffix("--b").unwrap_or(name);
                let fits = Anim::ALL.iter().any(|anim| {
                    let recipe = anim.recipe();
                    recipe.keyframes == base
                        && recipe.duration.var().reference() == duration
                        && recipe.easing.var().reference() == easing
                });
                if !fits {
                    failures.push(format!(
                        "{selector}: {one} is no Anim's keyframes, duration and easing"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
    // Two aliases per Anim at least; components add their own.
    assert!(
        checked >= Anim::ALL.len() * 2,
        "only {checked} animations were checked"
    );
}

#[test]
fn every_transition_uses_table_tokens() {
    let (rules, _) = parse(stylesheet());
    let durations: Vec<String> = DurationToken::ALL
        .iter()
        .map(|t| t.var().reference())
        .collect();
    let easings: Vec<String> = EasingToken::ALL
        .iter()
        .map(|t| t.var().reference())
        .collect();
    let mut checked = 0;
    let mut failures = Vec::new();
    for (selector, decls) in &rules {
        for (_, value) in decls.iter().filter(|(p, _)| p == "transition") {
            for one in split_list(value).into_iter().filter(|one| *one != "none") {
                checked += 1;
                let words: Vec<&str> = one.split_whitespace().collect();
                let ok = matches!(words[..], [_, duration, easing, ..]
                    if durations.iter().any(|d| d == duration) && easings.iter().any(|e| e == easing));
                if !ok {
                    failures.push(format!("{selector}: transition {one}"));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
    assert!(checked > 0, "no transition was checked");
}

/// The value of every custom property at `level`: the `.ds` token block, then that level's
/// block over it.
fn resolved(level: MotionLevel) -> BTreeMap<String, String> {
    let (rules, _) = parse(stylesheet());
    let mut values: BTreeMap<String, String> = rules
        .iter()
        .filter(|(selector, decls)| selector == ".ds" && decls.iter().any(|(n, _)| n == "--t-tap"))
        .flat_map(|(_, decls)| decls.iter().cloned())
        .collect();
    let selector = format!(".ds[*|data-motion={}]", level.slug());
    for (_, decls) in rules.iter().filter(|(s, _)| *s == selector) {
        values.extend(decls.iter().cloned());
    }
    values
}

#[test]
fn every_level_resolves_to_the_rust_table() {
    let mut failures = Vec::new();
    for level in MotionLevel::ALL {
        let values = resolved(level);
        let mut expect = |name: &str, want: String| {
            if values.get(name) != Some(&want) {
                failures.push(format!(
                    "{level:?} {name}: {:?}, table {want}",
                    values.get(name)
                ));
            }
        };
        for token in DurationToken::ALL {
            let ms = token.duration(level).as_millis();
            expect(token.var().as_str(), format!("{ms}ms"));
        }
        for token in EasingToken::ALL {
            expect(token.var().as_str(), token.easing(level).css());
        }
        for token in ScalarToken::ALL {
            expect(token.var().as_str(), token.value(level).css());
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn the_settle_table() {
    // design/05-MOTION.md section 7.1's worked values, index 0.
    #[rustfmt::skip]
    const CASES: &[(Anim, MotionLevel, u64)] = &[
        (Anim::Fold, MotionLevel::Standard, 454),
        (Anim::FoldHeavy, MotionLevel::Standard, 517),
        (Anim::Curl, MotionLevel::Standard, 594),
        (Anim::Heal, MotionLevel::Standard, 284),
        (Anim::Rise, MotionLevel::Standard, 284),
        (Anim::HcOut, MotionLevel::Standard, 154),
        (Anim::TabOut, MotionLevel::Standard, 284),
        (Anim::Gulp, MotionLevel::Standard, 454),
        (Anim::Floatup, MotionLevel::Standard, 934),
        (Anim::ComposeSend, MotionLevel::Standard, 654),
        (Anim::Park, MotionLevel::Standard, 454),
        (Anim::Spark, MotionLevel::Standard, 554),
        (Anim::RowIn, MotionLevel::Standard, 454),
        (Anim::Nudge, MotionLevel::Standard, 554),
        (Anim::Shake, MotionLevel::Standard, 594),
        (Anim::Fold, MotionLevel::Calm, 334),
        (Anim::Fold, MotionLevel::Extra, 594),
        (Anim::FoldHeavy, MotionLevel::Calm, 379),
        (Anim::FoldHeavy, MotionLevel::Extra, 678),
        (Anim::CrumpleHeavy, MotionLevel::Standard, 517),
        (Anim::CrumpleHeavy, MotionLevel::Calm, 379),
        (Anim::CurlHeavy, MotionLevel::Standard, 678),
        (Anim::CurlHeavy, MotionLevel::Extra, 678),
        // Wave 2 integration: the four recipe rows the overlays needed (section 5 rows 7, 26,
        // 37 and 64).
        (Anim::PaletteFade, MotionLevel::Standard, 204),
        (Anim::LinkPillIn, MotionLevel::Standard, 204),
        (Anim::BubblePop, MotionLevel::Standard, 204),
        (Anim::PeekFullIn, MotionLevel::Standard, 284),
        // mailo gaps 3: the four keyframes the catalogue had no motion for.
        (Anim::PillUp, MotionLevel::Standard, 454),
        (Anim::PillUp, MotionLevel::Calm, 334),
        (Anim::PillUp, MotionLevel::Extra, 594),
        (Anim::RingDrain, MotionLevel::Standard, 5034),
        (Anim::RingDrain, MotionLevel::Calm, 5034),
        (Anim::FadeIn, MotionLevel::Standard, 284),
        (Anim::Busy, MotionLevel::Standard, 5034),
        // sill Q80: the pane switch, both panes at `--t-move`, so one timer settles the pair.
        (Anim::PaneInR, MotionLevel::Standard, 284),
        (Anim::PaneInL, MotionLevel::Standard, 284),
        (Anim::PaneOutL, MotionLevel::Standard, 284),
        (Anim::PaneOutR, MotionLevel::Standard, 284),
        // sill Q75: the OSD's entrance at --t-quick, its exit at --t-move (neither token moves
        // with the look's level but under Reduced).
        (Anim::OsdIn, MotionLevel::Standard, 204),
        (Anim::OsdIn, MotionLevel::Extra, 204),
        (Anim::OsdOut, MotionLevel::Standard, 284),
        (Anim::OsdOut, MotionLevel::Calm, 284),
        // The level control's step mark, at --t-tap.
        (Anim::LevelTick, MotionLevel::Standard, 124),
        // sill Q90: the sheet's exit at --t-move, which only Reduced shortens.
        (Anim::SheetOut, MotionLevel::Standard, 284),
        (Anim::SheetOut, MotionLevel::Calm, 284),
        // sill Q121, Q122: the banner's exit at --t-move, which only Reduced shortens.
        (Anim::BannerOut, MotionLevel::Standard, 284),
        (Anim::BannerOut, MotionLevel::Extra, 284),
        (Anim::BannerIn, MotionLevel::Standard, 284),
        (Anim::BannerIn, MotionLevel::Calm, 284),
        // sill Q123: the center's edge panel, in and out at --t-move.
        (Anim::PanelIn, MotionLevel::Standard, 284),
        (Anim::PanelOut, MotionLevel::Standard, 284),
    ];
    for &(anim, level, ms) in CASES {
        assert_eq!(
            settle(anim, level, StaggerIndex::default()),
            Duration::from_millis(ms),
            "{anim:?} {level:?}"
        );
    }
    // Every anim settles to 94 ms under Reduced, except a hold (`DurationToken::kind`):
    // `ChipFlash` times `--t-flash`, which keeps its Standard 1200 ms so the mentioned-person
    // ring is still visible under Reduced (wave 2's timing tweak; FINDINGS.md "W1 integration"
    // left this as an open question, resolved here).
    for anim in Anim::ALL
        .into_iter()
        .filter(|anim| !matches!(anim, Anim::ChipFlash | Anim::RingDrain))
    {
        assert_eq!(
            settle(anim, MotionLevel::Reduced, StaggerIndex::default()),
            Duration::from_millis(94),
            "{anim:?} Reduced"
        );
    }
    assert_eq!(
        settle(
            Anim::ChipFlash,
            MotionLevel::Reduced,
            StaggerIndex::default()
        ),
        Duration::from_millis(1234),
        "ChipFlash Reduced (a hold, unaffected by Reduced)"
    );
    // The send ring is the undo window: a hold too (mailo gaps 3).
    assert_eq!(
        settle(
            Anim::RingDrain,
            MotionLevel::Reduced,
            StaggerIndex::default()
        ),
        Duration::from_millis(5034),
        "RingDrain Reduced (a hold, unaffected by Reduced)"
    );
}
