//! SettingsRow: a settings-style list row for networks, devices and outputs, usable outside a
//! Menu: a glyph, a title and a detail line in `MenuEntry::Row`'s type (the
//! shell text menu's, design/13 section 13.3.3), and a trailing mark; 44 px high, a hairline
//! between rows. A control-center list and a menu then read alike.
//!
//! A `div[role=button]`, as `ModuleTile`: a toggle row holds a button of its own. Enter or
//! Space on the row runs `onclick` on both renderers (Blitz synthesises no click from a key).
//!
//! An operation on the row's item (design/26-DETAILS.md 5.2.2, 5.2.3, 5.2.8) is its `phase`:
//! pending, the glyph breathes or a spinner takes the trailing slot ([`RowWork`]); a success
//! seals the glyph's disc once or draws the check on; a failure shakes the row once.

use crate::components::content::text_runs::{TextLine, text};
use crate::components::controls::press::PressListeners;
use crate::components::controls::spinner::{SpinnerKind, ring};
use crate::components::lists::settings_row_phase::{RowDisc, RowPhase, RowWork};
use crate::components::lists::settings_row_trailing::{RowTrailing, trailing as trailing_mark};
use crate::core::press::Press;
use crate::core::vocab::{Availability, Switch};
use crate::motion::detail::first_show::FirstShow;
use crate::motion::detail::{
    armed::use_armed,
    once::use_shake,
    pending::PendingFrame,
    settle::{SettleStyle, Settling},
    touch::Touch,
    use_detail::use_detail,
    use_operation::use_operation,
    use_pending::use_pending,
    use_settle::use_settle,
};
use crate::style::icon::Icon;
use crate::style::icon::render::{Glyph, IconSize};
use dioxus::prelude::*;

/// One settings row. `onclick` hears a press on the row (a toggle's own press is the toggle's).
///
/// `phase` is the operation on the row's item ([`RowPhase`], default `Rest`), `work` where its
/// pending loop shows (default the glyph), `disc` the disc the glyph sits on (default none). A
/// press on the row is kept for the change it causes, so the success it leads to seals with the
/// spring and one from elsewhere does not (R5). `first` is `FirstShow::Animate` on a pane just
/// opened: a device's battery (`RowTrailing::Battery`) then sweeps in; one that arrives while the
/// row shows always does.
#[component]
pub fn SettingsRow(
    #[props(default)] glyph: Option<Icon>,
    #[props(into)] title: TextLine,
    detail: Option<TextLine>,
    #[props(default)] trailing: RowTrailing,
    #[props(default)] availability: Availability,
    onclick: EventHandler<Press>,
    #[props(default)] phase: RowPhase,
    #[props(default)] work: RowWork,
    #[props(default)] disc: RowDisc,
    #[props(default)] first: FirstShow,
) -> Element {
    let live = availability == Availability::Enabled;
    let listen = PressListeners::new(onclick);
    let armed = use_armed();
    let moment = use_detail(phase, FirstShow::Still, armed.touch());
    armed.spend(moment.cue());
    let frame = use_pending(use_operation(moment.cue()), work.spec());
    let settling = use_settle(moment.cue(), success_style(&trailing));
    let shake = use_shake(moment.cue()).attrs();
    let spinning = spinner_frame(work, frame);
    let arriving = arrival(use_mount(), first);
    let mark = match spinning {
        Some(frame) => pending_trail(frame),
        None => trailing_mark(&trailing, &title, availability, settling, arriving),
    };
    let (class, alias) = match shake {
        Some((anim, alias)) => (format!("ds-settings-row {anim}"), Some(alias)),
        None => ("ds-settings-row".to_owned(), None),
    };
    rsx! {
        div {
            class,
            "data-pulse": alias,
            role: "button",
            tabindex: "0",
            "data-trailing": trailing.slug(),
            "data-phase": phase_slug(phase),
            "aria-pressed": trailing.pressed(),
            "aria-busy": busy(phase),
            "aria-disabled": availability.aria_disabled(),
            onclick: move |event| {
                if live {
                    armed.arm(Touch::from_event(&event));
                    listen.click(&event);
                }
            },
            onkeydown: move |event| {
                let key = event.key();
                if live && (key == Key::Enter || key == Key::Character(" ".into())) {
                    event.prevent_default();
                    armed.arm(Touch::from_event(&event));
                    onclick.call(Press::primary());
                }
            },
            if let Some(icon) = glyph {
                {glyph_slot(icon, disc, glyph_frame(work, frame), settling)}
            }
            span { class: "ds-settings-row-words",
                span { class: "ds-settings-row-title ds-truncate", {text(&title)} }
                if let Some(detail) = detail {
                    span { class: "ds-settings-row-detail ds-truncate", {text(&detail)} }
                }
            }
            {mark}
        }
    }
}

/// Whether the row is drawing its first frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mount {
    Opening,
    Shown,
}

/// The row's mount: `Opening` on its first render, `Shown` after.
fn use_mount() -> Mount {
    let mut seen = use_hook(|| CopyValue::new(Mount::Opening));
    let now = *seen.peek();
    if now == Mount::Opening {
        seen.set(Mount::Shown);
    }
    now
}

/// How a part that mounts now first shows: as the row was opened, or, arriving on a row already
/// showing, as a value that just appeared.
fn arrival(mount: Mount, first: FirstShow) -> FirstShow {
    match mount {
        Mount::Opening => first,
        Mount::Shown => FirstShow::Animate,
    }
}

/// How a success lands on this row: the check draws on where the row ends in one, else the
/// glyph (its disc) seals.
fn success_style(trailing: &RowTrailing) -> SettleStyle {
    match trailing {
        RowTrailing::Check(Switch::On) => SettleStyle::Check,
        RowTrailing::None
        | RowTrailing::Check(Switch::Off)
        | RowTrailing::Toggle { .. }
        | RowTrailing::Chevron
        | RowTrailing::Text(_)
        | RowTrailing::Glyph(_)
        | RowTrailing::Battery(_) => SettleStyle::LockIn,
    }
}

/// The spinner's frame when the loop shows in the trailing slot and is past its grace.
fn spinner_frame(work: RowWork, frame: PendingFrame) -> Option<PendingFrame> {
    match (work, frame) {
        (RowWork::Trailing, PendingFrame::Step(_) | PendingFrame::Stalled) => Some(frame),
        (RowWork::Trailing, PendingFrame::Idle) | (RowWork::Glyph, _) => None,
    }
}

/// The glyph's breathing frame when the loop shows on the glyph.
fn glyph_frame(work: RowWork, frame: PendingFrame) -> PendingFrame {
    match work {
        RowWork::Glyph => frame,
        RowWork::Trailing => PendingFrame::Idle,
    }
}

/// The trailing slot while a spinner holds it: a 14 px well the ring turns around.
fn pending_trail(frame: PendingFrame) -> Element {
    rsx! {
        span { class: "ds-settings-row-trail", "data-mark": "pending",
            span { class: "ds-settings-row-spin", {ring(SpinnerKind::Spin, frame)} }
        }
    }
}

/// The glyph's column: the glyph on its disc, breathing while `frame` steps, sealing while
/// `settling` says.
fn glyph_slot(icon: Icon, disc: RowDisc, frame: PendingFrame, settling: Settling) -> Element {
    let beat = match frame {
        PendingFrame::Step(n) if n.is_multiple_of(2) => Some("high"),
        PendingFrame::Step(_) => Some("low"),
        PendingFrame::Idle | PendingFrame::Stalled => None,
    };
    let seal = match settling {
        Settling::Sealing(key) => key.attrs(),
        Settling::Rest | Settling::Filling(_) | Settling::Drawing(_) => None,
    };
    let (class, alias) = match seal {
        Some((anim, alias)) => (format!("ds-settings-row-glyph {anim}"), Some(alias)),
        None => ("ds-settings-row-glyph".to_owned(), None),
    };
    rsx! {
        span {
            class,
            "data-pulse": alias,
            "data-disc": disc.slug(),
            "data-pending": pending_slug(frame),
            "data-beat": beat,
            Glyph { icon, size: IconSize::Base }
        }
    }
}

/// The glyph's `data-pending` word: written only while a loop shows on it, so a row at rest
/// keeps its markup.
fn pending_slug(frame: PendingFrame) -> Option<&'static str> {
    match frame {
        PendingFrame::Idle => None,
        PendingFrame::Step(_) | PendingFrame::Stalled => Some(frame.slug()),
    }
}

/// The `data-phase` word: written only while something is happening.
fn phase_slug(phase: RowPhase) -> Option<&'static str> {
    match phase {
        RowPhase::Rest => None,
        RowPhase::Pending(_) => Some("pending"),
        RowPhase::Succeeded(_) => Some("succeeded"),
        RowPhase::Failed(_) => Some("failed"),
    }
}

/// `aria-busy`: while the operation runs.
fn busy(phase: RowPhase) -> Option<&'static str> {
    match phase {
        RowPhase::Pending(_) => Some("true"),
        RowPhase::Rest | RowPhase::Succeeded(_) | RowPhase::Failed(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{glyph_frame, spinner_frame, success_style};
    use crate::components::lists::settings_row_phase::RowWork;
    use crate::components::lists::settings_row_trailing::RowTrailing;
    use crate::core::vocab::Switch;
    use crate::motion::detail::{pending::PendingFrame, settle::SettleStyle};
    use crate::style::icon::Icon;

    #[test]
    fn a_success_draws_the_check_where_there_is_one_and_seals_the_glyph_else() {
        assert_eq!(
            success_style(&RowTrailing::Check(Switch::On)),
            SettleStyle::Check
        );
        assert_eq!(
            success_style(&RowTrailing::Glyph(Icon::Lock)),
            SettleStyle::LockIn
        );
        assert_eq!(success_style(&RowTrailing::None), SettleStyle::LockIn);
    }

    #[test]
    fn the_loop_shows_where_the_row_says_and_only_past_its_grace() {
        let step = PendingFrame::Step(1);
        assert_eq!(spinner_frame(RowWork::Trailing, step), Some(step));
        assert_eq!(spinner_frame(RowWork::Trailing, PendingFrame::Idle), None);
        assert_eq!(spinner_frame(RowWork::Glyph, step), None);
        assert_eq!(glyph_frame(RowWork::Glyph, step), step);
        assert_eq!(glyph_frame(RowWork::Trailing, step), PendingFrame::Idle);
    }
}
