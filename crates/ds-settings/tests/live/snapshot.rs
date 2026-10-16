//! `Snapshot`: the trait's default, the bus method, the permission hook, and the client's
//! fallback against a module that predates the method.

use std::collections::BTreeMap;

use ds_settings::live::{
    Access, AnyOwner, Caller, INTERFACE, LiveClient, LiveError, LiveModule, LiveSchema, Snapshot,
    Verdict, serve,
};
use ds_settings::schema::KeyPath;
use zbus::interface;
use zbus::zvariant::Value;

use crate::bus::PrivateBus;
use crate::module::{Accounts, CLIENT, REMOVE, TOGGLE, schema};

const PATH: &str = "/org/quire/Test1/settings";
const NAME: &str = "org.quire.Test1";

fn key(path: &str) -> KeyPath {
    KeyPath(path.to_owned())
}

/// A module whose `Describe` still lists a key that `Get` has already forgotten: an account
/// removed between the two calls.
struct Vanishing {
    inner: Accounts,
    gone: &'static str,
}

impl LiveModule for Vanishing {
    async fn permit(&self, caller: &Caller, access: Access) -> Verdict {
        self.inner.permit(caller, access).await
    }
    async fn describe(&self) -> LiveSchema {
        self.inner.describe().await
    }
    async fn get(&self, path: &KeyPath) -> Result<toml::Value, LiveError> {
        match path.0 == self.gone {
            true => Err(LiveError::UnknownKey(path.0.clone())),
            false => self.inner.get(path).await,
        }
    }
    async fn set(&self, path: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        self.inner.set(path, value).await
    }
}

/// A module that answers no read.
struct Closed(Accounts);

impl LiveModule for Closed {
    async fn permit(&self, _: &Caller, _: Access) -> Verdict {
        Verdict::Refuse
    }
    async fn describe(&self) -> LiveSchema {
        self.0.describe().await
    }
    async fn get(&self, path: &KeyPath) -> Result<toml::Value, LiveError> {
        self.0.get(path).await
    }
    async fn set(&self, path: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        self.0.set(path, value).await
    }
}

/// The interface as it was before `Snapshot`: `Describe` and `Get` only.
struct Legacy(Accounts);

#[interface(name = "org.quire.SettingsModule1")]
impl Legacy {
    async fn describe(&self) -> Result<String, LiveError> {
        Ok(self.0.describe().await.to_json())
    }

    async fn get(&self, path: String) -> Result<Value<'static>, LiveError> {
        ds_settings::live::value::to_wire(&self.0.get(&KeyPath(path)).await?)
    }
}

fn described_and_got(schema: LiveSchema, accounts: &Accounts) -> Snapshot {
    let values: BTreeMap<KeyPath, toml::Value> = schema
        .key
        .iter()
        .map(|spec| (spec.path.clone(), spec.default.clone()))
        .collect();
    assert_eq!(accounts.schema, schema);
    Snapshot { schema, values }
}

#[tokio::test]
async fn the_default_snapshot_is_describe_then_every_get() {
    let accounts = Accounts::new(None);
    let expected = described_and_got(schema(), &accounts);
    assert_eq!(accounts.snapshot().await.expect("reads"), expected);
}

#[tokio::test]
async fn a_key_removed_between_describe_and_get_is_dropped_from_schema_and_values() {
    let module = Vanishing {
        inner: Accounts::new(None),
        gone: CLIENT,
    };
    let snapshot = module.snapshot().await.expect("reads");
    let listed: Vec<&str> = snapshot
        .schema
        .key
        .iter()
        .map(|k| k.path.0.as_str())
        .collect();
    let valued: Vec<&str> = snapshot.values.keys().map(|k| k.0.as_str()).collect();
    assert_eq!(listed, [TOGGLE, REMOVE]);
    assert_eq!(valued, [REMOVE, TOGGLE]);
}

#[tokio::test]
async fn another_get_failure_fails_the_snapshot() {
    let reads = [(key(TOGGLE), Err(LiveError::Failed("disk".to_owned())))];
    match Snapshot::from_reads(schema(), reads) {
        Err(LiveError::Failed(_)) => {}
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[tokio::test]
async fn snapshot_over_the_bus_equals_describe_and_get() {
    let bus = PrivateBus::start("snapshot");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    serve(&daemon, PATH, Accounts::new(None))
        .await
        .expect("serves");

    let client = LiveClient::new(&settings, NAME, PATH, AnyOwner).expect("proxy");
    let snapshot = client.snapshot().await.expect("snapshot");
    assert_eq!(snapshot.schema, client.describe().await.unwrap());
    assert_eq!(snapshot, client.snapshot_by_parts().await.unwrap());
    assert_eq!(snapshot.values.len(), schema().key.len());
    assert_eq!(
        snapshot.values[&key(TOGGLE)],
        client.get(&key(TOGGLE)).await.unwrap()
    );
}

#[tokio::test]
async fn a_vanished_key_is_absent_from_the_bus_snapshot_too() {
    let bus = PrivateBus::start("snapshot-gone");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    let module = Vanishing {
        inner: Accounts::new(None),
        gone: CLIENT,
    };
    serve(&daemon, PATH, module).await.expect("serves");

    let client = LiveClient::new(&settings, NAME, PATH, AnyOwner).expect("proxy");
    let snapshot = client.snapshot().await.expect("snapshot");
    assert!(snapshot.schema.key.iter().all(|spec| spec.path.0 != CLIENT));
    assert!(!snapshot.values.contains_key(&key(CLIENT)));
    assert_eq!(snapshot, client.snapshot_by_parts().await.unwrap());
}

#[tokio::test]
async fn a_refused_reader_gets_not_permitted() {
    let bus = PrivateBus::start("snapshot-refused");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    serve(&daemon, PATH, Closed(Accounts::new(None)))
        .await
        .expect("serves");

    let client = LiveClient::new(&settings, NAME, PATH, AnyOwner).expect("proxy");
    match client.snapshot().await {
        Err(LiveError::NotPermitted(_)) => {}
        other => panic!("expected NotPermitted, got {other:?}"),
    }
}

#[tokio::test]
async fn a_module_without_snapshot_is_read_by_the_client() {
    let bus = PrivateBus::start("snapshot-legacy");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    daemon
        .object_server()
        .at(PATH, Legacy(Accounts::new(None)))
        .await
        .expect("exports");

    // The raw call fails the way an old module fails it.
    let raw = zbus::Proxy::new(&settings, NAME, PATH, INTERFACE)
        .await
        .unwrap();
    assert!(raw.call_method("Snapshot", &()).await.is_err());

    let client = LiveClient::new(&settings, NAME, PATH, AnyOwner).expect("proxy");
    let snapshot = client.snapshot().await.expect("falls back");
    assert_eq!(snapshot.schema, schema());
    assert_eq!(snapshot.values.len(), schema().key.len());
}
