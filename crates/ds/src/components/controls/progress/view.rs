//! ProgressIndicator (design/30 section 2.9, `NSProgressIndicator`): a bar, a spoke spinner or
//! a ring. Markup: `div.ds-progress[data-style][data-size]` with `role="progressbar"`; a known
//! share is `--f` on the fill and `aria-valuenow`; an operation without a known share is
//! `data-pending` (`step` while it runs) and the loop's step as `--step` on the root, so nothing turns without an
//! [`Operation`](ds_motion::detail::operation::Operation).

use super::arc::{RingSpan, arc_path};
use super::model::{Progress, ProgressStyle};
use super::spokes::{SPIN, age, step_of};
use crate::components::content::icon_source::IconSource;
use crate::components::content::icon_view::IconView;
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Fraction;
use ds_core::word::Word;
use ds_motion::detail::operation::Operation;
use ds_motion::detail::pending::{PendingFrame, SPIN_STEPS};
use ds_motion::detail::tween::{TweenSpec, use_tween};
use ds_motion::detail::use_pending::use_pending;
use ds_style::icon::render::IconSize;
use ds_style::tokens::control_size::ControlSize;
use ds_style::tokens::{easing::EasingToken, timing::DurationToken};

/// How a determinate value follows its target: linearly over `--t-move`, retargeting from where
/// it stands (design/30 section 1.3, Progress sweep).
const FOLLOW: TweenSpec = TweenSpec {
    duration: DurationToken::Move,
    easing: EasingToken::Linear,
};

/// Progress: `progress` drawn as `style` at `size`. A `Ring` takes an optional centre `glyph`.
///
/// A `Spinner` has no known amount, so it takes `Progress::Unknown` and draws nothing while the
/// operation is idle; a `Bar` or `Ring` given `Unknown` is the barber pole or the turning arc.
/// `common.aria_label` names the work it shows.
#[component]
pub fn ProgressIndicator(
    #[props(default)] style: ProgressStyle,
    progress: Progress,
    #[props(default)] size: ControlSize,
    #[props(default)] glyph: Option<IconSource>,
    #[props(default)] common: Common,
) -> Element {
    let (share, operation) = match progress {
        Progress::Known(share) => (share.clamped(), Operation::Idle),
        Progress::Unknown(operation) => (Fraction(0), operation),
    };
    let shown = use_tween(share, FOLLOW);
    let frame = use_pending(operation, SPIN);
    let known = matches!(progress, Progress::Known(_));
    let running = matches!(frame, PendingFrame::Step(_));
    let class = common.class("ds-progress");
    let data = common.data_attributes();
    rsx! {
        div {
            id: common.id.clone(),
            class,
            role: "progressbar",
            "data-style": style.slug(),
            "data-size": size.slug(),
            "data-pending": if known { None } else { Some(frame.slug()) },
            "aria-label": common.aria_label.clone(),
            "aria-valuemin": if known { Some("0") } else { None },
            "aria-valuemax": if known { Some("100") } else { None },
            "aria-valuenow": progress.valuenow().map(|now| now.to_string()),
            "aria-busy": if running { Some("true") } else { None },
            style: step_of(frame).map(|step| format!("--step:{step}")),
            onmounted: move |event| common.mounted(event),
            ..data,
            match style {
                ProgressStyle::Bar => bar(shown, frame),
                ProgressStyle::Spinner => spinner(frame),
                ProgressStyle::Ring => ring(shown, frame, glyph),
            }
        }
    }
}

/// The bar: a track and its fill. A known share is the fill's width; a running operation fills
/// the track with stripes that slide one step at a time.
fn bar(shown: Fraction, frame: PendingFrame) -> Element {
    let step = step_of(frame);
    rsx! {
        div { class: "ds-progress-track",
            div {
                class: "ds-progress-fill",
                style: if step.is_some() { None } else { Some(format!("--f:{}", shown.css())) },
            }
        }
    }
}

/// The twelve spokes, each at its age behind the lit one.
fn spinner(frame: PendingFrame) -> Element {
    rsx! {
        for spoke in 0..SPIN_STEPS {
            span {
                key: "{spoke}",
                class: "ds-progress-indicator",
                "data-spoke": "{spoke}",
                "data-age": "{age(frame, spoke)}",
                "aria-hidden": "true",
            }
        }
    }
}

/// The ring: a full track and the arc over it, from twelve clockwise. Unknown, the arc is a
/// quarter turned by the loop's step.
fn ring(shown: Fraction, frame: PendingFrame, glyph: Option<IconSource>) -> Element {
    let span = match frame {
        PendingFrame::Step(_) => RingSpan::FULL.filled(Fraction(250)),
        PendingFrame::Idle => RingSpan::FULL.filled(shown),
    };
    let arc = arc_path(span);
    let track = arc_path(RingSpan::FULL);
    rsx! {
        svg { class: "ds-progress-track", "data-ds-svg": "progress", view_box: "0 0 100 100", "aria-hidden": "true",
            if let Some(d) = track {
                path {
                    d,
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "9.3",
                }
            }
        }
        if let Some(d) = arc {
            svg {
                class: "ds-progress-fill",
                "data-ds-svg": "progress",
                view_box: "0 0 100 100",
                "aria-hidden": "true",
                path {
                    d,
                    fill: "none",
                    stroke: "currentColor",
                    "stroke-width": "9.3",
                    "stroke-linecap": "round",
                }
            }
        }
        if let Some(source) = glyph {
            span { class: "ds-progress-glyph",
                IconView { source, size: IconSize::Compact }
            }
        }
    }
}
