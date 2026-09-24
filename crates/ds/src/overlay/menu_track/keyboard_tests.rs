//! The two keyboard events a ds `Menu` feeds (`Select`, `Expand`) and the tracker that starts
//! open (`MenuTrack::open`): quire gap Q7, where the Menu's submenus are driven by this machine.

use super::{
    Held, ItemPath, MenuKey, MenuPhase, MenuTiming, MenuTrack, MenuTrackEffect, MenuTrackEvent,
    Submenu,
};
use std::sync::LazyLock;
use std::time::{Duration, Instant};

type Event = MenuTrackEvent<()>;
type Effect = MenuTrackEffect<()>;

static ZERO: LazyLock<Instant> = LazyLock::new(Instant::now);

fn at(ms: u64) -> Instant {
    *ZERO + Duration::from_millis(ms)
}

fn path(i: u16) -> ItemPath {
    ItemPath(vec![i])
}

/// Feed `events` to a tracker that starts open; the last phase and every effect, in order.
fn run(events: Vec<Event>) -> (MenuPhase<()>, Vec<Effect>) {
    let start = (MenuTrack::open(MenuTiming::default(), ()), Vec::new());
    let (track, effects) =
        events
            .into_iter()
            .enumerate()
            .fold(start, |(track, mut all), (ms, event)| {
                let (track, effects) = track.step(event, at(ms as u64));
                all.extend(effects);
                (track, all)
            });
    (track.phase, effects)
}

fn sub(phase: &MenuPhase<()>) -> Submenu {
    match phase {
        MenuPhase::Tracking(session) => session.sub.clone(),
        MenuPhase::Closed => panic!("expected an open menu"),
    }
}

#[test]
fn an_open_tracker_is_in_click_mode() {
    let track = MenuTrack::open(MenuTiming::default(), ());
    assert_eq!(track.open_menu(), Some(&()));
    let MenuPhase::Tracking(session) = &track.phase else {
        panic!("expected an open menu");
    };
    assert_eq!(session.held, Held::Released);
}

#[test]
fn expand_opens_at_once_and_left_closes_it() {
    let (phase, effects) = run(vec![Event::Expand(path(2))]);
    assert_eq!(
        effects,
        vec![Effect::Highlight(Some(path(2))), Effect::OpenSub(path(2))]
    );
    assert_eq!(
        sub(&phase),
        Submenu::Open {
            item: path(2),
            guard: None
        }
    );
    let (phase, effects) = run(vec![Event::Expand(path(2)), Event::Key(MenuKey::Left)]);
    assert_eq!(effects.last(), Some(&Effect::CloseSub));
    assert_eq!(sub(&phase), Submenu::None);
}

#[test]
fn expanding_another_item_closes_the_first() {
    let (_, effects) = run(vec![Event::Expand(path(2)), Event::Expand(path(4))]);
    assert_eq!(
        effects[2..],
        [
            Effect::Highlight(Some(path(4))),
            Effect::CloseSub,
            Effect::OpenSub(path(4))
        ]
    );
    let (_, effects) = run(vec![Event::Expand(path(2)), Event::Expand(path(2))]);
    assert_eq!(
        effects.len(),
        2,
        "expanding the open item again changes nothing"
    );
}

#[test]
fn select_moves_the_highlight_and_closes_another_items_submenu() {
    // (events, the last step's effects, the submenu after)
    let cases: Vec<(Vec<Event>, Vec<Effect>, Submenu)> = vec![
        (
            vec![Event::Select(path(1))],
            vec![Effect::Highlight(Some(path(1)))],
            Submenu::None,
        ),
        (
            vec![Event::Expand(path(2)), Event::Select(path(3))],
            vec![Effect::Highlight(Some(path(3))), Effect::CloseSub],
            Submenu::None,
        ),
        (
            vec![Event::Expand(path(2)), Event::Select(path(2))],
            vec![],
            Submenu::Open {
                item: path(2),
                guard: None,
            },
        ),
    ];
    for (events, want, want_sub) in cases {
        let last = events.len() - 1;
        let (earlier, _) = run(events[..last].to_vec());
        let track = MenuTrack {
            timing: MenuTiming::default(),
            phase: earlier,
        };
        let (track, effects) = track.step(events[last].clone(), at(50));
        assert_eq!(effects, want, "{events:?}");
        assert_eq!(sub(&track.phase), want_sub, "{events:?}");
    }
}
