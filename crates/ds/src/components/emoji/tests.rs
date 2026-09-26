use super::id::EmojiId;
use super::script::{MoodChange, Playing, Shown, Step, script};
use super::sheet::{MANIFEST_JSON, SheetPx, durations, png, position, timing};
use crate::components::persona::Mood;
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

#[test]
fn idle_loops_the_users_emoji_for_most_of_the_window() {
    let steps = script(
        EmojiId::Wink,
        Mood::Idle,
        MoodChange::Same,
        Playing::Frames,
        WINDOW,
    );
    let loop_length: Duration = durations(EmojiId::Wink).into_iter().sum();
    assert!(waited(&steps) + loop_length > WINDOW);
    assert!(shown(&steps).iter().all(|f| f.emoji == EmojiId::Wink));
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
