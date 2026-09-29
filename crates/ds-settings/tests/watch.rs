//! `Store::watch` on a scratch directory: one change per renamed write, a file that appears after
//! the watch starts, a damaged file that keeps the last good value.

mod support;

use ds::{Spawner, Theme};
use ds_settings::{AppearanceFile, DEBOUNCE, WatchState};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use support::Scratch;

/// Generous slack over the 30 ms debounce for a test runner under load, without being so long
/// the test hangs if the watch is broken.
const MARGIN: Duration = Duration::from_millis(2_000);

/// A `Spawner` on the runtime the test is running in.
struct TestSpawner(tokio::runtime::Handle);

impl TestSpawner {
    fn current() -> Self {
        TestSpawner(tokio::runtime::Handle::current())
    }
}

impl Spawner for TestSpawner {
    fn spawn(&self, task: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.0.spawn(task);
    }
}

fn dark() -> AppearanceFile {
    let mut file = AppearanceFile::default();
    file.appearance.theme = Theme::Dark;
    file
}

#[tokio::test]
async fn a_renamed_write_fires_exactly_one_change() {
    let scratch = Scratch::new();
    let store = scratch.store();
    store
        .save(&AppearanceFile::default())
        .unwrap_or_else(|e| panic!("{e}"));

    let mut watch = store.watch::<AppearanceFile>(&TestSpawner::current());
    assert_eq!(watch.state(), &WatchState::Live);
    assert_eq!(watch.current(), AppearanceFile::default());

    // The same atomic temp-and-rename write the real writer does, not an in-place write.
    store.save(&dark()).unwrap_or_else(|e| panic!("{e}"));

    let got = tokio::time::timeout(MARGIN, watch.changed())
        .await
        .unwrap_or_else(|_| panic!("no change observed within the debounce plus margin"))
        .unwrap_or_else(|| panic!("the watch stopped"));
    assert_eq!(got.value, dark());
    assert_eq!(watch.current(), dark());

    // Exactly one: nothing further arrives once the burst from the rename has settled.
    let extra = tokio::time::timeout(DEBOUNCE * 4, watch.changed()).await;
    assert!(extra.is_err(), "expected no further change, got {extra:?}");
}

#[tokio::test]
async fn a_watch_started_before_the_file_exists_sees_its_first_write() {
    let scratch = Scratch::new();
    let store = scratch.store();
    let mut watch = store.watch::<AppearanceFile>(&TestSpawner::current());
    assert_eq!(watch.current(), AppearanceFile::default());

    store.save(&dark()).unwrap_or_else(|e| panic!("{e}"));

    let got = tokio::time::timeout(MARGIN, watch.changed())
        .await
        .unwrap_or_else(|_| panic!("no change observed within the debounce plus margin"))
        .unwrap_or_else(|| panic!("the watch stopped"));
    assert_eq!(got.value, dark());
}

#[tokio::test]
async fn a_file_that_is_not_toml_keeps_the_last_good_value_and_says_so() {
    let scratch = Scratch::new();
    let store = scratch.store();
    store.save(&dark()).unwrap_or_else(|e| panic!("{e}"));
    let mut watch = store.watch::<AppearanceFile>(&TestSpawner::current());

    let path = scratch.app_dir().join("appearance.toml");
    std::fs::write(&path, "not toml [[[ = =").unwrap_or_else(|e| panic!("{e}"));

    let got = tokio::time::timeout(MARGIN, watch.changed())
        .await
        .unwrap_or_else(|_| panic!("the damaged file was never reported"))
        .unwrap_or_else(|| panic!("the watch stopped"));
    assert_eq!(got.value, dark(), "the last good settings stay in force");
    assert_eq!(got.invalid.len(), 1, "{:?}", got.invalid);
    assert_eq!(got.invalid[0].path, "", "the whole file is the invalid key");
}
