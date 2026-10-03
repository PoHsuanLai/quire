//! design/12 §12.3.7's numbers and the swipe machine of 03 §4.2 and §1.5, as tables. Every
//! scripted gesture moves the fingers at a constant speed inside the velocity window, so the fit
//! is exact whichever samples the window keeps.

use std::time::Duration;

use ds_core::machine::{Elapsed, Machine};
use ds_core::time::stamp::Stamp;

use super::{
    PageMilli, PagesPerSecMilli, Reduced, Swipe, SwipeIn, SwipeOut, SwipeParams, SwipeSource,
    commit, finish_duration, rubber,
};

const FOUR: SwipeParams = SwipeParams {
    spaces: 4,
    reduced: Reduced::No,
};

const FOUR_REDUCED: SwipeParams = SwipeParams {
    spaces: 4,
    reduced: Reduced::Yes,
};

const MOUSE: SwipeIn = SwipeIn::Began {
    source: SwipeSource::MagicMouse,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn offset(p: i32) -> SwipeOut {
    SwipeOut::Offset { p: PageMilli(p) }
}

fn finishing(from: u32, to: u32, p0: i32, v0: i32, since: u64) -> Swipe {
    Swipe::Finishing {
        from,
        to,
        p0: PageMilli(p0),
        v0: PagesPerSecMilli(v0),
        since: Stamp(since),
    }
}

#[test]
fn commit_follows_half_a_page_and_the_flick() {
    #[rustfmt::skip]
    const CASES: &[(&str, i32, i32, i32)] = &[
        ("no movement commits nothing",                 0,     0,     0),
        ("no movement ignores a flick",                 0,     5000,  0),
        ("just under half a page returns",              499,   0,     0),
        ("half a page commits forward",                 500,   0,     1),
        ("half a page back commits back",               -500,  0,     -1),
        ("a long drag commits only one Space",          1800,  0,     1),
        ("past half, a weak reverse flick commits",     600,   -1499, 1),
        ("past half, a reverse flick returns",          600,   -1500, 0),
        ("past half back, a reverse flick returns",     -600,  1500,  0),
        ("under half, a forward flick commits",         300,   1500,  1),
        ("under half, a weaker flick returns",          300,   1499,  0),
        ("under half back, a flick back commits back",  -300,  -1500, -1),
        ("under half, a reverse flick returns",         300,   -2000, 0),
    ];
    for (name, p, v, want) in CASES {
        assert_eq!(commit(PageMilli(*p), PagesPerSecMilli(*v)), *want, "{name}");
    }
}

#[test]
fn the_finish_takes_the_lift_speed_within_100_to_400_ms() {
    #[rustfmt::skip]
    const CASES: &[(&str, i32, i32, u64)] = &[
        ("half a page at 5 pages/s",                    500,   5000,  300),
        ("backwards at 6 pages/s",                      -600,  -6000, 300),
        ("a quarter page at 3 pages/s",                 250,   3000,  250),
        ("0.7 page at 10 pages/s",                      700,   10000, 210),
        ("a slow lift counts as 2 pages/s, capped",     500,   0,     400),
        ("a speed under 2 pages/s counts as 2",         200,   1000,  300),
        ("the speed's sign does not matter",            200,   -1000, 300),
        ("a short fast finish is at least 100 ms",      300,   10000, 100),
        ("no distance is the shortest finish",          0,     0,     100),
    ];
    for (name, dp, v, want) in CASES {
        assert_eq!(
            finish_duration(PageMilli(*dp), PagesPerSecMilli(*v)),
            ms(*want),
            "{name}"
        );
    }
}

#[test]
fn the_rubber_band_gives_a_quarter_capped_at_a_quarter_page() {
    #[rustfmt::skip]
    const CASES: &[(&str, i32, u32, u32, i32)] = &[
        ("inside the row is itself",                    500,   0, 3, 500),
        ("on the first Space is itself",                0,     0, 3, 0),
        ("on the last Space is itself",                 3000,  0, 3, 3000),
        ("0.4 before the first shows 0.1",              -400,  0, 3, -100),
        ("half a page before shows an eighth",          -500,  0, 3, -125),
        ("a page before is capped at a quarter",        -1000, 0, 3, -250),
        ("two pages before is still a quarter",         -2000, 0, 3, -250),
        ("0.4 past the last shows 0.1",                 3400,  0, 3, 3100),
        ("a page past the last is capped",              4000,  0, 3, 3250),
        ("a lone Space bands both ways",                200,   0, 0, 50),
        ("a lone Space bands back too",                 -200,  0, 0, -50),
        ("a row starting later bands at its start",     1600,  2, 4, 1900),
        ("inside a row starting later",                 2000,  2, 4, 2000),
    ];
    for (name, p, first, last, want) in CASES {
        assert_eq!(
            rubber(PageMilli(*p), *first, *last),
            PageMilli(*want),
            "{name}"
        );
    }
}

/// Every output of a script run from `start`, the final state and its wake.
fn run(
    start: Swipe,
    script: &[(u64, SwipeIn)],
    params: &SwipeParams,
) -> (Swipe, Vec<SwipeOut>, Option<Stamp>) {
    let (state, outs) = script
        .iter()
        .fold((start, Vec::new()), |(s, mut all), &(at, input)| {
            let (s, out) = s.step(input, Stamp(at), params);
            all.extend(out);
            (s, all)
        });
    let wake = state.wake();
    (state, outs, wake)
}

/// Name, start, script, final state, outputs, wake.
type Case = (
    &'static str,
    Swipe,
    Vec<(u64, SwipeIn)>,
    Swipe,
    Vec<SwipeOut>,
    Option<Stamp>,
);

fn check(cases: Vec<Case>, params: &SwipeParams) {
    for (name, start, script, state, outs, wake) in cases {
        assert_eq!(run(start, &script, params), (state, outs, wake), "{name}");
    }
}

#[test]
fn the_row_follows_the_fingers() {
    let moved = |source, p: i32, dx: i32| {
        let (state, outs, _) = run(
            Swipe::Idle { at: 1 },
            &[
                (0, SwipeIn::Began { source }),
                (10, SwipeIn::Changed { dx }),
            ],
            &FOUR,
        );
        let Swipe::Tracking {
            from: 1,
            source: s,
            p: raw,
            ..
        } = state
        else {
            panic!("{source:?} {dx}: not tracking from 1: {state:?}");
        };
        assert_eq!(
            (s, raw),
            (source, PageMilli(p)),
            "{source:?} {dx}: raw position"
        );
        outs
    };
    assert_eq!(
        moved(SwipeSource::MagicMouse, 1300, -300),
        vec![offset(1300)],
        "fingers left bring the next Space in, a page per 1000 units"
    );
    assert_eq!(
        moved(SwipeSource::Touchpad, 500, 200),
        vec![offset(500)],
        "touchpad fingers right bring the previous Space in, a page per 400 px"
    );
    assert_eq!(
        moved(SwipeSource::MagicMouse, 3000, -2000),
        vec![offset(3000)],
        "two pages along stays inside the row"
    );
    let (state, outs, wake) = run(
        Swipe::Idle { at: 0 },
        &[(0, MOUSE), (10, SwipeIn::Changed { dx: 400 })],
        &FOUR,
    );
    assert!(
        matches!(
            state,
            Swipe::Tracking {
                from: 0,
                p: PageMilli(-400),
                ..
            }
        ),
        "the raw position keeps the full move: {state:?}"
    );
    assert_eq!(
        (outs, wake),
        (vec![offset(-100)], None),
        "before the first Space it bands"
    );
}

#[test]
fn a_lift_finishes_to_the_committed_space() {
    let changed = |dx| SwipeIn::Changed { dx };
    check(
        vec![
            (
                "a slow drag past half a page commits",
                Swipe::Idle { at: 1 },
                vec![(0, MOUSE), (10, changed(-600)), (200, SwipeIn::Ended)],
                finishing(1, 2, 1600, 0, 200),
                vec![offset(1600), SwipeOut::Committed { to: 2 }],
                Some(Stamp(600)),
            ),
            (
                "a quick flick under half a page commits at its speed",
                Swipe::Idle { at: 1 },
                vec![
                    (0, MOUSE),
                    (10, changed(-100)),
                    (20, changed(-100)),
                    (30, changed(-100)),
                    (30, SwipeIn::Ended),
                ],
                finishing(1, 2, 1300, 10000, 30),
                vec![
                    offset(1100),
                    offset(1200),
                    offset(1300),
                    SwipeOut::Committed { to: 2 },
                ],
                Some(Stamp(240)),
            ),
            (
                "a slow drag under half a page returns",
                Swipe::Idle { at: 1 },
                vec![(0, MOUSE), (10, changed(-300)), (200, SwipeIn::Ended)],
                finishing(1, 1, 1300, 0, 200),
                vec![offset(1300), SwipeOut::Committed { to: 1 }],
                Some(Stamp(600)),
            ),
            (
                "a reverse flick past half a page returns",
                Swipe::Idle { at: 1 },
                vec![
                    (0, MOUSE),
                    (0, changed(-700)),
                    (300, changed(50)),
                    (310, changed(50)),
                    (320, changed(50)),
                    (320, SwipeIn::Ended),
                ],
                finishing(1, 1, 1550, -5000, 320),
                vec![
                    offset(1700),
                    offset(1650),
                    offset(1600),
                    offset(1550),
                    SwipeOut::Committed { to: 1 },
                ],
                Some(Stamp(650)),
            ),
            (
                "a long drag moves one Space only",
                Swipe::Idle { at: 1 },
                vec![(0, MOUSE), (10, changed(-1800)), (200, SwipeIn::Ended)],
                finishing(1, 2, 2800, 0, 200),
                vec![offset(2800), SwipeOut::Committed { to: 2 }],
                Some(Stamp(600)),
            ),
            (
                "past the last Space it bands and returns",
                Swipe::Idle { at: 3 },
                vec![(0, MOUSE), (10, changed(-600)), (200, SwipeIn::Ended)],
                finishing(3, 3, 3150, 0, 200),
                vec![offset(3150), SwipeOut::Committed { to: 3 }],
                Some(Stamp(425)),
            ),
            (
                "a cancel returns with no speed",
                Swipe::Idle { at: 1 },
                vec![
                    (0, MOUSE),
                    (10, changed(-100)),
                    (20, changed(-100)),
                    (30, changed(-100)),
                    (30, SwipeIn::Cancelled),
                ],
                finishing(1, 1, 1300, 0, 30),
                vec![
                    offset(1100),
                    offset(1200),
                    offset(1300),
                    SwipeOut::Committed { to: 1 },
                ],
                Some(Stamp(430)),
            ),
            (
                "the finish's end settles",
                finishing(1, 2, 1600, 0, 200),
                vec![(599, SwipeIn::Elapsed), (600, SwipeIn::Elapsed)],
                Swipe::Idle { at: 2 },
                vec![SwipeOut::Settled { at: 2 }],
                None,
            ),
        ],
        &FOUR,
    );
}

#[test]
fn go_slides_and_redirects() {
    check(
        vec![
            (
                "Go slides from the shown Space",
                Swipe::Idle { at: 1 },
                vec![(100, SwipeIn::Go { to: 3 })],
                finishing(1, 3, 1000, 0, 100),
                vec![SwipeOut::Committed { to: 3 }],
                Some(Stamp(500)),
            ),
            (
                "Go to the shown Space does nothing",
                Swipe::Idle { at: 1 },
                vec![(100, SwipeIn::Go { to: 1 })],
                Swipe::Idle { at: 1 },
                vec![],
                None,
            ),
            (
                "Go past the last Space does nothing",
                Swipe::Idle { at: 1 },
                vec![(100, SwipeIn::Go { to: 4 })],
                Swipe::Idle { at: 1 },
                vec![],
                None,
            ),
            (
                "Go redirects a slide from where it is at its speed",
                finishing(0, 1, 0, 0, 0),
                vec![(0, SwipeIn::Go { to: 2 })],
                finishing(0, 2, 0, 7500, 0),
                vec![SwipeOut::Committed { to: 2 }],
                Some(Stamp(400)),
            ),
            (
                "Go after a slide's end, before its elapse, starts from its end",
                finishing(0, 1, 0, 0, 0),
                vec![(400, SwipeIn::Go { to: 2 })],
                finishing(0, 2, 1000, 0, 400),
                vec![SwipeOut::Committed { to: 2 }],
                Some(Stamp(800)),
            ),
            (
                "Go to a slide's own target changes nothing",
                finishing(0, 1, 0, 0, 0),
                vec![(100, SwipeIn::Go { to: 1 })],
                finishing(0, 1, 0, 0, 0),
                vec![],
                Some(Stamp(400)),
            ),
            (
                "Go while the fingers are down changes nothing",
                Swipe::Idle { at: 1 },
                vec![
                    (0, MOUSE),
                    (100, SwipeIn::Go { to: 3 }),
                    (200, SwipeIn::Ended),
                ],
                finishing(1, 1, 1000, 0, 200),
                vec![SwipeOut::Committed { to: 1 }],
                Some(Stamp(300)),
            ),
        ],
        &FOUR,
    );
}

#[test]
fn a_swipe_catches_a_slide_where_it_is() {
    // 0.6 to 1.0 over 400 ms; at 200 ms the ease-out-cubic has covered 0.875 of 0.4.
    let (state, outs, wake) = run(finishing(0, 1, 600, 0, 0), &[(200, MOUSE)], &FOUR);
    assert!(
        matches!(
            state,
            Swipe::Tracking {
                from: 1,
                source: SwipeSource::MagicMouse,
                p: PageMilli(950),
                ..
            }
        ),
        "tracking from the slide's target at its position: {state:?}"
    );
    assert_eq!((outs, wake), (vec![], None));
}

#[test]
fn strays_change_nothing() {
    let tracking = |dx| {
        run(
            Swipe::Idle { at: 1 },
            &[(0, MOUSE), (10, SwipeIn::Changed { dx })],
            &FOUR,
        )
        .0
    };
    let at_rest: &[(&str, SwipeIn)] = &[
        ("a change at rest", SwipeIn::Changed { dx: -300 }),
        ("a lift at rest", SwipeIn::Ended),
        ("a cancel at rest", SwipeIn::Cancelled),
        ("an elapse at rest", SwipeIn::Elapsed),
    ];
    for (name, input) in at_rest {
        assert_eq!(
            Swipe::Idle { at: 1 }.step(*input, Stamp(50), &FOUR),
            (Swipe::Idle { at: 1 }, vec![]),
            "{name}"
        );
    }
    let while_tracking: &[(&str, SwipeIn)] = &[
        ("a second begin while tracking", MOUSE),
        ("an elapse while tracking", SwipeIn::Elapsed),
    ];
    for (name, input) in while_tracking {
        let before = tracking(-300);
        assert_eq!(
            before.clone().step(*input, Stamp(50), &FOUR),
            (before, vec![]),
            "{name}"
        );
    }
    let slide = finishing(1, 2, 1000, 0, 0);
    for (name, input) in [
        ("a change while finishing", SwipeIn::Changed { dx: -300 }),
        ("a lift while finishing", SwipeIn::Ended),
        ("a cancel while finishing", SwipeIn::Cancelled),
        ("an early elapse", SwipeIn::Elapsed),
    ] {
        assert_eq!(
            slide.clone().step(input, Stamp(399), &FOUR),
            (slide.clone(), vec![]),
            "{name}"
        );
    }
}

#[test]
fn reduced_motion_settles_at_once_with_no_live_slide() {
    check(
        vec![
            (
                "a committed swipe settles at once and draws nothing on the way",
                Swipe::Idle { at: 1 },
                vec![
                    (0, MOUSE),
                    (10, SwipeIn::Changed { dx: -600 }),
                    (200, SwipeIn::Ended),
                ],
                Swipe::Idle { at: 2 },
                vec![SwipeOut::Committed { to: 2 }, SwipeOut::Settled { at: 2 }],
                None,
            ),
            (
                "a cancel settles back at once",
                Swipe::Idle { at: 1 },
                vec![
                    (0, MOUSE),
                    (10, SwipeIn::Changed { dx: -600 }),
                    (20, SwipeIn::Cancelled),
                ],
                Swipe::Idle { at: 1 },
                vec![SwipeOut::Committed { to: 1 }, SwipeOut::Settled { at: 1 }],
                None,
            ),
            (
                "Go settles at once",
                Swipe::Idle { at: 1 },
                vec![(100, SwipeIn::Go { to: 3 })],
                Swipe::Idle { at: 3 },
                vec![SwipeOut::Committed { to: 3 }, SwipeOut::Settled { at: 3 }],
                None,
            ),
        ],
        &FOUR_REDUCED,
    );
}

#[test]
fn a_new_swipe_rests_on_the_first_space() {
    assert_eq!(Swipe::default(), Swipe::Idle { at: 0 });
    assert_eq!(SwipeIn::from(Elapsed), SwipeIn::Elapsed);
}

#[test]
fn a_touchpad_page_is_400_px_and_its_flick_commits_the_same_way() {
    const TOUCHPAD: SwipeIn = SwipeIn::Began {
        source: SwipeSource::Touchpad,
    };
    let changed = |dx| SwipeIn::Changed { dx };
    check(
        vec![(
            "a quick 120 px flick is 0.3 page at 10 pages/s",
            Swipe::Idle { at: 1 },
            vec![
                (0, TOUCHPAD),
                (10, changed(-40)),
                (20, changed(-40)),
                (30, changed(-40)),
                (30, SwipeIn::Ended),
            ],
            finishing(1, 2, 1300, 10000, 30),
            vec![
                offset(1100),
                offset(1200),
                offset(1300),
                SwipeOut::Committed { to: 2 },
            ],
            Some(Stamp(240)),
        )],
        &FOUR,
    );
}

#[test]
fn a_redirect_off_the_row_leaves_the_slide_alone() {
    let slide = finishing(0, 1, 0, 0, 0);
    assert_eq!(
        slide.clone().step(SwipeIn::Go { to: 4 }, Stamp(100), &FOUR),
        (slide, vec![])
    );
}
