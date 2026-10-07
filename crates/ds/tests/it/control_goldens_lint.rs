//! Every control golden, rendered markup, linted against the components' stylesheet: only classes
//! it styles, no hand-written SVG or form control, and no literal paint outside the custom
//! properties a component computes. The stylesheet itself is linted, whole, in `ds-shell`.

use crate::support::golden;

use ds_lint::{LintConfig, markup};

/// The consumer classes the mail-app goldens put on a button through `extra_class`, and
/// the edit surface 2 golden on the surface (mailo's `.c-body`, positioned so its text stacks
/// over a selection layer): a consumer's own sheet styles them, as it would in the app, so they
/// are in scope here and nowhere in quire's sheet.
const CONSUMER_CSS: &str = ".row-reveal{opacity:0}\n.quiet-until-hover{opacity:0}\n.fold-more{opacity:0}\n\
     .c-body{position:relative}";

/// Coherence rule 2 on quire's own output: every control golden, rendered markup, uses only
/// classes the stylesheet styles, no hand-written SVG or form control, and no literal paint
/// outside the custom properties a component computes (the avatar's `--av-bg`, O-7), which
/// the markup lint allows on a `ds-*` element without an exception.
#[test]
fn every_control_golden_lints_clean() {
    let config = LintConfig::new(&ds::kits());
    let css = format!("{}\n{CONSUMER_CSS}", ds::stylesheet());
    let goldens = golden::all_in("controls");
    assert!(goldens.len() > 60, "only {} goldens", goldens.len());
    let failures: Vec<String> = goldens
        .iter()
        .flat_map(|(name, html)| {
            markup(html, &css, &config)
                .into_iter()
                .map(move |offence| format!("{name}: {:?} {}", offence.rule, offence.text))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
