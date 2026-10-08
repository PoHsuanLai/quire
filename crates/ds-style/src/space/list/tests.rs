use super::*;
use crate::appearance::theme::Theme;
use crate::space::look::{CardAccent, Grain};
use crate::space::palette::Dot;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
struct Note {
    #[serde(default)]
    folder: String,
}

type Set = Spaces<Note, String>;

fn set(count: usize) -> Set {
    Spaces::first_run(
        (0..count).map(|n| Note {
            folder: format!("f{n}"),
        }),
        Note::default,
    )
}

fn names(spaces: &Set) -> Vec<String> {
    spaces
        .list()
        .iter()
        .map(|space| space.name.clone())
        .collect()
}

fn ids(spaces: &Set) -> Vec<u64> {
    spaces.list().iter().map(|space| space.id.0).collect()
}

#[test]
fn a_first_run_has_one_space_per_payload_or_one_fallback() {
    assert_eq!(names(&set(3)), ["Space 1", "Space 2", "Space 3"]);
    assert_eq!(names(&set(0)), ["Space 1"]);
    assert_eq!(
        set(2).list()[1].look.dots,
        crate::space::presets::PRESETS[1].dots
    );
    assert_eq!(set(3).current().id, SpaceId(0));
}

#[test]
fn adding_names_tints_and_keeps_the_current_theme() {
    let mut spaces = set(1);
    spaces.edit(SpaceId(0), |space| space.look.theme = Theme::Dark);
    let made = spaces.add(Note::default());
    let space = spaces.get(made).expect("added");
    assert_eq!(space.name, "Space 2");
    assert_eq!(space.look.dots, crate::space::presets::PRESETS[1].dots);
    assert_eq!(space.look.theme, Theme::Dark);
    assert_eq!(spaces.current().id, SpaceId(0), "adding does not switch");
    assert_eq!(spaces.add(Note::default()), SpaceId(2));
}

#[test]
fn switching_remembers_where_you_were_and_says_which_way_to_slide() {
    // (from index, to index, slide)
    const CASES: &[(usize, usize, Option<SlideIn>)] = &[
        (0, 2, Some(SlideIn::Right)),
        (2, 0, Some(SlideIn::Left)),
        (1, 1, None),
        (0, 9, None),
    ];
    for &(from, to, slide) in CASES {
        let mut spaces = set(3);
        spaces.switch_to_index(from, "x".into());
        let got = spaces.switch_to_index(to, format!("left-{from}"));
        assert_eq!(got.as_ref().map(|s| s.slide), slide, "{from}->{to}");
        if slide.is_some() {
            assert_eq!(
                spaces.recall().of(SpaceId(from as u64)),
                format!("left-{from}")
            );
        }
    }
    let mut spaces = set(2);
    spaces.switch_to(SpaceId(1), "inbox".into());
    let back = spaces.switch_to(SpaceId(0), "sent".into()).expect("moves");
    assert_eq!(
        back.restore, "inbox",
        "Space 0 was left at inbox when Space 1 was chosen"
    );
}

#[test]
fn removing_picks_a_neighbour_and_never_empties_the_list() {
    // (count, current, removed, ids left, current after)
    const CASES: &[(usize, u64, u64, &[u64], u64)] = &[
        (3, 2, 0, &[1, 2], 2),
        (3, 0, 1, &[0, 2], 0),
        (3, 1, 1, &[0, 2], 2),
        (3, 2, 2, &[0, 1], 1),
        (2, 0, 0, &[1], 1),
    ];
    for &(count, current, removed, left, after) in CASES {
        let mut spaces = set(count);
        spaces.select(SpaceId(current));
        spaces.switch_to(SpaceId(current), "p".into());
        let gone = spaces.remove(SpaceId(removed)).expect("removable");
        assert_eq!(ids(&spaces), left, "{count}/{current}/{removed}");
        assert_eq!(spaces.current().id.0, after);
        assert_eq!(gone.now_current.0, after);
        assert_eq!(
            spaces.recall().of(SpaceId(removed)),
            String::new(),
            "its place is forgotten"
        );
    }
    let mut one = set(1);
    assert_eq!(one.remove(SpaceId(0)).map(|_| ()), Err(Refused::Last));
    let mut two = set(2);
    assert_eq!(two.remove(SpaceId(7)).map(|_| ()), Err(Refused::Missing));
    assert_eq!(two.count(), 2);
}

#[test]
fn reordering_keeps_the_space_on_screen() {
    let mut spaces = set(3);
    spaces.select(SpaceId(2));
    spaces.move_to(SpaceId(2), 0);
    assert_eq!(ids(&spaces), [2, 0, 1]);
    assert_eq!(spaces.current().id, SpaceId(2));
    spaces.move_to(SpaceId(2), 99);
    assert_eq!(ids(&spaces), [0, 1, 2]);
}

#[test]
fn an_edit_is_held_to_the_dot_limits_and_keeps_the_id() {
    let mut spaces = set(1);
    spaces.edit(SpaceId(0), |space| {
        space.id = SpaceId(9);
        space.look.dots = vec![
            Dot {
                hue: 400.0,
                chroma: 3.0
            };
            5
        ];
    });
    let space = &spaces.list()[0];
    assert_eq!(space.id, SpaceId(0));
    assert_eq!(space.look.dots.len(), MOST_DOTS);
    assert_eq!(
        space.look.dots[0],
        Dot {
            hue: 40.0,
            chroma: 1.0
        }
    );
    spaces.edit(SpaceId(0), |space| space.look.dots.clear());
    assert_eq!(spaces.list()[0].look.dots.len(), 1);
}

fn read(json: &str) -> Option<Set> {
    serde_json::from_str(json).ok()
}

#[test]
fn a_stored_file_round_trips() {
    let mut spaces = set(3);
    spaces.rename(SpaceId(1), "Work");
    spaces.switch_to(SpaceId(1), "inbox".into());
    let json = serde_json::to_string(&spaces).expect("serialises");
    assert_eq!(read(&json), Some(spaces));
}

#[test]
fn an_old_or_damaged_file_is_read_leniently() {
    // mailo's file from before the kit: no ids, positions in `recall`, a retired accent word.
    let old = r#"{"spaces":[
        {"name":"Work","dots":[{"hue":10,"chroma":0.5}],"theme":"dark","card_accent":"hint","folder":"a"},
        {"name":"Home","dots":[],"grain":500,"theme":"nonsense","folder":"b"},
        {"name":"Far","dots":[{"hue":1,"chroma":1},{"hue":2,"chroma":1},{"hue":3,"chroma":1},{"hue":4,"chroma":1}]}
      ],"current":9,"recall":{"0":"inbox","7":"gone"}}"#;
    let spaces = read(old).expect("reads");
    assert_eq!(ids(&spaces), [0, 1, 2]);
    assert_eq!(
        spaces.current().id,
        SpaceId(2),
        "current is held to a real Space"
    );
    assert_eq!(spaces.list()[0].look.theme, Theme::Dark);
    assert_eq!(spaces.list()[0].look.card_accent, CardAccent::SpaceHue);
    assert_eq!(spaces.list()[0].payload.folder, "a");
    assert_eq!(spaces.list()[1].look.theme, Theme::System);
    assert_eq!(
        spaces.list()[1].look.dots.len(),
        1,
        "empty is the neutral dot"
    );
    assert_eq!(spaces.list()[1].look.grain, Grain(100));
    assert_eq!(spaces.list()[2].look.dots.len(), MOST_DOTS);
    assert_eq!(
        spaces.list()[2].look.grain,
        Grain(0),
        "a missing grain is the preset's own"
    );
    assert_eq!(spaces.recall().of(SpaceId(0)), "inbox");
    assert_eq!(spaces.recall().len(), 1, "an entry for no Space is dropped");

    // A damaged recall costs only itself; duplicate ids are renumbered.
    let damaged = r#"{"spaces":[{"id":4,"name":"a"},{"id":4,"name":"b"}],"recall":[1,2]}"#;
    let spaces = read(damaged).expect("reads");
    assert_eq!(ids(&spaces), [4, 1]);
    assert!(spaces.recall().is_empty());
    assert_eq!(spaces.clone().add(Note::default()), SpaceId(5));
}

#[test]
fn a_file_with_no_space_or_no_json_is_not_a_list() {
    for text in [
        "",
        "null",
        "[]",
        r#"{"spaces":[]}"#,
        "{ nope",
        r#"{"spaces":"x"}"#,
    ] {
        assert_eq!(read(text), None, "{text:?}");
    }
}

fn at(secs: i64) -> Epoch {
    Epoch(1_700_000_000 + secs)
}

type Recent = Today<u32>;

fn live(today: &Recent, space: u64, now: Epoch) -> Vec<u32> {
    today
        .live(SpaceId(space), now)
        .iter()
        .map(|(entry, _)| entry.item)
        .collect()
}

#[test]
fn opening_refreshing_closing_and_clearing_are_per_space() {
    let mut today = Recent::default();
    today.opened(SpaceId(0), 1, at(0));
    today.opened(SpaceId(0), 2, at(1));
    today.opened(SpaceId(1), 1, at(2));
    assert_eq!(live(&today, 0, at(3)), [2, 1]);
    today.opened(SpaceId(0), 1, at(4));
    assert_eq!(live(&today, 0, at(5)), [1, 2], "refresh moves to the front");
    assert_eq!(today.entries[0].last_opened, at(4));
    today.close(SpaceId(0), &1);
    assert_eq!(live(&today, 0, at(5)), [2]);
    assert_eq!(live(&today, 1, at(5)), [1], "the other Space keeps its own");
    today.clear(SpaceId(0));
    assert!(live(&today, 0, at(5)).is_empty());
}

#[test]
fn entries_expire_twelve_hours_after_they_were_opened_and_parked_things_never_do() {
    let mut today: Today<u32, u8> = Today::default();
    today.opened(SpaceId(0), 1, at(0));
    today.park(SpaceId(0), 9, "Draft", at(0));
    let hours = |h: u64| at((h * 3600) as i64);
    // (now, entries live, left)
    const CASES: &[(u64, usize)] = &[(0, 1), (11, 1), (12, 1)];
    for &(h, count) in CASES {
        assert_eq!(today.live(SpaceId(0), hours(h)).len(), count, "{h} h");
    }
    assert_eq!(today.live(SpaceId(0), hours(12))[0].1, Duration::ZERO);
    assert_eq!(today.live(SpaceId(0), at(12 * 3600 + 1)).len(), 0);
    assert_eq!(today.prune(hours(11)), 0);
    assert_eq!(today.prune(at(12 * 3600 + 1)), 1);
    assert_eq!(
        today.parked_in(SpaceId(0)).len(),
        1,
        "parking is not a timer"
    );
    today.park(SpaceId(1), 9, "Moved", at(5));
    assert_eq!(today.parked.len(), 1, "parking again moves it");
    today.unpark(&9);
    assert!(today.parked.is_empty());
}

#[test]
fn deleting_a_space_forgets_only_its_entries() {
    let mut today: Today<u32, u8> = Today::default();
    today.opened(SpaceId(0), 1, at(0));
    today.opened(SpaceId(2), 1, at(0));
    today.park(SpaceId(0), 5, "a", at(0));
    today.park(SpaceId(2), 6, "b", at(0));
    today.drop_space(SpaceId(0));
    assert_eq!(today.entries.len(), 1);
    assert_eq!(
        today.entries[0].space,
        SpaceId(2),
        "ids are stable: nothing renumbers"
    );
    assert_eq!(today.parked.len(), 1);
    assert_eq!(today.parked[0].space, SpaceId(2));
}

#[test]
fn today_reads_mailos_old_draft_key() {
    let text = r#"{"entries":[],"drafts":[{"space":0,"item":3,"title":"t","parked":5}]}"#;
    let today: Today<u32, u8> = serde_json::from_str(text).expect("reads");
    assert_eq!(today.parked.len(), 1);
}

#[test]
fn edit_recall_changes_the_place_of_a_space_and_ignores_strangers() {
    let mut spaces = set(1);
    spaces.edit_recall(SpaceId(0), |place| place.push_str("inbox"));
    spaces.edit_recall(SpaceId(0), |place| place.push('!'));
    assert_eq!(spaces.recall().of(SpaceId(0)), "inbox!");
    spaces.edit_recall(SpaceId(9), |place| place.push('x'));
    assert_eq!(spaces.recall().len(), 1);
}
