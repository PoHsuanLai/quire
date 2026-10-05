//! The fragments of `CONSUMING.md` that need an app around them (`YourApp`, `YourPage`, a
//! handler), compiled here so the guide cannot name an API that is gone. Each function carries
//! the guide's text under the section named in its doc comment; the guide's self-contained
//! snippets are doc tests of `src/lib.rs`.
#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    unused_mut,
    clippy::derivable_impls,
    clippy::needless_return,
    clippy::redundant_closure
)]

use dioxus::prelude::*;
use ds::prelude::*;

#[component]
fn YourApp() -> Element {
    rsx! {}
}

#[component]
fn YourPage() -> Element {
    rsx! {}
}

fn send() {}
fn open_thread() {}
fn restore(_token: ds::stack::toast_hub::UndoToken) {}

/// Section 6, "Controls".
#[component]
fn Controls() -> Element {
    use ds::components::controls::button_model::Answers;

    rsx! {
        Button {
            answers: Answers::Return,
            label: "Send".to_owned(),
            icon: Some(Icon::Send),
            onclick: move |_| send(),
        }
    }
}

/// Section 6, "Lists".
#[component]
fn Lists() -> Element {
    use ds::base::vocab::RowState;
    use ds::components::app::thread_row::ThreadRow;

    rsx! {
        ThreadRow {
            state: RowState::default(),
            name: "Ada Lovelace".to_owned(),
            via: None,
            subject: "Re: the analytical engine".to_owned(),
            snippet: Some("I have translated the memoir...".into()),
            time: "2:14 PM".to_owned(),
            tags: rsx! {},
            star: None,
            strip: None,
            onclick: move |_| open_thread(),
        }
    }
}

/// Section 6, "Overlays".
#[component]
fn Overlays() -> Element {
    use ds::host::measure::{Anchor, MountedRef};
    use ds::root::common::Common;
    use ds::stack::toast_hub::UndoToken;

    let toasts = use_toasts();
    toasts.push("Sent".to_owned(), None);
    toasts.push_undoable(
        "Archived".to_owned(),
        UndoToken(7),
        EventHandler::new(move |token| restore(token)),
    );

    let items: Vec<MenuItem<u8>> = Vec::new();
    let mut open = use_signal(|| Shown::Hidden);
    let mut more = use_signal(|| None::<MountedRef>);
    rsx! {
        Button {
            label: "More".to_owned(),
            onclick: move |_| open.set(Shown::Visible),
            common: Common {
                mounted: Some(EventHandler::new(move |event: MountedEvent| more.set(Some(MountedRef(event.data()))))),
                ..Common::default()
            },
        }
        if let (Shown::Visible, Some(button)) = (open(), more()) {
            Menu { placement: MenuPlacement::Popup, anchor: Anchor::Mounted(button), items, onpick: move |_: u8| {}, onclose: move |()| open.set(Shown::Hidden) }
        }
    }
}

/// Section 6, "Frame".
#[component]
fn Frame() -> Element {
    use ds::base::vocab::ShortcutKey;

    rsx! {
        CommandPill {
            label: "Search or run a command".to_owned(),
            shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('k')]),
            onclick: move |()| open_palette(),
        }
    }
}

fn open_palette() {}
use ds::components::app::command_pill::CommandPill;

/// Section 6, "External icons".
#[component]
fn ExternalIcons() -> Element {
    let theme_path = std::path::PathBuf::from("/usr/share/icons/hicolor/scalable/apps/app.svg");
    let title = "Title".to_owned();
    let onclick = move |_: Press| {};
    let icon = ds::prelude::IconSource::Symbolic(ds::prelude::ExternalIcon {
        url: ds::style::icon::url::IconUrl::file(&theme_path).expect("an absolute path"),
        size: ds::prelude::IconSize::Base,
    });
    rsx! { Button { bezel: Bezel::Toolbar, image: ImagePosition::Only, icon: Some(icon), label: title, onclick } }
}
use ds::components::controls::button_model::{Bezel, ImagePosition};

/// Section 6, "Pointer buttons and ids".
fn pointer_buttons() -> impl Fn(Press) {
    let open_menu = || {};
    let activate = || {};
    move |press: Press| match press.button {
        PointerButton::Secondary => open_menu(),
        PointerButton::Primary | PointerButton::Middle => activate(),
    }
}
use ds::base::press::{PointerButton, Press};

/// Section 6, "A Blitz host that is not `ds_blitz::launch`".
#[component]
fn BarRoot() -> Element {
    let appearance = Appearance::default();
    let look = SpaceLook::default();
    ds_blitz::provide_host();
    rsx! { Ds { appearance, material: Material::Bar, look, YourPage {} } }
}

/// Section 6, "Status items".
#[component]
fn StatusItems() -> Element {
    use ds::style::tokens::status::StatusMetrics;
    use ds_shell::prelude::MenuBarItem;

    let metrics = StatusMetrics {
        box_size: Px(22.0),
        glyph: Px(16.0),
    };
    rsx! { div { class: "status", style: metrics.style_attr(), YourPage {} } }
}

/// Section 6, "Focus waits out a busy document".
#[component]
fn LauncherRoot() -> Element {
    let appearance = Appearance::default();
    let look = SpaceLook::default();
    ds_blitz::provide_host();
    rsx! { Ds { appearance, material: Material::Sheet, look, YourPage {} } }
}

#[derive(Clone, PartialEq)]
struct Hit;

fn is_actions(key: &KeyboardEvent) -> bool {
    false
}

/// Section 6, "Giving a field the keyboard back" and "An embedded palette".
#[component]
fn EmbeddedPalette() -> Element {
    use ds::focus::request::use_focus_request;
    use ds::host::measure::Anchor;

    let (label, placeholder, query, empty) =
        (String::new(), String::new(), String::new(), String::new());
    let groups: Vec<ds::components::menus::palette::palette_group::PaletteGroup<Hit>> = Vec::new();
    let items: Vec<MenuItem<u8>> = Vec::new();
    let mut actions = use_signal(|| false);
    let mut row = use_signal(|| None::<Rect>);
    let field = use_focus_request();
    rsx! {
        CommandPalette::<Hit> {
            label, placeholder, query, tokens: Vec::new(), groups, empty,
            oninput: move |_: String| {}, onpick: move |_: Hit| {}, onclose: move |()| {},
            host: CommandPaletteHost::Surface,
            id: "launcher-card".to_string(),
            focus: field,
            on_select_rect: move |rect: Rect| row.set(Some(rect)),
            onkey: move |key: KeyboardEvent| if is_actions(&key) {
                key.prevent_default();
                actions.set(true);
            },
        }
        if let (true, Some(rect)) = (actions(), row()) {
            Menu { placement: MenuPlacement::Popup, anchor: Anchor::Rect(rect), items, onpick: move |_: u8| {},
                   onclose: move |()| { actions.set(false); field.request(); } }
        }
    }
}

/// Section 6, "A palette kept mounted".
#[component]
fn KeptPalette() -> Element {
    let (label, placeholder, empty) = (String::new(), String::new(), String::new());
    let groups: Vec<ds::components::menus::palette::palette_group::PaletteGroup<Hit>> = Vec::new();
    let mut shown = use_signal(|| Shown::Hidden);
    let mut query = use_signal(String::new);
    rsx! {
        CommandPalette::<Hit> {
            label, placeholder, query: query(), tokens: Vec::new(), groups, empty,
            oninput: move |text: String| query.set(text),
            onpick: move |_: Hit| {}, onclose: move |()| shown.set(Shown::Hidden),
            host: CommandPaletteHost::Surface,
            shown: shown(),
        }
    }
}

/// Section 6, "Ctrl+K toggles the actions menu".
#[component]
fn ActionsKey() -> Element {
    use ds::focus::request::use_focus_request;

    let (label, placeholder, query, empty) =
        (String::new(), String::new(), String::new(), String::new());
    let groups: Vec<ds::components::menus::palette::palette_group::PaletteGroup<Hit>> = Vec::new();
    let items: Vec<MenuItem<u8>> = Vec::new();
    let mut actions = use_signal(|| false);
    let field = use_focus_request();
    rsx! {
        div {
            onkeydown: move |event: KeyboardEvent| {
                if actions() && is_actions(&event) {
                    event.prevent_default();
                    actions.set(false);
                    field.request();
                }
            },
            CommandPalette::<Hit> {
                label, placeholder, query, tokens: Vec::new(), groups, empty,
                oninput: move |_: String| {}, onpick: move |_: Hit| {}, onclose: move |()| {},
                focus: field,
                onkey: move |event: KeyboardEvent| if is_actions(&event) {
                    event.prevent_default();
                    event.stop_propagation();
                    actions.set(true);
                },
            }
            if actions() {
                Menu { placement: MenuPlacement::Popup, anchor: ds::host::measure::Anchor::Rect(Rect::default()), items, onpick: move |_: u8| {},
                       onclose: move |()| { actions.set(false); field.request(); } }
            }
        }
    }
}

/// Section 5, rule 2.
mod rule_two {
    use dioxus::core::VirtualDom;
    use ds_lint::{LintConfig, Rule, markup};

    const YOUR_CSS: &str = "";

    fn render_ssr() -> String {
        let mut dom = VirtualDom::new(super::YourApp);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    fn full_css() -> String {
        format!("{}\n{}", ds::stylesheet(), YOUR_CSS)
    }

    #[test]
    fn markup_lint_is_clean() {
        let offences = markup(&render_ssr(), &full_css(), &LintConfig::new(&ds::kits()));
        assert!(offences.is_empty(), "{offences:#?}");
    }

    #[test]
    fn no_raw_button_is_rendered() {
        let offences = markup(&render_ssr(), &full_css(), &LintConfig::new(&ds::kits()));
        assert!(
            offences.iter().all(|o| o.rule != Rule::RawMarkup),
            "{offences:#?}"
        );
    }
}

/// Section 5, rule 4.
#[test]
fn the_badge_times_out_on_ds_motions_own_clock() {
    use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
    use std::time::Duration;

    let resolved = resolve(
        Appearance::default(),
        SpaceLook::default().theme,
        SystemPrefs::default(),
    );
    let hold = settle(Anim::Fade, resolved.motion); // never a millisecond literal

    let view = Viewport {
        width: 480,
        height: 360,
        scale_percent: 100,
    };
    let mut harness = Harness::new(YourApp, HarnessConfig::new(view).with_clock(Clock::Virtual));
    if let Some(target) = harness.centre(".ds-button") {
        harness.send(Input::click(target));
    }
    harness.advance(hold - Duration::from_millis(10));
    // assert it has NOT settled yet
    harness.advance(Duration::from_millis(20));
    // assert it HAS settled now
    let _ = harness.now();
    let _ = harness.clock();
}

/// Section 10, "Menu tracking".
#[test]
fn menu_track_steps() {
    use ds::base::machine::Machine;
    use ds::base::time::stamp::Stamp;
    use ds::stack::menu_track::types::{MenuTiming, MenuTrack, MenuTrackEffect, MenuTrackEvent};

    let track: MenuTrack<u32> = MenuTrack::closed();
    let timing = MenuTiming::default();
    let (track, effects) = track.step(MenuTrackEvent::PressTitle(1), Stamp(0), &timing, &());
    assert!(!effects.is_empty());
}

/// Section 10, "Curves".
#[test]
fn easing_at() {
    use ds::style::tokens::easing::EasingToken;

    let level = MotionLevel::Standard;
    let progress = EasingToken::Out.easing(level).at(Fraction(500));
    assert!(progress.0 > 500);
}

/// Section 10, "Spaces store".
#[test]
fn space_store_looks() {
    use ds::style::space::store::{
        SpaceDefaults, SpaceStore, Workspace, WorkspaceId, WorkspaceIndex,
    };

    let store = SpaceStore::default();
    let workspace = Workspace {
        id: Some(WorkspaceId("w1".to_owned())),
        index: WorkspaceIndex(0),
    };
    let look = store.look_for_workspace(&workspace, SpaceDefaults::default());
    let store = store.with_look(&workspace, look);
    let _ = store.look_for(WorkspaceIndex(0), SpaceDefaults::default());
}

/// Section 10, "Settings files".
mod settings_files {
    use ds_settings::{AppName, ConfigRoot, FileName, Format, SettingsDoc, Store};

    #[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    #[serde(default)]
    struct YourFile {
        name: String,
    }

    impl SettingsDoc for YourFile {
        const FILE: FileName = FileName("your-file.toml");
        const FORMAT: Format = Format::Toml;
    }

    fn use_store(
        spawner: &dyn ds::base::spawner::Spawner,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = Store::new(ConfigRoot::Xdg, AppName("your-app-id"));
        let loaded = store.load::<YourFile>();
        store.save(&loaded.value)?;
        let mut watch = store.watch::<YourFile>(spawner);
        Ok(())
    }
}

/// Section 7, the settings schema.
mod schema {
    use ds::prelude::*;
    use ds_settings::SettingsSchema; // the derive macro (macro namespace)
    use ds_settings::schema::{Page, SettingsSchema}; // the trait `.schema()` needs (type namespace)
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, Word)]
    #[serde(rename_all = "snake_case")]
    #[word(case = snake)]
    pub enum OpenLinks {
        #[default]
        InApp,
        Externally,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SettingsSchema)]
    #[serde(default)]
    #[settings(file = "your-app/settings.toml", domain = "reader", page = Page::App("your-app".to_owned()))]
    pub struct ReaderSettings {
        #[settings(
            label = "Open links",
            help = "Where a link in a message opens.",
            section = "Reader"
        )]
        pub open_links: OpenLinks,
    }

    impl Default for ReaderSettings {
        fn default() -> Self {
            ReaderSettings {
                open_links: OpenLinks::default(),
            }
        }
    }

    fn main() {
        let args: Vec<String> = std::env::args().collect();
        if ds_settings::schema::maybe_write_schema(&ReaderSettings::schema(), &args)
            .unwrap_or(false)
        {
            return;
        }
        // ...normal startup...
    }
}

/// Section 12, user styles.
#[component]
fn UserStyled() -> Element {
    use ds_settings::{AppName, ConfigRoot, Store, UserStyle};

    let store = Store::new(ConfigRoot::Xdg, AppName("your-app-id"));
    let mut user_style = use_signal({
        let store = store.clone();
        move || store.load::<UserStyle>().value
    });
    use_future(move || {
        let store = store.clone();
        async move {
            let spawner = ds_blitz::TokioSpawner::current();
            let mut watch = store.watch::<UserStyle>(&spawner);
            while let Some(loaded) = watch.changed().await {
                user_style.set(loaded.value);
            }
        }
    });
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, user_style, YourPage {} }
    }
}

/// Section 2, "The `Ds` root".
#[component]
fn RootApp() -> Element {
    rsx! {
        Ds {
            appearance: Appearance::default(),   // theme, accent, motion — see section 3
            material: Material::Window,           // one of the eight Materials (03-COLOR §17)
            style { {ds::stylesheet()} }           // or Inject::Inline (the default) does this for you
            YourPage {}
        }
    }
}

/// Section 4, `Surface`.
#[component]
fn Nested() -> Element {
    use ds::style::appearance::blur::BlurState;

    rsx! {
        Surface { material: Material::Popover, theme: Some(Scheme::Dark),
            YourPage {}
        }
        Surface { material: Material::Widget, accent: Some(Accent::Green), blur: Some(BlurState::Unavailable),
            YourPage {}
        }
    }
}

/// Section 12, reporting on a user stylesheet.
#[test]
fn user_stylesheet_notes() {
    let notes = ds_lint::user_stylesheet(".ds { --accent: red; }", &ds::kits());
    let _ = notes.len();
}
