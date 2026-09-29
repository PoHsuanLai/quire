//! On a real Blitz document, on the virtual clock: a status glyph in the bar's
//! status item (same box, ink, pill and label as an icon, sized by the setting), the low-battery
//! nudge inside the item lifting the glyph alone (design/26), the control center's panel
//! header and tile disc holding a status glyph (the battery's fill sweeping in on a surface just
//! opened), and the Sound module's level reading the bar's `VolumeState`. Each motion ends at 0
//! frames.

use dioxus::prelude::*;
use ds::detail::{Detailed, EventStamp, FirstShow, Moment, Touch, use_detail};
use ds::{
    Appearance, BatteryPower, BatteryState, Ds, Fraction, Icon, IconButton, IconButtonVariant,
    IconSource, LevelControl, LevelGlyph, LowAt, Material, ModulePanel, ModuleState, ModuleTile,
    Muting, Px, StatusMetrics, StatusState, VolumeState, VolumeWaves, WifiBars, WifiReach,
    WifiState,
};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Clock, Harness, HarnessConfig, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn virtual_harness(app: fn() -> Element) -> Harness {
    Harness::with_config(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual))
}

fn battery(level: u16, power: BatteryPower) -> BatteryState {
    BatteryState {
        level: Fraction(level),
        power,
        low_at: LowAt(Fraction(200)),
    }
}

/// The bar's metrics under test: a 26 px box holding an 18 px glyph, so a glyph drawn at its own
/// written size (16) or at the box would both be caught.
const METRICS: StatusMetrics = StatusMetrics {
    box_size: Px(26.0),
    glyph: Px(18.0),
};

// ---- a status glyph in the bar's status item -------------------------------------------

static ITEM: GlobalSignal<StatusState> = Signal::global(|| {
    StatusState::Wifi(WifiState::Joined {
        bars: WifiBars::Two,
        reach: WifiReach::Internet,
    })
});

#[allow(non_snake_case)]
fn Bar() -> Element {
    let status = ITEM();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "bar", style: METRICS.style_attr(),
                IconButton {
                    variant: IconButtonVariant::Status,
                    icon: status,
                    label: status.words(),
                    onclick: |_| {},
                }
            }
        }
    }
}

const ITEM_SEL: &str = "#bar .ds-icon-button";

fn side(harness: &Harness, selector: &str) -> (f32, f32) {
    let rect = harness
        .rect(selector)
        .unwrap_or_else(|| panic!("{selector} is missing"));
    (rect.size.width.0, rect.size.height.0)
}

#[test]
fn a_status_glyph_fills_the_status_items_glyph_square_with_its_label() {
    let mut harness = virtual_harness(Bar);
    let glyph = format!("{ITEM_SEL} > .ds-status-glyph[*|data-kind=wifi]");
    assert_eq!(harness.count(&glyph), 1, "{}", harness.html());
    assert_eq!(
        side(&harness, ITEM_SEL),
        (30.0, 26.0),
        "the item's box: the bar's 30 wide slot"
    );
    assert_eq!(side(&harness, &glyph), (18.0, 18.0), "the setting's glyph");
    let part = format!("{glyph} > .ds-status-part[*|data-part=arc-3]");
    assert_eq!(side(&harness, &part), (18.0, 18.0), "each part fills it");
    let words = harness.within(|| ITEM.peek().words());
    assert_eq!(harness.attr(ITEM_SEL, "aria-label"), Some(words));
    assert_eq!(
        harness.attr(ITEM_SEL, "data-variant").as_deref(),
        Some("status")
    );

    // The volume glyph sizes itself inline; the item's square still wins.
    harness.within(|| *ITEM.write() = StatusState::Volume(VolumeState::Heard(VolumeWaves::Two)));
    harness.advance(ms(0));
    let volume = format!("{ITEM_SEL} > .ds-status-glyph[*|data-kind=volume]");
    assert_eq!(side(&harness, &volume), (18.0, 18.0), "{}", harness.html());
    let wave = format!("{volume} .ds-level-part[*|data-part=wave-1]");
    assert_eq!(side(&harness, &wave), (18.0, 18.0));
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn the_status_item_plays_the_glyphs_own_moments() {
    let mut harness = virtual_harness(Bar);
    harness.within(|| *ITEM.write() = StatusState::Wifi(WifiState::Idle));
    harness.advance(ms(0));
    let arc = |h: &Harness| {
        h.attr(
            &format!("{ITEM_SEL} .ds-status-part[*|data-part=arc-3]"),
            "data-show",
        )
    };
    assert_eq!(arc(&harness).as_deref(), Some("faint"));
    assert_settles_to_zero_frames(&mut harness);
    // Joining runs the searching loop after its grace, one layer lit at a time.
    harness.within(|| *ITEM.write() = StatusState::Wifi(WifiState::Joining(EventStamp(1))));
    harness.advance(ms(1_000));
    let lit = harness.count(&format!("{ITEM_SEL} .ds-status-part[*|data-show=lit]"));
    assert_eq!(lit, 1, "one layer lit while searching: {}", harness.html());
    harness.within(|| {
        *ITEM.write() = StatusState::Wifi(WifiState::Joined {
            bars: WifiBars::Three,
            reach: WifiReach::Internet,
        })
    });
    settle_until(&mut harness, |h| arc(h).as_deref() == Some("lit"));
    assert_settles_to_zero_frames(&mut harness);
}

// ---- the nudge inside the item ----------------------------------------------------------

/// A test's own low-battery watch: crossing into Low is the Attention (sill's `LowWatch`).
#[derive(Debug, Clone, Copy, PartialEq)]
enum Watch {
    Clear,
    Low,
}

impl Detailed for Watch {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Watch::Clear, Watch::Low) => Moment::Attention,
            (Watch::Clear | Watch::Low, Watch::Clear) | (Watch::Low, Watch::Low) => Moment::Rest,
        }
    }

    fn first(state: &Self) -> Moment {
        match state {
            Watch::Clear | Watch::Low => Moment::Rest,
        }
    }
}

static WATCH: GlobalSignal<Watch> = Signal::global(|| Watch::Clear);

#[allow(non_snake_case)]
fn NudgedBar() -> Element {
    let watch = WATCH();
    let cue = use_detail(watch, FirstShow::Still, Touch::Remote).cue();
    let state = match watch {
        Watch::Clear => battery(400, BatteryPower::Battery),
        Watch::Low => battery(190, BatteryPower::Battery),
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "bar", style: METRICS.style_attr(),
                IconButton {
                    variant: IconButtonVariant::Status,
                    icon: StatusState::Battery(state),
                    label: state.words(),
                    onclick: |_| {},
                    nudge: Some(cue),
                }
            }
        }
    }
}

const NUDGE: &str = "#bar .ds-icon-button > .ds-icon-nudge";

fn nudging(harness: &Harness) -> bool {
    harness.has_class(NUDGE, "a-nudge-up")
}

#[test]
fn a_new_attention_cue_lifts_the_glyph_once_and_never_the_pill() {
    let mut harness = virtual_harness(NudgedBar);
    let glyph = format!("{NUDGE} > .ds-status-glyph[*|data-kind=battery]");
    assert_eq!(harness.count(&glyph), 1, "{}", harness.html());
    assert_eq!(
        side(&harness, &glyph),
        (18.0, 18.0),
        "the nudge keeps the square"
    );
    assert!(!nudging(&harness), "no nudge on the first frame (R1)");
    harness.within(|| *WATCH.write() = Watch::Low);
    harness.advance(ms(0));
    assert!(
        nudging(&harness),
        "crossing into low nudges: {}",
        harness.html()
    );
    assert!(
        !harness.has_class("#bar .ds-icon-button", "a-nudge-up"),
        "the pill stays"
    );
    assert_settles_to_zero_frames(&mut harness);
    assert!(!nudging(&harness), "the nudge ends");
    // The same state rendered again replays nothing; a new crossing nudges once more.
    harness.within(|| *WATCH.write() = Watch::Low);
    harness.advance(ms(0));
    assert!(!nudging(&harness));
    harness.within(|| *WATCH.write() = Watch::Clear);
    harness.advance(ms(0));
    assert!(!nudging(&harness), "leaving low is no attention");
    harness.within(|| *WATCH.write() = Watch::Low);
    harness.advance(ms(0));
    assert!(nudging(&harness), "a second crossing nudges again");
    assert_settles_to_zero_frames(&mut harness);
}

// ---- the control center's slots ------------------------------------------------------

#[allow(non_snake_case)]
fn Center() -> Element {
    let status = StatusState::Battery(battery(930, BatteryPower::Battery));
    let wifi = StatusState::Wifi(WifiState::Joined {
        bars: WifiBars::Three,
        reach: WifiReach::NoInternet,
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "panel",
                ModulePanel { glyph: IconSource::Status(status), title: "Battery", first: FirstShow::Animate,
                    p { "93 %" }
                }
            }
            div { id: "tile",
                ModuleTile {
                    glyph: wifi,
                    title: "Wi-Fi",
                    status: Some(wifi.words().into()),
                    state: ModuleState::On,
                    onclick: |_| {},
                    first: FirstShow::Animate,
                }
            }
            div { id: "plain",
                ModuleTile { glyph: Icon::Moon, title: "Focus", status: None, state: ModuleState::Off, onclick: |_| {} }
            }
        }
    }
}

/// The battery fill's drawn width on the 24 grid, 0 when there is none.
fn fill(harness: &Harness) -> f32 {
    harness
        .attr("#panel [*|data-part=fill] rect", "width")
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(0.0)
}

#[test]
fn the_panel_header_sweeps_the_battery_in_and_the_disc_holds_the_wifi_glyph() {
    let mut harness = virtual_harness(Center);
    assert_eq!(
        harness.count("#panel .ds-module-panel-glyph > .ds-status-glyph[*|data-kind=battery]"),
        1,
        "{}",
        harness.html()
    );
    // An Appear sweeps the fill in from empty over `--t-sweep`, not all at once.
    let start = fill(&harness);
    assert!(start < 1.0, "the fill starts empty: {start}");
    harness.advance(ms(200));
    let midway = fill(&harness);
    assert!(midway > start && midway < 9.5, "midway: {midway}");
    harness.advance(ms(1_000));
    assert!(
        (fill(&harness) - 10.0).abs() < 0.6,
        "full: {}",
        fill(&harness)
    );
    let disc = "#tile .ds-module-disc > .ds-status-glyph[*|data-kind=wifi]";
    assert_eq!(harness.count(disc), 1, "{}", harness.html());
    assert_eq!(
        harness.attr(
            &format!("{disc} > .ds-status-part[*|data-part=badge]"),
            "data-show"
        ),
        Some("lit".to_owned()),
        "the no-internet badge"
    );
    assert_eq!(side(&harness, disc), (16.0, 16.0), "the disc's glyph size");
    assert_eq!(
        harness.count("#plain .ds-module-disc > svg.ds-ic"),
        1,
        "an Icon still draws"
    );
    assert_settles_to_zero_frames(&mut harness);
}

// ---- the Sound module reads the bar's VolumeState ------------------------------------

static VOLUME: GlobalSignal<VolumeState> = Signal::global(|| VolumeState::Muted);

#[allow(non_snake_case)]
fn Sound() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { id: "shared", style: "width: 300px",
                LevelControl { label: "Volume", value: Fraction(500), glyph: VOLUME(), onchange: |_| {} }
            }
            div { id: "own", style: "width: 300px",
                LevelControl { label: "Output", value: Fraction(500), glyph: LevelGlyph::Volume(Muting::Audible), onchange: |_| {} }
            }
        }
    }
}

fn part_on(harness: &Harness, root: &str, part: &str) -> Option<String> {
    harness.attr(
        &format!("{root} .ds-level-in .ds-level-part[*|data-part={part}]"),
        "data-on",
    )
}

#[test]
fn a_level_given_a_volume_state_draws_that_states_waves_and_slash() {
    let mut harness = virtual_harness(Sound);
    // Muted at a level of 50 %: the slash, no waves, as the bar's glyph draws it.
    assert_eq!(part_on(&harness, "#shared", "slash").as_deref(), Some("on"));
    assert_eq!(
        part_on(&harness, "#shared", "wave-1").as_deref(),
        Some("off")
    );
    // A LevelGlyph still follows the value: two waves at 50 %.
    assert_eq!(part_on(&harness, "#own", "wave-2").as_deref(), Some("on"));
    assert_eq!(part_on(&harness, "#own", "slash").as_deref(), Some("off"));
    harness.within(|| *VOLUME.write() = VolumeState::Heard(VolumeWaves::Three));
    harness.advance(ms(0));
    assert_eq!(
        part_on(&harness, "#shared", "slash").as_deref(),
        Some("off")
    );
    assert_eq!(
        part_on(&harness, "#shared", "wave-3").as_deref(),
        Some("on"),
        "the state's three waves, whatever the capsule's 50 %"
    );
    assert_eq!(
        harness
            .attr("#shared .ds-level", "aria-valuenow")
            .as_deref(),
        Some("50"),
        "the capsule still shows the value"
    );
    assert_settles_to_zero_frames(&mut harness);
}
