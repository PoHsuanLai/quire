//! The one call a consumer's test makes.

use std::collections::BTreeMap;

use super::rule::{LintConfig, Rule};
use super::stylesheet::every_offence;

/// Panic, listing every offence with its line and column, unless `css` is clean once
/// `config.exceptions` are applied. Either way it prints what each exception suppressed (to
/// stderr on success, in the panic otherwise), so an exception that suppresses nothing is seen.
pub fn assert_clean(css: &str, config: &LintConfig) {
    let (offences, suppressed) = config.partition(every_offence(css, config));
    if offences.is_empty() {
        eprint!("{}", suppressed_counts(config, &suppressed));
        return;
    }
    let mut by_rule: BTreeMap<Rule, Vec<_>> = BTreeMap::new();
    for offence in &offences {
        by_rule.entry(offence.rule).or_default().push(offence);
    }
    let mut message = format!("quire lint found {} offence(s):\n", offences.len());
    for (rule, group) in &by_rule {
        for offence in group {
            message.push_str(&format!(
                "  {rule:?} {}:{}: {}\n",
                offence.line, offence.column, offence.text
            ));
        }
    }
    message.push_str(&suppressed_counts(config, &suppressed));
    panic!("{message}");
}

/// One line per exception: how many offences it suppressed.
pub(super) fn suppressed_counts(config: &LintConfig, suppressed: &[super::Offence]) -> String {
    if config.exceptions.is_empty() {
        return String::new();
    }
    let mut text = format!(
        "{} offence(s) suppressed by exceptions:\n",
        suppressed.len()
    );
    for exception in config.exceptions {
        let count = suppressed
            .iter()
            .filter(|offence| exception.covers(offence))
            .count();
        text.push_str(&format!(
            "  {count} x {:?} on {} ({})\n",
            exception.rule, exception.selector, exception.reason
        ));
    }
    text
}
