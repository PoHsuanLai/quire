//! Two closed vocabularies the linter checks a stylesheet against, mirrored from the frozen
//! token and motion enums' own doc comments rather than called through their `var()`/`class()`
//! methods.
//!
//! Wave 1's tokens and motion crates fill in those methods in their own worktrees; at lint time
//! `ColourToken::var()` and friends may still be `todo!()`. Calling them here would make every
//! lint test that exercises a declared token panic on whichever agent merges last, which is
//! exactly backwards for a coherence check meant to run standalone. The custom-property names
//! and keyframe names are part of the frozen enums' documented shape already (each variant's
//! doc comment names its `--*` property or its CSS keyframe name), so this list is a transcript
//! of that frozen text, not a guess. `tests/self_lint.rs` (linting `ds::stylesheet()` against
//! itself, `#[ignore]`d until `w1-tokens` merges) is the check that this transcript has not
//! drifted from the real tables once tokens and motion are in.

/// Every custom property the design system's own token tables declare, `--` included.
///
/// Source: the doc comment on every variant of `ColourToken`, `DurationToken`, `EasingToken`,
/// `Radius`, `Shadow`, `Family`, `FontSize`, `ZLayer`, `ScalarToken`, `DelayToken::Fly`,
/// `LabelHue` x `HueMember`, `FrameVars`'s fields, and `MaterialRecipe`'s fields.
pub const DECLARED_VARS: &[&str] = &[
    // ColourToken (crate::tokens::colour)
    "--paper",
    "--surface",
    "--surface-2",
    "--raise",
    "--ink",
    "--ink-soft",
    "--ink-faint",
    "--line",
    "--line-soft",
    "--accent",
    "--accent-ink",
    "--accent-soft",
    "--seal",
    "--ok",
    "--warn",
    "--danger",
    "--scrim",
    "--foreign-ground",
    "--ok-wash",
    "--warn-wash",
    "--danger-wash",
    "--accent-ring",
    "--danger-ink",
    "--mark-ground",
    // DurationToken (crate::tokens::timing)
    "--t-tap",
    "--t-quick",
    "--t-move",
    "--t-big",
    "--t-ambient",
    "--t-big-heavy",
    "--t-spark",
    "--t-curl",
    "--t-curl-heavy",
    "--t-send",
    "--t-float",
    "--t-hc-out",
    "--t-scene",
    "--t-shake",
    "--t-park",
    "--t-nudge",
    "--t-shake-long",
    "--t-sail",
    "--t-boat-return",
    "--t-spin",
    "--t-send-ring",
    // EasingToken (crate::tokens::easing)
    "--e-out",
    "--e-spring",
    "--e-exit",
    "--e-shake",
    "--e-linear",
    "--e-in-out",
    // Radius (crate::tokens::shape)
    "--r-panel",
    "--r-card",
    "--r-btn",
    "--r-chip",
    "--r-pill",
    "--r-field",
    "--r-menu",
    "--r-item",
    "--r-tile",
    "--r-window",
    "--r-menu-item",
    "--r-bubble-button",
    "--r-small",
    "--r-kbd",
    "--r-tiny",
    "--r-micro",
    "--r-media",
    // Shadow (crate::tokens::elevation)
    "--shadow-1",
    "--shadow-2",
    "--shadow-pop",
    "--shadow-sheet",
    "--shadow-drag",
    "--shadow-bubble",
    "--shadow-current",
    "--shadow-pill-inset",
    "--shadow-window",
    "--shadow-card",
    "--shadow-handle",
    "--shadow-mark",
    "--shadow-mark-tile",
    // Family, FontSize (crate::tokens::type_scale)
    "--font-display",
    "--font-ui",
    "--font-data",
    "--fs-pico",
    "--fs-nano",
    "--fs-micro",
    "--fs-caption",
    "--fs-note",
    "--fs-eyebrow",
    "--fs-help",
    "--fs-small",
    "--fs-meta",
    "--fs-control",
    "--fs-body",
    "--fs-reading",
    "--fs-compose",
    "--fs-base",
    "--fs-subhead",
    "--fs-title",
    "--fs-heading-3",
    "--fs-subject",
    "--fs-heading",
    "--fs-amount",
    "--fs-day",
    "--fs-display",
    // ZLayer (crate::tokens::layer)
    "--z-scene",
    "--z-grain",
    "--z-raise",
    "--z-link-pill",
    "--z-toast",
    "--z-send-pill",
    "--z-scrim",
    "--z-peek",
    "--z-focus-page",
    "--z-edge",
    "--z-side-peek",
    "--z-palette",
    "--z-card",
    "--z-menu",
    "--z-bubble",
    "--z-drag",
    // ScalarToken (crate::tokens::scalar)
    "--overshoot",
    "--squish",
    "--lift",
    "--tilt",
    "--stagger",
    // DelayToken::var() (crate::tokens::delay): only the delays the stylesheet also reads.
    "--d-fly",
    "--d-heal-step",
    // LabelHue x HueMember (crate::tokens::label_hue): base, `-deep`, `-soft` per hue.
    "--c-red",
    "--c-red-deep",
    "--c-red-soft",
    "--c-amber",
    "--c-amber-deep",
    "--c-amber-soft",
    "--c-green",
    "--c-green-deep",
    "--c-green-soft",
    "--c-blue",
    "--c-blue-deep",
    "--c-blue-soft",
    "--c-violet",
    "--c-violet-deep",
    "--c-violet-soft",
    // FrameVars (crate::space::frame_vars): the `--f-*` variables a `.ds` root carries inline.
    "--f-ink",
    "--f-ink-soft",
    "--f-ink-faint",
    "--f-pill",
    "--f-pill-hover",
    "--f-line",
    "--f-solid",
    "--f-grad",
    "--f-grain",
    // MaterialRecipe (crate::material::recipe): the `--m-*` a material paints.
    "--m-tint",
    "--m-tint-solid",
    "--m-edge",
    "--m-shadow",
    "--m-radius",
];

/// Every `@keyframes` name [`crate::motion::Anim`] declares, plus each name's `X--b` restart
/// alias (design/05-MOTION.md section 9 rule 2, spike S5).
///
/// Source: the doc comment on every `Anim` variant, which names its keyframes in backticks.
/// `FoldHeavy` shares `fold`'s keyframes at a longer duration (`Anim` doc comment), so it is
/// not a second name here.
pub const ANIM_NAMES: &[&str] = &[
    "seal-pop",
    "gulp",
    "pop-in",
    "row-in",
    "rise",
    "star-pop",
    "spark",
    "chip-land",
    "chip-in",
    "fold",
    "crumple",
    "curl",
    "heal",
    "bump",
    "tab-in",
    "tab-out",
    "slide-r",
    "slide-l",
    "menu-in",
    "menu-pop",
    "cmdk-in",
    "peek-in",
    "fade",
    "hc-in",
    "hc-out",
    "page-in",
    "park",
    "shake-x",
    "nudge",
    "shake",
    "compose-rise",
    "compose-send",
    "floatup",
    "sail",
    "boat-return",
    "dest",
    "breathe",
    "spin",
];

/// Whether `name` (an `animation-name` value, without the `--b` suffix considered separately)
/// is a keyframes name the design system exports.
pub fn is_known_anim(name: &str) -> bool {
    if name.eq_ignore_ascii_case("none") {
        return true;
    }
    let base = name.strip_suffix("--b").unwrap_or(name);
    ANIM_NAMES
        .iter()
        .any(|known| known.eq_ignore_ascii_case(base))
}
