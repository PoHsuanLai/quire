//! The two closed vocabularies the linter checks a stylesheet against, derived from the token
//! and motion tables themselves: every custom property the tables emit, and every keyframes
//! name an [`Anim`] plays. Nothing here is a hand list, so a token added to a table is known to
//! the lint the moment it exists (the tests below check it against the generated stylesheet).

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::appearance::{Accent, Scheme};
use crate::components::StatusMetrics;
use crate::css::accents_css::swatch_var;
use crate::css::materials_css::{MATERIAL_VARS, TINT_ALPHA};
use crate::css::shape_css::{SHAPE_VARS, SQUIRCLE_VARS};
use crate::icon::family::{PLATE_GLYPH, PLATE_INSET};
use crate::material::stack::STACK_INPUTS;
use crate::motion::Anim;
use crate::space::{CardAccent, FrameVars, SpaceLook};
use crate::tokens::dock::DOCK_TOKENS;
use crate::tokens::shell::SHELL_TOKENS;
use crate::tokens::{
    ColourToken, DelayToken, DurationToken, EasingToken, Family, FontSize, HueMember, LabelHue,
    OpacityToken, PersonSwatch, PixelToken, Radius, ScalarToken, Shadow, SpacingToken, VarName,
    ZLayer,
};

/// Every custom property the design system declares, `--` included: the token table's
/// (colours, label hues, durations, the CSS delays, easings, scalars, radii, spacing steps,
/// shadows, faces, sizes, layers), the accent swatches, the frame's `--f-*` a root writes inline, and the
/// material's `--m-*` with the tint-alpha key, and the bar status item's two properties a
/// consumer sets from its settings.
pub fn declared_vars() -> &'static HashSet<String> {
    static VARS: LazyLock<HashSet<String>> = LazyLock::new(collect);
    &VARS
}

fn collect() -> HashSet<String> {
    let tables: Vec<VarName> = ColourToken::ALL
        .map(ColourToken::var)
        .into_iter()
        .chain(DurationToken::ALL.map(DurationToken::var))
        .chain(DelayToken::ALL.into_iter().filter_map(DelayToken::var))
        .chain(EasingToken::ALL.map(EasingToken::var))
        .chain(ScalarToken::ALL.map(ScalarToken::var))
        .chain(Radius::ALL.map(Radius::var))
        .chain(SpacingToken::ALL.map(SpacingToken::var))
        .chain(Shadow::ALL.map(Shadow::var))
        .chain(FontSize::ALL.map(FontSize::var))
        .chain(ZLayer::ALL.map(ZLayer::var))
        .chain(OpacityToken::ALL.map(OpacityToken::var))
        .chain([Family::Display, Family::Ui, Family::Data].map(Family::var))
        .chain(MATERIAL_VARS)
        .chain([TINT_ALPHA])
        .chain([StatusMetrics::BOX_VAR, StatusMetrics::GLYPH_VAR])
        .chain(tuned_vars())
        .chain(STACK_INPUTS)
        .chain(SQUIRCLE_VARS)
        .chain(SHAPE_VARS)
        .collect();
    let hues = LabelHue::ALL
        .into_iter()
        .flat_map(|hue| HueMember::ALL.map(|member| hue.var(member)));
    let swatches = Accent::ALL.into_iter().map(swatch_var);
    let people = PersonSwatch::ALL.into_iter().map(PersonSwatch::var);
    // Ask the frame for its names rather than restating them; a Space that lends its hue
    // writes the most.
    let look = SpaceLook {
        card_accent: CardAccent::SpaceHue,
        ..SpaceLook::default()
    };
    let frame = FrameVars::of(&look, Scheme::Light)
        .names()
        .into_iter()
        .map(str::to_owned);
    tables
        .into_iter()
        .map(|name| name.as_str().to_owned())
        .chain(hues)
        .chain(swatches)
        .chain(people)
        .chain(frame)
        .collect()
}

/// The tuned tokens (the shell type scale, the dock's geometry, the plate's shares, the pixel
/// tokens) and the inputs written for them.
fn tuned_vars() -> impl Iterator<Item = VarName> {
    SHELL_TOKENS
        .into_iter()
        .chain(DOCK_TOKENS)
        .chain([PLATE_GLYPH, PLATE_INSET])
        .chain(PixelToken::ALL.map(PixelToken::tuned))
        .flat_map(|token| [token.token, token.input])
}

/// Whether `name` (an `animation-name` value, its `X--b` restart alias included) is a
/// keyframes name some [`Anim`] plays (design/05-MOTION.md section 9 rule 2, spike S5).
pub fn is_known_anim(name: &str) -> bool {
    if name.eq_ignore_ascii_case("none") {
        return true;
    }
    let base = name.strip_suffix("--b").unwrap_or(name);
    Anim::ALL
        .iter()
        .any(|anim| anim.recipe().keyframes.eq_ignore_ascii_case(base))
}

#[cfg(test)]
mod tests {
    use super::{declared_vars, is_known_anim};
    use crate::lint::{tokenize, walk};

    /// Every custom property the generated stylesheet declares on a `.ds` root block (the token,
    /// accent and material sections), asked of the stylesheet itself.
    fn root_declarations() -> Vec<String> {
        let (rules, _) = walk::walk(&tokenize::tokens(crate::stylesheet()));
        rules
            .into_iter()
            .filter(|rule| {
                rule.selector
                    .strip_prefix(".ds")
                    .is_some_and(|rest| rest.is_empty() || rest.starts_with('['))
            })
            .flat_map(|rule| rule.declarations)
            .map(|decl| decl.property.text)
            .filter(|name| name.starts_with("--"))
            .collect()
    }

    #[test]
    fn every_root_declaration_is_registered() {
        let missing: Vec<String> = root_declarations()
            .into_iter()
            .filter(|name| !declared_vars().contains(name))
            .collect();
        assert!(
            missing.is_empty(),
            "declared but not registered: {missing:?}"
        );
    }

    #[test]
    fn the_tables_reach_the_registry() {
        const CASES: &[&str] = &[
            "--paper",
            "--handle-ring",
            "--t-flash",
            "--d-heal",
            "--d-fly",
            "--c-violet-soft",
            "--swatch-postmark",
            "--f-grad",
            "--m-tint",
            "--m-tint-alpha",
            "--font-data",
            "--s-1",
            "--s-36",
        ];
        for name in CASES {
            assert!(declared_vars().contains(*name), "{name}");
        }
        assert!(!declared_vars().contains("--d-heal-step"));
    }

    #[test]
    fn an_anim_and_its_alias_are_known() {
        const CASES: &[(&str, bool)] = &[
            ("gulp", true),
            ("gulp--b", true),
            ("chip-flash", true),
            ("none", true),
            ("wobble", false),
            ("gulp--c", false),
        ];
        for (name, want) in CASES {
            assert_eq!(is_known_anim(name), *want, "{name}");
        }
    }
}
