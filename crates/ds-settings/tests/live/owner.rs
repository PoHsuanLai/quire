//! The owner check and the caps: a module whose name was taken by another program is sent nothing,
//! and a module cannot hand the client more than a page can show.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use ds_settings::live::limits::{CHOICES_MAX, KEYS_MAX, TEXT_MAX, VALUE_ITEMS_MAX, VALUE_TEXT_MAX};
use ds_settings::live::{
    Access, AnyOwner, Caller, LiveClient, LiveError, LiveModule, LiveSchema, OwnerAnswer,
    OwnerCheck, SenderName, Verdict, serve,
};
use ds_settings::schema::{KeyKind, KeyPath};

use crate::bus::PrivateBus;
use crate::module::{Accounts, CLIENT, REMOVE, TOGGLE, schema, spec};

const PATH: &str = "/org/quire/Owner1/settings";
const NAME: &str = "org.quire.Owner1";

fn key(path: &str) -> KeyPath {
    KeyPath(path.to_owned())
}

/// The owners the check vouches for, and every owner it was asked about.
#[derive(Debug, Clone, Default)]
struct Roster {
    allowed: Arc<Mutex<HashSet<String>>>,
    asked: Arc<Mutex<Vec<String>>>,
}

impl Roster {
    fn vouch_for(&self, owner: &zbus::Connection) {
        let name = owner.unique_name().expect("named").to_string();
        self.allowed.lock().expect("allowed").insert(name);
    }

    fn asked(&self) -> Vec<String> {
        self.asked.lock().expect("asked").clone()
    }
}

impl OwnerCheck for Roster {
    fn holds_role<'a>(&'a self, _name: &'a str, owner: &'a SenderName) -> OwnerAnswer<'a> {
        self.asked.lock().expect("asked").push(owner.0.clone());
        let known = self.allowed.lock().expect("allowed").contains(&owner.0);
        Box::pin(async move {
            match known {
                true => Verdict::Allow,
                false => Verdict::Refuse,
            }
        })
    }
}

/// A module that says how often it was asked anything, and keeps the strings it was set to.
#[derive(Debug, Clone, Default)]
struct Witness {
    asked: Arc<AtomicU32>,
    set: Arc<Mutex<Vec<String>>>,
}

impl LiveModule for Witness {
    async fn permit(&self, _: &Caller, _: Access) -> Verdict {
        self.asked.fetch_add(1, Ordering::SeqCst);
        Verdict::Allow
    }

    async fn describe(&self) -> LiveSchema {
        schema()
    }

    async fn get(&self, _: &KeyPath) -> Result<toml::Value, LiveError> {
        Ok(toml::Value::String(String::new()))
    }

    async fn set(&self, _: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        let text = value.as_str().unwrap_or_default().to_owned();
        self.set.lock().expect("set").push(text);
        Ok(())
    }
}

fn not_the_service<T>(answer: Result<T, LiveError>) -> bool {
    matches!(answer, Err(LiveError::NotTheService(_)))
}

#[tokio::test]
async fn a_refused_owner_is_sent_nothing_and_no_secret() {
    let bus = PrivateBus::start("refused");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    let witness = Witness::default();
    serve(&daemon, PATH, witness.clone()).await.expect("serves");

    let roster = Roster::default();
    let client = LiveClient::new(&settings, NAME, PATH, roster.clone()).expect("client");
    let secret = toml::Value::String("hunter2".to_owned());

    assert!(not_the_service(client.set(&key(CLIENT), &secret).await));
    assert!(not_the_service(client.invoke(&key(REMOVE)).await));
    assert!(not_the_service(client.snapshot().await));
    assert!(not_the_service(client.snapshot_by_parts().await));
    assert!(not_the_service(client.describe().await));
    assert!(not_the_service(client.get(&key(TOGGLE)).await));
    assert!(not_the_service(client.changes().await));
    assert_eq!(
        witness.asked.load(Ordering::SeqCst),
        0,
        "no call reached the module"
    );
    assert!(
        witness.set.lock().expect("set").is_empty(),
        "no secret reached it"
    );

    // The module would have taken the call: it was the client that refused.
    roster.vouch_for(&daemon);
    client
        .set(&key(CLIENT), &secret)
        .await
        .expect("vouched for");
    assert_eq!(*witness.set.lock().expect("set"), ["hunter2"]);
}

#[tokio::test]
async fn a_new_owner_of_the_name_is_checked_again() {
    let bus = PrivateBus::start("handover");
    let first = bus.connect().await;
    let second = bus.connect().await;
    let settings = bus.connect().await;
    let (at_first, at_second) = (Witness::default(), Witness::default());
    first.request_name(NAME).await.expect("name");
    serve(&first, PATH, at_first.clone()).await.expect("serves");
    serve(&second, PATH, at_second.clone())
        .await
        .expect("serves");

    let roster = Roster::default();
    roster.vouch_for(&first);
    let client = LiveClient::new(&settings, NAME, PATH, roster.clone()).expect("client");
    client.snapshot().await.expect("the first owner passes");
    client.snapshot().await.expect("and is not asked again");
    assert_eq!(roster.asked().len(), 1);

    // The name changes hands to a program the check does not know.
    assert!(first.release_name(NAME).await.expect("released"));
    second.request_name(NAME).await.expect("name");
    let secret = toml::Value::String("hunter2".to_owned());
    assert!(not_the_service(client.set(&key(CLIENT), &secret).await));
    assert_eq!(roster.asked().len(), 2, "the new owner was asked");
    assert!(at_second.set.lock().expect("set").is_empty());
    assert_eq!(at_second.asked.load(Ordering::SeqCst), 0);

    roster.vouch_for(&second);
    client
        .set(&key(CLIENT), &secret)
        .await
        .expect("vouched for");
    assert_eq!(*at_second.set.lock().expect("set"), ["hunter2"]);
    assert!(
        at_first.set.lock().expect("set").is_empty(),
        "the old owner heard nothing"
    );
}

fn texts(count: usize) -> Vec<ds_settings::schema::KeySpec> {
    (0..count)
        .map(|n| {
            spec(
                &format!("accounts.k{n}"),
                KeyKind::Text,
                toml::Value::String(String::new()),
            )
        })
        .collect()
}

fn one(kind: KeyKind, default: toml::Value) -> Vec<ds_settings::schema::KeySpec> {
    vec![spec("accounts.one", kind, default)]
}

fn long_label() -> Vec<ds_settings::schema::KeySpec> {
    let mut keys = texts(1);
    keys[0].label.0 = "x".repeat(TEXT_MAX + 1);
    keys
}

fn many_choices() -> Vec<ds_settings::schema::KeySpec> {
    let variants = (0..=CHOICES_MAX).map(|n| format!("w{n}")).collect();
    one(
        KeyKind::Menu { variants },
        toml::Value::String("w0".to_owned()),
    )
}

/// Schemas each over one cap, by name.
fn over_the_caps() -> Vec<(&'static str, Vec<ds_settings::schema::KeySpec>)> {
    vec![
        ("rows", texts(KEYS_MAX + 1)),
        ("label", long_label()),
        ("choices", many_choices()),
        (
            "text",
            one(
                KeyKind::Text,
                toml::Value::String("x".repeat(VALUE_TEXT_MAX + 1)),
            ),
        ),
        (
            "list",
            one(
                KeyKind::List(Box::new(KeyKind::Text)),
                toml::Value::Array(vec!["a".into(); VALUE_ITEMS_MAX + 1]),
            ),
        ),
    ]
}

#[tokio::test]
async fn a_schema_over_a_cap_is_refused_and_one_at_the_cap_is_read() {
    let bus = PrivateBus::start("caps");
    let settings = bus.connect().await;
    let mut cases = over_the_caps();
    cases.push(("at the rows cap", texts(KEYS_MAX)));
    for (n, (case, key)) in cases.into_iter().enumerate() {
        let daemon = bus.connect().await;
        let name = format!("org.quire.Caps{n}");
        daemon.request_name(name.as_str()).await.expect("name");
        let module = Accounts::with_schema(None, LiveSchema { version: 1, key });
        serve(&daemon, PATH, module).await.expect("serves");
        let client = LiveClient::new(&settings, &name, PATH, AnyOwner).expect("client");
        let (described, snapshot) = (client.describe().await, client.snapshot().await);
        match case.starts_with("at the") {
            true => assert!(described.is_ok() && snapshot.is_ok(), "{case}"),
            false => assert!(
                matches!(described, Err(LiveError::Failed(_)))
                    && matches!(snapshot, Err(LiveError::Failed(_))),
                "{case}: {described:?} {snapshot:?}"
            ),
        }
    }
}

/// A module whose schema is small and whose values are not.
struct Bulky;

impl LiveModule for Bulky {
    async fn permit(&self, _: &Caller, _: Access) -> Verdict {
        Verdict::Allow
    }

    async fn describe(&self) -> LiveSchema {
        LiveSchema {
            version: 1,
            key: texts(1),
        }
    }

    async fn get(&self, _: &KeyPath) -> Result<toml::Value, LiveError> {
        Ok(toml::Value::String("x".repeat(VALUE_TEXT_MAX + 1)))
    }

    async fn set(&self, _: &KeyPath, _: toml::Value) -> Result<(), LiveError> {
        Ok(())
    }
}

#[tokio::test]
async fn a_value_over_the_cap_is_refused() {
    let bus = PrivateBus::start("bulky");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    serve(&daemon, PATH, Bulky).await.expect("serves");
    let client = LiveClient::new(&settings, NAME, PATH, AnyOwner).expect("client");
    assert!(matches!(
        client.get(&key("accounts.k0")).await,
        Err(LiveError::Failed(_))
    ));
    assert!(matches!(client.snapshot().await, Err(LiveError::Failed(_))));
}
