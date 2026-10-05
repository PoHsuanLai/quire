//! The switcher's tables (design/13 §13.4 acceptance 8, the quick-tap rule, App Exposé from 03
//! §1.3). The app key is a plain string here; the machine never looks inside it.

use std::time::Duration;

use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;

use super::{Phase, SwIn, SwKey, SwOut, Switcher, SwitcherParams};
use crate::dir::Dir;

/// One scripted input sequence: `(ms, input)` pairs.
type Script = Vec<(u64, SwIn)>;

/// Name, script, final state, every output in order.
type Case = (&'static str, Script, Phase, Vec<SwOut<&'static str>>);

const APPS: [&str; 4] = ["files", "firefox", "foot", "mail"];

fn params() -> SwitcherParams {
    SwitcherParams {
        show_delay: Duration::from_millis(150),
    }
}

fn apps() -> Vec<&'static str> {
    APPS.to_vec()
}

fn run(inputs: &[(u64, SwIn)]) -> (Phase, Vec<SwOut<&'static str>>) {
    let p = params();
    let (switcher, outs) = inputs.iter().fold(
        (Switcher::hidden(), Vec::new()),
        |(s, mut all), &(ms, input)| {
            let (s, out) = s.step(input, Stamp(ms), &p, &apps());
            all.extend(out);
            (s, all)
        },
    );
    (switcher.phase(), outs)
}

fn check(cases: Vec<Case>) {
    for (name, inputs, want_state, want_out) in cases {
        assert_eq!(run(&inputs), (want_state, want_out), "{name}");
    }
}

#[test]
fn design_13_acceptance_8() {
    let chord = (0, SwIn::Chord(Dir::Next));
    check(vec![
        (
            "a quick tap activates the previous app with no UI",
            vec![chord, (80, SwIn::ModifierReleased)],
            Phase::Hidden,
            vec![SwOut::Activate("firefox")],
        ),
        (
            "held: the panel appears at 150 ms with index 1",
            vec![chord, (149, SwIn::Elapsed), (150, SwIn::Elapsed)],
            Phase::Shown { sel: 1 },
            vec![SwOut::Show, SwOut::Select(1)],
        ),
        (
            "Tab twice then release activates index 3",
            vec![
                chord,
                (150, SwIn::Elapsed),
                (200, SwIn::Key(SwKey::Tab)),
                (250, SwIn::Key(SwKey::Tab)),
                (300, SwIn::ModifierReleased),
            ],
            Phase::Hidden,
            vec![
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
            vec![chord, (150, SwIn::Elapsed), (200, SwIn::Key(SwKey::Escape))],
            Phase::Hidden,
            vec![SwOut::Show, SwOut::Select(1), SwOut::Hide],
        ),
        (
            "Q quits the selection and stays",
            vec![chord, (150, SwIn::Elapsed), (200, SwIn::Key(SwKey::Q))],
            Phase::Shown { sel: 1 },
            vec![SwOut::Show, SwOut::Select(1), SwOut::Quit("firefox")],
        ),
        (
            "grave steps back and wraps",
            vec![
                chord,
                (150, SwIn::Elapsed),
                (200, SwIn::Key(SwKey::Grave)),
                (220, SwIn::Key(SwKey::Grave)),
            ],
            Phase::Shown { sel: 3 },
            vec![
                SwOut::Show,
                SwOut::Select(1),
                SwOut::Select(0),
                SwOut::Select(3),
            ],
        ),
        (
            "a click activates the clicked app",
            vec![chord, (150, SwIn::Elapsed), (200, SwIn::Click(2))],
            Phase::Hidden,
            vec![
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
            Phase::Hidden,
            vec![SwOut::Activate("mail")],
        ),
    ]);
}

/// The quick-tap rule when the release may go unseen: what arrives inside the show delay
/// decides.
#[test]
fn quick_taps_whose_release_is_not_seen() {
    let chord = (0, SwIn::Chord(Dir::Next));
    check(vec![
        (
            "a second chord inside the delay switches to the previous app, no UI",
            vec![chord, (90, SwIn::Chord(Dir::Next))],
            Phase::Hidden,
            vec![SwOut::Activate("firefox")],
        ),
        (
            "a second previous chord inside the delay switches to what the first chose",
            vec![
                (0, SwIn::Chord(Dir::Previous)),
                (90, SwIn::Chord(Dir::Previous)),
            ],
            Phase::Hidden,
            vec![SwOut::Activate("mail")],
        ),
        (
            "a second chord after the delay, before its tick, steps",
            vec![chord, (170, SwIn::Chord(Dir::Next)), (175, SwIn::Elapsed)],
            Phase::Shown { sel: 2 },
            vec![SwOut::Show, SwOut::Select(2)],
        ),
        (
            "a Tab seen while armed steps (the chord is held)",
            vec![
                chord,
                (60, SwIn::Key(SwKey::Tab)),
                (80, SwIn::ModifierReleased),
            ],
            Phase::Hidden,
            vec![SwOut::Activate("foot")],
        ),
        (
            "Q while armed does nothing",
            vec![chord, (60, SwIn::Key(SwKey::Q)), (150, SwIn::Elapsed)],
            Phase::Shown { sel: 1 },
            vec![SwOut::Show, SwOut::Select(1)],
        ),
        (
            "Escape while armed cancels with nothing shown",
            vec![chord, (60, SwIn::Key(SwKey::Escape)), (150, SwIn::Elapsed)],
            Phase::Hidden,
            vec![],
        ),
        (
            "H hides the selection and stays; hover selects",
            vec![
                chord,
                (150, SwIn::Elapsed),
                (200, SwIn::Key(SwKey::H)),
                (220, SwIn::Hover(3)),
                (230, SwIn::Hover(9)),
            ],
            Phase::Shown { sel: 3 },
            vec![
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
fn down_or_up_opens_app_expose_for_the_selection() {
    let chord = (0, SwIn::Chord(Dir::Next));
    check(vec![
        (
            "Down while shown hides the panel and exposes the selection",
            vec![chord, (150, SwIn::Elapsed), (200, SwIn::Key(SwKey::Down))],
            Phase::Hidden,
            vec![
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
                (150, SwIn::Elapsed),
                (200, SwIn::Key(SwKey::Tab)),
                (250, SwIn::Key(SwKey::Up)),
            ],
            Phase::Hidden,
            vec![
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
            Phase::Hidden,
            vec![SwOut::Activate("firefox")],
        ),
    ]);
}

/// One step from `phase` over `apps`, as the phase it leaves and what it wants.
fn once(
    phase: Phase,
    input: SwIn,
    at: u64,
    apps: &[&'static str],
) -> (Phase, Vec<SwOut<&'static str>>) {
    let (next, outs) = Switcher::in_phase(phase).step(input, Stamp(at), &params(), &apps.to_vec());
    (next.phase(), outs)
}

#[test]
fn no_apps_is_no_switcher() {
    assert_eq!(
        once(Phase::Hidden, SwIn::Chord(Dir::Next), 0, &[]),
        (Phase::Hidden, vec![])
    );
    assert_eq!(
        once(Phase::Shown { sel: 2 }, SwIn::Elapsed, 0, &[]),
        (Phase::Hidden, vec![SwOut::Hide])
    );
}

#[test]
fn a_selection_past_a_shrunk_list_falls_back_to_the_last_app() {
    let two = &APPS[..2];
    assert_eq!(
        once(Phase::Shown { sel: 3 }, SwIn::Key(SwKey::Q), 0, two),
        (Phase::Shown { sel: 1 }, vec![SwOut::Quit("firefox")])
    );
    let armed = Phase::Armed {
        until: Stamp(150),
        sel: 3,
    };
    assert_eq!(
        once(armed, SwIn::ModifierReleased, 50, two),
        (Phase::Hidden, vec![SwOut::Activate("firefox")])
    );
}

#[test]
fn only_an_armed_switcher_wakes_and_at_the_end_of_the_show_delay() {
    let p = params();
    let wake = |phase| Switcher::<&str>::in_phase(phase).wake();
    assert_eq!(wake(Phase::Hidden), None);
    assert_eq!(wake(Phase::Shown { sel: 1 }), None);
    let (armed, _) = Switcher::hidden().step(SwIn::Chord(Dir::Next), Stamp(40), &p, &apps());
    assert_eq!(armed.wake(), Some(Stamp(190)));
}
