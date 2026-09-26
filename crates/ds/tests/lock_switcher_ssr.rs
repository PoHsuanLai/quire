//! The lock screen, the polkit prompt and the app switcher as markup (M11): every part in every
//! state matches its golden under `tests/snapshots/lock_switcher/`, lints clean, and uses only
//! `ds-` classes the stylesheet styles; the markup carries what the props say, and no state
//! writes a password field's `value`.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test lock_switcher_ssr` rewrites the goldens.

#[path = "support/golden.rs"]
mod golden;

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::lint::{LintConfig, markup};
use ds::{
    AppKey, AppSwitcher, Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, CapsLock, Ds,
    Icon, IconSource, ImageSource, Inject, LockClock, LockLook, LockPrompt, LockScreen, LockUser,
    Material, PersonaSpec, PlateFamily, PolkitPrompt, PromptState, Px, RootChrome, SwitcherApp,
    Text, Theme, TilePresence, person_hue,
};

fn user() -> LockUser {
    LockUser::new(
        "Dana Reyes",
        AvatarFace {
            initial: 'D',
            size: AvatarSize::Size20,
            tone: AvatarTone::Person(person_hue("dana")),
            shape: AvatarShape::Round,
        },
    )
}

/// Dana with her photo, a portrait-shaped PNG the disc crops.
fn photo_user() -> LockUser {
    LockUser::new(
        "Dana Reyes",
        ImageSource("data:image/png;base64,AAAA".to_owned()),
    )
}

/// Dana as her persona.
fn persona_user() -> LockUser {
    LockUser::new("Dana Reyes", PersonaSpec::from_seed(7))
}

fn root(material: Material, body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme: Theme::Light, ..Appearance::default() }, material, stylesheet: Inject::Host,
            {body}
        }
    }
}

fn clock(look: LockLook) -> Element {
    root(
        Material::Window,
        rsx! { LockClock { time: "9:41", date: "Saturday 26 September", look } },
    )
}

fn prompt(state: PromptState, caps: CapsLock, look: LockLook) -> Element {
    root(
        Material::Window,
        rsx! {
            LockPrompt { user: user(), state, caps, look, hint: Some(Text::from("Press Enter to unlock")),
                oninput: |_| {}, onsubmit: |_| {} }
        },
    )
}

fn prompt_for(user: LockUser, state: PromptState) -> Element {
    root(
        Material::Window,
        rsx! {
            LockPrompt { user, state, hint: Some(Text::from("Press Enter to unlock")),
                oninput: |_| {}, onsubmit: |_| {} }
        },
    )
}

fn screen() -> Element {
    root(
        Material::Window,
        rsx! {
            LockScreen {
                wallpaper: ImageSource("data:image/png;base64,AAAA".to_owned()),
                clock: rsx! { LockClock { time: "9:41", date: "Saturday 26 September" } },
                prompt: rsx! { LockPrompt { user: user(), oninput: |_| {}, onsubmit: |_| {} } },
            }
        },
    )
}

fn polkit(state: PromptState, caps: CapsLock) -> Element {
    polkit_with(user(), state, caps)
}

fn polkit_for(user: LockUser, state: PromptState) -> Element {
    polkit_with(user, state, CapsLock::Off)
}

fn polkit_with(user: LockUser, state: PromptState, caps: CapsLock) -> Element {
    root(
        Material::Sheet,
        rsx! {
            PolkitPrompt {
                action: "Authentication is required to change the system's time zone.",
                detail: Some(Text::from("org.freedesktop.timedate1.set-timezone")),
                user,
                state,
                caps,
                oninput: |_| {},
                onsubmit: |_| {},
                oncancel: |_| {},
            }
        },
    )
}

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

fn apps(count: usize, leaving: Option<&str>) -> Vec<SwitcherApp> {
    APPS.iter()
        .take(count)
        .map(|(key, name, icon, family)| SwitcherApp {
            plate: Some(*family),
            presence: if Some(*key) == leaving {
                TilePresence::Leaving
            } else {
                TilePresence::Present
            },
            ..SwitcherApp::new(*key, *name, IconSource::Glyph(*icon))
        })
        .collect()
}

fn switcher(count: usize, selected: &str, output: f32, leaving: Option<&str>) -> Element {
    let body = rsx! {
        AppSwitcher {
            apps: apps(count, leaving),
            selected: AppKey(selected.to_owned()),
            output: Some(Px(output)),
            onhover: |_| {},
            onactivate: |_| {},
        }
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Osd, chrome: Some(RootChrome::Transparent), stylesheet: Inject::Host,
            {body}
        }
    }
}

type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("clock-clear", || clock(LockLook::Clear)),
    ("clock-space", || clock(LockLook::Space)),
    ("prompt-idle", || {
        prompt(PromptState::Idle, CapsLock::Off, LockLook::Clear)
    }),
    ("prompt-checking", || {
        prompt(PromptState::Checking, CapsLock::Off, LockLook::Clear)
    }),
    ("prompt-wrong-caps", || {
        prompt(PromptState::Wrong, CapsLock::On, LockLook::Clear)
    }),
    ("prompt-locked-out", || {
        prompt(
            PromptState::LockedOut {
                until: "9:52".to_owned(),
            },
            CapsLock::Off,
            LockLook::Clear,
        )
    }),
    ("prompt-space", || {
        prompt(PromptState::Idle, CapsLock::Off, LockLook::Space)
    }),
    ("screen", screen),
    ("polkit-idle", || polkit(PromptState::Idle, CapsLock::Off)),
    ("polkit-wrong-caps", || {
        polkit(PromptState::Wrong, CapsLock::On)
    }),
    ("polkit-checking", || {
        polkit(PromptState::Checking, CapsLock::Off)
    }),
    ("polkit-locked-out", || {
        polkit(
            PromptState::LockedOut {
                until: "9:52".to_owned(),
            },
            CapsLock::Off,
        )
    }),
    ("switcher-5", || switcher(5, "files", 1440.0, None)),
    ("switcher-14-shrunk", || switcher(14, "files", 1216.0, None)),
    ("switcher-14-scrolled", || {
        switcher(14, "downloads", 800.0, None)
    }),
    ("switcher-leaving", || {
        switcher(5, "terminal", 1440.0, Some("terminal"))
    }),
    ("prompt-photo", || {
        prompt_for(photo_user(), PromptState::Idle)
    }),
    ("prompt-persona", || {
        prompt_for(persona_user(), PromptState::Idle)
    }),
    ("prompt-persona-accepted", || {
        prompt_for(persona_user(), PromptState::Accepted)
    }),
    ("polkit-photo", || {
        polkit_for(photo_user(), PromptState::Idle)
    }),
    ("polkit-persona", || {
        polkit_for(persona_user(), PromptState::Idle)
    }),
];

/// Build the dom and flush the effects that register overlays, so a sheet is in the markup.
fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .filter_map(|(name, make)| {
            golden::check(&format!("lock_switcher/{name}.html"), &render(*make)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, make) in SPECIMENS {
        let html = render(*make);
        for offence in markup(&html, sheet, &LintConfig::default()) {
            failures.push(format!("{name}: {:?} {}", offence.rule, offence.text));
        }
        for class in html
            .split("class=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .filter(|class| class.starts_with("ds-"))
        {
            let needle = format!(".{class}");
            let styled = sheet.match_indices(&needle).any(|(at, _)| {
                !sheet[at + needle.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            });
            if !styled {
                failures.push(format!("{name}: .{class} is not styled"));
            }
        }
    }
    failures.dedup();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Every password field is a secret: `data-kind="secret"` and no `value` attribute, whatever
/// the state.
#[test]
fn no_password_field_writes_a_value() {
    for (name, make) in SPECIMENS
        .iter()
        .filter(|(name, _)| name.starts_with("prompt") || name.starts_with("polkit"))
    {
        let html = render(*make);
        let field = html
            .split("<input")
            .nth(1)
            .and_then(|rest| rest.split('>').next())
            .unwrap_or_else(|| panic!("{name}: a field in {html}"));
        assert!(field.contains("data-kind=\"secret\""), "{name}: {field}");
        assert!(!field.contains("value="), "{name}: {field}");
    }
}

/// The markup says what the props say: the state, the shake on a wrong password, the caps
/// mark, the closed field and the lock-out line; the switcher's fit and selection; a leaving
/// tile's fold.
#[test]
fn the_markup_carries_the_props() {
    let wrong = render(SPECIMENS[4].1);
    for want in [
        "data-state=\"wrong\"",
        "class=\"ds-lock-field a-shake-x\"",
        "data-pulse=\"a\"",
        "aria-label=\"Caps Lock is on\"",
        "data-size=\"64\"",
    ] {
        assert!(wrong.contains(want), "{want} in {wrong}");
    }
    let idle = render(SPECIMENS[2].1);
    assert!(!idle.contains("a-shake-x"), "{idle}");
    assert!(!idle.contains("Caps Lock"), "{idle}");
    let checking = render(SPECIMENS[3].1);
    for want in ["aria-busy=\"true\"", "ds-spinner", "aria-disabled=\"true\""] {
        assert!(checking.contains(want), "{want} in {checking}");
    }
    let out = render(SPECIMENS[5].1);
    assert!(out.contains("Try again at 9:52"), "{out}");
    let polkit = render(SPECIMENS[11].1);
    assert!(
        polkit.contains("Too many tries. Try again at 9:52."),
        "{polkit}"
    );
    assert!(polkit.contains("data-width=\"narrow\""), "{polkit}");

    let scrolled = render(SPECIMENS[14].1);
    assert!(
        scrolled.contains(
            "--switcher-cell:64px;--switcher-icon:48px;--switcher-gap:8px;--switcher-view:704px;--switcher-shift:296px;--switcher-at:9"
        ),
        "{scrolled}"
    );
    assert_eq!(scrolled.matches("aria-selected=\"true\"").count(), 1);
    let shrunk = render(SPECIMENS[13].1);
    assert!(
        shrunk.contains("--switcher-cell:72px;--switcher-icon:56px"),
        "{shrunk}"
    );
    let leaving = render(SPECIMENS[15].1);
    assert!(
        leaving.contains("class=\"ds-switcher-cell a-fold\""),
        "{leaving}"
    );
    assert!(leaving.contains("data-presence=\"leaving\""), "{leaving}");
}

/// A photo is cropped round at the face's size (64 at the lock, 48 in the polkit sheet), with
/// no letter; a persona is drawn at Medium, idle at rest and happy once accepted.
#[test]
fn the_picture_is_the_kind_the_user_carries() {
    let by_name = |name: &str| {
        let make = SPECIMENS
            .iter()
            .find(|(named, _)| *named == name)
            .unwrap_or_else(|| panic!("a specimen {name}"))
            .1;
        render(make)
    };
    let photo = by_name("prompt-photo");
    for want in [
        "class=\"ds-user-photo\" data-size=\"64\"",
        "class=\"ds-user-photo-image\"",
        "src=\"data:image/png;base64,AAAA\"",
    ] {
        assert!(photo.contains(want), "{want} in {photo}");
    }
    assert!(!photo.contains("ds-avatar"), "{photo}");
    assert!(
        by_name("polkit-photo").contains("class=\"ds-user-photo\" data-size=\"48\""),
        "the polkit photo at 48"
    );
    let persona = by_name("prompt-persona");
    assert!(
        persona.contains("data-size=\"64\" data-mood=\"idle\""),
        "{persona}"
    );
    assert!(!persona.contains("ds-avatar"), "{persona}");
    let accepted = by_name("prompt-persona-accepted");
    assert!(accepted.contains("data-mood=\"happy\""), "{accepted}");
    assert!(accepted.contains("data-state=\"accepted\""), "{accepted}");
}
