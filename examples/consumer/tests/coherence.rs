//! The four coherence rules (`ORCHESTRATION.md` "Coherence rules"), proven from outside the
//! quire workspace the way a real consumer's own test suite would: this crate's `Cargo.toml`
//! depends on `ds`, `ds-settings` and `ds-native` by path, exactly as `CONSUMING.md` "Adding
//! quire" tells a consumer to, and these tests are what `CONSUMING.md` "The four coherence
//! rules" tells that consumer to write.
//!
//! One test per rule, in ORCHESTRATION's order, so a failure names which rule broke:
//! 1. [`our_stylesheet_lints_clean`] — no literal colour, duration, easing, keyframe,
//!    font-family or `:root` in `consumer`'s own CSS.
//! 2. [`rendered_markup_uses_only_styled_classes`] and
//!    [`no_raw_form_control_or_svg_is_rendered`] — no raw markup, and no class nothing styles.
//! 3. (Rule 3, "a missing component is added to quire first, never patched locally", is a
//!    review discipline, not a runtime check; nothing to assert.)
//! 4. [`the_sent_badge_times_out_on_ds_motions_own_clock`] — motion is timed by `ds::motion`,
//!    never an ad-hoc sleep.

use consumer::{App, STYLE};
use ds::lint::{Exception, LintConfig, Profile, Rule, assert_clean, markup};
use ds::{Anim, MotionLevel, StaggerIndex, settle};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 360,
    scale_percent: 100,
};

/// A gap in quire itself, not this crate: `Ds`'s `Material::Window` frame layers
/// (`crates/ds/src/root/ds.rs::FrameLayers::render`) render the hidden layer as
/// `class="ds-layer back"`, a second CSS class, while `crates/ds/src/css/utilities.css` styles
/// it as `.ds-layer[*|data-layer=back]`, an attribute. The class and the selector never match,
/// so the hidden layer never gets `opacity:0` and the Space-switch cross-fade is silently
/// broken; a markup-lint test is what caught it, which is exactly why every consumer runs one
/// (reported in this wave's report, not fixed here: `root/ds.rs` is outside this crate's
/// ownership). Kept as one exception with its own reason, matching how quire's own
/// `self_lint.rs` documents its avatar exception, rather than silently narrowing the test.
const KNOWN_GAPS: &[Exception] = &[Exception {
    rule: Rule::UnstyledClass,
    selector: "div.ds-layer.back",
    reason: "Ds's own frame layers write class=\"ds-layer back\" but utilities.css styles \
             [*|data-layer=back]; a quire bug, not this app's markup (see the wave report)",
}];

/// An SSR render of [`App`]'s first frame, the input every markup-lint test shares.
fn render() -> String {
    let mut dom = dioxus::prelude::VirtualDom::new(App);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// What [`ds::lint::markup`] needs to know is styled: quire's own stylesheet, plus ours.
fn consumer_css() -> String {
    format!("{}\n{STYLE}", ds::stylesheet())
}

fn markup_config() -> LintConfig {
    LintConfig {
        exceptions: KNOWN_GAPS,
        ..LintConfig::default()
    }
}

// ---- Rule 1: no stylesheet outside quire may contain a literal colour, duration, easing, ----
// ---- keyframe, font-family or `:root` selector. -----------------------------------------

/// `STYLE` (`src/style.css`) under the strictest profile: this is the exact call
/// `CONSUMING.md` "The four coherence rules" tells a consumer to add, on its own CSS.
#[test]
fn our_stylesheet_lints_clean() {
    assert_clean(
        STYLE,
        &LintConfig {
            profile: Profile::Strict,
            ..LintConfig::default()
        },
    );
}

// ---- Rule 2: no surface writes raw markup; every class is one a rule styles. -------------

/// Every class the first frame carries is one quire's own stylesheet or `STYLE` styles. If a
/// class were misspelled, or a rule renamed without updating its CSS, this fails.
#[test]
fn rendered_markup_uses_only_styled_classes() {
    let html = render();
    let offences = markup(&html, &consumer_css(), &markup_config());
    let unstyled: Vec<_> = offences
        .iter()
        .filter(|offence| offence.rule == Rule::UnstyledClass)
        .collect();
    assert!(unstyled.is_empty(), "{unstyled:#?}\nin:\n{html}");
}

/// No hand-written `<button>`, `<input>`, `<select>`, `<textarea>` or `<svg>` anywhere in the
/// page: `Page` draws its field with [`ds::TextInput`], its action with [`ds::Button`], and its
/// menu with [`ds::Menu`], never the bare element. If `Page` ever grows a raw
/// `button { "Actions" }` in place of `ds::Button`, this is the test that fails — `markup`'s
/// [`Rule::RawMarkup`] is exactly the check ORCHESTRATION coherence rule 2 names.
#[test]
fn no_raw_form_control_or_svg_is_rendered() {
    let html = render();
    let offences = markup(&html, &consumer_css(), &markup_config());
    let raw: Vec<_> = offences
        .iter()
        .filter(|offence| offence.rule == Rule::RawMarkup)
        .collect();
    assert!(raw.is_empty(), "{raw:#?}\nin:\n{html}");
}

// ---- Rule 4: motion state is driven by quire timers, never an ad-hoc sleep. --------------

/// The "Sent" confirmation is timed by `ds::use_motion_timer(Anim::Fade)`
/// (`src/lib.rs::Page`), not a `std::thread::sleep` or a literal millisecond count in this
/// crate. Proof: it is still showing one frame short of `settle()`'s own duration, and it is
/// gone once that duration has fully elapsed on [`Harness`]'s clock — the same clock every
/// `ds-native` test in quire itself advances, never this crate's own timer.
#[test]
fn the_sent_badge_times_out_on_ds_motions_own_clock() {
    let mut harness = Harness::new(App, VIEW);
    let shown = |harness: &Harness| harness.attr(".sent-badge", "data-shown");
    assert_eq!(
        shown(&harness).as_deref(),
        Some("hidden"),
        "shown before any Send"
    );

    let send = harness
        .centre(".ds-button")
        .unwrap_or_else(|| panic!("Send is not on screen:\n{}", harness.html()));
    harness.click(send);
    assert_eq!(
        shown(&harness).as_deref(),
        Some("shown"),
        "Send did not open the badge"
    );

    let full = settle(Anim::Fade, MotionLevel::Standard, StaggerIndex::default());
    let short = full
        .checked_sub(Duration::from_millis(30))
        .unwrap_or(Duration::ZERO);
    harness.advance(short);
    assert_eq!(
        shown(&harness).as_deref(),
        Some("shown"),
        "hid before settle() finished: advanced {short:?} of {full:?}"
    );
    harness.advance(Duration::from_millis(60));
    assert_eq!(
        shown(&harness).as_deref(),
        Some("hidden"),
        "did not hide once settle() finished"
    );
}

// ---- Bonus: the anchored `Menu` opens and closes under `Harness`. ------------------------

/// Not one of the four coherence rules, but worth pinning: `Page`'s "More" button anchors its
/// `Menu` through `ds::use_rect` (`onmounted` + a rect read one frame later), and driving that
/// through `ds_native::Harness` used to panic with `RefCell already borrowed`
/// (`CONSUMING.md` §9) until wave 2 integration's `ds::HostMeasure` fix landed. This is the
/// regression test for that fix, from this crate's own (unrelated) reason to open a menu.
#[test]
fn the_menu_opens_and_closes_under_harness() {
    let mut harness = Harness::new(App, VIEW);
    let more = harness
        .centre(".ds-button[*|data-variant=secondary]")
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
    assert_eq!(
        harness.count(".ds-menu-item"),
        0,
        "the menu did not close after picking an entry"
    );
}
