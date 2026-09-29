//! The motion level a detail plays at, read when it starts (R7 lives inside every primitive).

use crate::style::appearance::motion::MotionLevel;
use crate::style::scope::Scope;
use dioxus::prelude::*;

/// The enclosing root's motion level, if there is a root; Standard outside one (a golden render
/// of a bare component).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level(Option<Signal<Scope>>);

impl Level {
    /// The level now, read without subscribing.
    pub fn now(self) -> MotionLevel {
        self.0
            .and_then(|env| env.try_peek().ok().map(|env| env.resolved.motion))
            .unwrap_or(MotionLevel::Standard)
    }
}

/// The level of the enclosing `Ds` or `Surface`.
pub fn use_level() -> Level {
    Level(try_use_context::<Signal<Scope>>())
}
