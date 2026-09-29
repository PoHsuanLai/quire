//! Linting a consumer stylesheet.

use std::collections::HashSet;

use super::declaration;
use super::hig;
use super::registry;
use super::rule::{LintConfig, Offence};
use super::selector;
use super::severity;
use super::tokenize;
use super::walk;

/// Every error in `css` under `config` that none of `config.exceptions` covers. Warnings
/// ([`super::Severity::Warning`]) are [`warnings`]'.
///
/// `config.own_vars` names custom properties (`--` included) the consumer declares itself,
/// beyond the design system's own token table; a property the stylesheet declares on some
/// selector (`.foo { --local: 4px; ... }`) is always allowed too, whether or not it is listed
/// there, since the declaration is right there to review.
pub fn stylesheet(css: &str, config: &LintConfig) -> Vec<Offence> {
    severity::split(config.partition(every_offence(css, config)).0).0
}

/// Every warning in `css` under `config` that none of `config.exceptions` covers: the rules
/// that report without failing (the HIG guardrails, design/27 section 7).
pub fn warnings(css: &str, config: &LintConfig) -> Vec<Offence> {
    severity::split(config.partition(every_offence(css, config)).0).1
}

/// Every offence in `css`, errors and warnings, exceptions not applied: what
/// [`super::assert_clean`] partitions.
pub(super) fn every_offence(css: &str, config: &LintConfig) -> Vec<Offence> {
    // Whitespace and comment tokens stay in: [`walk`] needs them to tell `.a.b` apart from
    // `.a .b`, and [`super::text::render`] needs them to reproduce a selector's own text.
    let tokens = tokenize::tokens(css);
    let (rules, mut offences) = walk::walk(&tokens);

    for rule in &rules {
        offences.extend(selector::offences(&rule.selector_tokens));
    }

    let mut known_vars: HashSet<String> = registry::declared_vars().clone();
    known_vars.extend(config.own_vars.iter().cloned());
    for rule in &rules {
        for decl in &rule.declarations {
            if decl.property.text.starts_with("--") {
                known_vars.insert(decl.property.text.clone());
            }
        }
    }

    for rule in &rules {
        let targets_svg = selector::targets_svg_children(&rule.selector_tokens);
        for decl in &rule.declarations {
            offences.extend(declaration::offences(
                &rule.selector,
                targets_svg,
                decl,
                &known_vars,
                config.profile,
            ));
        }
    }

    offences.extend(rules.iter().flat_map(hig::offences));

    offences.sort_by_key(|a| (a.line, a.column));
    offences
}
