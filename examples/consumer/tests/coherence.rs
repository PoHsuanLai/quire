//! The four coherence rules (`CONSUMING.md` section 5), proven from outside the quire workspace
//! the way a real consumer's own test suite would: this crate depends on `ds`, `ds-settings` and
//! `ds-blitz` by path, and `ds-lint` and `ds-harness` as dev-dependencies, exactly as
//! `CONSUMING.md` tells a consumer to.
//!
//! One test per rule, so a failure names which rule broke:
//! 1. [`our_stylesheet_lints_clean`]: no literal design value in `consumer`'s own CSS.
//! 2. [`rendered_markup_uses_only_styled_classes`] and
//!    [`no_raw_form_control_or_svg_is_rendered`]: no raw markup, no class nothing styles.
//! 3. Rule 3 (a missing component is added to quire first) is a review discipline, not a check.
//! 4. [`the_sent_badge_times_out_on_ds_motions_own_clock`]: motion is timed by `ds`'s timers.

use consumer::{App, STYLE};
use ds::{Anim, Appearance, SpaceLook, SystemPrefs, resolve, settle};
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};
use ds_lint::{LintConfig, Profile, Rule, assert_clean, markup};

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

/// An SSR render of [`App`]'s first frame, the input every markup-lint test shares.
fn render() -> String {
    let mut dom = dioxus::prelude::VirtualDom::new(App);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// What `markup` needs to know is styled: quire's own stylesheet, plus ours.
fn consumer_css() -> String {
    format!("{}\n{STYLE}", ds::stylesheet())
}

/// A harness on the virtual clock, so a timed assertion holds exactly under any load.
fn harness() -> Harness {
    Harness::with_config(App, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

/// Rule 1: `STYLE` under the strictest profile, the call `CONSUMING.md` section 5 shows.
#[test]
fn our_stylesheet_lints_clean() {
    assert_clean(
        STYLE,
        &LintConfig {
            profile: Profile::Strict,
            ..LintConfig::new(&ds::kits())
        },
    );
}

/// Rule 2: every class the first frame carries is one quire's stylesheet or `STYLE` styles.
#[test]
fn rendered_markup_uses_only_styled_classes() {
    let html = render();
    let offences = markup(&html, &consumer_css(), &LintConfig::new(&ds::kits()));
    let unstyled: Vec<_> = offences
        .iter()
        .filter(|offence| offence.rule == Rule::UnstyledClass)
        .collect();
    assert!(unstyled.is_empty(), "{unstyled:#?}\nin:\n{html}");
}

/// Rule 2: no hand-written `<button>`, `<input>`, `<select>`, `<textarea>` or `<svg>`.
#[test]
fn no_raw_form_control_or_svg_is_rendered() {
    let html = render();
    let offences = markup(&html, &consumer_css(), &LintConfig::new(&ds::kits()));
    let raw: Vec<_> = offences
        .iter()
        .filter(|offence| offence.rule == Rule::RawMarkup)
        .collect();
    assert!(raw.is_empty(), "{raw:#?}\nin:\n{html}");
}

/// Rule 4: the "Sent" badge is timed by `use_motion_timer(Anim::Fade)`, so it is still up just
/// short of `settle` and gone just past it, on the harness's virtual clock.
#[test]
fn the_sent_badge_times_out_on_ds_motions_own_clock() {
    let mut harness = harness();
    let shown = |harness: &Harness| harness.attr(".sent-badge", "data-shown");
    assert_eq!(shown(&harness).as_deref(), Some("hidden"));

    let send = harness
        .centre(".ds-button")
        .unwrap_or_else(|| panic!("Send is not on screen:\n{}", harness.html()));
    harness.click(send);
    assert_eq!(
        shown(&harness).as_deref(),
        Some("shown"),
        "Send did not show the badge"
    );

    let resolved = resolve(
        Appearance::default(),
        SpaceLook::default().theme,
        SystemPrefs::default(),
    );
    let hold = settle(Anim::Fade, resolved.motion);
    harness.advance(hold - std::time::Duration::from_millis(10));
    assert_eq!(
        shown(&harness).as_deref(),
        Some("shown"),
        "hid before settle"
    );
    harness.advance(std::time::Duration::from_millis(20));
    assert_eq!(
        shown(&harness).as_deref(),
        Some("hidden"),
        "did not hide once settle finished"
    );
}

/// Not one of the four rules: the "More" button anchors a `Menu` through its own element, and
/// the field gets the keyboard back when the menu closes.
#[test]
fn the_menu_opens_and_closes_under_harness() {
    let mut harness = harness();
    let more = harness
        .centre(".ds-button:not([*|data-answers])")
        .unwrap_or_else(|| panic!("More is not on screen:\n{}", harness.html()));
    harness.click(more);
    assert!(
        harness.count(".ds-menu-item") > 0,
        "the menu did not open:\n{}",
        harness.html()
    );

    let item = harness
        .centre(".ds-menu-item")
        .unwrap_or_else(|| panic!("no menu item on screen:\n{}", harness.html()));
    harness.click(item);
    harness.advance(
        settle(
            Anim::MenuOut,
            resolve(
                Appearance::default(),
                SpaceLook::default().theme,
                SystemPrefs::default(),
            )
            .motion,
        ) * 4,
    );
    assert_eq!(
        harness.count(".ds-menu-item"),
        0,
        "the menu did not close after a pick"
    );
}
