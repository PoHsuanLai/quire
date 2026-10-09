//! What every rendered specimen must be, shared by the suites that keep specimens and swept by
//! each crate's `golden_hygiene`: its markup lints clean against the stylesheet, and every `ds-`
//! class in it is one the stylesheet styles.

use ds_lint::{LintConfig, markup};

/// Whether `sheet` has a rule that selects `.class`.
fn styled(sheet: &str, class: &str) -> bool {
    let needle = format!(".{class}");
    sheet.match_indices(&needle).any(|(at, _)| {
        !sheet[at + needle.len()..]
            .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    })
}

/// The `ds-` classes `html` writes that no rule of `sheet` selects, as failures of `name`.
pub fn unstyled(name: &str, html: &str, sheet: &str) -> Vec<String> {
    html.split("class=\"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .flat_map(str::split_whitespace)
        .filter(|class| class.starts_with("ds-") && !styled(sheet, class))
        .map(|class| format!("{name}: .{class} is not styled"))
        .collect()
}

/// The lint offences of `html` under `config`, as failures of `name`.
pub fn offences(name: &str, html: &str, sheet: &str, config: &LintConfig) -> Vec<String> {
    markup(html, sheet, config)
        .into_iter()
        .map(|offence| format!("{name}: {:?} {}", offence.rule, offence.text))
        .collect()
}

/// The lint offences of `html` under `config`, and its unstyled classes, as failures of `name`.
pub fn failures(name: &str, html: &str, sheet: &str, config: &LintConfig) -> Vec<String> {
    let mut found = offences(name, html, sheet, config);
    found.extend(unstyled(name, html, sheet));
    found
}
