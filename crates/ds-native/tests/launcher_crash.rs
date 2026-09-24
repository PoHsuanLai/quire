//! The two launcher crashes (sill FINDINGS Q43, Q45), reproduced on a real Blitz document.
//!
//! Q45: a motion hook's settle task outlived its component, so unmounting a palette or a menu
//! before its entrance settled set a dropped signal ("ValueDroppedError"). Q43: a field's
//! `Focus::OnMount` set the focus from a task polled inside the render pass, where the renderer
//! holds the document ("RefCell already borrowed"). Each test drives mounts and unmounts from a
//! script inside the page, on the harness's clock, and passes only if nothing panics.

use dioxus::prelude::*;
use ds::{
    Anchor, Anim, Appearance, Availability, CommandPalette, Ds, Material, Menu, MenuEntry,
    MenuKind, Motion, MotionLevel, Point, Px, StaggerIndex, Trail, settle, sleep,
};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 420,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn item(value: u8, title: &str) -> MenuEntry<u8> {
    MenuEntry::Item {
        value,
        title: title.to_string(),
        detail: None,
        tile: None,
        trail: Trail::None,
        check: None,
        availability: Availability::Enabled,
    }
}

fn entries() -> Vec<MenuEntry<u8>> {
    vec![item(1, "Open"), item(2, "Pause"), item(3, "Quit")]
}

/// What the script mounts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Subject {
    Palette,
    Menu,
}

/// One scripted step: wait, then show or hide the subject.
type Step = (u64, Shown);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shown {
    Yes,
    No,
}

/// A tiny component, remounted every tick: it keeps dioxus reusing freed scope slots, as the
/// launcher's result rows do. A task that outlived the palette is polled again only once its
/// old slot holds a live scope, and then it wrote to the palette's dropped signals.
#[component]
fn Churn(n: u32) -> Element {
    rsx! { i { "{n}" } }
}

/// A page that mounts `subject` and unmounts it on `script`'s schedule, counting mounts, while
/// the palette's results change under it every 16 ms (a re-render in the same turn as the
/// mount, which is when Q43 fired most) and small components come and go beside it.
#[component]
fn Scripted(subject: Subject, script: Vec<Step>, motion: Motion) -> Element {
    let mut shown = use_signal(|| Shown::No);
    let mut mounts = use_signal(|| 0u32);
    let mut tick = use_signal(|| 0u32);
    use_hook(move || {
        spawn(async move {
            for (wait, next) in script {
                sleep(ms(wait)).await;
                if next == Shown::Yes && *shown.peek() == Shown::No {
                    mounts += 1;
                }
                shown.set(next);
            }
        });
        spawn(async move {
            loop {
                sleep(ms(16)).await;
                tick += 1;
            }
        });
    });
    let count = tick() % 3 + 1;
    let rows: Vec<MenuEntry<u8>> = entries().into_iter().take(count as usize).collect();
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Window,
            p { class: "mounts", "{mounts}" }
            for n in 0..count {
                Churn { key: "{tick}-{n}", n }
            }
            if shown() == Shown::Yes {
                match subject {
                    Subject::Palette => rsx! {
                        CommandPalette::<u8> {
                            label: "Search".to_string(),
                            placeholder: "Search".to_string(),
                            query: String::new(),
                            tokens: Vec::new(),
                            groups: vec![("Results".to_string(), rows)],
                            empty: "Nothing".to_string(),
                            oninput: move |_| {},
                            onpick: move |_| {},
                            onclose: move |()| shown.set(Shown::No),
                        }
                    },
                    Subject::Menu => rsx! {
                        Menu::<u8> {
                            kind: MenuKind::Slim,
                            anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                            entries: rows,
                            onpick: move |_| {},
                            onclose: move |()| shown.set(Shown::No),
                        }
                    },
                }
            }
        }
    }
}

/// Mount at 10 ms, unmount 100 ms later: before any entrance (`peek-in`, `menu-pop`) settles.
fn early_unmount() -> Vec<Step> {
    vec![(10, Shown::Yes), (100, Shown::No)]
}

#[allow(non_snake_case)]
fn PaletteGoneEarly() -> Element {
    rsx! { Scripted { subject: Subject::Palette, script: early_unmount(), motion: Motion::Standard } }
}

#[allow(non_snake_case)]
fn MenuGoneEarly() -> Element {
    rsx! { Scripted { subject: Subject::Menu, script: early_unmount(), motion: Motion::Standard } }
}

/// `sill launcher toggle` pairs 500 ms apart: open, close, open, close. At Extra motion the
/// palette's entrance runs past the first close, so each close lands mid `peek-in`.
#[allow(non_snake_case)]
fn PaletteToggledTwice() -> Element {
    rsx! {
        Scripted {
            subject: Subject::Palette,
            script: vec![(10, Shown::Yes), (500, Shown::No), (500, Shown::Yes), (500, Shown::No)],
            motion: Motion::Extra,
        }
    }
}

/// Fifty mounts, each unmounted after a varying time (some before the entrance settles, some
/// after the field has focused).
#[allow(non_snake_case)]
fn PaletteFiftyTimes() -> Element {
    let script = (0..50u64)
        .flat_map(|round| {
            [
                (7 + round % 5, Shown::Yes),
                (20 + (round * 37) % 400, Shown::No),
            ]
        })
        .collect();
    rsx! { Scripted { subject: Subject::Palette, script, motion: Motion::Standard } }
}

fn mounts(harness: &Harness) -> String {
    harness.text_of(".mounts").unwrap_or_default()
}

/// How long `anim`'s entrance runs, asked of the motion table: the early unmount is inside it.
fn entrance(anim: Anim) -> Duration {
    settle(anim, MotionLevel::Standard, StaggerIndex::default())
}

// The harness's time is the wall clock (its module documentation says why), so a test reads
// what the script did from the page's own count rather than catching the palette mid-way.

#[test]
fn a_palette_unmounted_before_its_entrance_settles_does_not_panic() {
    assert!(entrance(Anim::PeekIn) > ms(110), "the unmount is early");
    let mut harness = Harness::new(PaletteGoneEarly, VIEW);
    harness.advance(entrance(Anim::PeekIn) + ms(400));
    assert_eq!(mounts(&harness), "1", "the palette mounted");
    assert_eq!(harness.count(".ds-palette"), 0, "and is gone");
}

#[test]
fn a_menu_unmounted_before_its_entrance_settles_does_not_panic() {
    assert!(entrance(Anim::MenuPop) > ms(110), "the unmount is early");
    let mut harness = Harness::new(MenuGoneEarly, VIEW);
    harness.advance(entrance(Anim::MenuPop) + ms(400));
    assert_eq!(mounts(&harness), "1", "the menu mounted");
    assert_eq!(harness.count(".ds-menu"), 0, "and is gone");
}

#[test]
fn a_palette_toggled_twice_half_a_second_apart_does_not_panic() {
    assert!(settle(Anim::PeekIn, MotionLevel::Extra, StaggerIndex::default()) > ms(510));
    let mut harness = Harness::new(PaletteToggledTwice, VIEW);
    harness.advance(ms(2200));
    assert_eq!(mounts(&harness), "2");
    assert_eq!(harness.count(".ds-palette"), 0);
}

#[test]
fn a_palette_mounted_fifty_times_does_not_panic() {
    let mut harness = Harness::new(PaletteFiftyTimes, VIEW);
    // The script's own length, and a margin for the last unmount.
    let total: u64 = (0..50u64)
        .map(|round| 7 + round % 5 + 20 + (round * 37) % 400)
        .sum();
    harness.advance(ms(total + 200));
    assert_eq!(mounts(&harness), "50");
    assert_eq!(harness.count(".ds-palette"), 0);
}
