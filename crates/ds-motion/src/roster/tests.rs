//! The roster's table: what each input does to the rows and to the one wake.

use super::{
    LeaveBy, Measured, RosterIn, RosterOut, RosterParams, RosterState, RowPitch, presence_slug,
};
use crate::anim::Anim;
use crate::presence::{Exit, Presence};
use crate::settle::settle;
use ds_core::geometry::units::Px;
use ds_core::machine::Machine;
use ds_core::time::stamp::Stamp;
use ds_style::appearance::motion::MotionLevel;

fn params(leave: LeaveBy) -> RosterParams {
    RosterParams {
        leave,
        exit: Exit::Row,
        pitch: RowPitch(Px(79.0)),
        motion: MotionLevel::Standard,
    }
}

/// `anim`'s settle at Standard, in ms.
fn ms(anim: Anim) -> u64 {
    u64::try_from(settle(anim, MotionLevel::Standard).as_millis()).unwrap()
}

type Roster = RosterState<&'static str>;
type Input = RosterIn<&'static str>;

fn step(
    state: Roster,
    input: Input,
    at: u64,
    leave: LeaveBy,
) -> (Roster, Vec<RosterOut<&'static str>>) {
    state.step(input, Stamp(at), &params(leave), &Measured(Vec::new()))
}

fn rows(state: &Roster) -> Vec<(&'static str, &'static str)> {
    state
        .entries()
        .iter()
        .map(|entry| (entry.key, presence_slug(entry.presence, entry.heal)))
        .collect()
}

fn abcd() -> Roster {
    RosterState::first_show(&["a", "b", "c", "d"])
}

#[test]
fn a_first_show_is_simply_there_and_asks_for_no_wake() {
    let state = abcd();
    assert_eq!(state.wake(), None);
    let (same, outs) = step(
        state.clone(),
        RosterIn::List(vec!["a", "b", "c", "d"]),
        10,
        LeaveBy::Action,
    );
    assert_eq!(
        (same, outs),
        (state, vec![]),
        "the same keys change nothing"
    );
}

#[test]
fn a_new_key_enters_and_rests_when_its_animation_has_settled() {
    let (listed, _) = step(
        abcd(),
        RosterIn::List(vec!["a", "b", "c", "d", "x"]),
        100,
        LeaveBy::Action,
    );
    assert_eq!(rows(&listed)[4], ("x", "entering"));
    assert_eq!(listed.wake(), Some(Stamp(100 + ms(Anim::RowIn))));
    let (early, _) = step(
        listed.clone(),
        RosterIn::Elapsed,
        99 + ms(Anim::RowIn),
        LeaveBy::Action,
    );
    assert_eq!(early, listed, "a wake before the deadline changes nothing");
    let (rested, _) = step(
        listed,
        RosterIn::Elapsed,
        100 + ms(Anim::RowIn),
        LeaveBy::Action,
    );
    assert_eq!(rows(&rested)[4], ("x", "present"));
    assert_eq!(rested.wake(), None);
}

#[test]
fn two_listings_before_the_rest_share_one_deadline_that_only_moves_later() {
    let (one, _) = step(
        abcd(),
        RosterIn::List(vec!["a", "b", "c", "d", "x"]),
        100,
        LeaveBy::Action,
    );
    let (two, _) = step(
        one,
        RosterIn::List(vec!["a", "b", "c", "d", "x", "y"]),
        130,
        LeaveBy::Action,
    );
    assert_eq!(two.wake(), Some(Stamp(130 + ms(Anim::RowIn))));
    let (rested, _) = step(
        two,
        RosterIn::Elapsed,
        130 + ms(Anim::RowIn),
        LeaveBy::Action,
    );
    assert!(rows(&rested).iter().all(|(_, word)| *word == "present"));
}

#[test]
fn an_action_leave_drops_the_row_at_its_settle_and_heals_the_rows_below() {
    let (leaving, _) = step(abcd(), RosterIn::Leave(vec!["b"]), 50, LeaveBy::Action);
    let drop_at = 50 + ms(Anim::RowOut);
    assert_eq!(rows(&leaving)[1], ("b", "leaving"));
    assert_eq!(leaving.wake(), Some(Stamp(drop_at)));
    let (early, outs) = step(
        leaving.clone(),
        RosterIn::Elapsed,
        drop_at - 1,
        LeaveBy::Action,
    );
    assert_eq!(
        (early, outs),
        (leaving.clone(), vec![]),
        "mid exit: still there"
    );

    let (dropped, outs) = step(leaving, RosterIn::Elapsed, drop_at, LeaveBy::Action);
    assert_eq!(outs, vec![RosterOut::Settled("b")]);
    assert_eq!(
        rows(&dropped),
        vec![("a", "present"), ("c", "healing"), ("d", "healing")]
    );
    assert_eq!(dropped.wake(), Some(Stamp(drop_at + ms(Anim::Heal))));
    let (rested, _) = step(
        dropped,
        RosterIn::Elapsed,
        drop_at + ms(Anim::Heal),
        LeaveBy::Action,
    );
    assert!(rows(&rested).iter().all(|(_, word)| *word == "present"));
    assert_eq!(rested.wake(), None);
}

#[test]
fn a_batch_drops_together_and_heals_by_the_measured_pitches() {
    let (leaving, _) = step(abcd(), RosterIn::Leave(vec!["a", "c"]), 0, LeaveBy::Action);
    let measured = Measured(vec![("a", RowPitch(Px(40.0)))]);
    let at = Stamp(ms(Anim::RowOut));
    let (dropped, outs) = leaving.step(RosterIn::Elapsed, at, &params(LeaveBy::Action), &measured);
    assert_eq!(outs, vec![RosterOut::Settled("a"), RosterOut::Settled("c")]);
    let dy: Vec<_> = dropped
        .entries()
        .iter()
        .map(|entry| entry.heal.map(|heal| heal.dy))
        .collect();
    assert_eq!(
        dy,
        vec![Some(Px(40.0)), Some(Px(40.0 + 79.0))],
        "b below a heals by a's pitch, d below both by a's and c's (c unmeasured: the fallback)"
    );
}

#[test]
fn a_delisted_row_plays_its_exit_and_a_removal_without_one_drops_at_once() {
    let keys = vec!["a", "c", "d"];
    let (delisted, _) = step(abcd(), RosterIn::List(keys.clone()), 0, LeaveBy::Delist);
    assert_eq!(rows(&delisted)[1], ("b", "leaving"), "still drawn");
    assert_eq!(delisted.wake(), Some(Stamp(ms(Anim::RowOut))));
    let (gone, outs) = step(
        delisted,
        RosterIn::Elapsed,
        ms(Anim::RowOut),
        LeaveBy::Delist,
    );
    assert_eq!(outs, vec![RosterOut::Settled("b")]);
    assert_eq!(gone.entries().len(), 3);

    let (removed, outs) = step(abcd(), RosterIn::List(keys), 0, LeaveBy::Action);
    assert_eq!(
        (removed.entries().len(), outs),
        (3, vec![]),
        "no exit, no wake"
    );
    assert_eq!(removed.wake(), None);
}

#[test]
fn a_row_taken_back_stays_and_is_not_dropped_by_its_old_deadline() {
    let (leaving, _) = step(abcd(), RosterIn::Leave(vec!["b"]), 0, LeaveBy::Action);
    let (stayed, _) = step(leaving, RosterIn::Stay("b"), 40, LeaveBy::Action);
    assert_eq!(rows(&stayed)[1], ("b", "present"));
    assert_eq!(stayed.wake(), None, "the exit's deadline went with it");
    let (leaving_again, _) = step(stayed, RosterIn::Leave(vec!["b"]), 100, LeaveBy::Action);
    let (still, outs) = step(
        leaving_again,
        RosterIn::Elapsed,
        ms(Anim::RowOut),
        LeaveBy::Action,
    );
    assert_eq!(
        outs,
        vec![],
        "the first leave's time has passed; the second's has not"
    );
    assert_eq!(rows(&still)[1], ("b", "leaving"));
}

#[test]
fn a_leaving_row_listed_again_stays_and_one_still_listed_keeps_leaving() {
    let (leaving, _) = step(
        abcd(),
        RosterIn::List(vec!["a", "c", "d"]),
        0,
        LeaveBy::Delist,
    );
    let (back, _) = step(
        leaving,
        RosterIn::List(vec!["a", "b", "c", "d"]),
        30,
        LeaveBy::Delist,
    );
    assert_eq!(rows(&back)[1], ("b", "present"), "listed again: taken back");

    let (leaving, _) = step(abcd(), RosterIn::Leave(vec!["b"]), 0, LeaveBy::Action);
    let (other, _) = step(
        leaving,
        RosterIn::List(vec!["a", "b", "c", "d", "x"]),
        30,
        LeaveBy::Action,
    );
    assert_eq!(rows(&other)[1], ("b", "leaving"), "b was not newly listed");
}

#[test]
fn a_leaving_presence_is_the_exit_asked_for() {
    let (leaving, _) = step(abcd(), RosterIn::Leave(vec!["b"]), 0, LeaveBy::Action);
    assert_eq!(leaving.entries()[1].presence, Presence::Leaving(Exit::Row));
}

#[test]
fn a_windowed_list_starts_the_exit_of_a_removed_row_and_drops_a_scrolled_out_one_at_once() {
    // A list that mounts only a window of its keys: `Leave` for the key the caller removed, then
    // `List` with the window now. The leaver stays after the row that preceded it, a key that
    // only left the window is dropped with no exit and no heal, and the rows below the leaver
    // heal when it settles.
    let (state, _) = step(abcd(), RosterIn::Leave(vec!["b"]), 10, LeaveBy::Action);
    let (state, _) = step(
        state,
        RosterIn::List(vec!["c", "d", "e"]),
        10,
        LeaveBy::Action,
    );
    assert_eq!(
        rows(&state),
        vec![
            ("b", "leaving"),
            ("c", "present"),
            ("d", "present"),
            ("e", "entering")
        ],
        "a (scrolled out) is gone; b plays out first, nothing above it to follow"
    );
    let drop_at = 10 + ms(Anim::RowOut);
    let (dropped, outs) = step(state, RosterIn::Elapsed, drop_at, LeaveBy::Action);
    assert_eq!(outs, vec![RosterOut::Settled("b")]);
    assert_eq!(
        rows(&dropped),
        vec![("c", "healing"), ("d", "healing"), ("e", "healing")],
        "the rows that were below b heal"
    );
}
