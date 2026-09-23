//! quire's own generated stylesheet, linted against itself under the strictest profile
//! (ORCHESTRATION coherence rule 1: every downstream crate runs this same check on its own
//! CSS, and quire's own output has to pass it too).

/// Ignored: `ds::stylesheet()` is still `todo!()` in this worktree (`w1-tokens` has not merged
/// its bodies yet, and `crates/ds/src/css/*.css` are still the freeze's empty placeholders), so
/// calling it here panics on the token wave's `todo!()`, not on anything this crate's lint got
/// wrong. Once `w1-tokens` lands, un-ignore this and, per the brief, exclude the token/material/
/// icon-generated sections first: those sections *are* the source of the design system's own
/// colours, `@keyframes` and `.ds[data-*]` selectors, which is exactly what this lint bans a
/// consumer from writing, so linting them verbatim would fail on the lint doing its job rather
/// than on a real defect.
#[test]
#[ignore = "needs w1-tokens merged"]
fn self_lint() {
    let config = ds::lint::LintConfig {
        profile: ds::lint::Profile::Strict,
        own_vars: Vec::new(),
    };
    ds::lint::assert_clean(ds::stylesheet(), &config);
}
