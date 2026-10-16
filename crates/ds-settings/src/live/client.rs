//! The client side: what the Settings app (detent) uses to read and set a live module.

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use ds_peer::{Deadline, within};
use zbus::Connection;
use zbus::export::futures_core::Stream;
use zbus::zvariant::{OwnedValue, Value};

use super::link::Link;
use super::value::{from_wire, to_wire};
use super::{LiveError, LiveSchema, OwnerCheck, Snapshot, Timeouts, limits};
use crate::schema::KeyPath;

/// The raw proxy of `org.quire.SettingsModule1`: the frozen signatures, no conversion.
#[zbus::proxy(interface = "org.quire.SettingsModule1", gen_blocking = false)]
pub trait SettingsModule1 {
    fn describe(&self) -> Result<String, LiveError>;
    fn get(&self, key: &str) -> Result<OwnedValue, LiveError>;
    fn snapshot(&self) -> Result<(String, HashMap<String, OwnedValue>), LiveError>;
    fn set(&self, key: &str, value: &Value<'_>) -> Result<(), LiveError>;
    #[zbus(signal)]
    fn changed(&self, key: &str, value: Value<'_>) -> zbus::Result<()>;
}

/// Who a screen says is not answering when a call to the module runs out of time.
const PEER: &str = "The service";

/// `work` under `deadline`: a module that does not answer is [`LiveError::Timeout`].
async fn bounded<T>(
    deadline: Deadline,
    work: impl Future<Output = Result<T, LiveError>>,
) -> Result<T, LiveError> {
    match within(deadline, PEER, work).await {
        Ok(answer) => answer,
        Err(silent) => Err(LiveError::Timeout(silent.to_string())),
    }
}

/// One `Changed` signal.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub key: KeyPath,
    pub value: toml::Value,
}

/// A typed client of one live module: a bus name and an object path, and the check that the
/// program owning that name is the one the module belongs to.
///
/// Before the first read or `Set`, and again whenever the name's owner changes, the client asks
/// its [`OwnerCheck`] about the owner. Only an owner that passes is spoken to, and then by its
/// unique name, so the name cannot change hands between the check and the call. When the check
/// fails every call is [`LiveError::NotTheService`] and nothing is sent: a typed secret never
/// reaches a program that merely took the name.
#[derive(Debug, Clone)]
pub struct LiveClient {
    link: Link,
    timeouts: Timeouts,
}

impl LiveClient {
    /// A client of the module at `path` on the peer named `destination`
    /// (`org.quire.Accounts1`, ...), whose owner `check` vouches for. Touches the bus only when
    /// first used.
    pub fn new(
        connection: &Connection,
        destination: &str,
        path: &str,
        check: impl OwnerCheck,
    ) -> Result<LiveClient, LiveError> {
        Ok(LiveClient {
            link: Link::new(connection, destination, path, Arc::new(check))?,
            timeouts: Timeouts::default(),
        })
    }

    /// This client with `timeouts` for each call to the module.
    pub fn with_timeouts(mut self, timeouts: Timeouts) -> LiveClient {
        self.timeouts = timeouts;
        self
    }

    /// The module's schema (checked: a schema that breaks the never-settable rule is refused here
    /// too, so a misbehaving daemon cannot hand the Settings app an agent-settable `accounts.` key;
    /// a schema over the [`limits`] is refused as well).
    pub async fn describe(&self) -> Result<LiveSchema, LiveError> {
        bounded(self.timeouts.read, async {
            describe_on(&self.link.proxy().await?).await
        })
        .await
    }

    pub async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        bounded(self.timeouts.read, async {
            get_on(&self.link.proxy().await?, key).await
        })
        .await
    }

    /// The module's schema and every value, read once. A module that predates `Snapshot`
    /// answers `UnknownMethod`; the client then reads `describe` and each key itself, dropping
    /// a key that went away, so it gets the same shape (without the one-read guarantee).
    pub async fn snapshot(&self) -> Result<Snapshot, LiveError> {
        bounded(self.timeouts.read, async {
            snapshot_on(&self.link.proxy().await?).await
        })
        .await
    }

    /// `describe` then a `get` per key: what [`LiveClient::snapshot`] does against a module
    /// without `Snapshot`.
    ///
    /// The whole read has one read deadline, not one per key.
    pub async fn snapshot_by_parts(&self) -> Result<Snapshot, LiveError> {
        bounded(self.timeouts.read, async {
            parts_on(&self.link.proxy().await?).await
        })
        .await
    }

    /// Sets a key. The owner is checked before the value is put on the wire.
    pub async fn set(&self, key: &KeyPath, value: &toml::Value) -> Result<(), LiveError> {
        let wire = to_wire(value)?;
        bounded(self.timeouts.action, async {
            self.link.proxy().await?.set(&key.0, &wire).await
        })
        .await
    }

    /// Runs an action key (`KeyKind::Live`): a `Set` of `true`.
    pub async fn invoke(&self, key: &KeyPath) -> Result<(), LiveError> {
        self.set(key, &toml::Value::Boolean(true)).await
    }

    /// The module's `Changed` signals, in order. The subscription is made first and the module is
    /// then asked for its schema, which is how the module learns this peer may read: it sends
    /// `Changed` to the peers it let read, and to no one else. A peer the module refuses gets
    /// [`LiveError::NotPermitted`] here instead of a stream that never speaks.
    ///
    /// The stream hears the owner that passed the check and no other: when the name changes hands
    /// it goes quiet, and a new `changes()` checks the new owner.
    pub async fn changes(&self) -> Result<Changes, LiveError> {
        bounded(self.timeouts.read, async {
            let proxy = self.link.proxy().await?;
            let inner = proxy.receive_changed().await?;
            describe_on(&proxy).await?;
            Ok(Changes { inner })
        })
        .await
    }
}

async fn describe_on(proxy: &SettingsModule1Proxy<'static>) -> Result<LiveSchema, LiveError> {
    limits::schema(&proxy.describe().await?)
}

async fn get_on(
    proxy: &SettingsModule1Proxy<'static>,
    key: &KeyPath,
) -> Result<toml::Value, LiveError> {
    limits::value(from_wire(&Value::from(proxy.get(&key.0).await?))?)
}

async fn snapshot_on(proxy: &SettingsModule1Proxy<'static>) -> Result<Snapshot, LiveError> {
    match proxy.snapshot().await {
        Ok((json, values)) => {
            let schema = limits::schema(&json)?;
            limits::rows(values.len())?;
            let values = values
                .iter()
                .map(|(key, value)| Ok((KeyPath(key.clone()), limits::value(from_wire(value)?)?)))
                .collect::<Result<BTreeMap<_, _>, LiveError>>()?;
            Ok(Snapshot { schema, values })
        }
        Err(why) if is_unknown_method(&why) => parts_on(proxy).await,
        Err(why) => Err(why),
    }
}

async fn parts_on(proxy: &SettingsModule1Proxy<'static>) -> Result<Snapshot, LiveError> {
    let schema = describe_on(proxy).await?;
    let mut reads = Vec::with_capacity(schema.key.len());
    for spec in &schema.key {
        reads.push((spec.path.clone(), get_on(proxy, &spec.path).await));
    }
    Snapshot::from_reads(schema, reads)
}

/// Whether the peer answered that it has no such method (a module built before `Snapshot`).
fn is_unknown_method(why: &LiveError) -> bool {
    const UNKNOWN_METHOD: &str = "org.freedesktop.DBus.Error.UnknownMethod";
    match why {
        LiveError::ZBus(zbus::Error::MethodError(name, ..)) => name.as_str() == UNKNOWN_METHOD,
        LiveError::ZBus(zbus::Error::FDO(inner)) => {
            matches!(**inner, zbus::fdo::Error::UnknownMethod(_))
        }
        _ => false,
    }
}

/// The stream [`LiveClient::changes`] returns.
#[derive(Debug)]
pub struct Changes {
    inner: ChangedStream,
}

impl Stream for Changes {
    type Item = Result<Change, LiveError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.inner).poll_next(cx).map(|next| {
            next.map(|signal| {
                let args = signal.args()?;
                Ok(Change {
                    key: KeyPath(args.key().to_string()),
                    value: limits::value(from_wire(args.value())?)?,
                })
            })
        })
    }
}
