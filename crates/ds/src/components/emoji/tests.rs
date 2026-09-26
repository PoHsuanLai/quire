use super::id::EmojiId;
use super::script::{IDLE_REST, MoodChange, Pace, Playing, Shown, Step, pace, reaction, script};
use super::sheet::{MANIFEST_JSON, SheetPx, durations, png, position, timing};
use crate::components::user_picture::Mood;
use std::time::Duration;

const WINDOW: Duration = Duration::from_secs(20);

#[test]
fn the_manifest_lists_every_emoji_in_order() {
    let manifest: serde_json::Value = serde_json::from_str(MANIFEST_JSON).expect("manifest");
    let listed: Vec<&str> = manifest["emoji"]
        .as_array()
        .expect("emoji")
        .iter()
        .filter_map(|entry| entry["slug"].as_str())
        .collect();
    let ours: Vec<&str> = EmojiId::ALL.iter().map(|emoji| emoji.slug()).collect();
    assert_eq!(listed, ours);
    assert_eq!(manifest["licence"], "CC-BY-4.0");
}

#[test]
fn every_sheet_is_its_grid_of_frames() {
    for emoji in EmojiId::ALL {
        let grid = timing(emoji);
        assert!(grid.frames > 1, "{emoji:?} is still");
        assert!(grid.frames <= grid.columns * grid.rows, "{emoji:?}");
        assert!(
            grid.frames > grid.columns * (grid.rows - 1),
            "{emoji:?}: an empty row"
        );
        for px in [SheetPx::Px128, SheetPx::Px256] {
            let image = image::load_from_memory(png(emoji, px)).expect("png");
            assert_eq!(
                (image.width(), image.height()),
                (
                    u32::from(grid.columns) * px.px(),
                    u32::from(grid.rows) * px.px()
                ),
                "{emoji:?} {px:?}"
            );
            // The renderer's texture limit.
            assert!(image.width() <= 4096 && image.height() <= 4096);
        }
    }
}

#[test]
fn the_id_is_stored_as_its_slug() {
    for emoji in EmojiId::ALL {
        let json = serde_json::to_string(&emoji).expect("serialise");
        assert_eq!(json, format!("\"{}\"", emoji.slug()));
        let back: EmojiId = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(back, emoji);
    }
    assert_eq!(EmojiId::default(), EmojiId::Blush);
}

#[test]
fn a_frame_is_a_share_of_the_sheet() {
    let grid = timing(EmojiId::Wink);
    assert_eq!(position(grid, 0).0, "0% 0%");
    let (last, fit) = position(grid, grid.columns - 1);
    assert_eq!(last, "100% 0%");
    assert_eq!(fit, format!("{}% {}%", grid.columns * 100, grid.rows * 100));
    let (below, _) = position(grid, grid.columns);
    assert!(below.starts_with("0% "), "{below}");
}

fn waited(steps: &[Step]) -> Duration {
    steps
        .iter()
        .map(|step| match step {
            Step::Wait(hold) => *hold,
            Step::Show(_) => Duration::ZERO,
        })
        .sum()
}

fn shown(steps: &[Step]) -> Vec<Shown> {
    steps
        .iter()
        .filter_map(|step| match step {
            Step::Show(frame) => Some(*frame),
            Step::Wait(_) => None,
        })
        .collect()
}

#[test]
fn every_script_rests_inside_the_window() {
    let moods = [
        Mood::Idle,
        Mood::Attentive,
        Mood::Wince,
        Mood::Happy,
        Mood::Asleep,
    ];
    for user in EmojiId::ALL {
        for mood in moods {
            for change in [MoodChange::Changed, MoodChange::Same] {
                for playing in [Playing::Frames, Playing::Stills] {
                    let steps = script(user, mood, change, playing, WINDOW);
                    assert!(waited(&steps) <= WINDOW, "{user:?} {mood:?}");
                    let last = shown(&steps).last().copied().expect("a frame");
                    assert_eq!(last.frame, 0, "{user:?} {mood:?} ends mid-loop");
                    if playing == Playing::Stills {
                        assert!(shown(&steps).iter().all(|f| f.frame == 0), "{user:?}");
                    }
                }
            }
        }
    }
}

/// How many of `emoji`'s loops a script plays: each loop shows its frame 1 once.
fn passes(steps: &[Step], emoji: EmojiId) -> usize {
    shown(steps)
        .iter()
        .filter(|f| f.emoji == emoji && f.frame == 1)
        .count()
}

#[test]
fn idle_plays_the_users_loop_now_and_then() {
    let steps = script(
        EmojiId::Wink,
        Mood::Idle,
        MoodChange::Same,
        Playing::Frames,
        WINDOW,
    );
    let loop_length: Duration = durations(EmojiId::Wink).into_iter().sum();
    let count = passes(&steps, EmojiId::Wink);
    // One loop, a rest, another: as many as fit, with a rest between each.
    let cycle = loop_length + IDLE_REST;
    let fit = (WINDOW + IDLE_REST).as_millis() / cycle.as_millis();
    assert_eq!(count as u128, fit, "{count} loops");
    assert!(count >= 2, "idle plays more than once");
    assert!(shown(&steps).iter().all(|f| f.emoji == EmojiId::Wink));
    assert!(
        steps.contains(&Step::Wait(IDLE_REST)),
        "idle rests between loops"
    );
}

#[test]
fn attentive_glances_then_plays_steadily() {
    let steps = script(
        EmojiId::Wink,
        Mood::Attentive,
        MoodChange::Changed,
        Playing::Frames,
        WINDOW,
    );
    let frames = shown(&steps);
    let glance = timing(EmojiId::ATTENTIVE).frames as usize;
    assert!(frames[..glance].iter().all(|f| f.emoji == EmojiId::Eyes));
    assert!(frames[glance..].iter().all(|f| f.emoji == EmojiId::Wink));
    assert!(
        !steps.contains(&Step::Wait(IDLE_REST)),
        "attentive never rests"
    );
    let loop_length: Duration = durations(EmojiId::Wink).into_iter().sum();
    assert!(
        waited(&steps) + loop_length > WINDOW,
        "attentive fills the window"
    );
}

/// The mapping, one row per mood: the emoji a change to it swaps in, and the pace after.
#[test]
fn every_mood_has_its_reaction_and_pace() {
    const CASES: [(Mood, Option<EmojiId>, Pace); 5] = [
        (Mood::Idle, None, Pace::Slow),
        (Mood::Attentive, Some(EmojiId::Eyes), Pace::Steady),
        (Mood::Wince, Some(EmojiId::Confounded), Pace::Slow),
        (Mood::Happy, Some(EmojiId::Partying), Pace::Slow),
        (Mood::Asleep, None, Pace::Still),
    ];
    for (mood, swap, speed) in CASES {
        assert_eq!(reaction(mood), swap, "{mood:?}");
        assert_eq!(pace(mood), speed, "{mood:?}");
    }
}

#[test]
fn a_wince_plays_the_reaction_once_then_the_users_own() {
    let steps = script(
        EmojiId::Wink,
        Mood::Wince,
        MoodChange::Changed,
        Playing::Frames,
        WINDOW,
    );
    let frames = shown(&steps);
    let reaction = timing(EmojiId::WRONG).frames as usize;
    assert!(frames[..reaction].iter().all(|f| f.emoji == EmojiId::WRONG));
    assert!(frames[reaction..].iter().all(|f| f.emoji == EmojiId::Wink));
    // Unchanged mood (a new wake stamp while wincing): no reaction.
    let again = script(
        EmojiId::Wink,
        Mood::Wince,
        MoodChange::Same,
        Playing::Frames,
        WINDOW,
    );
    assert!(shown(&again).iter().all(|f| f.emoji == EmojiId::Wink));
}

#[test]
fn asleep_is_the_sleeping_face_still() {
    let steps = script(
        EmojiId::Wink,
        Mood::Asleep,
        MoodChange::Changed,
        Playing::Frames,
        WINDOW,
    );
    assert_eq!(steps, vec![Step::Show(Shown::rest(EmojiId::ASLEEP))]);
}
