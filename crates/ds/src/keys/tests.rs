use super::source::KeySource;
use super::state::KeyState;
use super::use_keys_provider;
use chordkit::{
    Action, AppAction, AppId, Chord, Conflict, Context, DefaultChord, Desktop, KeyInput, Keymap,
    KeymapSource, Modifier, Overrides, Platform, Registration, SourceError, StandardAction,
    SystemBinding,
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
    // A Remap with no chords unbinds Copy; keep it on Ctrl+C so the app's chord clashes.
    let remap = Remap::default();
    remap.set_copy("Ctrl+C");
    let mut state = windows_state(remap);
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

fn ours() -> Platform {
    Platform::Linux {
        desktop: Desktop::Ours,
    }
}

fn ours_state() -> KeyState {
    KeyState::load(KeySource::conventions(ours()))
}

fn action(id: &str) -> AppAction {
    AppAction::new(id).expect("an action id")
}

fn app_action(id: &str) -> Option<Action> {
    Some(Action::App(action(id)))
}

#[test]
fn an_app_forgoes_what_it_does_not_offer_and_never_what_every_app_honours() {
    let mut state = ours_state();
    let command_t = press(&[Modifier::Super], 't');
    let tab = row("mail.tab", "Primary+T");
    let refused = state.register(&mail(), std::slice::from_ref(&tab));
    assert!(
        matches!(refused, Err(Conflict::Standard { .. })),
        "{refused:?}"
    );
    let forgoing = Registration::new(&mail())
        .forgo(StandardAction::ShowFonts)
        .action(tab.0, tab.1);
    assert_eq!(state.register(&mail(), &forgoing), Ok(()));
    assert_eq!(
        state.keymap.resolve(&command_t, Context::Normal),
        app_action("mail.tab")
    );
    state.reload();
    assert_eq!(
        state.keymap.resolve(&command_t, Context::Normal),
        app_action("mail.tab")
    );
    let universal = Registration::new(&mail()).forgo(StandardAction::Copy);
    let refused = state.register(&mail(), &universal);
    assert!(
        matches!(refused, Err(Conflict::Universal { .. })),
        "{refused:?}"
    );
}

#[test]
fn registering_an_app_again_replaces_its_earlier_registration() {
    let mut state = ours_state();
    let command_shift_n = press(&[Modifier::Super, Modifier::Shift], 'n');
    let old = row("mail.compose", "Primary+Shift+N");
    let new = row("mail.write", "Primary+Shift+N");
    assert_eq!(state.register(&mail(), &[old]), Ok(()));
    // The changed defaults give the chord to another action: no clash with the app's own old one.
    assert_eq!(state.register(&mail(), std::slice::from_ref(&new)), Ok(()));
    assert_eq!(
        state.keymap.resolve(&command_shift_n, Context::Normal),
        app_action("mail.write")
    );
    state.reload();
    assert!(state.problems().is_empty(), "{:?}", state.problems());
    assert_eq!(
        state.keymap.resolve(&command_shift_n, Context::Normal),
        app_action("mail.write")
    );
    // A refused replacement leaves the earlier registration standing.
    let refused = state.register(&mail(), &[row("mail.copy", "Primary+C")]);
    assert!(refused.is_err());
    assert_eq!(
        state.keymap.resolve(&command_shift_n, Context::Normal),
        app_action("mail.write")
    );
}

#[test]
fn app_overrides_lie_on_the_source_and_survive_a_reload() {
    let (overrides, parsed) =
        Overrides::parse("mail.compose = Primary+Alt+N\nmail.other = Primary+Space\n");
    assert!(parsed.is_empty(), "{parsed:?}");
    let source = KeySource::conventions(ours()).with_overrides(overrides);
    let mut state = KeyState::load(source);
    let compose = row("mail.compose", "Primary+Shift+N");
    assert_eq!(state.register(&mail(), &[compose]), Ok(()));
    let rebound = press(&[Modifier::Super, Modifier::Alt], 'n');
    let default = press(&[Modifier::Super, Modifier::Shift], 'n');
    for _ in 0..2 {
        assert_eq!(
            state.keymap.resolve(&rebound, Context::Normal),
            app_action("mail.compose")
        );
        assert_eq!(state.keymap.resolve(&default, Context::Normal), None);
        assert_eq!(state.problems().len(), 1, "{:?}", state.problems());
        assert!(state.problems()[0].contains("Space"));
        state.reload();
    }
}

#[test]
fn a_probe_registers_with_forgo_through_the_handle() {
    thread_local! {
        static TAKEN: RefCell<Option<Option<Action>>> = const { RefCell::new(None) };
    }
    fn forgoing() -> Element {
        let keys = use_keys_provider();
        use_hook(|| {
            let (tab, chord) = row("mail.tab", "Primary+T");
            let registration = Registration::new(&mail())
                .forgo(StandardAction::ShowFonts)
                .action(tab, chord);
            assert_eq!(keys.register(&mail(), &registration), Ok(()));
            let taken = keys.resolve(
                &Key::Character("t".into()),
                Modifiers::META,
                Context::Normal,
            );
            TAKEN.with(|seen| *seen.borrow_mut() = Some(taken));
        });
        rsx! {}
    }
    let mut dom = VirtualDom::new(forgoing);
    dom.rebuild_in_place();
    let seen = TAKEN.with(|seen| seen.borrow().clone());
    assert_eq!(seen, Some(app_action("mail.tab")));
}
