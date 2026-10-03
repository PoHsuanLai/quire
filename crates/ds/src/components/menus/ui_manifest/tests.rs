use super::*;
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu_bar::{BarCommand, BarSection, MenuBarModel};
use ds_core::command::{
    ActionName, AppCommand, CommandFace, CommandId, IntentsApp, ShortcutBinding, UiOnlyReason,
};
use ds_core::standard_action::StandardAction;
use ds_core::vocab::{Shortcut, ShortcutKey};
use ds_core::word::Word;

/// An app's command type: the bar's standard items, and three of its own.
#[derive(Debug, Clone, PartialEq)]
enum Cmd {
    Bar(BarCommand),
    NewDraft,
    Archive,
    Minimize,
}

fn chrome(reason: &str) -> CommandFace {
    CommandFace::UiOnly(UiOnlyReason::new(reason).expect("a reason"))
}

impl AppCommand for Cmd {
    fn id(&self) -> CommandId {
        CommandId(match self {
            Cmd::Bar(BarCommand::Standard(action)) => format!("standard.{action:?}").to_lowercase(),
            Cmd::Bar(other) => format!("bar.{other:?}").to_lowercase(),
            Cmd::NewDraft => "file.new".to_owned(),
            Cmd::Archive => "thread.archive".to_owned(),
            Cmd::Minimize => "window.minimize".to_owned(),
        })
    }

    fn face(&self) -> CommandFace {
        match self {
            Cmd::NewDraft => CommandFace::Action(ActionName("mail.draft.create".to_owned())),
            Cmd::Archive => CommandFace::Action(ActionName("mail.thread.archive".to_owned())),
            Cmd::Minimize => chrome("window chrome"),
            Cmd::Bar(_) => chrome("standard item"),
        }
    }
}

fn app() -> IntentsApp {
    IntentsApp("org.quire.Mail".to_owned())
}

fn archive_key() -> Vec<ShortcutBinding<Cmd>> {
    vec![ShortcutBinding {
        keys: Shortcut(vec![ShortcutKey::Char('e')]),
        command: Cmd::Archive,
    }]
}

fn manifest_of(rows: Vec<CommandRow>) -> UiManifest {
    UiManifest::new(app(), rows).expect("a manifest")
}

#[test]
fn the_file_is_the_shape_the_checker_reads() {
    let rows = [
        menu_rows(&[MenuItem::new(Cmd::NewDraft, "New Message")
            .with_key(Shortcut::standard(StandardAction::New))]),
        menu_rows(&[MenuItem::new(Cmd::Minimize, "Minimize")]),
        shortcut_rows(&archive_key()),
    ]
    .concat();
    let text = manifest_of(rows).to_toml().expect("text");
    assert_eq!(
        text,
        r#"vocab = 1
app = "org.quire.Mail"

[[commands]]
id = "file.new"
source = "menu"
chord = "cmd+n"
action = "mail.draft.create"

[[commands]]
id = "window.minimize"
source = "menu"

[[commands]]
id = "thread.archive"
source = "shortcut"
chord = "e"
action = "mail.thread.archive"

[[ui_only]]
id = "window.minimize"
reason = "window chrome"
"#
    );
}

#[test]
fn the_checkers_own_file_is_read_back_as_written() {
    // The sample the checker documents for its reader: bytes it produced or accepts.
    const FROM_CHECKER: &str = r#"
vocab = 1
app = "org.quire.Mail"

[[commands]]
id = "file.new"
source = "menu"
chord = "cmd+n"
action = "mail.draft.create"

[[commands]]
id = "window.minimize"
source = "menu"

[[ui_only]]
id = "window.minimize"
reason = "window chrome"
"#;
    let read: UiManifest = toml::from_str(FROM_CHECKER).expect("the checker's sample");
    assert_eq!(read.commands.len(), 2);
    assert_eq!(read.commands[0].source, CommandSource::Menu);
    let written = read.to_toml().expect("text");
    assert_eq!(toml::from_str::<UiManifest>(&written), Ok(read));
}

#[test]
fn the_standard_bar_and_the_apps_own_menus_are_all_read() {
    let own = vec![
        MenuItem::new(Cmd::NewDraft, "New Message"),
        MenuItem::Separator,
        MenuItem::Header("Thread".to_owned()),
        MenuItem::Submenu {
            title: "Move".to_owned(),
            image: None,
            availability: ds_core::vocab::Availability::Enabled,
            children: vec![MenuItem::new(Cmd::Archive, "Archive")],
        },
    ];
    let bar = MenuBarModel::standard("Mail", Cmd::Bar).with_items(BarSection::File, own);
    let ids: Vec<String> = bar_rows(&bar).into_iter().map(|row| row.id.0).collect();
    // The App menu's five, the File menu's five standard items then the app's New Message and
    // its submenu's Archive, Edit's seven, View's three, Window's two, Help's search and help.
    assert_eq!(ids.len(), 5 + (5 + 2) + 7 + 3 + 2 + 2);
    assert_eq!(&ids[10..12], ["file.new", "thread.archive"]);
    assert!(ids.contains(&"standard.paste".to_owned()));
}

#[test]
fn a_command_in_two_places_is_one_row_and_one_reason() {
    let item = || MenuItem::new(Cmd::Minimize, "Minimize");
    let rows = [menu_rows(&[item()]), menu_rows(&[item(), item()])].concat();
    let manifest = manifest_of(rows);
    assert_eq!(manifest.commands.len(), 1);
    assert_eq!(manifest.ui_only.len(), 1);
}

#[test]
fn a_command_in_the_menu_and_on_a_key_is_a_row_for_each() {
    let menu = menu_rows(&[MenuItem::new(Cmd::Archive, "Archive")]);
    let manifest = manifest_of([menu, shortcut_rows(&archive_key())].concat());
    let sources: Vec<CommandSource> = manifest.commands.iter().map(|c| c.source).collect();
    assert_eq!(sources, [CommandSource::Menu, CommandSource::Shortcut]);
}

#[test]
fn one_id_with_two_faces_is_refused() {
    let mut rows = menu_rows(&[MenuItem::new(Cmd::Minimize, "Minimize")]);
    rows.push(CommandRow {
        face: CommandFace::Action(ActionName("window.minimize".to_owned())),
        ..rows[0].clone()
    });
    assert_eq!(
        UiManifest::new(app(), rows),
        Err(UiManifestError::ConflictingFaces(CommandId(
            "window.minimize".to_owned()
        )))
    );
}

#[test]
fn a_file_for_no_app_is_refused() {
    assert_eq!(
        UiManifest::new(IntentsApp("  ".to_owned()), Vec::new()),
        Err(UiManifestError::NoApp)
    );
}

#[test]
fn the_source_names_match_the_files() {
    ds_core::testing::word_matches_serde::<CommandSource>();
    assert_eq!(CommandSource::ALL.len(), 2);
}
