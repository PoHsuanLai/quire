//! The Symbols page: every symbol effect (design/35-SYMBOL-EFFECTS.md) on a few icons, a button
//! that fires each `Once` effect, a toggle that runs each `While` effect, and the buttons that
//! move a transition.

use crate::pages::Section;
use crate::pages::details::overview::{Cell, mini};
use dioxus::prelude::*;
use ds::prelude::*;
use ds_core::vocab::Shown;
use ds_core::word::Word;

/// The icons a whole-icon effect plays on.
const WHOLE: [Icon; 3] = [Icon::Inbox, Icon::Settings, Icon::Search];
/// The icons with a moving part, each with the effect that moves it.
const PARTS: [(Icon, OnceEffect); 11] = [
    (Icon::Trash, OnceEffect::Part),
    (Icon::Bell, OnceEffect::Part),
    (Icon::Mail, OnceEffect::Part),
    (Icon::Folder, OnceEffect::Part),
    (Icon::Refresh, OnceEffect::Part),
    (Icon::Wifi, OnceEffect::VariableColor),
    (Icon::Volume2, OnceEffect::VariableColor),
    (Icon::BatteryFull, OnceEffect::VariableColor),
    (Icon::Lock, OnceEffect::Part),
    (Icon::Star, OnceEffect::Part),
    (Icon::Heart, OnceEffect::Part),
];

/// The page.
#[component]
pub fn SymbolsPage() -> Element {
    rsx! {
        OnceSection {}
        WhileSection {}
        TransitionSection {}
        PartSection {}
    }
}

/// A row of symbols of `effect`, the way a cell shows them.
fn row(effect: impl Fn(Icon) -> SymbolEffect) -> Element {
    rsx! {
        div { class: "g-detail",
            for icon in WHOLE {
                Symbol { icon, size: IconSize::Bar, effect: effect(icon) }
            }
        }
    }
}

#[component]
fn OnceSection() -> Element {
    rsx! {
        Section { title: "Once", note: "Each plays when its trigger changes: one press, one play. The first render plays nothing, and a re-render with the same trigger plays nothing.",
            div { class: "g-detail-grid",
                for effect in OnceEffect::ALL.iter().copied() {
                    OnceCell { effect }
                }
            }
        }
    }
}

#[component]
fn OnceCell(effect: OnceEffect) -> Element {
    let mut trigger = use_signal(Trigger::default);
    rsx! {
        Cell { name: effect.label(), code: "SymbolEffect::Once({effect:?}, trigger)",
            controls: rsx! {
                {mini("Play", move |_| trigger.set(trigger().next()))}
            },
            {row(move |_| SymbolEffect::Once(effect, trigger()))}
        }
    }
}

#[component]
fn WhileSection() -> Element {
    rsx! {
        Section { title: "While", note: "Each runs for as long as its toggle is on and stops when it is off: a loop is the one thing on the page that keeps asking for frames, and only while it is on. Scale up and Scale down hold the symbol larger or smaller.",
            div { class: "g-detail-grid",
                for effect in LoopEffect::ALL.iter().copied() {
                    WhileCell { effect }
                }
            }
        }
    }
}

#[component]
fn WhileCell(effect: LoopEffect) -> Element {
    let mut activity = use_signal(Activity::default);
    let (label, next) = match activity() {
        Activity::Idle => ("Start", Activity::Active),
        Activity::Active => ("Stop", Activity::Idle),
    };
    rsx! {
        Cell { name: effect.label(), code: "SymbolEffect::While({effect:?}, activity)",
            controls: rsx! {
                {mini(label, move |_| activity.set(next))}
            },
            {row(move |_| SymbolEffect::While(effect, activity()))}
        }
    }
}

#[component]
fn TransitionSection() -> Element {
    rsx! {
        Section { title: "Transitions", note: "Appear grows a symbol in as it fades up and Disappear fades it out; Draw On draws its strokes along their paths and Draw Off draws them back; Replace cross-fades into the next icon.",
            div { class: "g-detail-grid",
                for effect in [TransitionEffect::Appear, TransitionEffect::Disappear, TransitionEffect::DrawOn] {
                    ShownCell { effect }
                }
                ReplaceCell {}
            }
        }
    }
}

#[component]
fn ShownCell(effect: TransitionEffect) -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    rsx! {
        Cell { name: effect.label(), code: "SymbolEffect::Transition({effect:?}, shown)",
            controls: rsx! {
                {mini("Show", move |_| shown.set(Shown::Visible))}
                {mini("Hide", move |_| shown.set(Shown::Hidden))}
            },
            {row(move |_| SymbolEffect::Transition(effect, shown()))}
        }
    }
}

#[component]
fn ReplaceCell() -> Element {
    let mut flipped = use_signal(|| false);
    let icon = if flipped() { Icon::Check } else { Icon::Trash };
    rsx! {
        Cell { name: "Replace", code: "SymbolEffect::Transition(Replace, ..)",
            controls: rsx! {
                {mini("Swap", move |_| flipped.set(!flipped()))}
            },
            div { class: "g-detail",
                Symbol { icon, size: IconSize::Bar, effect: SymbolEffect::Transition(TransitionEffect::Replace, Shown::Visible) }
            }
        }
    }
}

#[component]
fn PartSection() -> Element {
    rsx! {
        Section { title: "By part", note: "The icons Apple animates by part: the trash can's lid lifts, the bell swings from its top, the envelope's flap opens, the folder leans open, refresh turns, the Wi-Fi bars, the volume waves and the battery cells light in order (Variable Color), the lock's shackle lifts, and the star and heart swell and fill. The drawings are Lucide's; only the annotated shapes move.",
            div { class: "g-detail-grid",
                for (icon , effect) in PARTS {
                    PartCell { icon, effect }
                }
            }
        }
    }
}

#[component]
fn PartCell(icon: Icon, effect: OnceEffect) -> Element {
    let mut trigger = use_signal(Trigger::default);
    let mut activity = use_signal(Activity::default);
    let loop_effect = match effect {
        OnceEffect::VariableColor => LoopEffect::VariableColor,
        _ => LoopEffect::Part,
    };
    let (label, next) = match activity() {
        Activity::Idle => ("Loop", Activity::Active),
        Activity::Active => ("Stop", Activity::Idle),
    };
    rsx! {
        Cell { name: format!("{icon:?}"), code: "SymbolEffect::Once({effect:?}, trigger)",
            controls: rsx! {
                {mini("Play", move |_| trigger.set(trigger().next()))}
                {mini(label, move |_| activity.set(next))}
            },
            div { class: "g-detail",
                Symbol { icon, size: IconSize::Tile96, effect: SymbolEffect::Once(effect, trigger()) }
                Symbol { icon, size: IconSize::Tile96, effect: SymbolEffect::While(loop_effect, activity()) }
            }
        }
    }
}
