//! The skeleton and the proxy over a private bus.

use std::pin::Pin;

use ds_settings::live::{Change, INTERFACE, LiveClient, LiveError, serve};
use ds_settings::schema::{AgentSetting, KeyKind, KeyPath};
use zbus::export::futures_core::Stream;

use crate::bus::PrivateBus;
use crate::module::{Accounts, CLIENT, REMOVE, TOGGLE, schema, spec};

const PATH: &str = "/org/quire/Test1/settings";
const NAME: &str = "org.quire.Test1";

/// The next item of `stream`, without a stream-combinator dependency.
async fn next<S: Stream + Unpin>(stream: &mut S) -> Option<S::Item> {
    std::future::poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
}

fn key(path: &str) -> KeyPath {
    KeyPath(path.to_owned())
}

#[tokio::test]
async fn set_get_describe_and_changed_round_trip() {
    let bus = PrivateBus::start("round");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    let role = settings.unique_name().expect("named").to_string();
    daemon.request_name(NAME).await.expect("name");
    serve(&daemon, PATH, Accounts::new(Some(role)))
        .await
        .expect("serves");

    let client = LiveClient::new(&settings, NAME, PATH).await.expect("proxy");
    assert_eq!(client.describe().await.expect("describe"), schema());
    assert_eq!(
        client.get(&key(TOGGLE)).await.unwrap(),
        toml::Value::Boolean(true)
    );

    let mut changes = client.changes().await.expect("subscribed");
    client
        .set(&key(TOGGLE), &toml::Value::Boolean(false))
        .await
        .expect("allowed");
    client
        .set(&key(CLIENT), &toml::Value::String("id-123".to_owned()))
        .await
        .expect("allowed");
    assert_eq!(
        client.get(&key(TOGGLE)).await.unwrap(),
        toml::Value::Boolean(false)
    );
    assert_eq!(
        client.get(&key(CLIENT)).await.unwrap(),
        toml::Value::String("id-123".to_owned())
    );
    assert_eq!(
        next(&mut changes)
            .await
            .expect("a signal")
            .expect("decodes"),
        Change {
            key: key(TOGGLE),
            value: toml::Value::Boolean(false)
        }
    );
    assert_eq!(
        next(&mut changes)
            .await
            .expect("a signal")
            .expect("decodes"),
        Change {
            key: key(CLIENT),
            value: toml::Value::String("id-123".to_owned())
        }
    );
}

#[tokio::test]
async fn a_daemon_originated_change_is_broadcast() {
    let bus = PrivateBus::start("served");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    let served = serve(&daemon, PATH, Accounts::new(None))
        .await
        .expect("serves");
    let client = LiveClient::new(&settings, NAME, PATH).await.expect("proxy");
    let mut changes = client.changes().await.expect("subscribed");
    served
        .changed(&key(TOGGLE), &toml::Value::Boolean(false))
        .await
        .expect("emits");
    assert_eq!(
        next(&mut changes)
            .await
            .expect("a signal")
            .expect("decodes"),
        Change {
            key: key(TOGGLE),
            value: toml::Value::Boolean(false)
        }
    );
    assert_eq!(served.path().as_str(), PATH);
}

#[tokio::test]
async fn an_action_key_runs_through_set() {
    let bus = PrivateBus::start("action");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    let role = settings.unique_name().expect("named").to_string();
    daemon.request_name(NAME).await.expect("name");
    let module = std::sync::Arc::new(Accounts::new(Some(role)));
    serve(&daemon, PATH, Shared(module.clone()))
        .await
        .expect("serves");
    let client = LiveClient::new(&settings, NAME, PATH).await.expect("proxy");
    client.invoke(&key(REMOVE)).await.expect("allowed");
    assert_eq!(*module.removed.lock().unwrap(), 1);
}

#[tokio::test]
async fn set_from_a_caller_without_the_role_is_refused_by_name() {
    let bus = PrivateBus::start("refused");
    let daemon = bus.connect().await;
    let agent = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    let module = std::sync::Arc::new(Accounts::new(Some(":1.nobody".to_owned())));
    serve(&daemon, PATH, Shared(module.clone()))
        .await
        .expect("serves");

    let client = LiveClient::new(&agent, NAME, PATH).await.expect("proxy");
    // Reading is allowed to anyone; writing is not.
    assert!(client.describe().await.is_ok());
    for refused in [
        client.set(&key(TOGGLE), &toml::Value::Boolean(false)).await,
        client.invoke(&key(REMOVE)).await,
    ] {
        match refused {
            Err(LiveError::NotPermitted(_)) => {}
            other => panic!("expected NotPermitted, got {other:?}"),
        }
    }
    assert_eq!(*module.removed.lock().unwrap(), 0);
    assert_eq!(
        client.get(&key(TOGGLE)).await.unwrap(),
        toml::Value::Boolean(true)
    );
    // The error crosses the bus under its documented name.
    let raw = zbus::Proxy::new(&agent, NAME, PATH, INTERFACE)
        .await
        .unwrap();
    let why: zbus::Error = raw
        .call_method("Set", &(TOGGLE, zbus::zvariant::Value::Bool(false)))
        .await
        .unwrap_err();
    match why {
        zbus::Error::MethodError(name, _, _) => {
            assert_eq!(
                name.as_str(),
                "org.quire.SettingsModule1.Error.NotPermitted"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn unknown_keys_and_bad_values_have_their_own_names() {
    let bus = PrivateBus::start("errors");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    let role = settings.unique_name().expect("named").to_string();
    daemon.request_name(NAME).await.expect("name");
    serve(&daemon, PATH, Accounts::new(Some(role)))
        .await
        .expect("serves");
    let client = LiveClient::new(&settings, NAME, PATH).await.expect("proxy");
    assert!(matches!(
        client.get(&key("accounts.nope")).await,
        Err(LiveError::UnknownKey(_))
    ));
    assert!(matches!(
        client.set(&key(TOGGLE), &toml::Value::Integer(3)).await,
        Err(LiveError::BadValue(_))
    ));
    assert!(matches!(
        client.set(&key(TOGGLE), &toml::Value::Float(1.5)).await,
        Err(LiveError::BadValue(_))
    ));
}

#[tokio::test]
async fn a_module_with_an_agent_settable_accounts_key_is_not_served() {
    let bus = PrivateBus::start("never");
    let daemon = bus.connect().await;
    let mut bad = schema();
    let mut agent_key = spec(
        "accounts.a1.service.chat",
        KeyKind::Text,
        toml::Value::String(String::new()),
    );
    agent_key.agent = AgentSetting::Settable;
    bad.key.push(agent_key);
    let refused = serve(&daemon, PATH, Accounts::with_schema(None, bad)).await;
    assert!(matches!(refused, Err(LiveError::Failed(_))), "{refused:?}");
}

/// `serve` takes the module by value; a test that inspects the module afterwards shares it.
struct Shared(std::sync::Arc<Accounts>);

impl ds_settings::live::LiveModule for Shared {
    async fn permit(
        &self,
        caller: &ds_settings::live::Caller,
        access: ds_settings::live::Access,
    ) -> ds_settings::live::Verdict {
        self.0.permit(caller, access).await
    }
    async fn describe(&self) -> ds_settings::live::LiveSchema {
        self.0.describe().await
    }
    async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        self.0.get(key).await
    }
    async fn set(&self, key: &KeyPath, value: toml::Value) -> Result<(), LiveError> {
        self.0.set(key, value).await
    }
}

#[tokio::test]
async fn a_set_to_an_unavailable_choice_is_refused_with_its_reason() {
    use ds_settings::schema::{ChoiceWord, UnavailableReason};
    let bus = PrivateBus::start("unavailable");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    let role = settings.unique_name().expect("named").to_string();
    daemon.request_name(NAME).await.expect("name");
    let mut schema = schema();
    let pick = schema
        .key
        .iter_mut()
        .find(|spec| spec.path.0 == CLIENT)
        .expect("the text key");
    pick.unavailable.insert(
        ChoiceWord("gated".to_owned()),
        UnavailableReason("Add an account to use".to_owned()),
    );
    serve(&daemon, PATH, Accounts::with_schema(Some(role), schema))
        .await
        .expect("serves");
    let client = LiveClient::new(&settings, NAME, PATH).await.expect("proxy");

    let refused = client
        .set(&key(CLIENT), &toml::Value::String("gated".to_owned()))
        .await;
    assert!(
        matches!(&refused, Err(LiveError::Unavailable(why)) if why == "Add an account to use"),
        "{refused:?}"
    );
    assert_eq!(
        client.get(&key(CLIENT)).await.unwrap(),
        toml::Value::String(String::new())
    );
    client
        .set(&key(CLIENT), &toml::Value::String("open".to_owned()))
        .await
        .expect("an available choice still sets");
    let described = client.describe().await.expect("describe");
    assert!(
        described
            .key
            .iter()
            .any(|spec| !spec.unavailable.is_empty())
    );
}
