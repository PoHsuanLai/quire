//! The Level page (the user's brief of 2026-09-25): the capsule slider's two looks live on an OSD
//! card, the level indicator's two styles as a grid of states on two grounds, and the OSD card at
//! the top right, shown and hidden with its fade. The same grid, in both schemes and at 1x and 2x, is
//! `ds-gallery --level-sheet DIR`'s contact sheet.

use crate::pages::shell::level_tile::{Ground, LevelTile, STATES, theme, work};
use crate::pages::{Section, Specimen};
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::content::level_glyph::vocab::LevelGlyph;
use ds::components::controls::level_indicator::LevelStyle;
use ds::components::controls::slider_model::SliderLook;
use ds::prelude::*;
use ds_core::vocab::Muting;
use ds_shell::osd::{OsdLevel, OsdPosition};
use ds_shell::prelude::*;
use ds_shell::tokens::osd::OsdMetrics;
use ds_style::appearance::blur::BlurState;
use ds_style::tokens::control_size::ControlSize;

/// The Level page.
#[component]
pub fn LevelPage() -> Element {
    rsx! {
        LiveSection {}
        LooksSection {}
        OsdSection {}
    }
}

/// A slider look's caption.
fn describe_look(look: SliderLook) -> &'static str {
    match look {
        SliderLook::Linear => {
            "Linear: a thin track under a round knob, the form slider of a settings row"
        }
        SliderLook::Capsule => {
            "Capsule (recommended): a capsule, the fill in the material's bright ink, the glyph inside at the left, knocked out where the fill covers it"
        }
        SliderLook::CapsuleKnob => {
            "Capsule and knob: the capsule with a round knob riding the fill's end, the glyph before it"
        }
    }
}

/// A level indicator style's caption.
fn describe_style(style: LevelStyle) -> &'static str {
    match style {
        LevelStyle::Continuous => {
            "Continuous: a capsule, the fill in the material's bright ink, the glyph inside at the left"
        }
        LevelStyle::Discrete => {
            "Discrete: sixteen rounded squares that fill in one after another, the glyph before them"
        }
    }
}

#[component]
fn LiveSection() -> Element {
    let scheme = use_scope().scheme;
    let mut volume = use_signal(|| Fraction(400));
    let mut brightness = use_signal(|| Fraction(300));
    let mut muting = use_signal(|| Muting::Audible);
    let glyph = LevelGlyph::Volume(muting());
    rsx! {
        Section { title: "Live", note: "Drag, or focus and use the arrows (Shift for fine steps). Under the pointer the fill follows with no easing. The buttons set the level from outside, which slides over --t-quick --e-out.",
            div { class: "g-row g-row-top",
                for look in [SliderLook::Capsule, SliderLook::CapsuleKnob] {
                    Specimen { name: look.slug().to_owned(), code: describe_look(look).to_owned(),
                        OsdCard { scheme, title: "Sound",
                            div { class: "g-level-live",
                                Slider { label: "Volume", value: volume(), glyph, look, onchange: move |next| volume.set(next) }
                                Slider { label: "Brightness", value: brightness(), glyph: LevelGlyph::Brightness, look, onchange: move |next| brightness.set(next) }
                            }
                        }
                    }
                }
            }
            div { class: "g-row",
                Button { size: ControlSize::Mini, label: "Volume 0", onclick: move |_| volume.set(Fraction(0)) }
                Button { size: ControlSize::Mini, label: "Volume 40", onclick: move |_| volume.set(Fraction(400)) }
                Button { size: ControlSize::Mini, label: "Volume 100", onclick: move |_| volume.set(Fraction(1000)) }
                Button {
                    size: ControlSize::Mini,
                    label: "Mute",
                    onclick: move |_| {
                        let next = match muting() {
                            Muting::Audible => Muting::Muted,
                            Muting::Muted => Muting::Audible,
                        };
                        muting.set(next);
                    },
                }
                Button { size: ControlSize::Mini, label: "Brightness 30", onclick: move |_| brightness.set(Fraction(300)) }
            }
        }
    }
}

/// An OSD card in `scheme` holding `children`, always shown, on the Work Space's tint.
#[component]
fn OsdCard(scheme: Scheme, title: String, children: Element) -> Element {
    let theme = theme(scheme);
    rsx! {
        Ds {
            appearance: Appearance { theme, ..Appearance::default() },
            look: work(theme),
            material: Material::Osd,
            chrome: Some(RootChrome::Transparent),
            blur: BlurState::Unavailable,
            stylesheet: Inject::Host,
            Osd { shown: Shown::Visible, label: title, position: OsdPosition::BottomCentre, {children} }
        }
    }
}

#[component]
fn LooksSection() -> Element {
    let scheme = use_scope().scheme;
    rsx! {
        Section { title: "Styles", note: "Each style at volume 0, 40 and 100 %, muted, and brightness 30 %, on the OSD card over the Work Space's frame and over a light ground. The speaker shows a wave per third of the range and a slash when muted; the sun's rays grow with the level.",
            for style in LevelStyle::ALL.iter().copied() {
                Specimen { name: style.slug().to_owned(), code: describe_style(style).to_owned(),
                    div { class: "g-level-grid",
                        for ground in Ground::ALL.iter().copied() {
                            div { class: "g-row g-row-top",
                                for state in STATES {
                                    LevelTile { style, scheme, ground, state }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn OsdSection() -> Element {
    let scheme = use_scope().scheme;
    let theme = theme(scheme);
    let mut shown = use_signal(|| Shown::Visible);
    let mut hidden_at = use_signal(|| 0u32);
    let cards = [
        (LevelGlyph::Volume(Muting::Audible), "MA270U", Fraction(620)),
        (LevelGlyph::Brightness, "Display", Fraction(300)),
    ];
    rsx! {
        Section { title: "OSD at the top right", note: "The Osd card under the bar's reserve (osd.position TopRight, osd.margin_px 24): a title line over a read-only capsule, a control-center module's shape. Hide it: it fades over --t-move --e-exit, then on_hidden runs (the host unmaps the surface there); show it: it fades in over --t-quick. Showing it while it fades takes the hide back.",
            div { class: "g-row",
                Button { size: ControlSize::Mini, label: "Show", onclick: move |_| shown.set(Shown::Visible) }
                Button { size: ControlSize::Mini, label: "Hide", onclick: move |_| shown.set(Shown::Hidden) }
                span { class: "g-code", "on_hidden ran {hidden_at} times" }
            }
            div { class: "g-grid2",
                for (glyph , title , value) in cards {
                    Specimen { name: title.to_owned(), code: "Osd { position: TopRight, style: Continuous }".to_owned(),
                        div { class: "g-level-screen", style: OsdMetrics::default().style_attr(),
                            Ds {
                                appearance: Appearance { theme, ..Appearance::default() },
                                look: work(theme),
                                material: Material::Window,
                                stylesheet: Inject::Host,
                                div { class: "g-level-screen",
                                    Ds {
                                        appearance: Appearance { theme, ..Appearance::default() },
                                        look: work(theme),
                                        material: Material::Osd,
                                        chrome: Some(RootChrome::Transparent),
                                        blur: BlurState::Unavailable,
                                        stylesheet: Inject::Host,
                                        Osd {
                                            shown: shown(),
                                            label: title,
                                            level: OsdLevel { value, glyph },
                                            on_hidden: move |()| hidden_at += 1,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
