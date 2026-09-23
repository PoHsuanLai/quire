//! Gaps: what Blitz cannot do and what quire draws instead (FINDINGS spike S1-S16), each shown
//! working or visibly absent; and what the design names that quire does not draw yet, as the
//! gallery found it.

use super::{Scope, Section, Specimen};
use crate::axes::{Axes, Showcase};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    BlurState, Glyph, Icon, IconSize, InputModality, Material, clip_chars, sleep, use_env, use_rect,
};
use std::time::Duration;

/// Each spike finding: its id, what Blitz does, and what quire does about it.
const LIMITS: [(&str, &str, &str); 16] = [
    (
        "S1",
        "A <style> in the body applies.",
        "Ds inlines the stylesheet; so does this gallery.",
    ),
    (
        "S2",
        "[data-x=v] never matches an attribute Dioxus sets.",
        "Every attribute selector is written [*|data-x=v]; the lint rejects the bare form.",
    ),
    (
        "S3",
        "@keyframes work, var() inside them too.",
        "motion.css is used as designed: the motion lab fires each.",
    ),
    (
        "S4",
        "transition on a var()-driven value works.",
        "Toggles and sliders transition through tokens.",
    ),
    (
        "S5",
        "Swapping animation-name restarts it.",
        "use_pulse flips data-pulse between X and X--b.",
    ),
    (
        "S6",
        "SVG stroke=currentColor follows color; CSS stroke on children does nothing.",
        "Glyph writes stroke as attributes; the lint bans stroke and fill in CSS.",
    ),
    (
        "S7",
        "mask-image: url(data:…) paints only with a data: net provider.",
        "ds-native installs one; the truncation fade and masks work.",
    ),
    (
        "S8",
        "background-image: url(data:…) tiles, the same condition.",
        "The grain tile and this page's wallpaper are data: PNGs.",
    ),
    (
        "S9",
        "onmounted's rect is 0 x 0; correct after the first layout.",
        "use_rect reads after a frame (below).",
    ),
    (
        "S10",
        "futures-timer sleeps wake the document.",
        "Settle timers, hover intent and the toast hold run on ds::sleep (the ticker below).",
    ),
    (
        "S11",
        "Registered TTFs are picked by font-family.",
        "ds-native registers the three faces once (the Type page).",
    ),
    (
        "S12",
        ":focus-visible and :focus-within never match; a click does not focus a button.",
        "The host stamps data-modality; rings key on keyboard modality (below).",
    ),
    (
        "S13",
        "text-overflow: ellipsis draws no ellipsis.",
        ".ds-truncate fades the end; clip_chars cuts known text in Rust (below).",
    ),
    (
        "S14",
        "color-mix() works.",
        "Washes stay precomputed for determinism.",
    ),
    (
        "S15",
        "backdrop-filter: blur() is ignored on every backend.",
        "Blur comes from the compositor; data-blur=off paints the solid tint (below).",
    ),
    (
        "S16",
        "filter: saturate() is ignored; blur() only on hybrid.",
        "filter is banned; vibrancy is baked into the tints.",
    ),
];

/// What the gallery could not show with quire as it is: each a component that cannot express a
/// documented state, a missing token, or a contract the code breaks.
const DS_GAPS: [&str; 15] = [
    "No spacing tokens: design/01-LAYOUT.md section 2 names the scale, but every padding and gap is a raw length.",
    "Surface overrides only the material and the scheme. A specimen in another accent or blur state needs a nested Ds (the Matrix and Materials pages).",
    "The root marks the back frame layer with the class \"back\", but the stylesheet hides .ds-layer[*|data-layer=back]: the class is unstyled and both layers stay opaque, so the Space switch never cross-fades.",
    "HoverCard has no parts: its person header, stats, flag and foot are hand-written ds-hovercard-* markup.",
    "ToastHub reports an undo only as last_undo(); a consumer restores by watching it in an effect.",
    "The Avatar's computed hue and account colour are inline hexes (O-7), which a consumer's markup lint has to except.",
    "Button has no mounted handle, so a menu anchored to a button measures a wrapper instead.",
    "No token or component for a wallpaper, a stage or a specimen grid: the gallery's own layout CSS covers them.",
    "TextInput is 176 px tall on Blitz: the input takes the 300 x 150 replaced-element default and text_input.css sets no height (Controls page).",
    "Every Ds renders a ToastHost; the hidden toast is an empty pill whose translateY(160%) does not clear the root, so a small dark pill shows at the bottom of every nested root (Tokens, Materials, Matrix).",
    "The toast's pull tab paints as an empty pale pill in the snapshot: its Undo glyph and label are not visible (Overlays page).",
    "A place SidebarItem without a count centres its icon and label instead of starting them at the left (Lists page).",
    "On master a float's two rect probes wake on the same poll and the second runs inside Blitz's render borrow: RefCell already borrowed. The wave 2 integration branch fixes it with a host measurer; until then the posed snapshot opens no menu.",
    "The Space editor's colour field and its handles are blank in a snapshot (Space page), and its preset swatches render as 140 px discs.",
    "The LinkPill enters with --t-move --e-spring; the design says --t-quick --e-out and no Anim has that recipe (link_pill.css TODO).",
];

/// The gaps page.
#[component]
pub fn GapsPage() -> Element {
    rsx! {
        Section { title: "Blitz limits, spike S1-S16", note: "What the renderer does, and what quire does about it. The demonstrations follow.",
            div { class: "g-table g-cols5",
                for (id , does , answer) in LIMITS {
                    span { class: "g-code", "{id}" }
                    span { class: "g-note", style: "grid-column:span 2", "{does}" }
                    span { class: "g-note", style: "grid-column:span 2", "{answer}" }
                }
            }
        }
        Truncation {}
        Grain {}
        BlurOff {}
        Icons {}
        Live {}
        Section { title: "What quire does not draw yet", note: "Found while building this gallery.",
            div { class: "g-col",
                for gap in DS_GAPS {
                    p { class: "g-note", "· {gap}" }
                }
            }
        }
    }
}

/// S13: the mask fade, and the Rust clip for known text.
#[component]
fn Truncation() -> Element {
    const TEXT: &str = "The quick brown fox jumps over the lazy dog and keeps running";
    rsx! {
        Section { title: "S13: truncation",
            div { class: "g-grid3",
                Specimen { name: ".ds-truncate: the end fades", code: "mask-image, spike S7 and S13".to_string(),
                    div { class: "ds-truncate", style: "width:220px", "{TEXT}" }
                }
                Specimen { name: "clip_chars(text, 28): a real ellipsis", code: "text/clip.rs".to_string(),
                    div { style: "width:220px", "{clip_chars(TEXT, 28)}" }
                }
                Specimen { name: "neither: a hard clip", code: "overflow:hidden alone".to_string(),
                    div { style: "width:220px;overflow:hidden;white-space:nowrap", "{TEXT}" }
                }
            }
        }
    }
}

/// S8: the grain tile over the frame gradient, at the Space's own strength.
#[component]
fn Grain() -> Element {
    let grain = use_context::<Signal<Axes>>().read().look.grain.0;
    rsx! {
        Section { title: "S8: grain tile", note: "The generated 128 px alpha-noise PNG, at --f-grain over the Space gradient. Change the grain on the Space page.",
            div { class: "g-stage", style: "background:var(--f-grad)",
                div { class: "ds-grain" }
                p { class: "g-note g-stage-pad", "grain {grain}" }
            }
        }
    }
}

/// S15: nothing behind a panel is blurred; `data-blur=off` paints the solid fallback.
#[component]
fn BlurOff() -> Element {
    let env = use_env();
    let accent = env.resolved.accent;
    let scheme = env.scheme;
    rsx! {
        Section { title: "S15: no backdrop blur", note: "Both panels sit over one-pixel stripes. With blur on the stripes show through the tint unblurred (the compositor blurs on the desktop, Blitz cannot); with blur off the solid tint hides them.",
            div { class: "g-wall g-grid2", style: "background-image:url(\"{wallpaper::uri()}\")",
                for blur in [BlurState::Available, BlurState::Unavailable] {
                    Scope { scheme, accent, material: Material::Popover, blur,
                        div { class: "g-panel",
                            span { class: "g-name", "data-blur={blur.slug()}" }
                            span { "Popover tint over the wallpaper." }
                        }
                    }
                }
            }
        }
    }
}

/// S6: a Glyph takes the colour of its text.
#[component]
fn Icons() -> Element {
    let tones = [
        "--ink",
        "--ink-soft",
        "--accent",
        "--ok",
        "--warn",
        "--danger",
    ];
    rsx! {
        Section { title: "S6: icon colour", note: "Each Glyph's stroke is currentColor as an attribute, so it follows the colour around it.",
            div { class: "g-row",
                for tone in tones {
                    span { class: "g-row", style: "color:var({tone})",
                        Glyph { icon: Icon::Star, size: IconSize::Large }
                        span { class: "g-code", "{tone}" }
                    }
                }
            }
        }
    }
}

/// S9, S10, S12: a measured rect, a ticking timer, the input modality.
#[component]
fn Live() -> Element {
    let probe = use_rect();
    let mut ticks = use_signal(|| 0u32);
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    use_hook(move || {
        if showcase == Showcase::Posed {
            return;
        }
        spawn(async move {
            loop {
                sleep(Duration::from_secs(1)).await;
                ticks += 1;
            }
        });
    });
    let modality = match use_env().modality {
        InputModality::Pointer => "pointer: no focus rings",
        InputModality::Keyboard => "keyboard: focus rings show",
    };
    let rect = probe.rect().map_or("not measured yet".to_string(), |rect| {
        format!(
            "{} x {} at ({}, {})",
            rect.size.width.0, rect.size.height.0, rect.origin.x.0, rect.origin.y.0
        )
    });
    rsx! {
        Section { title: "S9, S10, S12: measured, timed, driven",
            div { class: "g-grid3",
                Specimen { name: "use_rect after layout", code: rect,
                    div { class: "g-stage", onmounted: move |event| probe.on_mounted(event) }
                }
                Specimen { name: "ds::sleep ticks", code: format!("{} s since this page opened", ticks()),
                    div {}
                }
                Specimen { name: "data-modality", code: modality.to_string(),
                    div {}
                }
            }
        }
    }
}
