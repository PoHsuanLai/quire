//! Settle as a hook: a success lands once per new Success cue, then rests (design/26 R3, R14).

use super::cue::Cue;
use super::level::{Level, use_level};
use super::moment::Moment;
use super::settle::{SettleStyle, Settling};
use super::touch::Touch;
use crate::core::task::{Gone, spawn_in, try_get, try_set};
use crate::core::time::clock::sleep;
use crate::core::vocab::Fraction;
use crate::core::vocab::StaggerIndex;
use crate::motion::pulse_key::PulseKey;
use crate::motion::timeline::glide::Glide;
use crate::motion::timeline::playback::tick;
use crate::motion::{anim::Anim, settle::settle};
use crate::style::appearance::motion::MotionLevel;
use crate::style::tokens::{delay::DelayToken, easing::EasingToken, timing::DurationToken};
use dioxus::core::{Task, current_scope_id, queue_effect};
use dioxus::prelude::*;

/// What a success cue settles as this frame: `Rest` until a new Success cue, then the style's
/// landing once (Fill steps its layers, Check draws and holds `SettleHold`, LockIn seals with the
/// spring only on `Touch::Contact`), then `Rest` again. Under Reduced Fill and LockIn show their
/// final state at once, and Check shows the whole check for its hold (R7).
pub fn use_settle(cue: Cue, style: SettleStyle) -> Settling {
    let lander = Lander {
        now: use_signal(|| Settling::Rest),
        task: use_signal(|| None),
        env: use_level(),
        scope: use_hook(current_scope_id),
    };
    let mut seen = use_hook(|| CopyValue::new(cue.serial()));
    if *seen.peek() != cue.serial() {
        seen.set(cue.serial());
        queue_effect(move || {
            let _ = lander.land(cue, style);
        });
    }
    (lander.now)()
}

/// The frame a success is at and the timer that moves it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Lander {
    now: Signal<Settling>,
    task: Signal<Option<Task>>,
    env: Level,
    scope: ScopeId,
}

impl Lander {
    /// Stop the last landing; start this cue's if it is a success.
    fn land(self, cue: Cue, style: SettleStyle) -> Result<(), Gone> {
        if let Some(running) = try_get(self.task)? {
            running.cancel();
        }
        try_set(self.task, None)?;
        let last = try_get(self.now)?;
        try_set(self.now, Settling::Rest)?;
        if cue.moment() != Moment::Success {
            return Ok(());
        }
        let level = self.env.now();
        let task = spawn_in(self.scope, async move {
            let _ = self.play(style, cue.touch(), level, last).await;
            let _ = try_set(self.now, Settling::Rest);
            let _ = try_set(self.task, None);
        });
        try_set(self.task, Some(task))
    }

    async fn play(
        self,
        style: SettleStyle,
        touch: Touch,
        level: MotionLevel,
        last: Settling,
    ) -> Result<(), Gone> {
        match (style, level) {
            (SettleStyle::Fill(_) | SettleStyle::LockIn, MotionLevel::Reduced) => Ok(()),
            (SettleStyle::Fill(layers), _) => self.fill(layers.0, level).await,
            (SettleStyle::Check, _) => self.check(level).await,
            (SettleStyle::LockIn, _) => self.seal(touch, level, last).await,
        }
    }

    async fn fill(self, layers: u8, level: MotionLevel) -> Result<(), Gone> {
        let step = DurationToken::PendingStep.duration(level);
        for layer in 0..layers {
            try_set(self.now, Settling::Filling(layer))?;
            sleep(step).await;
        }
        Ok(())
    }

    async fn check(self, level: MotionLevel) -> Result<(), Gone> {
        let hold = DelayToken::SettleHold.delay();
        if level != MotionLevel::Reduced {
            let draw = Glide::between(
                0,
                1000,
                DurationToken::Move.duration(level),
                EasingToken::Out.easing(level),
            );
            let started = crate::core::time::clock::now();
            tick(&draw, started, |pose| {
                try_set(self.now, Settling::Drawing(drawn(pose.value)))
            })
            .await?;
        }
        try_set(self.now, Settling::Drawing(Fraction(1000)))?;
        sleep(hold).await;
        Ok(())
    }

    async fn seal(self, touch: Touch, level: MotionLevel, last: Settling) -> Result<(), Gone> {
        let anim = match touch {
            Touch::Contact(_) => Anim::Gulp,
            Touch::Remote => Anim::SealOut,
        };
        let before = match last {
            Settling::Sealing(key) => key,
            Settling::Rest | Settling::Filling(_) | Settling::Drawing(_) => PulseKey::rest(anim),
        };
        try_set(self.now, Settling::Sealing(before.fired_as(anim)))?;
        sleep(settle(anim, level, StaggerIndex::default())).await;
        Ok(())
    }
}

/// The share of a check drawn at a pose's `value`.
fn drawn(value: i64) -> Fraction {
    Fraction(u16::try_from(value.clamp(0, 1000)).unwrap_or(1000))
}
