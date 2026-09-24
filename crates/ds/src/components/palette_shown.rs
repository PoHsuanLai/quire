//! A command palette kept mounted while hidden (sill FINDINGS Q63): a launcher that mounts a
//! palette per opening pays 15-20 ms for it, so it keeps one and says whether it is shown.
//!
//! Hidden, the card is `display:none` (nothing laid out or painted, as an empty `ToastHost`) and
//! off the layer stack. Each time it is shown again it replays its entrance by swapping the
//! keyframe's `X`/`X--b` alias (`data-pulse`, design/05-MOTION.md section 9 rule 2) and restarting
//! the settle timer, starts over from an empty query and the first choice unless [`Retain`]
//! says otherwise, and takes the keyboard.

use crate::components::tooltip::Shown;
use crate::motion::anim::Anim;
use crate::motion::presence::Presence;
use crate::motion::timer::{MotionTimer, TimerPhase, use_motion_timer};
use dioxus::prelude::*;

/// What a palette shown again keeps from its last showing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Retain {
    /// Nothing: the query is emptied (through `oninput`) and the selection is the first choice.
    #[default]
    Nothing,
    /// The query, and the selection made under it: the palette opens where it was left.
    Query,
}

/// What one render's `shown` does to the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Change {
    /// Nothing: as it was (or mounted shown, which plays the entrance as it mounts).
    Stay,
    /// Shown after being hidden: replay the entrance, start over, take the keyboard.
    Show,
    /// Hidden: leave the layer stack.
    Hide,
}

/// The change from `last` (`None` before the first render) to `now`.
pub(crate) fn change(last: Option<Shown>, now: Shown) -> Change {
    match (last, now) {
        (Some(Shown::Hidden), Shown::Visible) => Change::Show,
        (None | Some(Shown::Visible), Shown::Hidden) => Change::Hide,
        (None | Some(Shown::Visible), Shown::Visible) | (Some(Shown::Hidden), Shown::Hidden) => {
            Change::Stay
        }
    }
}

/// Which of the entrance keyframe's two names the card plays: flipping it restarts the
/// animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alias {
    A,
    B,
}

impl Alias {
    fn flipped(self) -> Self {
        match self {
            Alias::A => Alias::B,
            Alias::B => Alias::A,
        }
    }

    fn slug(self) -> &'static str {
        match self {
            Alias::A => "a",
            Alias::B => "b",
        }
    }
}

/// The palette's showing, for one render.
#[derive(Clone, Copy)]
pub(crate) struct Showing {
    /// Whether the card is shown now.
    pub shown: Shown,
    /// What this render's `shown` changed.
    pub change: Change,
    /// Whether the card has been shown since it mounted: a palette mounted hidden does not
    /// take the keyboard until it is first shown.
    pub seen: Seen,
    timer: MotionTimer,
    alias: Signal<Alias>,
    /// The timer's settle callback, made in render: an effect has no scope to make one in.
    settled: EventHandler<()>,
}

/// Whether a palette has been shown since it mounted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seen {
    /// Shown at least once.
    Yes,
    /// Mounted hidden and not shown yet.
    Not,
}

/// The showing hook: `shown` is the caller's (`None`, always shown), `anim` the entrance.
pub(crate) fn use_showing(shown: Option<Shown>, anim: Anim) -> Showing {
    let now = shown.unwrap_or(Shown::Visible);
    let timer = use_motion_timer(anim);
    let mut last = use_hook(|| CopyValue::new(None::<Shown>));
    let mut seen = use_hook(|| {
        if now == Shown::Visible {
            timer.start(EventHandler::new(|()| {}));
        }
        CopyValue::new(match now {
            Shown::Visible => Seen::Yes,
            Shown::Hidden => Seen::Not,
        })
    });
    let change = change(*last.peek(), now);
    last.set(Some(now));
    if change == Change::Show {
        seen.set(Seen::Yes);
    }
    Showing {
        shown: now,
        change,
        seen: *seen.peek(),
        timer,
        alias: use_signal(|| Alias::A),
        settled: EventHandler::new(|()| {}),
    }
}

impl Showing {
    /// `data-presence`: entering until the entrance settles, then present.
    pub(crate) fn presence(&self) -> Presence {
        match self.timer.phase() {
            TimerPhase::Settled => Presence::Present,
            TimerPhase::Idle | TimerPhase::Running => Presence::Entering,
        }
    }

    /// `data-pulse`: which of the entrance keyframe's names the card plays.
    pub(crate) fn alias(&self) -> &'static str {
        self.alias.read().slug()
    }

    /// Play the entrance again from its first frame. Call it from an effect or a handler.
    pub(crate) fn replay(&self) {
        let alias = self.alias;
        if let Ok(now) = crate::task::try_get(alias) {
            let _ = crate::task::try_set(alias, now.flipped());
        }
        self.timer.start(self.settled);
    }
}

#[cfg(test)]
mod tests {
    use super::{Change, change};
    use crate::components::tooltip::Shown;

    #[test]
    fn only_a_hidden_palette_shown_again_replays() {
        #[rustfmt::skip]
        let cases = [
            (None, Shown::Visible, Change::Stay),
            (None, Shown::Hidden, Change::Hide),
            (Some(Shown::Visible), Shown::Visible, Change::Stay),
            (Some(Shown::Visible), Shown::Hidden, Change::Hide),
            (Some(Shown::Hidden), Shown::Hidden, Change::Stay),
            (Some(Shown::Hidden), Shown::Visible, Change::Show),
        ];
        for (last, now, want) in cases {
            assert_eq!(change(last, now), want, "{last:?} -> {now:?}");
        }
    }
}
