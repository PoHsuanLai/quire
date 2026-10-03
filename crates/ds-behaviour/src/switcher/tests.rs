//! The switcher's tables (design/13 §13.4 acceptance 8, the quick-tap rule, App Exposé from 03
//! §1.3). The app key is a plain string here; the machine never looks inside it.

use std::time::Duration;

use ds_core::time::stamp::Stamp;

use super::{SwIn, SwKey, SwOut, Switcher, SwitcherParams, step};
use crate::dir::Dir;

/// One scripted input sequence: `(ms, input)` pairs.
type Script = Vec<(u64, SwIn)>;

/// Name, script, final state, every output in order.
type Case = (&'static str, Script, Switcher, Vec<SwOut<&'static str>>);

const APPS: [&str; 4] = ["files", "firefox", "foot", "mail"];

fn params() -> SwitcherParams {
    SwitcherParams {
        show_delay: Duration::from_millis(150),
    }
}

fn run(inputs: &[(u64, SwIn)]) -> (Switcher, Vec<SwOut<&'static str>>) {
    let p = params();
    inputs.iter().fold(
        (Switcher::Hidden, Vec::new()),
        |(s, mut all), &(ms, input)| {
            let (s, out) = step(s, input, Stamp(ms), &APPS, &p);
            all.extend(out);
            (s, all)
        },
    )
}

fn check(cases: Vec<Case>) {
    for (name, inputs, want_state, want_out) in cases {
        assert_eq!(run(&inputs), (want_state, want_out), "{name}");
    }
}

#[test]
#[ignore = "S0-QB-F"]
fn design_13_acceptance_8() {
    let chord = (0, SwIn::Chord(Dir::Next));
    check(vec![
        (
            "a quick tap activates the previous app with no UI",
            vec![chord, (80, SwIn::ModifierReleased)],
            Switcher::Hidden,
            vec![SwOut::RequestTick(Stamp(150)), SwOut::Activate("firefox")],
        ),
        (
            "held: the panel appears at 150 ms with index 1",
            vec![chord, (149, SwIn::Tick), (150, SwIn::Tick)],
            Switcher::Shown { sel: 1 },
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
            ],
        ),
        (
            "Tab twice then release activates index 3",
            vec![
                chord,
                (150, SwIn::Tick),
                (200, SwIn::Key(SwKey::Tab)),
                (250, SwIn::Key(SwKey::Tab)),
                (300, SwIn::ModifierReleased),
            ],
            Switcher::Hidden,
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Select(2),
                SwOut::Select(3),
                SwOut::Hide,
                SwOut::Activate("mail"),
            ],
        ),
        (
            "Escape activates nothing",
            vec![chord, (150, SwIn::Tick), (200, SwIn::Key(SwKey::Escape))],
            Switcher::Hidden,
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Hide,
            ],
        ),
        (
            "Q quits the selection and stays",
            vec![chord, (150, SwIn::Tick), (200, SwIn::Key(SwKey::Q))],
            Switcher::Shown { sel: 1 },
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Quit("firefox"),
            ],
        ),
        (
            "grave steps back and wraps",
            vec![
                chord,
                (150, SwIn::Tick),
                (200, SwIn::Key(SwKey::Grave)),
                (220, SwIn::Key(SwKey::Grave)),
            ],
            Switcher::Shown { sel: 3 },
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Select(0),
                SwOut::Select(3),
            ],
        ),
        (
            "a click activates the clicked app",
            vec![chord, (150, SwIn::Tick), (200, SwIn::Click(2))],
            Switcher::Hidden,
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Hide,
                SwOut::Activate("foot"),
            ],
        ),
        (
            "previous from hidden starts at the last app",
            vec![
                (0, SwIn::Chord(Dir::Previous)),
                (50, SwIn::ModifierReleased),
            ],
            Switcher::Hidden,
            vec![SwOut::RequestTick(Stamp(150)), SwOut::Activate("mail")],
        ),
    ]);
}

/// The quick-tap rule when the release may go unseen: what arrives inside the show delay
/// decides.
#[test]
#[ignore = "S0-QB-F"]
fn quick_taps_whose_release_is_not_seen() {
    let chord = (0, SwIn::Chord(Dir::Next));
    check(vec![
        (
            "a second chord inside the delay switches to the previous app, no UI",
            vec![chord, (90, SwIn::Chord(Dir::Next))],
            Switcher::Hidden,
            vec![SwOut::RequestTick(Stamp(150)), SwOut::Activate("firefox")],
        ),
        (
            "a second previous chord inside the delay switches to what the first chose",
            vec![
                (0, SwIn::Chord(Dir::Previous)),
                (90, SwIn::Chord(Dir::Previous)),
            ],
            Switcher::Hidden,
            vec![SwOut::RequestTick(Stamp(150)), SwOut::Activate("mail")],
        ),
        (
            "a second chord after the delay, before its tick, steps",
            vec![chord, (170, SwIn::Chord(Dir::Next)), (175, SwIn::Tick)],
            Switcher::Shown { sel: 2 },
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(2),
            ],
        ),
        (
            "a Tab seen while armed steps (the chord is held)",
            vec![
                chord,
                (60, SwIn::Key(SwKey::Tab)),
                (80, SwIn::ModifierReleased),
            ],
            Switcher::Hidden,
            vec![SwOut::RequestTick(Stamp(150)), SwOut::Activate("foot")],
        ),
        (
            "Q while armed does nothing",
            vec![chord, (60, SwIn::Key(SwKey::Q)), (150, SwIn::Tick)],
            Switcher::Shown { sel: 1 },
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
            ],
        ),
        (
            "Escape while armed cancels with nothing shown",
            vec![chord, (60, SwIn::Key(SwKey::Escape)), (150, SwIn::Tick)],
            Switcher::Hidden,
            vec![SwOut::RequestTick(Stamp(150))],
        ),
        (
            "H hides the selection and stays; hover selects",
            vec![
                chord,
                (150, SwIn::Tick),
                (200, SwIn::Key(SwKey::H)),
                (220, SwIn::Hover(3)),
                (230, SwIn::Hover(9)),
            ],
            Switcher::Shown { sel: 3 },
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::HideApp("firefox"),
                SwOut::Select(3),
            ],
        ),
    ]);
}

/// 03 §1.3: while an app is selected, Down (or Up) opens App Exposé for it and the panel goes.
#[test]
#[ignore = "S0-QB-F"]
fn down_or_up_opens_app_expose_for_the_selection() {
    let chord = (0, SwIn::Chord(Dir::Next));
    check(vec![
        (
            "Down while shown hides the panel and exposes the selection",
            vec![chord, (150, SwIn::Tick), (200, SwIn::Key(SwKey::Down))],
            Switcher::Hidden,
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Hide,
                SwOut::Expose("firefox"),
            ],
        ),
        (
            "Up after a Tab exposes the app the Tab selected",
            vec![
                chord,
                (150, SwIn::Tick),
                (200, SwIn::Key(SwKey::Tab)),
                (250, SwIn::Key(SwKey::Up)),
            ],
            Switcher::Hidden,
            vec![
                SwOut::RequestTick(Stamp(150)),
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Select(2),
                SwOut::Hide,
                SwOut::Expose("foot"),
            ],
        ),
        (
            "Down while armed moves nothing; the release still switches",
            vec![
                chord,
                (60, SwIn::Key(SwKey::Down)),
                (80, SwIn::ModifierReleased),
            ],
            Switcher::Hidden,
            vec![SwOut::RequestTick(Stamp(150)), SwOut::Activate("firefox")],
        ),
    ]);
}

#[test]
#[ignore = "S0-QB-F"]
fn no_apps_is_no_switcher() {
    let p = params();
    let none: &[&str] = &[];
    assert_eq!(
        step(Switcher::Hidden, SwIn::Chord(Dir::Next), Stamp(0), none, &p),
        (Switcher::Hidden, vec![])
    );
    assert_eq!(
        step(Switcher::Shown { sel: 2 }, SwIn::Tick, Stamp(0), none, &p),
        (Switcher::Hidden, vec![SwOut::Hide])
    );
}
