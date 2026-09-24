//! The one call a consumer's test makes.

use std::collections::BTreeMap;

use super::Offence;
use super::rule::{Exception, LintConfig, Rule, Stale};
use super::stylesheet::every_offence;

/// Panic, listing every offence with its line and column, unless `css` is clean once
/// `config.exceptions` are applied and every exception suppressed at least one offence. An
/// exception that suppressed nothing is stale, and a stale exception is a failure unless
/// `config.stale` is [`Stale::Report`], which prints it and passes. Either way it prints what
/// each exception suppressed (to stderr on success, in the panic otherwise).
pub fn assert_clean(css: &str, config: &LintConfig) {
    let (offences, suppressed) = config.partition(every_offence(css, config));
    let stale = stale(config, &suppressed);
    let counts = suppressed_counts(config, &suppressed);
    match (offences.is_empty(), stale.is_empty(), config.stale) {
        (true, true, _) | (true, false, Stale::Report) => {
            eprint!("{counts}{}", stale_text(&stale));
        }
        (true, false, Stale::Fail) => panic!("{}{counts}", stale_text(&stale)),
        (false, _, _) => panic!("{}{counts}{}", offence_text(&offences), stale_text(&stale)),
    }
}

/// The exceptions of `config` that suppressed none of `suppressed`.
fn stale<'a>(config: &'a LintConfig, suppressed: &[Offence]) -> Vec<&'a Exception> {
    config
        .exceptions
        .iter()
        .filter(|exception| !suppressed.iter().any(|offence| exception.covers(offence)))
        .collect()
}

/// The offences, grouped by rule, each with its line and column.
fn offence_text(offences: &[Offence]) -> String {
    let mut by_rule: BTreeMap<Rule, Vec<&Offence>> = BTreeMap::new();
    for offence in offences {
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
    message
}

/// One line per stale exception, or nothing.
fn stale_text(stale: &[&Exception]) -> String {
    if stale.is_empty() {
        return String::new();
    }
    let mut text = format!(
        "quire lint found {} stale exception(s), which suppressed nothing:\n",
        stale.len()
    );
    for exception in stale {
        text.push_str(&format!(
            "  {:?} on {} ({})\n",
            exception.rule, exception.selector, exception.reason
        ));
    }
    text
}

/// One line per exception: how many offences it suppressed.
fn suppressed_counts(config: &LintConfig, suppressed: &[Offence]) -> String {
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
