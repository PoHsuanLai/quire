//! The switcher's transition (design/13 §13.4).
//!
//! The quick tap when the release may go unseen: what arrives inside the show delay decides. A
//! `ModifierReleased` switches to the armed selection with no panel; so does a second chord (a
//! second tap whose first release went unseen: the first tap wins, the second is spent). A key
//! seen while armed (Tab, Grave, the arrows) proves the chord is held and steps as it would
//! shown. Past the delay a chord steps.

use ds_core::time::stamp::Stamp;

use super::model::{SwIn, SwKey, SwOut, Switcher, SwitcherParams};
use crate::dir::Dir;
use crate::span;

/// What a step wants done, beside the phase it leaves.
type Step<K> = (Switcher, Vec<SwOut<K>>);

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

/// `s` after `input` at `now`, over the MRU list `apps` (the current app first). An empty list
/// is no switcher: the machine goes to [`Switcher::Hidden`], hiding the panel if it was shown.
pub fn step<K: Clone + Eq>(
    s: Switcher,
    input: SwIn,
    now: Stamp,
    apps: &[K],
    p: &SwitcherParams,
) -> Step<K> {
    let Some(ring) = Ring::over(apps) else {
        return no_apps(s);
    };
    match s {
        Switcher::Hidden => hidden(input, now, &ring, p),
        Switcher::Armed { since, sel } => armed(input, now, since, ring.clamp(sel), &ring, p),
        Switcher::Shown { sel } => shown(input, ring.clamp(sel), &ring),
    }
}

/// No apps: nothing to switch, and a panel that was up goes.
fn no_apps<K>(s: Switcher) -> Step<K> {
    match s {
        Switcher::Shown { .. } => (Switcher::Hidden, vec![SwOut::Hide]),
        Switcher::Hidden | Switcher::Armed { .. } => (Switcher::Hidden, Vec::new()),
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
            Switcher::Armed {
                since: now,
                sel: ring.moved(0, dir),
            },
            vec![SwOut::RequestTick(span::after(now, p.show_delay))],
        ),
        _ => (Switcher::Hidden, Vec::new()),
    }
}

/// The chord is down and the panel waits for the show delay.
fn armed<K: Clone>(
    input: SwIn,
    now: Stamp,
    since: Stamp,
    sel: usize,
    ring: &Ring<'_, K>,
    p: &SwitcherParams,
) -> Step<K> {
    let shows_at = span::after(since, p.show_delay);
    let wait = |sel| (Switcher::Armed { since, sel }, Vec::new());
    match input {
        SwIn::ModifierReleased => (Switcher::Hidden, vec![SwOut::Activate(ring.app(sel))]),
        SwIn::Key(SwKey::Escape) => (Switcher::Hidden, Vec::new()),
        SwIn::Chord(_) if now < shows_at => {
            (Switcher::Hidden, vec![SwOut::Activate(ring.app(sel))])
        }
        SwIn::Chord(dir) => wait(ring.moved(sel, dir)),
        SwIn::Key(key) => wait(direction(key).map_or(sel, |dir| ring.moved(sel, dir))),
        SwIn::Tick if now >= shows_at => (
            Switcher::Shown { sel },
            vec![SwOut::Show, SwOut::Select(sel)],
        ),
        SwIn::Tick | SwIn::Hover(_) | SwIn::Click(_) => wait(sel),
    }
}

/// The panel is up on `sel`.
fn shown<K: Clone>(input: SwIn, sel: usize, ring: &Ring<'_, K>) -> Step<K> {
    let stay = |out| (Switcher::Shown { sel }, out);
    let select = |sel| (Switcher::Shown { sel }, vec![SwOut::Select(sel)]);
    let activate = |index| {
        (
            Switcher::Hidden,
            vec![SwOut::Hide, SwOut::Activate(ring.app(index))],
        )
    };
    match input {
        SwIn::Chord(dir) => select(ring.moved(sel, dir)),
        SwIn::ModifierReleased => activate(sel),
        SwIn::Key(SwKey::Escape) => (Switcher::Hidden, vec![SwOut::Hide]),
        SwIn::Key(SwKey::Up | SwKey::Down) => (
            Switcher::Hidden,
            vec![SwOut::Hide, SwOut::Expose(ring.app(sel))],
        ),
        SwIn::Key(SwKey::Q) => stay(vec![SwOut::Quit(ring.app(sel))]),
        SwIn::Key(SwKey::H) => stay(vec![SwOut::HideApp(ring.app(sel))]),
        SwIn::Key(key) => {
            direction(key).map_or_else(|| stay(Vec::new()), |dir| select(ring.moved(sel, dir)))
        }
        SwIn::Hover(index) if ring.has(index) => select(index),
        SwIn::Click(index) if ring.has(index) => activate(index),
        SwIn::Hover(_) | SwIn::Click(_) | SwIn::Tick => stay(Vec::new()),
    }
}
