//! An app's exported menu bar, read and driven over a private bus: the Edit menu lists Undo with
//! its title and ⌘Z, a click on Undo runs the app's Undo, and disabling an item announces a new
//! layout. The bus is a scratch `dbus-daemon`; the real session bus is never reached.

#[path = "menu_export/bus.rs"]
mod bus;

use bus::PrivateBus;
use dioxus::prelude::*;
use ds::components::menus::export::{AppMenuAddress, Change, MenuTree};
use ds::components::menus::item::item::MenuItem;
use ds::components::menus::menu_bar::{BarCommand, BarSection, MenuBarModel};
use ds_blitz::AppId;
use ds_blitz::menus::{
    AllowAnyCaller, Bus, Caller, MenuExport, MenuExportConfig, MenuExportError, MenuPermit, Verdict,
};
use ds_core::command::{AppCommand, CommandFace, CommandId, UiOnlyReason};
use ds_core::standard_action::StandardAction;
use ds_core::vocab::Availability;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{OwnedValue, Type, Value};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 200,
    scale_percent: 100,
};
const APP_ID: &str = "org.quire.HarnessNotes";
const DBUSMENU: &str = "com.canonical.dbusmenu";

/// The harness app's command type: the bar's standard items and one of its own.
#[derive(Debug, Clone, PartialEq)]
enum Cmd {
    Bar(BarCommand),
    Archive,
}

impl AppCommand for Cmd {
    fn id(&self) -> CommandId {
        match self {
            Cmd::Bar(bar) => bar.id(),
            Cmd::Archive => CommandId("file.archive".to_owned()),
        }
    }

    fn face(&self) -> CommandFace {
        CommandFace::UiOnly(UiOnlyReason::new("harness").expect("a reason"))
    }
}

/// The app's bar: the standard six and Archive; `undo` says whether Undo can run.
fn bar(undo: Availability) -> MenuBarModel<Cmd> {
    MenuBarModel::standard("Notes", Cmd::Bar)
        .with_items(
            BarSection::File,
            vec![MenuItem::new(Cmd::Archive, "Archive")],
        )
        .availability(move |cmd| match cmd {
            Cmd::Bar(BarCommand::Standard(StandardAction::Undo)) => undo,
            _ => Availability::Enabled,
        })
}

thread_local! {
    /// Where the bus thread's activations reach the page; each test thread makes its own.
    static ACTIVATIONS: RefCell<Option<UnboundedReceiver<CommandId>>> =
        const { RefCell::new(None) };
}

/// The app's one handler: what its own menu runs on a pick.
fn handle(command: &Cmd, undone: &mut Signal<u32>) {
    if matches!(
        command,
        Cmd::Bar(BarCommand::Standard(StandardAction::Undo))
    ) {
        *undone += 1;
    }
}

#[allow(non_snake_case)]
fn Notes() -> Element {
    let mut undone = use_signal(|| 0_u32);
    use_hook(|| {
        let Some(mut activations) = ACTIVATIONS.with(|slot| slot.borrow_mut().take()) else {
            return;
        };
        let bar = bar(Availability::Enabled);
        spawn(async move {
            while let Some(id) = activations.recv().await {
                if let Some(command) = bar.command(&id) {
                    handle(command, &mut undone);
                }
            }
        });
    });
    rsx! { div { id: "undone", "data-count": "{undone}" } }
}

fn undone(harness: &Harness) -> u32 {
    harness
        .attr("#undone", "data-count")
        .and_then(|text| text.parse().ok())
        .unwrap_or(0)
}

/// `(ia{sv}av)` as a client reads it.
#[derive(Debug, Type, Value, OwnedValue, serde::Deserialize)]
#[zvariant(crate = "zbus::zvariant")]
struct Layout {
    id: i32,
    props: HashMap<String, OwnedValue>,
    children: Vec<OwnedValue>,
}

impl Layout {
    fn label(&self) -> Option<String> {
        self.props.get("label")?.try_clone().ok()?.try_into().ok()
    }

    fn child(&self, label: &str) -> Layout {
        self.children
            .iter()
            .filter_map(|child| Layout::try_from(child.try_clone().ok()?).ok())
            .find(|child| child.label().as_deref() == Some(label))
            .unwrap_or_else(|| panic!("no item labelled {label}"))
    }

    fn flag(&self, name: &str) -> Option<bool> {
        self.props.get(name)?.try_clone().ok()?.try_into().ok()
    }

    fn shortcut(&self) -> Option<Vec<Vec<String>>> {
        self.props
            .get("shortcut")?
            .try_clone()
            .ok()?
            .try_into()
            .ok()
    }
}

/// A client of the app's menu on the private bus.
struct Client {
    proxy: Proxy<'static>,
}

/// A new client connection to the private bus, and its unique name (what a permit sees).
fn connect(bus: &PrivateBus) -> (Connection, String) {
    let connection = zbus::blocking::connection::Builder::address(bus.address.as_str())
        .expect("a unix address")
        .build()
        .expect("the client connects to the private bus");
    let unique = connection
        .unique_name()
        .map(ToString::to_string)
        .unwrap_or_default();
    (connection, unique)
}

impl Client {
    fn new(bus: &PrivateBus, service: &str) -> Client {
        Client::on(&connect(bus).0, service)
    }

    fn on(connection: &Connection, service: &str) -> Client {
        let proxy = Proxy::new(
            connection,
            service.to_owned(),
            AppMenuAddress::PATH,
            DBUSMENU,
        )
        .expect("a proxy");
        Client { proxy }
    }

    fn layout(&self) -> (u32, Layout) {
        self.proxy
            .call("GetLayout", &(0_i32, -1_i32, Vec::<String>::new()))
            .expect("GetLayout")
    }

    fn click(&self, id: i32) -> zbus::Result<()> {
        self.proxy
            .call("Event", &(id, "clicked", Value::from(0_i32), 0_u32))
    }

    /// One `EventGroup` call that clicks item `id` `times` times; the ids it refused.
    fn click_group(&self, id: i32, times: usize) -> zbus::Result<Vec<i32>> {
        let events: Vec<(i32, &str, Value<'_>, u32)> = (0..times)
            .map(|_| (id, "clicked", Value::from(0_i32), 0_u32))
            .collect();
        self.proxy.call("EventGroup", &(events,))
    }
}

/// What a method error says, whichever form zbus hands it over in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    AccessDenied,
    LimitsExceeded,
    InvalidArgs,
    Other,
}

fn refusal(error: &zbus::Error) -> Refusal {
    const DENIED: &str = "org.freedesktop.DBus.Error.AccessDenied";
    const LIMITS: &str = "org.freedesktop.DBus.Error.LimitsExceeded";
    const ARGS: &str = "org.freedesktop.DBus.Error.InvalidArgs";
    match error {
        zbus::Error::FDO(inner) => match **inner {
            zbus::fdo::Error::AccessDenied(_) => Refusal::AccessDenied,
            zbus::fdo::Error::LimitsExceeded(_) => Refusal::LimitsExceeded,
            zbus::fdo::Error::InvalidArgs(_) => Refusal::InvalidArgs,
            _ => Refusal::Other,
        },
        zbus::Error::MethodError(name, ..) => match name.as_str() {
            DENIED => Refusal::AccessDenied,
            LIMITS => Refusal::LimitsExceeded,
            ARGS => Refusal::InvalidArgs,
            _ => Refusal::Other,
        },
        _ => Refusal::Other,
    }
}

/// A permit that lets one connection (by unique name) press items.
struct OnlyFrom(String);

impl MenuPermit for OnlyFrom {
    async fn permit(&self, caller: &Caller) -> Verdict {
        match caller.sender.0 == self.0 {
            true => Verdict::Allow,
            false => Verdict::Refuse,
        }
    }
}

/// Run `harness` until `done`, in short real-time steps: the bus delivers on its own thread.
fn settle(harness: &mut Harness, done: impl Fn(&Harness) -> bool, within: Duration) {
    let until = Instant::now() + within;
    while !done(harness) && Instant::now() < until {
        harness.advance(Duration::from_millis(20));
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn start(bus: &PrivateBus, tx: tokio::sync::mpsc::UnboundedSender<CommandId>) -> MenuExport {
    start_with(bus, tx, AllowAnyCaller)
}

fn start_with(
    bus: &PrivateBus,
    tx: tokio::sync::mpsc::UnboundedSender<CommandId>,
    permit: impl MenuPermit,
) -> MenuExport {
    let config = MenuExportConfig {
        app_id: AppId(APP_ID.to_owned()),
        bus: Bus::Address(bus.address.clone()),
    };
    let tree = MenuTree::from_bar(&bar(Availability::Enabled)).expect("a tree");
    MenuExport::start(
        &config,
        tree,
        move |command| {
            let _ = tx.send(command);
        },
        permit,
    )
    .expect("the export starts on the private bus")
}

#[test]
fn the_edit_menu_is_served_and_a_click_on_undo_runs_the_apps_undo() {
    let bus = PrivateBus::start("click");
    let (tx, rx) = unbounded_channel();
    ACTIVATIONS.with(|slot| *slot.borrow_mut() = Some(rx));
    let mut harness = Harness::new(Notes, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let export = start(&bus, tx);
    let service = AppMenuAddress::for_app_id(APP_ID)
        .expect("an address")
        .service;

    let client = Client::new(&bus, &service);
    let (revision, root) = client.layout();
    assert_eq!(revision, 1);
    let edit = root.child("Edit");
    let undo = edit.child("Undo");
    assert_eq!(undo.label().as_deref(), Some("Undo"));
    assert_eq!(
        undo.shortcut(),
        Some(vec![vec!["Super".to_owned(), "Z".to_owned()]])
    );
    assert_eq!(undo.flag("enabled"), Some(true));
    let titles: Vec<Option<String>> = root
        .children
        .iter()
        .filter_map(|child| Layout::try_from(child.try_clone().ok()?).ok())
        .map(|menu| menu.label())
        .collect();
    assert_eq!(titles.first(), Some(&Some("Notes".to_owned())));
    assert_eq!(titles.last(), Some(&Some("Help".to_owned())));

    assert_eq!(undone(&harness), 0);
    client.click(undo.id).expect("Event");
    settle(
        &mut harness,
        |harness| undone(harness) == 1,
        Duration::from_secs(10),
    );
    assert_eq!(undone(&harness), 1, "the click ran the app's Undo");
    drop(export);
}

#[test]
fn disabling_an_item_announces_a_new_layout_and_refuses_its_click() {
    let bus = PrivateBus::start("disable");
    let (tx, rx) = unbounded_channel();
    ACTIVATIONS.with(|slot| *slot.borrow_mut() = Some(rx));
    let mut harness = Harness::new(Notes, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    let export = start(&bus, tx);
    let service = AppMenuAddress::for_app_id(APP_ID)
        .expect("an address")
        .service;
    let client = Client::new(&bus, &service);
    let undo_id = client.layout().1.child("Edit").child("Undo").id;

    // Listen first, so the announcement cannot be missed.
    let (heard, announced) = mpsc::channel();
    let signals = client
        .proxy
        .receive_signal("LayoutUpdated")
        .expect("a match rule");
    std::thread::spawn(move || {
        for signal in signals {
            let body: Result<(u32, i32), _> = signal.body().deserialize();
            let _ = heard.send(body.ok());
        }
    });

    let change = export.update(&bar(Availability::Disabled)).expect("update");
    assert!(matches!(change, Change::Properties(ids) if ids.len() == 1));
    assert_eq!(
        announced.recv_timeout(Duration::from_secs(10)),
        Ok(Some((2, 0))),
        "LayoutUpdated(revision 2, parent 0)"
    );
    let (revision, root) = client.layout();
    assert_eq!(revision, 2);
    assert_eq!(
        root.child("Edit").child("Undo").flag("enabled"),
        Some(false)
    );

    assert!(
        client.click(undo_id).is_err(),
        "a disabled item cannot be picked"
    );
    // Give a (wrong) delivery time to arrive before checking that none did.
    settle(&mut harness, |_| false, Duration::from_millis(400));
    assert_eq!(undone(&harness), 0, "the app's Undo did not run");

    // The same bar again is no change: the revision stays.
    assert_eq!(
        export.update(&bar(Availability::Disabled)).expect("update"),
        Change::Same
    );
    assert_eq!(export.revision().0, 2);
}

#[test]
fn a_second_process_of_the_same_app_id_is_refused_the_name() {
    let bus = PrivateBus::start("second");
    let (tx, _rx) = unbounded_channel();
    let first = start(&bus, tx.clone());
    let service = AppMenuAddress::for_app_id(APP_ID)
        .expect("an address")
        .service;

    let config = MenuExportConfig {
        app_id: AppId(APP_ID.to_owned()),
        bus: Bus::Address(bus.address.clone()),
    };
    let tree = MenuTree::from_bar(&bar(Availability::Enabled)).expect("a tree");
    let second = MenuExport::start(&config, tree, |_| {}, AllowAnyCaller);
    assert!(
        matches!(&second, Err(MenuExportError::NameTaken(name)) if *name == service),
        "the name stays with its owner and the second is told"
    );

    let by_name = Client::new(&bus, &service);
    assert_eq!(by_name.layout().0, 1, "the first still owns the name");
    drop(first);
}

#[test]
fn only_a_caller_the_permit_lets_may_press_an_item_but_anyone_may_read() {
    let bus = PrivateBus::start("permit");
    let (tx, mut activations) = unbounded_channel();
    let service = AppMenuAddress::for_app_id(APP_ID)
        .expect("an address")
        .service;
    let ((shell_bus, shell_name), (intruder_bus, _)) = (connect(&bus), connect(&bus));
    let export = start_with(&bus, tx, OnlyFrom(shell_name));
    let (shell, intruder) = (
        Client::on(&shell_bus, &service),
        Client::on(&intruder_bus, &service),
    );
    let undo = shell.layout().1.child("Edit").child("Undo").id;

    assert_eq!(intruder.layout().0, 1, "reading the menu is open");
    let refused = intruder.click(undo).expect_err("refused");
    assert_eq!(refusal(&refused), Refusal::AccessDenied);
    let refused = intruder.click_group(undo, 1).expect_err("refused");
    assert_eq!(refusal(&refused), Refusal::AccessDenied);
    assert!(activations.try_recv().is_err(), "nothing ran");

    shell.click(undo).expect("the permitted caller presses");
    assert!(activations.try_recv().is_ok(), "the press ran the command");
    drop(export);
}

#[test]
fn an_event_group_is_limited_in_size_and_in_calls_per_second() {
    let bus = PrivateBus::start("group");
    let (tx, mut activations) = unbounded_channel();
    let export = start(&bus, tx);
    let service = AppMenuAddress::for_app_id(APP_ID)
        .expect("an address")
        .service;
    let client = Client::new(&bus, &service);
    let undo = client.layout().1.child("Edit").child("Undo").id;

    let too_many = client.click_group(undo, 65).expect_err("over the size cap");
    assert_eq!(refusal(&too_many), Refusal::InvalidArgs);
    assert!(activations.try_recv().is_err(), "none of the 65 ran");
    assert_eq!(client.click_group(undo, 64), Ok(Vec::new()));
    assert_eq!(
        std::iter::from_fn(|| activations.try_recv().ok()).count(),
        64,
        "all 64 ran"
    );

    // A burst beyond eight calls in a second is told to wait.
    let waited = (0..30)
        .filter_map(|_| client.click_group(undo, 1).err())
        .map(|error| refusal(&error))
        .collect::<Vec<_>>();
    assert!(
        waited.contains(&Refusal::LimitsExceeded),
        "a burst of 30 calls was not paced: {waited:?}"
    );
    drop(export);
}
