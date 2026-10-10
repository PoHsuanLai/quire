use super::source::KeySource;
use super::state::KeyState;
use super::use_keys_provider;
use chordkit::{
    Action, AppAction, AppId, Chord, Conflict, Context, DefaultChord, Desktop, KeyInput, Keymap,
    KeymapSource, Modifier, Platform, SourceError, StandardAction, SystemBinding,
};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

fn mail() -> AppId {
    AppId::new("mail").expect("an app name")
}

fn row(id: &str, chord: &str) -> (AppAction, DefaultChord) {
    (
        AppAction::new(id).expect("an action id"),
        chord.parse().expect("a chord"),
    )
}

fn press(modifiers: &[Modifier], key: char) -> KeyInput {
    KeyInput::new(
        modifiers.iter().copied().collect::<chordkit::Modifiers>(),
        chordkit::Key::Char(key),
    )
}

/// A source whose Copy chords the test changes, as a person editing their settings would.
#[derive(Clone, Default)]
struct Remap(Arc<Mutex<Vec<Chord>>>);

impl Remap {
    fn set_copy(&self, text: &str) {
        let chord: Chord = text.parse().expect("a chord");
        *self.0.lock().expect("lock") = vec![chord];
    }
}

impl KeymapSource for Remap {
    fn load(&self, platform: Platform) -> Result<Keymap, SourceError> {
        let chords = self.0.lock().expect("lock").clone();
        let binding = SystemBinding {
            action: StandardAction::Copy,
            chords,
        };
        Ok(Keymap::conventional(platform).with_system(vec![binding]))
    }
}

/// A source that cannot be read.
struct Broken;

impl KeymapSource for Broken {
    fn load(&self, _: Platform) -> Result<Keymap, SourceError> {
        Err(SourceError::Read {
            place: "the shortcut settings".to_owned(),
            reason: "no such device".to_owned(),
        })
    }
}

fn windows_state(source: impl KeymapSource + Send + Sync + 'static) -> KeyState {
    KeyState::load(KeySource::new(Platform::Windows, Box::new(source)))
}

#[test]
fn a_reload_applies_the_sources_change_and_registers_again_over_it() {
    let remap = Remap::default();
    let mut state = windows_state(remap.clone());
    let compose = row("mail.compose", "Primary+Shift+N");
    assert_eq!(
        state.register(&mail(), std::slice::from_ref(&compose)),
        Ok(())
    );
    let ctrl_shift_n = press(&[Modifier::Ctrl, Modifier::Shift], 'n');
    assert_eq!(
        state.keymap.resolve(&ctrl_shift_n, Context::Normal),
        Some(Action::App(compose.0.clone()))
    );
    // The person gives Copy the chord the app registered: the system's settings win a chord.
    remap.set_copy("Ctrl+Shift+N");
    state.reload();
    assert_eq!(
        state.keymap.resolve(&ctrl_shift_n, Context::Normal),
        Some(Action::Standard(StandardAction::Copy))
    );
    assert_eq!(state.problems().len(), 1, "{:?}", state.problems());
    assert!(state.problems()[0].contains("mail.compose"));
}

#[test]
fn a_refused_registration_is_not_remembered() {
    let mut state = windows_state(Remap::default());
    let taken = row("mail.copy", "Primary+C");
    let refused = state.register(&mail(), &[taken]);
    assert!(
        matches!(refused, Err(Conflict::Standard { .. })),
        "{refused:?}"
    );
    state.reload();
    assert!(state.problems().is_empty(), "{:?}", state.problems());
}

#[test]
fn an_unreadable_source_falls_back_to_the_conventions_and_says_why() {
    let state = windows_state(Broken);
    assert_eq!(state.problems().len(), 1);
    assert!(state.problems()[0].contains("no such device"));
    assert_eq!(
        state
            .keymap
            .resolve(&press(&[Modifier::Ctrl], 'c'), Context::Normal),
        Some(Action::Standard(StandardAction::Copy))
    );
}

#[test]
fn the_sources_change_flag_is_taken_once() {
    let state = windows_state(Remap::default());
    assert!(!state.take_changed());
    state
        .changed_flag()
        .store(true, std::sync::atomic::Ordering::Release);
    assert!(state.take_changed());
    assert!(!state.take_changed());
}

/// What the probe saw: the platform, what Ctrl+Shift+N resolved to, and Ctrl+C in a terminal.
type Seen = (Platform, Option<Action>, Option<Action>);

thread_local! {
    static SEEN: RefCell<Option<Seen>> = const { RefCell::new(None) };
}

fn probe() -> Element {
    let keys = use_keys_provider();
    use_hook(|| {
        let registered = keys.register_actions(&mail(), &[row("mail.compose", "Primary+Shift+N")]);
        assert_eq!(registered, Ok(()));
        let held = Modifiers::CONTROL | Modifiers::SHIFT;
        let compose = keys.resolve(&Key::Character("N".into()), held, Context::Normal);
        let copy = keys.resolve(
            &Key::Character("c".into()),
            Modifiers::CONTROL,
            Context::Terminal,
        );
        SEEN.with(|seen| *seen.borrow_mut() = Some((keys.platform(), compose, copy)));
    });
    rsx! {}
}

#[test]
fn the_root_provides_the_injected_sources_keymap() {
    let mut dom = VirtualDom::new(probe);
    dom.provide_root_context(KeySource::conventions(Platform::Windows));
    dom.rebuild_in_place();
    let seen = SEEN.with(|seen| seen.borrow().clone());
    let compose = AppAction::new("mail.compose").expect("an action id");
    assert_eq!(
        seen,
        Some((Platform::Windows, Some(Action::App(compose)), None))
    );
}

#[test]
fn without_a_source_the_root_takes_our_desktops_conventions() {
    let mut dom = VirtualDom::new(probe);
    dom.rebuild_in_place();
    let seen = SEEN.with(|seen| seen.borrow().clone());
    let platform = seen.map(|(platform, ..)| platform);
    assert_eq!(
        platform,
        Some(Platform::Linux {
            desktop: Desktop::Ours
        })
    );
}
