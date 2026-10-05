use super::*;
use crate::components::menus::item::item::MenuItem;
use crate::components::menus::menu_bar::{BarCommand, BarSection, MenuBarModel};
use ds_core::command::{AppCommand, CommandFace, CommandId, UiOnlyReason};
use ds_core::standard_action::StandardAction;
use ds_core::vocab::{Availability, Check};

/// An app's command type: the bar's standard items and two of its own.
#[derive(Debug, Clone, PartialEq)]
enum Cmd {
    Bar(BarCommand),
    Export,
    Wrap,
    Nameless,
}

impl AppCommand for Cmd {
    fn id(&self) -> CommandId {
        match self {
            Cmd::Bar(bar) => bar.id(),
            Cmd::Export => CommandId("file.export".to_owned()),
            Cmd::Wrap => CommandId("view.wrap_lines".to_owned()),
            Cmd::Nameless => CommandId(String::new()),
        }
    }

    fn face(&self) -> CommandFace {
        CommandFace::UiOnly(UiOnlyReason::new("test").expect("a reason"))
    }
}

fn bar() -> MenuBarModel<Cmd> {
    MenuBarModel::standard("Notes", Cmd::Bar)
        .with_items(
            BarSection::File,
            vec![MenuItem::new(Cmd::Export, "Export_As…")],
        )
        .with_items(
            BarSection::View,
            vec![MenuItem::new(Cmd::Wrap, "Wrap Lines").with_check(Check::On)],
        )
}

fn tree_of(bar: &MenuBarModel<Cmd>) -> MenuTree {
    MenuTree::from_bar(bar).expect("a tree")
}

fn undo() -> CommandId {
    CommandId("standard.undo".to_owned())
}

fn paste_disabled() -> MenuBarModel<Cmd> {
    bar().availability(|cmd| match cmd {
        Cmd::Bar(BarCommand::Standard(StandardAction::Paste)) => Availability::Disabled,
        _ => Availability::Enabled,
    })
}

#[test]
fn every_command_item_carries_the_command_it_runs() {
    let tree = tree_of(&bar());
    let commands: Vec<&CommandId> = tree
        .walk()
        .into_iter()
        .filter_map(|(node, _)| match &node.kind {
            NodeKind::Command { command, .. } => Some(command),
            _ => None,
        })
        .collect();
    assert!(commands.len() > 20, "{} commands", commands.len());
    assert!(commands.iter().all(|command| !command.0.is_empty()));
    assert!(commands.contains(&&undo()));
}

#[test]
fn an_item_with_no_command_id_is_refused() {
    let bar = bar().with_items(
        BarSection::File,
        vec![MenuItem::new(Cmd::Nameless, "Mystery")],
    );
    assert_eq!(
        MenuTree::from_bar(&bar),
        Err(TreeError::EmptyCommandId {
            title: "Mystery".to_owned()
        })
    );
}

#[test]
fn ids_are_positive_unique_and_stable() {
    let tree = tree_of(&bar());
    let ids: Vec<NodeId> = tree.walk().into_iter().map(|(node, _)| node.id).collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "unique");
    assert!(ids.iter().all(|id| id.0 > 0), "positive, the root is 0");
    assert_eq!(tree_of(&bar()), tree, "the same bar is the same tree");
    // A change of availability moves no id.
    let after: Vec<NodeId> = tree_of(&paste_disabled())
        .walk()
        .into_iter()
        .map(|(node, _)| node.id)
        .collect();
    assert_eq!(after, ids);
}

#[test]
fn a_command_in_two_menus_gets_two_ids_and_the_first_wins_the_lookup() {
    let twice = bar().with_items(
        BarSection::Edit,
        vec![MenuItem::new(
            Cmd::Bar(BarCommand::Standard(StandardAction::Undo)),
            "Undo Again",
        )],
    );
    let tree = tree_of(&twice);
    let undos: Vec<NodeId> = tree
        .walk()
        .into_iter()
        .filter(|(node, _)| tree.command_of(node.id) == Some(&undo()))
        .map(|(node, _)| node.id)
        .collect();
    assert_eq!(undos.len(), 2);
    assert_ne!(undos[0], undos[1]);
    assert_eq!(tree.id_of(&undo()), Some(undos[0]));
}

#[test]
fn a_command_id_round_trips_through_its_node_id() {
    let bar = bar();
    let tree = tree_of(&bar);
    for (node, _) in tree.walk() {
        let NodeKind::Command { command, .. } = &node.kind else {
            continue;
        };
        let id = tree.id_of(command).expect("an id");
        assert_eq!(tree.command_of(id), Some(command));
        assert_eq!(
            bar.command(command).map(AppCommand::id).as_ref(),
            Some(command)
        );
    }
}

fn prop_of(layout: &DbusLayout, name: &str) -> Option<Prop> {
    layout
        .item
        .props
        .iter()
        .find(|prop| prop.name() == name)
        .cloned()
}

fn find_child<'a>(layout: &'a DbusLayout, label: &str) -> &'a DbusLayout {
    layout
        .children
        .iter()
        .find(|child| prop_of(child, "label") == Some(Prop::Label(label.to_owned())))
        .unwrap_or_else(|| panic!("no child labelled {label}"))
}

fn full(tree: &MenuTree) -> DbusLayout {
    tree.layout(NodeId::ROOT, Depth::All, &PropNames::default())
        .expect("a root")
}

#[test]
fn the_edit_menu_lists_undo_with_its_title_and_command_z() {
    let layout = full(&tree_of(&bar()));
    assert_eq!(
        prop_of(&layout, "children-display"),
        Some(Prop::ChildrenDisplay(ChildrenDisplay::Submenu))
    );
    let titles: Vec<Option<Prop>> = layout
        .children
        .iter()
        .map(|menu| prop_of(menu, "label"))
        .collect();
    let want: Vec<Option<Prop>> = ["Notes", "File", "Edit", "View", "Window", "Help"]
        .iter()
        .map(|title| Some(Prop::Label((*title).to_owned())))
        .collect();
    assert_eq!(titles, want, "the six menus in the bar's order");
    let undo = find_child(find_child(&layout, "Edit"), "Undo");
    assert_eq!(
        undo.item.props,
        vec![
            Prop::Label("Undo".to_owned()),
            Prop::Enabled(Availability::Enabled),
            Prop::Shortcut(Chord(vec!["Super".to_owned(), "Z".to_owned()])),
            Prop::CommandId(CommandId("standard.undo".to_owned())),
        ]
    );
}

#[test]
fn properties_follow_what_the_item_is() {
    let tree = tree_of(&paste_disabled());
    let layout = full(&tree);
    let file = find_child(&layout, "File");
    let wrap = find_child(find_child(&layout, "View"), "Wrap Lines");
    let paste = find_child(find_child(&layout, "Edit"), "Paste");
    let rule = &find_child(&layout, "Notes").children[1];
    assert_eq!(
        prop_of(wrap, "toggle-type"),
        Some(Prop::ToggleType(ToggleType::Checkmark))
    );
    assert_eq!(
        prop_of(wrap, "toggle-state"),
        Some(Prop::ToggleState(ToggleState::On))
    );
    assert_eq!(
        prop_of(paste, "enabled"),
        Some(Prop::Enabled(Availability::Disabled))
    );
    assert_eq!(rule.item.props, vec![Prop::Type(ItemType::Separator)]);
    assert!(rule.children.is_empty());
    // A literal underscore is doubled, so no client reads it as a mnemonic.
    assert_eq!(
        prop_of(find_child(file, "Export__As…"), "label"),
        Some(Prop::Label("Export__As…".to_owned()))
    );
}

#[test]
fn depth_and_names_narrow_a_layout() {
    let tree = tree_of(&bar());
    let only_labels = PropNames(vec!["label".to_owned()]);
    let cases: &[(Depth, usize, usize)] = &[
        (Depth::Levels(0), 0, 0),
        (Depth::Levels(1), 6, 0),
        (Depth::All, 6, 7),
    ];
    for &(depth, menus, edit_rows) in cases {
        let layout = tree
            .layout(NodeId::ROOT, depth, &only_labels)
            .expect("a root");
        assert_eq!(layout.children.len(), menus, "{depth:?}");
        let rows = layout
            .children
            .iter()
            .find(|menu| prop_of(menu, "label") == Some(Prop::Label("Edit".to_owned())))
            .map_or(0, |edit| edit.children.len());
        assert_eq!(rows, edit_rows, "{depth:?}");
    }
    let layout = tree
        .layout(NodeId::ROOT, Depth::Levels(1), &only_labels)
        .expect("a root");
    assert!(layout.item.props.is_empty(), "the root has no label");
    assert!(
        layout
            .children
            .iter()
            .all(|menu| menu.item.props.len() == 1)
    );
    assert_eq!(Depth::from_wire(-1), Depth::All);
    assert_eq!(Depth::from_wire(2), Depth::Levels(2));
    let edit = tree.menu(BarSection::Edit).expect("Edit").id;
    assert!(
        tree.layout(edit, Depth::Levels(1), &PropNames::default())
            .is_some()
    );
    assert!(
        tree.layout(NodeId(7), Depth::All, &PropNames::default())
            .is_none()
    );
}

#[test]
fn group_properties_and_single_properties_read_the_same_props() {
    let tree = tree_of(&bar());
    let id = tree.id_of(&undo()).expect("undo");
    let rows = tree.group_properties(&[id, NodeId(7)], &PropNames::default());
    assert_eq!(rows.len(), 1, "an unknown id has no row");
    assert_eq!(
        tree.property(id, "label"),
        Some(Prop::Label("Undo".to_owned()))
    );
    assert_eq!(tree.property(id, "icon-name"), None);
    assert_eq!(
        tree.group_properties(&[], &PropNames::default()).len(),
        tree.walk().len(),
        "no ids means every item"
    );
}

#[test]
fn an_event_maps_back_to_the_command() {
    let tree = tree_of(&paste_disabled());
    let id_of = |command: &str| tree.id_of(&CommandId(command.to_owned())).expect(command);
    let edit = tree.menu(BarSection::Edit).expect("Edit").id;
    let cases: Vec<(&str, NodeId, &str, Result<Dispatch, EventError>)> = vec![
        (
            "click",
            id_of("standard.undo"),
            "clicked",
            Ok(Dispatch::Run(undo())),
        ),
        (
            "disabled",
            id_of("standard.paste"),
            "clicked",
            Err(EventError::Disabled(CommandId("standard.paste".to_owned()))),
        ),
        (
            "hover",
            id_of("standard.undo"),
            "hovered",
            Ok(Dispatch::Nothing),
        ),
        ("submenu", edit, "clicked", Ok(Dispatch::Nothing)),
        ("open", edit, "opened", Ok(Dispatch::Nothing)),
        ("root", NodeId::ROOT, "opened", Ok(Dispatch::Nothing)),
        (
            "unknown id",
            NodeId(9),
            "clicked",
            Err(EventError::UnknownItem(9)),
        ),
        (
            "unknown event",
            id_of("standard.undo"),
            "dragged",
            Err(EventError::UnknownEvent("dragged".to_owned())),
        ),
    ];
    for (name, id, event, want) in cases {
        assert_eq!(tree.event(id, event), want, "{name}");
    }
}

#[test]
fn the_revision_rises_only_when_the_tree_changes() {
    let state = MenuState::new(tree_of(&bar()));
    assert_eq!(state.revision, Revision(1));
    let (state, same) = state.replaced(tree_of(&bar()));
    assert_eq!((same, state.revision), (Change::Same, Revision(1)));

    let paste = state
        .tree
        .id_of(&CommandId("standard.paste".to_owned()))
        .expect("paste");
    let (state, disabled) = state.replaced(tree_of(&paste_disabled()));
    assert_eq!(
        (disabled, state.revision),
        (Change::Properties(vec![paste]), Revision(2))
    );

    let (state, enabled) = state.replaced(tree_of(&bar()));
    assert_eq!(
        (enabled, state.revision),
        (Change::Properties(vec![paste]), Revision(3))
    );

    let more = bar().with_items(BarSection::Edit, vec![MenuItem::new(Cmd::Export, "Extra")]);
    let (state, layout) = state.replaced(tree_of(&more));
    assert_eq!((layout, state.revision), (Change::Layout, Revision(4)));
}

#[test]
fn the_tree_is_json_and_reads_back() {
    let tree = tree_of(&paste_disabled());
    let json = serde_json::to_string(&tree).expect("json");
    assert!(json.contains(r#""command":"standard.undo""#));
    assert!(json.contains(r#""kind":"command""#));
    assert!(json.contains(r#""shortcut":["Super","Z"]"#));
    assert_eq!(serde_json::from_str::<MenuTree>(&json).expect("back"), tree);
}
