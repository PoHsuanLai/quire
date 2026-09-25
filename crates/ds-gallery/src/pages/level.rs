//! The Level page (the user's brief of 2026-09-25): the level control's three looks to choose
//! between, live on an OSD card and as a grid of states on two grounds, and the OSD card at the
//! top right, shown and hidden with its fade. The same grid, in both schemes and at 1x and 2x, is
//! `ds-gallery --level-sheet DIR`'s contact sheet.

use super::level_tile::{Ground, LevelTile, STATES, theme, work};
use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Appearance, BlurState, Button, ButtonVariant, Ds, Fraction, Inject, Level, LevelControl,
    LevelGlyph, LevelLook, Material, Muting, Osd, OsdMetrics, OsdPosition, RootChrome, Scheme,
    Shown, Tick, use_env,
};

/// The Level page.
#[component]
pub fn LevelPage() -> Element {
    rsx! {
        LiveSection {}
        LooksSection {}
        OsdSection {}
    }
}

/// A look's caption.
fn describe(look: LevelLook) -> &'static str {
    match look {
        LevelLook::Capsule => {
            "Capsule (recommended): a 26 px capsule, the fill in the material's bright ink, the glyph inside at the left, knocked out where the fill covers it"
        }
        LevelLook::CapsuleKnob => {
            "Capsule and knob: the capsule with a round knob riding the fill's end, the glyph before it"
        }
        LevelLook::Segments => {
            "Segments: sixteen rounded squares that fill in one --stagger after another, the glyph before them"
        }
    }
}

#[component]
fn LiveSection() -> Element {
    let scheme = use_env().scheme;
    let mut volume = use_signal(|| Fraction(400));
    let mut brightness = use_signal(|| Fraction(300));
    let mut muting = use_signal(|| Muting::Audible);
    let glyph = LevelGlyph::Volume(muting());
    rsx! {
        Section { title: "Live", note: "Drag, or focus and use the arrows (Shift for fine steps). Under the pointer the fill follows with no easing; a press swells the track; past either end the capsule stretches and springs back on release. The buttons set the level from outside, which slides over --t-quick --e-out. Each control ticks as it crosses a sixteenth.",
            div { class: "g-row g-row-top",
                for look in LevelLook::ALL {
                    Specimen { name: look.slug().to_owned(), code: describe(look).to_owned(),
                        OsdCard { scheme, title: "Sound",
                            div { class: "g-level-live",
                                LevelControl { label: "Volume", value: volume(), glyph, look, tick: Tick::Quiet, onchange: move |next| volume.set(next) }
                                LevelControl { label: "Brightness", value: brightness(), glyph: LevelGlyph::Brightness, look, tick: Tick::Quiet, onchange: move |next| brightness.set(next) }
                            }
                        }
                    }
                }
            }
            div { class: "g-row",
                Button { variant: ButtonVariant::Mini, label: "Volume 0", onclick: move |_| volume.set(Fraction(0)) }
                Button { variant: ButtonVariant::Mini, label: "Volume 40", onclick: move |_| volume.set(Fraction(400)) }
                Button { variant: ButtonVariant::Mini, label: "Volume 100", onclick: move |_| volume.set(Fraction(1000)) }
                Button {
                    variant: ButtonVariant::Mini,
                    label: "Mute",
                    onclick: move |_| {
                        let next = match muting() {
                            Muting::Audible => Muting::Muted,
                            Muting::Muted => Muting::Audible,
                        };
                        muting.set(next);
                    },
                }
                Button { variant: ButtonVariant::Mini, label: "Brightness 30", onclick: move |_| brightness.set(Fraction(300)) }
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
    let scheme = use_env().scheme;
    rsx! {
        Section { title: "Looks", note: "Each look at volume 0, 40 and 100 %, muted, and brightness 30 %, on the OSD card over the Work Space's frame and over a light ground. The speaker shows a wave per third of the range and a slash when muted; the sun's rays grow with the level.",
            for look in LevelLook::ALL {
                Specimen { name: look.slug().to_owned(), code: describe(look).to_owned(),
                    div { class: "g-level-grid",
                        for ground in Ground::ALL {
                            div { class: "g-row g-row-top",
                                for state in STATES {
                                    LevelTile { look, scheme, ground, state }
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
    let scheme = use_env().scheme;
    let theme = theme(scheme);
    let mut shown = use_signal(|| Shown::Visible);
    let mut hidden_at = use_signal(|| 0u32);
    let cards = [
        (LevelGlyph::Volume(Muting::Audible), "MA270U", Fraction(620)),
        (LevelGlyph::Brightness, "Display", Fraction(300)),
    ];
    rsx! {
        Section { title: "OSD at the top right", note: "The Osd card under the bar's reserve (osd.position TopRight, osd.margin_px 24): a title line over a read-only capsule, a control-center module's shape. Hide it: it fades and lifts over --t-move --e-exit, then on_hidden runs (the host unmaps the surface there); show it: it drops in over --t-quick --e-out. Showing it while it fades takes the hide back.",
            div { class: "g-row",
                Button { variant: ButtonVariant::Mini, label: "Show", onclick: move |_| shown.set(Shown::Visible) }
                Button { variant: ButtonVariant::Mini, label: "Hide", onclick: move |_| shown.set(Shown::Hidden) }
                span { class: "g-code", "on_hidden ran {hidden_at} times" }
            }
            div { class: "g-grid2",
                for (glyph , title , value) in cards {
                    Specimen { name: title.to_owned(), code: "Osd { position: TopRight, look: Capsule }".to_owned(),
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
                                            level: Level { value, glyph },
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
