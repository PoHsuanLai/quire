//! The switcher's transition (design/13 §13.4).
//!
//! The quick tap when the release may go unseen: what arrives inside the show delay decides. A
//! `ModifierReleased` switches to the armed selection with no panel; so does a second chord (a
//! second tap whose first release went unseen: the first tap wins, the second is spent). A key
//! seen while armed (Tab, Grave, the arrows) proves the chord is held and steps as it would
//! shown. Past the delay a chord steps.

use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

use super::model::{Phase, SwIn, SwKey, SwOut, Switcher, SwitcherParams};
use crate::dir::Dir;
use crate::span;

/// What a step wants done, beside the phase it leaves.
type Step<K> = (Phase, Vec<SwOut<K>>);

/// The MRU app list, never empty, that selections index.
struct Ring<'a, K>(&'a [K]);

impl<'a, K: Clone> Ring<'a, K> {
    /// The ring over `apps`, or none when there are no apps.
    fn over(apps: &'a [K]) -> Option<Ring<'a, K>> {
        (!apps.is_empty()).then_some(Ring(apps))
    }

    /// `sel` brought inside the list, in case it shrank since the selection was made.
    fn clamp(&self, sel: usize) -> usize {
        sel.min(self.0.len().saturating_sub(1))
    }

    /// The app at `sel`.
    fn app(&self, sel: usize) -> K {
        self.0[self.clamp(sel)].clone()
    }

    /// Whether `index` names an app.
    fn has(&self, index: usize) -> bool {
        index < self.0.len()
    }

    /// The selection one step from `sel` toward `dir`, wrapping.
    fn moved(&self, sel: usize, dir: Dir) -> usize {
        let len = self.0.len();
        match dir {
            Dir::Next => (sel + 1) % len,
            Dir::Previous => (sel + len - 1) % len,
        }
    }
}

impl<K: Clone + Eq + 'static> Machine for Switcher<K> {
    type In = SwIn;
    type Out = SwOut<K>;
    type Params = SwitcherParams;
    /// The MRU app list, the current app first.
    type Ctx = Vec<K>;

    /// The switcher after `input` at `at`, over the MRU list `apps`. An empty list is no
    /// switcher: the machine goes to [`Phase::Hidden`], hiding the panel if it was shown.
    fn step(
        self,
        input: SwIn,
        at: Stamp,
        p: &SwitcherParams,
        apps: &Vec<K>,
    ) -> (Self, Vec<SwOut<K>>) {
        let (phase, outs) = phase_step(self.phase, input, at, apps, p);
        (Switcher::in_phase(phase), outs)
    }

    fn wake(&self) -> Option<Stamp> {
        match self.phase {
            Phase::Armed { until, .. } => Some(until),
            Phase::Hidden | Phase::Shown { .. } => None,
        }
    }
}

fn phase_step<K: Clone + Eq>(
    s: Phase,
    input: SwIn,
    now: Stamp,
    apps: &[K],
    p: &SwitcherParams,
) -> Step<K> {
    let Some(ring) = Ring::over(apps) else {
        return no_apps(s);
    };
    match s {
        Phase::Hidden => hidden(input, now, &ring, p),
        Phase::Armed { until, sel } => armed(input, now, until, ring.clamp(sel), &ring),
        Phase::Shown { sel } => shown(input, ring.clamp(sel), &ring),
    }
}

/// No apps: nothing to switch, and a panel that was up goes.
fn no_apps<K>(s: Phase) -> Step<K> {
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
fn hidden<K: Clone>(input: SwIn, now: Stamp, ring: &Ring<'_, K>, p: &SwitcherParams) -> Step<K> {
    match input {
        SwIn::Chord(dir) => (
            Phase::Armed {
                until: span::after(now, p.show_delay),
                sel: ring.moved(0, dir),
            },
            Vec::new(),
        ),
        _ => (Phase::Hidden, Vec::new()),
    }
}

/// The chord is down and the panel waits for the show delay.
fn armed<K: Clone>(
    input: SwIn,
    now: Stamp,
    shows_at: Stamp,
    sel: usize,
    ring: &Ring<'_, K>,
) -> Step<K> {
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
        SwIn::ModifierReleased => (Phase::Hidden, vec![SwOut::Activate(ring.app(sel))]),
        SwIn::Key(SwKey::Escape) => (Phase::Hidden, Vec::new()),
        SwIn::Chord(_) if now < shows_at => (Phase::Hidden, vec![SwOut::Activate(ring.app(sel))]),
        SwIn::Chord(dir) => wait(ring.moved(sel, dir)),
        SwIn::Key(key) => wait(direction(key).map_or(sel, |dir| ring.moved(sel, dir))),
        SwIn::Elapsed if now >= shows_at => {
            (Phase::Shown { sel }, vec![SwOut::Show, SwOut::Select(sel)])
        }
        SwIn::Elapsed | SwIn::Hover(_) | SwIn::Click(_) => wait(sel),
    }
}

/// The panel is up on `sel`.
fn shown<K: Clone>(input: SwIn, sel: usize, ring: &Ring<'_, K>) -> Step<K> {
    let stay = |out| (Phase::Shown { sel }, out);
    let select = |sel| (Phase::Shown { sel }, vec![SwOut::Select(sel)]);
    let activate = |index| {
        (
            Phase::Hidden,
            vec![SwOut::Hide, SwOut::Activate(ring.app(index))],
        )
    };
    match input {
        SwIn::Chord(dir) => select(ring.moved(sel, dir)),
        SwIn::ModifierReleased => activate(sel),
        SwIn::Key(SwKey::Escape) => (Phase::Hidden, vec![SwOut::Hide]),
        SwIn::Key(SwKey::Up | SwKey::Down) => (
            Phase::Hidden,
            vec![SwOut::Hide, SwOut::Expose(ring.app(sel))],
        ),
        SwIn::Key(SwKey::Q) => stay(vec![SwOut::Quit(ring.app(sel))]),
        SwIn::Key(SwKey::H) => stay(vec![SwOut::HideApp(ring.app(sel))]),
        SwIn::Key(key) => {
            direction(key).map_or_else(|| stay(Vec::new()), |dir| select(ring.moved(sel, dir)))
        }
        SwIn::Hover(index) if ring.has(index) => select(index),
        SwIn::Click(index) if ring.has(index) => activate(index),
        SwIn::Hover(_) | SwIn::Click(_) | SwIn::Elapsed => stay(Vec::new()),
    }
}
