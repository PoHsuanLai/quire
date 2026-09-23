//! Count: an unread or item count, empty at zero, bumping when it changes
//! (design/04-COMPONENTS.md section 14).

use crate::components::vocab::PulseKey;
use crate::motion::Anim;
use dioxus::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

/// Where a count sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CountPlace {
    /// Trailing a sidebar item.
    #[default]
    Item,
    /// In an account tile's corner.
    Tile,
}

impl CountPlace {
    /// The `data-place` word.
    fn slug(self) -> &'static str {
        match self {
            CountPlace::Item => "item",
            CountPlace::Tile => "tile",
        }
    }
}

/// The text a count shows: nothing at zero, so the layout does not shift (`S:1247`).
fn text(value: u32) -> String {
    match value {
        0 => String::new(),
        n => n.to_string(),
    }
}

/// The value a count last rendered and the bump it was playing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Seen {
    value: u32,
    bump: PulseKey,
}

impl Seen {
    /// A count first shown at `value`: nothing plays on mount.
    fn first(value: u32) -> Self {
        Seen {
            value,
            bump: PulseKey::rest(Anim::Bump),
        }
    }

    /// What rendering `value` next makes of it: a different value fires the bump again (the
    /// other alias, so it restarts), the same value leaves it as it was.
    fn next(self, value: u32) -> Self {
        let bump = if value == self.value {
            self.bump
        } else {
            self.bump.fired()
        };
        Seen { value, bump }
    }
}

/// A count. It plays `bump` whenever `value` differs from the value it last rendered, and not
/// on mount. A count that should not bump across a Space switch (O-8) is given a new `key`
/// there, so it mounts at rest instead.
///
/// The last value lives in a plain cell, not a signal: the bump is decided while rendering, and
/// a signal written during render would schedule a second render for nothing.
#[component]
pub fn Count(value: u32, #[props(default)] place: CountPlace) -> Element {
    let seen = use_hook(|| Rc::new(Cell::new(Seen::first(value))));
    let now = seen.get().next(value);
    seen.set(now);
    let (class, alias) = match now.bump.attrs() {
        Some((anim, alias)) => (format!("ds-count {anim}"), Some(alias)),
        None => ("ds-count".to_string(), None),
    };
    rsx! {
        span {
            class,
            "data-place": place.slug(),
            "data-pulse": alias,
            {text(value)}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Seen;
    use crate::components::vocab::PulsePhase;

    #[test]
    fn a_change_fires_the_bump_and_a_repeat_does_not() {
        const STEPS: &[(u32, PulsePhase)] = &[
            (4, PulsePhase::Rest),
            (5, PulsePhase::A),
            (5, PulsePhase::A),
            (6, PulsePhase::B),
            (0, PulsePhase::A),
        ];
        let mut seen = Seen::first(4);
        for &(value, want) in STEPS {
            seen = seen.next(value);
            assert_eq!(seen.bump.phase(), want, "rendering {value}");
        }
    }
}
