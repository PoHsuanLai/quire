//! The one call a consumer's test makes.

use std::collections::BTreeMap;

use super::rule::{LintConfig, Rule};
use super::stylesheet::stylesheet;

/// Panic, listing every offence with its line and column, unless `css` is clean.
pub fn assert_clean(css: &str, config: &LintConfig) {
    let offences = stylesheet(css, config);
    if offences.is_empty() {
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
    panic!("{message}");
}
