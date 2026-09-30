//! `style.css` as a settings document: raw text that round-trips byte for byte, a missing file is
//! an empty style, no text is refused, and a watch publishes each change whole (a file that is
//! not UTF-8 keeps the last good text and says so).

mod support;

use ds_core::spawner::Spawner;
use ds_settings::{Store, UserStyle, WatchState};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use support::Scratch;

/// Generous slack over the 30 ms debounce for a test runner under load.
const MARGIN: Duration = Duration::from_millis(2_000);

struct TestSpawner(tokio::runtime::Handle);

impl Spawner for TestSpawner {
    fn spawn(&self, task: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.0.spawn(task);
    }
}

fn style(text: &str) -> UserStyle {
    UserStyle(text.to_owned())
}

fn watch(store: &Store) -> ds_settings::Watch<UserStyle> {
    store.watch::<UserStyle>(&TestSpawner(tokio::runtime::Handle::current()))
}

async fn next(watch: &mut ds_settings::Watch<UserStyle>) -> ds_settings::Loaded<UserStyle> {
    tokio::time::timeout(MARGIN, watch.changed())
        .await
        .unwrap_or_else(|_| panic!("no change observed within the debounce plus margin"))
        .unwrap_or_else(|| panic!("the watch stopped"))
}

#[test]
fn a_missing_file_is_an_empty_style() {
    let scratch = Scratch::new();
    let loaded = scratch.store().load::<UserStyle>();
    assert_eq!(loaded.value, UserStyle::default());
    assert!(loaded.value.is_blank());
    assert!(loaded.invalid.is_empty() && loaded.unknown.is_empty());
}

#[test]
fn the_file_is_style_css_and_its_text_round_trips_byte_for_byte() {
    let scratch = Scratch::new();
    let store = scratch.store();
    let text = ".ds { --accent: #f00 }\n/* a note */\n[*|data-surface=bar] { color: red; }\n";
    store.save(&style(text)).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        std::fs::read_to_string(scratch.app_dir().join("style.css")).ok(),
        Some(text.to_owned())
    );
    assert_eq!(store.load::<UserStyle>().value, style(text));
}

#[test]
fn text_that_is_not_css_still_loads() {
    let scratch = Scratch::new();
    let text = ".a { color: red \n} } ((( \"unterminated";
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(scratch.app_dir().join("style.css"), text).unwrap_or_else(|e| panic!("{e}"));
    let loaded = scratch.store().load::<UserStyle>();
    assert_eq!(loaded.value, style(text));
    assert!(loaded.invalid.is_empty(), "{:?}", loaded.invalid);
}

#[test]
fn a_file_that_is_not_utf8_at_load_is_empty_and_reported() {
    let scratch = Scratch::new();
    std::fs::create_dir_all(scratch.app_dir()).unwrap_or_else(|e| panic!("{e}"));
    std::fs::write(scratch.app_dir().join("style.css"), [0x2e, 0xff, 0xfe])
        .unwrap_or_else(|e| panic!("{e}"));
    let loaded = scratch.store().load::<UserStyle>();
    assert_eq!(loaded.value, UserStyle::default());
    assert_eq!(loaded.invalid.len(), 1, "{:?}", loaded.invalid);
}

#[tokio::test]
async fn the_watch_publishes_each_edit_whole_and_an_emptied_file_as_empty() {
    let scratch = Scratch::new();
    let store = scratch.store();
    let mut watch = watch(&store);
    assert_eq!(watch.state(), &WatchState::Live);
    assert_eq!(watch.current(), UserStyle::default());

    store
        .save(&style(".a { }"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(next(&mut watch).await.value, style(".a { }"));

    store
        .save(&style(".b { }"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(next(&mut watch).await.value, style(".b { }"));

    std::fs::remove_file(scratch.app_dir().join("style.css")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(next(&mut watch).await.value, UserStyle::default());
    assert_eq!(watch.current(), UserStyle::default());
}

#[tokio::test]
async fn a_file_that_is_not_utf8_keeps_the_last_good_text_and_says_so() {
    let scratch = Scratch::new();
    let store = scratch.store();
    store
        .save(&style(".a { }"))
        .unwrap_or_else(|e| panic!("{e}"));
    let mut watch = watch(&store);

    std::fs::write(scratch.app_dir().join("style.css"), [0x2e, 0xff, 0xfe])
        .unwrap_or_else(|e| panic!("{e}"));
    let got = next(&mut watch).await;
    assert_eq!(
        got.value,
        style(".a { }"),
        "the last good text stays in force"
    );
    assert_eq!(got.invalid.len(), 1, "{:?}", got.invalid);
    assert_eq!(got.invalid[0].path, "", "the whole file is the invalid key");
}
