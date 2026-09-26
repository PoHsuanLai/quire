//! design/26 D0 on a real Blitz document, for the primitives the other details tests leave out:
//! a `LayerGlyph` searching then filling once (Pending, Settle(Fill)), a `CheckMark` drawing on,
//! a seal that springs only on contact (Settle(LockIn), R5), and the OffUp and CrossFade morphs.
//! Each ends at 0 frames (R3).

use dioxus::prelude::*;
use ds::detail::{
    CheckMark, Detailed, EventStamp, FirstShow, LayerGlyph, Layering, Layers, Moment, MorphGlyph,
    MorphStyle, PendingSpec, PendingStyle, SettleStyle, Settling, Touch, use_detail, use_operation,
    use_pending, use_settle,
};
use ds::{Appearance, Ds, Icon, IconSize, Material};
use ds_native::harness::{assert_settles_to_zero_frames, settle_until};
use ds_native::{Harness, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 240,
    height: 200,
    scale_percent: 100,
};

/// A Wi-Fi item's state.
#[derive(Debug, Clone, PartialEq)]
enum Net {
    Off,
    Joining,
    Joined,
}

impl Detailed for Net {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Net::Off | Net::Joining | Net::Joined, Net::Joining) => Moment::Pending,
            (Net::Joining, Net::Joined) => Moment::Success,
            (Net::Off | Net::Joined, Net::Joined) => Moment::Change,
            (Net::Off | Net::Joining | Net::Joined, Net::Off) => Moment::Unavailable,
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Net::Joining => Moment::Pending,
            Net::Off | Net::Joined => Moment::Rest,
        }
    }
}

/// A seal, sealed once per stamp.
#[derive(Debug, Clone, PartialEq)]
enum Seal {
    Open,
    Sealed(EventStamp),
}

impl Detailed for Seal {
    fn moment(from: &Self, to: &Self) -> Moment {
        match (from, to) {
            (Seal::Open | Seal::Sealed(_), Seal::Sealed(_)) => Moment::Success,
            (Seal::Open | Seal::Sealed(_), Seal::Open) => Moment::Change,
        }
    }
    fn first(state: &Self) -> Moment {
        match state {
            Seal::Open | Seal::Sealed(_) => Moment::Rest,
        }
    }
}

const SEARCHING: PendingSpec = PendingSpec {
    style: PendingStyle::Iterate,
    layers: Layers(4),
};

static NET: GlobalSignal<Net> = Signal::global(|| Net::Off);
static SEAL: GlobalSignal<(Seal, Touch)> = Signal::global(|| (Seal::Open, Touch::Remote));
static ICON: GlobalSignal<Icon> = Signal::global(|| Icon::Play);

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            Wifi {}
            Check {}
            SealDisc {}
            div { id: "offup", MorphGlyph { icon: ICON(), size: IconSize::Base, style: MorphStyle::OffUp } }
            div { id: "fade", MorphGlyph { icon: ICON(), size: IconSize::Base, style: MorphStyle::CrossFade } }
        }
    }
}

#[allow(non_snake_case)]
#[component]
fn Wifi() -> Element {
    let detail = use_detail(NET(), FirstShow::Still, Touch::Remote);
    let frame = use_pending(use_operation(detail.cue()), SEARCHING);
    let layering = match use_settle(detail.cue(), SettleStyle::Fill(Layers(4))) {
        Settling::Filling(upto) => Layering::Filling(upto),
        Settling::Rest | Settling::Drawing(_) | Settling::Sealing(_) => {
            Layering::Pending(frame, SEARCHING)
        }
    };
    rsx! { div { id: "wifi", LayerGlyph { icon: Icon::Wifi, size: IconSize::Base, layering } } }
}

#[allow(non_snake_case)]
#[component]
fn Check() -> Element {
    let detail = use_detail(NET(), FirstShow::Still, Touch::Remote);
    let settling = use_settle(detail.cue(), SettleStyle::Check);
    rsx! { div { id: "check", CheckMark { settling, size: IconSize::Base } } }
}

#[allow(non_snake_case)]
#[component]
fn SealDisc() -> Element {
    let (seal, touch) = SEAL();
    let detail = use_detail(seal, FirstShow::Still, touch);
    let key = match use_settle(detail.cue(), SettleStyle::LockIn) {
        Settling::Sealing(key) => key.attrs(),
        Settling::Rest | Settling::Filling(_) | Settling::Drawing(_) => None,
    };
    let class = key.as_ref().map(|(class, _)| class.clone());
    rsx! {
        div {
            id: "seal",
            style: "width:40px;height:40px",
            class,
            "data-pulse": key.map(|(_, alias)| alias),
            onclick: move |event| {
                let next = match SEAL.peek().0 {
                    Seal::Sealed(EventStamp(n)) => n + 1,
                    Seal::Open => 1,
                };
                *SEAL.write() = (Seal::Sealed(EventStamp(next)), Touch::from_event(&event));
            },
        }
    }
}

fn lit(harness: &Harness) -> usize {
    harness.count("#wifi .ds-layer-part[*|data-lit=on]")
}

#[test]
fn a_layer_glyph_searches_one_layer_at_a_time_then_fills_once_and_rests() {
    let mut harness = Harness::new(Page, VIEW);
    assert_eq!(harness.count("#wifi .ds-layer-part"), 4);
    assert_eq!(lit(&harness), 4, "at rest the glyph is whole");
    harness.within(|| *NET.write() = Net::Joining);
    settle_until(&mut harness, |h| lit(h) == 1);
    harness.within(|| *NET.write() = Net::Joined);
    // The fill lights the layers from the dot out, once, then the glyph is whole again.
    settle_until(&mut harness, |h| (2..4).contains(&lit(h)));
    settle_until(&mut harness, |h| lit(h) == 4);
    assert_settles_to_zero_frames(&mut harness);
    assert_eq!(lit(&harness), 4);
}

#[test]
fn a_check_mark_draws_on_holds_and_goes() {
    let mut harness = Harness::new(Page, VIEW);
    assert_eq!(harness.count("#check svg"), 0, "no check before a success");
    harness.within(|| *NET.write() = Net::Joining);
    harness.advance(Duration::from_millis(20));
    harness.within(|| *NET.write() = Net::Joined);
    let offset = |h: &Harness| {
        h.attr("#check path", "stroke-dashoffset")
            .and_then(|value| value.parse::<f32>().ok())
    };
    settle_until(&mut harness, |h| {
        offset(h).is_some_and(|o| o > 0.0 && o < 22.0)
    });
    settle_until(&mut harness, |h| offset(h) == Some(0.0));
    settle_until(&mut harness, |h| h.count("#check svg") == 0);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn a_seal_springs_only_when_it_was_pressed() {
    let mut harness = Harness::new(Page, VIEW);
    harness.within(|| *SEAL.write() = (Seal::Sealed(EventStamp(1)), Touch::Remote));
    settle_until(&mut harness, |h| h.has_class("#seal", "a-seal-out"));
    assert!(
        !harness.has_class("#seal", "a-gulp"),
        "a remote seal sprang"
    );
    settle_until(&mut harness, |h| !h.has_class("#seal", "a-seal-out"));
    assert_settles_to_zero_frames(&mut harness);

    let at = harness.centre("#seal").expect("the seal is laid out");
    harness.click(at);
    settle_until(&mut harness, |h| h.has_class("#seal", "a-gulp"));
    settle_until(&mut harness, |h| !h.has_class("#seal", "a-gulp"));
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn off_up_and_cross_fade_morphs_end_on_one_still_layer() {
    let mut harness = Harness::new(Page, VIEW);
    harness.within(|| *ICON.write() = Icon::Pause);
    settle_until(&mut harness, |h| {
        h.has_class("#offup [*|data-morph=in]", "a-morph-in")
            && h.has_class("#fade [*|data-morph=in]", "a-morph-fade-in")
    });
    settle_until(&mut harness, |h| {
        h.count("#offup .ds-morph-layer") == 1 && h.count("#fade .ds-morph-layer") == 1
    });
    assert_settles_to_zero_frames(&mut harness);
}
