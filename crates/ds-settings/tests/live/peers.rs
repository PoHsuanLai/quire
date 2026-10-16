//! Who hears `Changed`, and what a client does with a module that does not answer.

use std::pin::Pin;
use std::time::{Duration, Instant};

use ds_settings::live::{
    Access, AnyOwner, Caller, Deadline, INTERFACE, LiveClient, LiveError, LiveModule, LiveSchema,
    Timeouts, Verdict, serve,
};
use ds_settings::schema::KeyPath;
use zbus::export::futures_core::Stream;
use zbus::message::Type;
use zbus::{MatchRule, MessageStream};

use crate::bus::PrivateBus;
use crate::module::{Accounts, TOGGLE};

const PATH: &str = "/org/quire/Peers1/settings";
const NAME: &str = "org.quire.Peers1";

async fn next<S: Stream + Unpin>(stream: &mut S) -> Option<S::Item> {
    std::future::poll_fn(|cx| Pin::new(&mut *stream).poll_next(cx)).await
}

fn key(path: &str) -> KeyPath {
    KeyPath(path.to_owned())
}

#[tokio::test]
async fn changed_reaches_the_peers_that_may_read_and_not_one_that_was_refused() {
    let bus = PrivateBus::start("hears");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    let spy = bus.connect().await;
    let role = settings.unique_name().expect("named").to_string();
    let spy_name = spy.unique_name().expect("named").to_string();
    daemon.request_name(NAME).await.expect("name");
    let served = serve(
        &daemon,
        PATH,
        Accounts::new(Some(role)).refusing_reads_from(spy_name),
    )
    .await
    .expect("serves");

    // The spy asks the bus for every `Changed`, as any app may.
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .interface(INTERFACE)
        .expect("interface")
        .member("Changed")
        .expect("member")
        .build();
    let mut heard = MessageStream::for_match_rule(rule, &spy, None)
        .await
        .expect("subscribed");
    let spy_client = LiveClient::new(&spy, NAME, PATH, AnyOwner).expect("proxy");
    assert!(matches!(
        spy_client.changes().await,
        Err(LiveError::NotPermitted(_))
    ));

    let client = LiveClient::new(&settings, NAME, PATH, AnyOwner).expect("proxy");
    let mut changes = client.changes().await.expect("subscribed");
    client
        .set(&key(TOGGLE), &toml::Value::Boolean(false))
        .await
        .expect("allowed");
    served
        .changed(&key(TOGGLE), &toml::Value::Boolean(true))
        .await
        .expect("emits");
    for want in [false, true] {
        let change = next(&mut changes)
            .await
            .expect("a signal")
            .expect("decodes");
        assert_eq!(change.value, toml::Value::Boolean(want));
    }
    let overheard = tokio::time::timeout(Duration::from_millis(300), next(&mut heard)).await;
    assert!(
        overheard.is_err(),
        "a refused peer was sent a value: {overheard:?}"
    );
}

/// A module that is up but never answers a read or a write.
struct Hanging;

impl LiveModule for Hanging {
    async fn permit(&self, _: &Caller, _: Access) -> Verdict {
        Verdict::Allow
    }
    async fn describe(&self) -> LiveSchema {
        // One key, so a snapshot has a value to wait for.
        crate::module::schema()
    }
    async fn get(&self, _: &KeyPath) -> Result<toml::Value, LiveError> {
        std::future::pending().await
    }
    async fn set(&self, _: &KeyPath, _: toml::Value) -> Result<(), LiveError> {
        std::future::pending().await
    }
}

#[tokio::test]
async fn a_module_that_does_not_answer_ends_every_call_as_a_timeout() {
    let bus = PrivateBus::start("hangs");
    let daemon = bus.connect().await;
    let settings = bus.connect().await;
    daemon.request_name(NAME).await.expect("name");
    serve(&daemon, PATH, Hanging).await.expect("serves");
    let wait = Deadline::of(Duration::from_millis(150));
    let client = LiveClient::new(&settings, NAME, PATH, AnyOwner)
        .expect("proxy")
        .with_timeouts(Timeouts::new(wait, wait));

    let started = Instant::now();
    assert!(matches!(
        client.get(&key(TOGGLE)).await,
        Err(LiveError::Timeout(_))
    ));
    assert!(matches!(
        client.snapshot().await,
        Err(LiveError::Timeout(_))
    ));
    let set = client.set(&key(TOGGLE), &toml::Value::Boolean(true)).await;
    assert!(
        matches!(&set, Err(LiveError::Timeout(why)) if why == "The service is not answering"),
        "{set:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(5));
}
