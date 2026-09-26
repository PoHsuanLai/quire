//! The Lock and switcher page (M11): the shell's own lock screen over the calm wallpaper (at
//! rest, a wrong password posed at its shake, and checking in the Space's colour), the polkit
//! prompt's sheet, and the app switcher with five and fourteen apps.

use super::Section;
use crate::axes::Axes;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::{
    AppKey, AppSwitcher, Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, CapsLock, Ds,
    Icon, IconSource, ImageSource, Inject, LockClock, LockLook, LockPrompt, LockScreen, LockUser,
    Material, PlateFamily, PolkitPrompt, PromptState, Px, RootChrome, SwitcherApp, Text,
    person_hue, use_env,
};

/// The person at the lock screen.
fn user() -> LockUser {
    LockUser {
        name: "Po-Hsuan Lai".to_owned(),
        avatar: AvatarFace {
            initial: 'P',
            size: AvatarSize::Size64,
            tone: AvatarTone::Person(person_hue("pohsuan")),
            shape: AvatarShape::Round,
        },
    }
}

/// One lock specimen: its caption, state, caps lock, look, hint and pose.
#[derive(Debug, Clone, PartialEq)]
struct Pose {
    caption: &'static str,
    state: PromptState,
    caps: CapsLock,
    look: LockLook,
    hint: Option<&'static str>,
    /// Posed at `shake-x`'s 20 % frame (the field 6 px left), since a snapshot is taken long
    /// after the shake has played.
    shaking: Shaking,
}

/// Whether a specimen is posed mid-shake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shaking {
    Posed,
    Still,
}

fn poses() -> [Pose; 3] {
    [
        Pose {
            caption: "Idle",
            state: PromptState::Idle,
            caps: CapsLock::Off,
            look: LockLook::Clear,
            hint: Some("Press Enter to unlock"),
            shaking: Shaking::Still,
        },
        Pose {
            caption: "Wrong, posed at shake-x's 20 % frame, caps lock on",
            state: PromptState::Wrong,
            caps: CapsLock::On,
            look: LockLook::Clear,
            hint: Some("Caps Lock is on"),
            shaking: Shaking::Posed,
        },
        Pose {
            caption: "Checking, in the Space's colour (LockLook::Space)",
            state: PromptState::Checking,
            caps: CapsLock::Off,
            look: LockLook::Space,
            hint: Some("Checking…"),
            shaking: Shaking::Still,
        },
    ]
}

/// The page.
#[component]
pub fn LockSwitcherPage() -> Element {
    let [idle, wrong, checking] = poses();
    rsx! {
        Section { title: "Lock screen", note: "LockScreen with a LockClock and a LockPrompt over the calm wallpaper under --lock-veil, drawn at a 16:10 output's proportions (1216 x 760; the two below at half scale). The date, then the time in the display face at --fs-lock-clock (140) 700, white; at the bottom the avatar at 64, the name 16/700, a 260 x 38 pill of flat white glass (--lock-glass) holding the Secret field, the caps-lock mark and the enter arrow (shown once something is typed), and the hint line. Wrong plays shake-x once (420 ms, --e-shake) and empties the field as it settles; Checking closes the field and spins the arrow; under LockLook::Space the date sits in a pill and the field is painted with the Space gradient.",
            div { class: "g-lock-full",
                LockStage { pose: idle }
            }
            div { class: "g-row g-row-top",
                div { class: "g-col",
                    div { class: "g-lock-mini",
                        LockStage { pose: wrong.clone() }
                    }
                    span { class: "g-name", "{wrong.caption}" }
                }
                div { class: "g-col",
                    div { class: "g-lock-mini",
                        LockStage { pose: checking.clone() }
                    }
                    span { class: "g-name", "{checking.caption}" }
                }
            }
        }
        Section { title: "Polkit prompt", note: "PolkitPrompt: a narrow centred Sheet (SheetWidth::Narrow, 340) over the modal scrim, entering with peek-in. The avatar at 48, the title, the program's message, Details as a hover card, the name over a boxed Secret field, Cancel (Secondary) and Authenticate (Primary) at equal width. Left at rest; right locked out after too many tries, the field and Authenticate closed.",
            div { class: "g-row g-row-top",
                PolkitStage { state: PromptState::Idle }
                PolkitStage { state: PromptState::LockedOut { until: "9:52".to_owned() } }
            }
        }
        Section { title: "App switcher", note: "AppSwitcher on the Osd material painted as the OSD card is (the Space gradient at the frame alpha, radius 18), padding 16: cells of 112 with icons of 96, 8 apart; the selection a --f-pill square (--r-tile 12) that springs between cells (--t-quick --e-spring); the selected app's name under it as its Fly in the UI face 13/600. Five apps on a 1440 output; fourteen on a 1216 output (this wall's width) shrink to fit output - 64 (cells of 72, icons of 56); fourteen on an 800 output stop at the 48 px minimum and scroll to keep the selection (the tenth app) in view.",
            SwitcherStage { count: 5, selected: 1, output: 1440.0 }
            SwitcherStage { count: 14, selected: 1, output: 1216.0 }
            SwitcherStage { count: 14, selected: 9, output: 800.0 }
        }
    }
}

/// A lock screen in the page's scheme, over the calm wallpaper.
#[component]
fn LockStage(pose: Pose) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion) = {
        let axes = axes.read();
        (axes.theme, axes.accent, axes.motion)
    };
    let scheme = use_env().scheme;
    let wallpaper = ImageSource(wallpaper::calm_uri(scheme).to_owned());
    let pose_class = match pose.shaking {
        Shaking::Posed => Some("g-shake-pose"),
        Shaking::Still => None,
    };
    rsx! {
        div { class: "g-lock-stage",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Window,
                stylesheet: Inject::Host,
                div { class: "g-lock-size",
                LockScreen {
                    wallpaper,
                    clock: rsx! { LockClock { time: "9:41", date: "Saturday 26 September", look: pose.look } },
                    prompt: rsx! {
                        div { class: pose_class,
                            LockPrompt {
                                user: user(),
                                state: pose.state.clone(),
                                caps: pose.caps,
                                look: pose.look,
                                hint: pose.hint.map(Text::from),
                                oninput: |_| {},
                                onsubmit: |_| {},
                            }
                        }
                    },
                }
                }
            }
        }
    }
}

/// A polkit prompt in `state`, in a Sheet root of its own over the calm wallpaper.
#[component]
fn PolkitStage(state: PromptState) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion) = {
        let axes = axes.read();
        (axes.theme, axes.accent, axes.motion)
    };
    let scheme = use_env().scheme;
    rsx! {
        div { class: "g-modal g-polkit", style: "background-image:url(\"{wallpaper::calm_uri(scheme)}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Sheet,
                stylesheet: Inject::Host,
                div { class: "g-polkit-stage" }
                PolkitPrompt {
                    action: "Authentication is required to change the system's time zone.",
                    detail: Some(Text::from("org.freedesktop.timedate1.set-timezone")),
                    user: user(),
                    state,
                    oninput: |_| {},
                    onsubmit: |_| {},
                    oncancel: |_| {},
                }
            }
        }
    }
}

/// The apps the switcher shows, most recent first.
const APPS: [(&str, &str, Icon, PlateFamily); 14] = [
    ("mail", "Mail", Icon::Mail, PlateFamily::Blue),
    ("files", "Files", Icon::Folder, PlateFamily::Blue),
    ("terminal", "Terminal", Icon::Terminal, PlateFamily::Neutral),
    ("notes", "Notes", Icon::StickyNote, PlateFamily::Amber),
    ("photos", "Photos", Icon::Image, PlateFamily::Violet),
    ("settings", "Settings", Icon::Settings, PlateFamily::Neutral),
    ("calendar", "Calendar", Icon::Clock, PlateFamily::Red),
    ("music", "Music", Icon::Headphones, PlateFamily::Red),
    ("camera", "Camera", Icon::Camera, PlateFamily::Neutral),
    ("downloads", "Downloads", Icon::Download, PlateFamily::Green),
    ("monitor", "System Monitor", Icon::Gauge, PlateFamily::Green),
    ("printer", "Printers", Icon::Printer, PlateFamily::Neutral),
    ("keyboard", "Keyboard", Icon::Keyboard, PlateFamily::Violet),
    ("display", "Displays", Icon::Monitor, PlateFamily::Amber),
];

/// A switcher of the first `count` apps with the one at `selected` selected, on an output
/// `output` wide, in a transparent Osd root on the calm wallpaper.
#[component]
fn SwitcherStage(count: usize, selected: usize, output: f32) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion, blur) = {
        let axes = axes.read();
        (axes.theme, axes.accent, axes.motion, axes.blur)
    };
    let scheme = use_env().scheme;
    let apps: Vec<SwitcherApp> = APPS
        .iter()
        .take(count)
        .map(|(key, name, icon, family)| SwitcherApp {
            plate: Some(*family),
            ..SwitcherApp::new(*key, *name, IconSource::Glyph(*icon))
        })
        .collect();
    let chosen = AppKey(APPS[selected.min(count - 1)].0.to_owned());
    rsx! {
        div { class: "g-switcher-wall", style: "background-image:url(\"{wallpaper::calm_uri(scheme)}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Osd,
                blur,
                stylesheet: Inject::Host,
                chrome: Some(RootChrome::Transparent),
                AppSwitcher {
                    apps,
                    selected: chosen,
                    output: Some(Px(output)),
                    onhover: |_| {},
                    onactivate: |_| {},
                }
            }
        }
    }
}
