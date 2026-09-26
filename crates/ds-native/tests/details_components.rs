//! design/26 D0b on the two components whose spinners looped forever (section 6): a module tile
//! that stays Busy, and a lock prompt that stays Checking, now step after the grace, hold their
//! still frame at `PendingCap` and paint 0 frames from then on, with nothing new from the caller
//! (the operation is derived from the state's own Pending moment).

use dioxus::prelude::*;
use ds::{
    Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Ds, Icon, LockPrompt, LockUser,
    Material, ModuleState, PromptState, person_hue,
};
use ds::{ModuleTile, Text};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Harness, Viewport};
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 360,
    height: 240,
    scale_percent: 100,
};

/// `PendingCap` and a little: the spinner holds still by then.
const PAST_CAP: Duration = Duration::from_millis(10_500);

static MODULE: GlobalSignal<ModuleState> = Signal::global(|| ModuleState::Off);
static PROMPT: GlobalSignal<PromptState> = Signal::global(|| PromptState::Idle);

#[allow(non_snake_case)]
fn Tile() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            div { style: "width:300px;padding:12px",
                ModuleTile { glyph: Icon::Wifi, title: Text::from("Wi-Fi"), status: None, state: MODULE(), onclick: |_| {} }
            }
        }
    }
}

fn pending(harness: &Harness) -> Option<String> {
    harness.attr(".ds-spinner", "data-pending")
}

/// Wait until the spinner steps, then until it holds still, and check it holds still no sooner
/// than the cap after `busy` and paints nothing after.
fn steps_then_holds(harness: &mut Harness, busy: Instant) {
    settle_until(harness, |h| pending(h).as_deref() == Some("step"));
    while busy.elapsed() < PAST_CAP && pending(harness).as_deref() != Some("still") {
        harness.advance(Duration::from_millis(250));
    }
    assert_eq!(
        pending(harness).as_deref(),
        Some("still"),
        "{}",
        harness.html()
    );
    assert!(
        busy.elapsed() >= Duration::from_secs(10),
        "held before the cap"
    );
    assert_settles_to_zero_frames(harness);
}

#[test]
fn a_busy_module_tile_stops_at_the_cap() {
    let mut harness = Harness::new(Tile, VIEW);
    harness.within(|| *MODULE.write() = ModuleState::Busy);
    let busy = Instant::now();
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        pending(&harness).as_deref(),
        Some("idle"),
        "no ring inside the grace"
    );
    steps_then_holds(&mut harness, busy);
    harness.within(|| *MODULE.write() = ModuleState::On);
    harness.advance(Duration::from_millis(30));
    assert_eq!(harness.count(".ds-spinner"), 0);
    assert_settles_to_zero_frames(&mut harness);
}

fn user() -> LockUser {
    LockUser::new(
        "Dana Reyes",
        AvatarFace {
            initial: 'D',
            size: AvatarSize::Size34,
            tone: AvatarTone::Person(person_hue("dana")),
            shape: AvatarShape::Round,
        },
    )
}

#[allow(non_snake_case)]
fn Lock() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            LockPrompt { user: user(), state: PROMPT(), oninput: |_| {}, onsubmit: |_| {} }
        }
    }
}

#[test]
fn a_checking_lock_prompt_stops_at_the_cap() {
    let mut harness = Harness::new(Lock, VIEW);
    harness.within(|| *PROMPT.write() = PromptState::Checking);
    let checking = Instant::now();
    steps_then_holds(&mut harness, checking);
    // The try fails: the ring goes, the field shakes once, and the prompt rests.
    harness.within(|| *PROMPT.write() = PromptState::Wrong);
    harness.advance(Duration::from_millis(30));
    assert_eq!(harness.count(".ds-spinner"), 0);
    assert_settles_to_zero_frames(&mut harness);
}
