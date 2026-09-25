//! The keyframes with their `X--b` aliases, the `a-<anim>` pulse classes and their
//! `data-pulse` alias swap (design/05-MOTION.md section 9 rule 2).
//!
//! `motion.css` is the canonical text. Every `@keyframes X` in it is written again as
//! `@keyframes X--b`, identical; the pulse class plays `X` under `data-pulse=a` and `X--b`
//! under `data-pulse=b`, so flipping the attribute changes the animation name and Stylo starts
//! it over (spike S5). Each class's declaration is the [`Anim`]'s recipe, written from the same
//! table [`crate::settle`] times.

use super::MOTION;
use super::emit::{attr_selector, property, rule};
use crate::appearance::MotionLevel;
use crate::motion::{Anim, Fill, Iteration, Recipe};

/// The suffix of every keyframe's second name.
pub(crate) const ALIAS: &str = "--b";

/// Keyframes, aliases and pulse classes.
pub fn motion_css() -> String {
    let mut css = String::from(MOTION.trim_end());
    css.push('\n');
    for (name, body) in keyframes(MOTION) {
        css.push_str(&format!("@keyframes {name}{ALIAS}{body}\n"));
    }
    for anim in Anim::ALL {
        for (phase, name) in [("a", String::new()), ("b", ALIAS.to_owned())] {
            let selector = format!(".{}{}", anim.class(), attr_selector("data-pulse", phase));
            let value = animation(anim.recipe(), &name);
            css.push_str(&rule(&selector, &[property("animation", &value)]));
        }
    }
    // Under Reduced every animation runs once (section 3.2): a loop would never settle.
    let reduced = format!(
        ".ds{}",
        attr_selector("data-motion", MotionLevel::Reduced.slug())
    );
    css.push_str(&rule(
        &format!("{reduced},{reduced} *"),
        &[property("animation-iteration-count", "1")],
    ));
    css
}

/// The `animation` shorthand for `recipe`, its keyframes name followed by `suffix`:
/// `gulp var(--t-big) var(--e-spring)`, then the iteration and the fill when they are not the
/// defaults.
pub(crate) fn animation(recipe: Recipe, suffix: &str) -> String {
    let mut value = format!(
        "{}{suffix} {} {}",
        recipe.keyframes,
        recipe.duration.var().reference(),
        recipe.easing.var().reference()
    );
    match recipe.iteration {
        Iteration::Once => {}
        Iteration::Infinite => value.push_str(" infinite"),
        Iteration::InfiniteAlternate => value.push_str(" infinite alternate"),
    }
    match recipe.fill {
        Fill::None => {}
        Fill::Forwards => value.push_str(" forwards"),
        Fill::Backwards => value.push_str(" backwards"),
    }
    value
}

/// Every `@keyframes` in `css`, as its name and its body from `{` to the matching `}`.
///
/// `css` is our own `motion.css`: comments hold no braces and no `@keyframes` inside them.
pub(crate) fn keyframes(css: &str) -> Vec<(&str, &str)> {
    const AT: &str = "@keyframes ";
    let mut found = Vec::new();
    let mut rest = css;
    while let Some(start) = rest.find(AT) {
        let after = &rest[start + AT.len()..];
        let Some(open) = after.find('{') else { break };
        let name = after[..open].trim();
        let Some(close) = matching(&after[open..]) else {
            break;
        };
        found.push((name, &after[open..open + close + 1]));
        rest = &after[open + close + 1..];
    }
    found
}

/// The index of the `}` that closes the `{` at the start of `text`.
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

#[cfg(test)]
mod tests {
    use super::{animation, keyframes, motion_css};
    use crate::css::MOTION;
    use crate::motion::Anim;

    #[test]
    fn every_anim_has_its_keyframes_and_every_keyframe_an_anim() {
        let names: Vec<&str> = keyframes(MOTION)
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        for anim in Anim::ALL {
            let wanted = anim.recipe().keyframes;
            assert!(names.contains(&wanted), "{anim:?}: no @keyframes {wanted}");
        }
        for name in &names {
            assert!(
                Anim::ALL
                    .iter()
                    .any(|anim| anim.recipe().keyframes == *name),
                "@keyframes {name} has no Anim"
            );
        }
        assert_eq!(names.len(), 55);
    }

    #[test]
    fn every_contact_keyframe_follows_the_motion_level() {
        // Calm and Reduced set `--overshoot` to 1, which flattens a keyframe only if it reads
        // it (mailo gaps 3): the springs that answer a touch or a change all do. The painted
        // shapes at each level are measured in ds-native's `contact_motion` test.
        let bodies = keyframes(MOTION);
        for anim in [
            Anim::PopIn,
            Anim::RowIn,
            Anim::ComposeRise,
            Anim::ChipIn,
            Anim::CmdkIn,
            Anim::Gulp,
            Anim::Bump,
            Anim::SealPop,
        ] {
            let name = anim.recipe().keyframes;
            let body = bodies
                .iter()
                .find(|(found, _)| *found == name)
                .map(|(_, body)| *body)
                .unwrap_or_default();
            assert!(
                body.contains("var(--overshoot)"),
                "{anim:?}: @keyframes {name} ignores the motion level"
            );
        }
    }

    #[test]
    fn the_osd_pair_follows_the_cards_anchor() {
        // One pair for both positions (sill Q75): each keyframe moves by `--osd-dy`, which the
        // card declares per position, so the top right drops in and the bottom centre rises.
        let bodies = keyframes(MOTION);
        for anim in [Anim::OsdIn, Anim::OsdOut] {
            let name = anim.recipe().keyframes;
            let body = bodies
                .iter()
                .find(|(found, _)| *found == name)
                .map(|(_, body)| *body)
                .unwrap_or_default();
            assert!(
                body.contains("var(--osd-dy"),
                "{anim:?}: @keyframes {name} is fixed"
            );
        }
    }

    #[test]
    fn the_alias_is_the_same_text_under_the_second_name() {
        let css = motion_css();
        for (name, body) in keyframes(MOTION) {
            let alias = format!("@keyframes {name}--b{body}");
            assert!(css.contains(&alias), "{name}: alias missing");
        }
    }

    #[test]
    fn the_shorthand_is_the_recipe() {
        const CASES: &[(Anim, &str, &str)] = &[
            (Anim::Gulp, "", "gulp var(--t-big) var(--e-spring)"),
            (
                Anim::FoldHeavy,
                "--b",
                "fold--b var(--t-big-heavy) var(--e-exit) forwards",
            ),
            (Anim::Rise, "", "rise var(--t-move) var(--e-out) backwards"),
            (
                Anim::Dest,
                "",
                "dest var(--t-float) var(--e-out) infinite alternate",
            ),
            (
                Anim::Spin,
                "",
                "spin var(--t-spin) var(--e-linear) infinite",
            ),
        ];
        for &(anim, suffix, want) in CASES {
            assert_eq!(animation(anim.recipe(), suffix), want, "{anim:?}");
        }
    }
}
