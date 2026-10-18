//! The switcher's transition (design/13 §13.4).
//!
//! The quick tap when the release may go unseen: what arrives inside the show delay decides. A
//! `ModifierReleased` switches to the armed selection with no panel; so does a second chord (a
//! second tap whose first release went unseen: the first tap wins, the second is spent). A key
//! seen while armed (Tab, Grave, the arrows) proves the chord is held and steps as it would
//! shown. Past the delay a chord steps.

use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

use super::model::{Phase, SwIn, SwKey, SwOut, SwitcherParams};
use crate::dir::Dir;

/// What a step wants done, beside the phase it leaves.
type Step = (Phase, Vec<SwOut>);

/// The MRU app list, never empty, that selections index: its length.
struct Ring(usize);

impl Ring {
    /// The ring over `count` apps, or none when there are no apps.
    fn over(count: usize) -> Option<Ring> {
        (count > 0).then_some(Ring(count))
    }

    /// `sel` brought inside the list, in case it shrank since the selection was made.
    fn clamp(&self, sel: usize) -> usize {
        sel.min(self.0.saturating_sub(1))
    }

    /// Whether `index` names an app.
    fn has(&self, index: usize) -> bool {
        index < self.0
    }

    /// The selection one step from `sel` toward `dir`, wrapping.
    fn moved(&self, sel: usize, dir: Dir) -> usize {
        let len = self.0;
        match dir {
            Dir::Next => (sel + 1) % len,
            Dir::Previous => (sel + len - 1) % len,
        }
    }
}

impl Machine for Phase {
    type In = SwIn;
    type Out = SwOut;
    type Params = SwitcherParams;
    /// How many apps the MRU list holds, the current app first.
    type Ctx = usize;

    /// The switcher after `input` at `at`, over an MRU list of `count` apps. An empty list is no
    /// switcher: the machine goes to [`Phase::Hidden`], hiding the panel if it was shown.
    fn step(self, input: SwIn, at: Stamp, p: &SwitcherParams, count: &usize) -> (Self, Vec<SwOut>) {
        phase_step(self, input, at, *count, p)
    }

    fn wake(&self) -> Option<Stamp> {
        match self {
            Phase::Armed { until, .. } => Some(*until),
            Phase::Hidden | Phase::Shown { .. } => None,
        }
    }
}

fn phase_step(s: Phase, input: SwIn, now: Stamp, count: usize, p: &SwitcherParams) -> Step {
    let Some(ring) = Ring::over(count) else {
        return no_apps(s);
    };
    match s {
        Phase::Hidden => hidden(input, now, &ring, p),
        Phase::Armed { until, sel } => armed(input, now, until, ring.clamp(sel), &ring),
        Phase::Shown { sel } => shown(input, ring.clamp(sel), &ring),
    }
}

/// No apps: nothing to switch, and a panel that was up goes.
fn no_apps(s: Phase) -> Step {
    match s {
        Phase::Shown { .. } => (Phase::Hidden, vec![SwOut::Hide]),
        Phase::Hidden | Phase::Armed { .. } => (Phase::Hidden, Vec::new()),
    }
}

/// The way a key steps the selection, if it steps it.
fn direction(key: SwKey) -> Option<Dir> {
    match key {
        SwKey::Tab | SwKey::Right => Some(Dir::Next),
        SwKey::ShiftTab | SwKey::Grave | SwKey::Left => Some(Dir::Previous),
        SwKey::Up | SwKey::Down | SwKey::Q | SwKey::H | SwKey::Escape => None,
    }
}

/// Only a chord starts anything: the selection is the app after (or before) the current one,
/// and the panel waits out the show delay.
fn hidden(input: SwIn, now: Stamp, ring: &Ring, p: &SwitcherParams) -> Step {
    match input {
        SwIn::Chord(dir) => (
            Phase::Armed {
                until: now.after_span(p.show_delay),
                sel: ring.moved(0, dir),
            },
            Vec::new(),
        ),
        _ => (Phase::Hidden, Vec::new()),
    }
}

/// The chord is down and the panel waits for the show delay.
fn armed(input: SwIn, now: Stamp, shows_at: Stamp, sel: usize, ring: &Ring) -> Step {
    let wait = |sel| {
        (
            Phase::Armed {
                until: shows_at,
                sel,
            },
            Vec::new(),
        )
    };
    match input {
        SwIn::ModifierReleased => (Phase::Hidden, vec![SwOut::Activate(sel)]),
        SwIn::Key(SwKey::Escape) => (Phase::Hidden, Vec::new()),
        SwIn::Chord(_) if now < shows_at => (Phase::Hidden, vec![SwOut::Activate(sel)]),
        SwIn::Chord(dir) => wait(ring.moved(sel, dir)),
        SwIn::Key(key) => wait(direction(key).map_or(sel, |dir| ring.moved(sel, dir))),
        SwIn::Elapsed if now >= shows_at => {
            (Phase::Shown { sel }, vec![SwOut::Show, SwOut::Select(sel)])
        }
        SwIn::Elapsed | SwIn::Hover(_) | SwIn::Click(_) => wait(sel),
    }
}

/// The panel is up on `sel`.
fn shown(input: SwIn, sel: usize, ring: &Ring) -> Step {
    let stay = |out| (Phase::Shown { sel }, out);
    let select = |sel| (Phase::Shown { sel }, vec![SwOut::Select(sel)]);
    let activate = |index| (Phase::Hidden, vec![SwOut::Hide, SwOut::Activate(index)]);
    match input {
        SwIn::Chord(dir) => select(ring.moved(sel, dir)),
        SwIn::ModifierReleased => activate(sel),
        SwIn::Key(SwKey::Escape) => (Phase::Hidden, vec![SwOut::Hide]),
        SwIn::Key(SwKey::Up | SwKey::Down) => {
            (Phase::Hidden, vec![SwOut::Hide, SwOut::Expose(sel)])
        }
        SwIn::Key(SwKey::Q) => stay(vec![SwOut::Quit(sel)]),
        SwIn::Key(SwKey::H) => stay(vec![SwOut::HideApp(sel)]),
        SwIn::Key(key) => {
            direction(key).map_or_else(|| stay(Vec::new()), |dir| select(ring.moved(sel, dir)))
        }
        SwIn::Hover(index) if ring.has(index) => select(index),
        SwIn::Click(index) if ring.has(index) => activate(index),
        SwIn::Hover(_) | SwIn::Click(_) | SwIn::Elapsed => stay(Vec::new()),
    }
}
