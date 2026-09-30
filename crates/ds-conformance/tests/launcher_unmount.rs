//! The two launcher crashes, reproduced on a real Blitz document.
//!
//! A motion hook's settle task outlived its component, so unmounting a palette or a menu
//! before its entrance settled set a dropped signal ("ValueDroppedError"). A field's
//! `Focus::OnMount` set the focus from a task polled inside the render pass, where the renderer
//! holds the document ("RefCell already borrowed"). Each test drives mounts and unmounts from a
//! script inside the page, on the harness's clock, and passes only if nothing panics.

use dioxus::prelude::*;
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteRow};
use ds::host::measure::Anchor;
use ds::prelude::*;
use ds_core::time::clock::sleep;
use ds_harness::{Driver, Harness, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 640,
    height: 420,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

/// The titles both subjects list.
const TITLES: [&str; 3] = ["Open", "Pause", "Quit"];

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
/// mount, which is when that crash fired most) and small components come and go beside it.
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
    let taken = TITLES.iter().zip(1u8..).take(count as usize);
    let rows: Vec<MenuItem<u8>> = taken
        .clone()
        .map(|(title, value)| MenuItem::new(value, *title))
        .collect();
    let palette_rows: Vec<PaletteRow<u8>> = taken
        .map(|(title, value)| PaletteRow::new(value, *title))
        .collect();
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
                            groups: vec![PaletteGroup::list("Results", palette_rows)],
                            empty: "Nothing".to_string(),
                            oninput: move |_| {},
                            onpick: move |_| {},
                            onclose: move |()| shown.set(Shown::No),
                        }
                    },
                    Subject::Menu => rsx! {
                        Menu::<u8> {
                            placement: MenuPlacement::Popup,
                            anchor: Anchor::Point(Point { x: Px(40.0), y: Px(60.0) }),
                            items: rows,
                            onpick: move |_| {},
                            onclose: move |()| shown.set(Shown::No),
                        }
                    },
                }
            }
        }
    }
}

/// Mount at 10 ms, unmount 100 ms later: before the palette's entrance (`peek-in`) settles.
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

/// `sill launcher toggle` pairs 200 ms apart: open, close, open, close. The palette's entrance
/// runs past each close, so each close lands mid `peek-in`.
#[allow(non_snake_case)]
fn PaletteToggledTwice() -> Element {
    rsx! {
        Scripted {
            subject: Subject::Palette,
            script: vec![(10, Shown::Yes), (200, Shown::No), (200, Shown::Yes), (200, Shown::No)],
            motion: Motion::Standard,
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

/// Let the script run for `time`, then on until it has mounted `want` times and the subject is
/// gone, for at most as long again: a loaded machine stretches every wait the script makes.
fn run_script(harness: &mut Harness, time: Duration, want: &str, subject: &str) {
    harness.advance(time);
    for _ in 0..(time.as_millis() / 100) {
        if mounts(harness) == want && harness.count(subject) == 0 {
            return;
        }
        harness.advance(ms(100));
    }
}

/// How long `anim`'s entrance runs, asked of the motion table: the early unmount is inside it.
fn entrance(anim: Anim) -> Duration {
    settle(anim, MotionLevel::Standard)
}

// The harness's time is the wall clock (its module documentation says why), so a test reads
// what the script did from the page's own count rather than catching the palette mid-way.

#[test]
fn a_palette_unmounted_before_its_entrance_settles_does_not_panic() {
    assert!(entrance(Anim::PeekIn) > ms(110), "the unmount is early");
    let mut harness = Harness::new(PaletteGoneEarly, VIEW);
    run_script(
        &mut harness,
        entrance(Anim::PeekIn) + ms(400),
        "1",
        ".ds-palette",
    );
    assert_eq!(mounts(&harness), "1", "the palette mounted");
    assert_eq!(harness.count(".ds-palette"), 0, "and is gone");
}

#[test]
fn a_menu_unmounted_before_its_fade_settles_does_not_panic() {
    let mut harness = Harness::new(MenuGoneEarly, VIEW);
    run_script(
        &mut harness,
        settle(Anim::MenuOut, MotionLevel::Standard) + ms(400),
        "1",
        ".ds-menu",
    );
    assert_eq!(mounts(&harness), "1", "the menu mounted");
    assert_eq!(harness.count(".ds-menu"), 0, "and is gone");
}

#[test]
fn a_palette_toggled_twice_within_its_entrance_does_not_panic() {
    assert!(settle(Anim::PeekIn, MotionLevel::Standard) > ms(210));
    let mut harness = Harness::new(PaletteToggledTwice, VIEW);
    run_script(&mut harness, ms(2200), "2", ".ds-palette");
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
    run_script(&mut harness, ms(total + 200), "50", ".ds-palette");
    assert_eq!(mounts(&harness), "50");
    assert_eq!(harness.count(".ds-palette"), 0);
}
