//! Which of a symbol's three drivers plays an effect on an icon: the stylesheet on the wrapper,
//! Rust on the parts inside the SVG, or the draw-on tween (design/35-SYMBOL-EFFECTS.md section 3).

use super::effect::{Activity, LoopEffect, OnceEffect, SymbolEffect, TransitionEffect, Trigger};
use crate::anim::Anim;
use ds_core::vocab::Shown;
use ds_style::icon::Icon;
use ds_style::icon::parts::PartGesture;

/// How the part driver runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PartRun {
    /// One play per change of the trigger.
    Once(Trigger),
    /// Plays again and again while active.
    While(Activity),
}

/// The driver an effect has on an icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Route {
    /// The stylesheet, on the wrapper.
    Css(SymbolEffect),
    /// Rust, on the icon's annotated parts.
    Parts(PartGesture, PartRun),
    /// Rust, drawing the strokes on or off.
    Draw(Shown),
    /// The glyph morph.
    Replace,
}

/// The driver for `effect` on `icon`: an effect that moves a part falls back to a whole-icon one
/// (a bounce, a pulse) on an icon with no part to move.
pub fn route(effect: SymbolEffect, icon: Icon) -> Route {
    let gesture = icon.parts().map(|parts| parts.gesture);
    match (effect, gesture) {
        (SymbolEffect::Once(OnceEffect::Part, trigger), Some(gesture)) => {
            Route::Parts(gesture, PartRun::Once(trigger))
        }
        (SymbolEffect::Once(OnceEffect::VariableColor, trigger), Some(PartGesture::Layers)) => {
            Route::Parts(PartGesture::Layers, PartRun::Once(trigger))
        }
        (SymbolEffect::While(LoopEffect::Part, activity), Some(gesture)) => {
            Route::Parts(gesture, PartRun::While(activity))
        }
        (SymbolEffect::While(LoopEffect::VariableColor, activity), Some(PartGesture::Layers)) => {
            Route::Parts(PartGesture::Layers, PartRun::While(activity))
        }
        (SymbolEffect::Once(OnceEffect::Part, trigger), None) => {
            Route::Css(SymbolEffect::Once(OnceEffect::Bounce, trigger))
        }
        (SymbolEffect::Once(OnceEffect::VariableColor, trigger), _) => {
            Route::Css(SymbolEffect::Once(OnceEffect::Pulse, trigger))
        }
        (SymbolEffect::While(LoopEffect::Part, activity), None) => {
            Route::Css(SymbolEffect::While(LoopEffect::Bounce, activity))
        }
        (SymbolEffect::While(LoopEffect::VariableColor, activity), _) => {
            Route::Css(SymbolEffect::While(LoopEffect::Pulse, activity))
        }
        (SymbolEffect::Transition(TransitionEffect::DrawOn, shown), _) => Route::Draw(shown),
        (SymbolEffect::Transition(TransitionEffect::Replace, _), _) => Route::Replace,
        (effect, _) => Route::Css(effect),
    }
}

/// The animation a `Once` effect plays on the wrapper; the part effects are routed away first.
pub fn once_anim(effect: OnceEffect) -> Anim {
    match effect {
        OnceEffect::Bounce | OnceEffect::Part => Anim::Bounce,
        OnceEffect::Pulse | OnceEffect::VariableColor => Anim::Pulse,
        OnceEffect::Wiggle => Anim::Wiggle,
        OnceEffect::Breathe => Anim::Breathe,
        OnceEffect::Rotate => Anim::RotateOnce,
    }
}

/// The animation a `While` effect plays on the wrapper while active.
pub fn loop_anim(effect: LoopEffect) -> Anim {
    match effect {
        LoopEffect::Bounce | LoopEffect::Part => Anim::BounceLoop,
        LoopEffect::Pulse | LoopEffect::VariableColor => Anim::PulseLoop,
        LoopEffect::ScaleUp => Anim::ScaleUp,
        LoopEffect::ScaleDown => Anim::ScaleDown,
        LoopEffect::Wiggle => Anim::WiggleLoop,
        LoopEffect::Breathe => Anim::BreatheLoop,
        LoopEffect::Rotate => Anim::Turn,
    }
}

#[cfg(test)]
mod tests {
    use super::{PartRun, Route, route};
    use crate::symbol::effect::{
        Activity, LoopEffect, OnceEffect, SymbolEffect, TransitionEffect, Trigger,
    };
    use ds_core::vocab::Shown;
    use ds_style::icon::Icon;
    use ds_style::icon::parts::PartGesture;

    #[test]
    fn a_part_effect_moves_the_part_where_there_is_one_and_falls_back_where_there_is_not() {
        let once = |effect| SymbolEffect::Once(effect, Trigger(1));
        let looped = |effect| SymbolEffect::While(effect, Activity::Active);
        // (effect, icon, driver).
        let cases = [
            (
                once(OnceEffect::Part),
                Icon::Trash,
                Route::Parts(PartGesture::Lift, PartRun::Once(Trigger(1))),
            ),
            (
                once(OnceEffect::Part),
                Icon::Inbox,
                Route::Css(once(OnceEffect::Bounce)),
            ),
            (
                once(OnceEffect::VariableColor),
                Icon::Wifi,
                Route::Parts(PartGesture::Layers, PartRun::Once(Trigger(1))),
            ),
            (
                once(OnceEffect::VariableColor),
                Icon::Trash,
                Route::Css(once(OnceEffect::Pulse)),
            ),
            (
                looped(LoopEffect::VariableColor),
                Icon::Volume2,
                Route::Parts(PartGesture::Layers, PartRun::While(Activity::Active)),
            ),
            (
                looped(LoopEffect::Part),
                Icon::Inbox,
                Route::Css(looped(LoopEffect::Bounce)),
            ),
            (
                once(OnceEffect::Bounce),
                Icon::Trash,
                Route::Css(once(OnceEffect::Bounce)),
            ),
            (
                SymbolEffect::Transition(TransitionEffect::DrawOn, Shown::Hidden),
                Icon::Mail,
                Route::Draw(Shown::Hidden),
            ),
            (
                SymbolEffect::Transition(TransitionEffect::Replace, Shown::Visible),
                Icon::Mail,
                Route::Replace,
            ),
        ];
        for (effect, icon, want) in cases {
            assert_eq!(route(effect, icon), want, "{effect:?} on {icon:?}");
        }
    }
}
