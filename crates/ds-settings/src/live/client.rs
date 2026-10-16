//! The client side: what the Settings app (detent) uses to read and set a live module.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use ds_peer::{Deadline, within};
use zbus::Connection;
use zbus::export::futures_core::Stream;
use zbus::zvariant::{OwnedValue, Value};

use super::value::{from_wire, to_wire};
use super::{LiveError, LiveSchema, Snapshot, Timeouts};
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

/// A typed client of one live module: a bus name and an object path, and no state.
#[derive(Debug, Clone)]
pub struct LiveClient {
    proxy: SettingsModule1Proxy<'static>,
    timeouts: Timeouts,
}

impl LiveClient {
    /// A client of the module at `path` on the peer named `destination`
    /// (`org.quire.Accounts1`, ...).
    pub async fn new(
        connection: &Connection,
        destination: &str,
        path: &str,
    ) -> Result<LiveClient, LiveError> {
        let timeouts = Timeouts::default();
        let proxy = bounded(timeouts.read, async {
            Ok(SettingsModule1Proxy::builder(connection)
                .destination(destination.to_owned())?
                .path(path.to_owned())?
                .build()
                .await?)
        })
        .await?;
        Ok(LiveClient { proxy, timeouts })
    }

    /// This client with `timeouts` for each call to the module.
    pub fn with_timeouts(mut self, timeouts: Timeouts) -> LiveClient {
        self.timeouts = timeouts;
        self
    }

    /// The module's schema (checked: a schema that breaks the never-settable rule is refused here
    /// too, so a misbehaving daemon cannot hand the Settings app an agent-settable `accounts.` key).
    pub async fn describe(&self) -> Result<LiveSchema, LiveError> {
        bounded(self.timeouts.read, self.describe_unbounded()).await
    }

    async fn describe_unbounded(&self) -> Result<LiveSchema, LiveError> {
        let json = self.proxy.describe().await?;
        LiveSchema::from_json(&json).map_err(|why| LiveError::Failed(why.to_string()))
    }

    pub async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        bounded(self.timeouts.read, self.get_unbounded(key)).await
    }

    async fn get_unbounded(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        from_wire(&Value::from(self.proxy.get(&key.0).await?))
    }

    /// The module's schema and every value, read once. A module that predates `Snapshot`
    /// answers `UnknownMethod`; the client then reads `describe` and each key itself, dropping
    /// a key that went away, so it gets the same shape (without the one-read guarantee).
    pub async fn snapshot(&self) -> Result<Snapshot, LiveError> {
        bounded(self.timeouts.read, self.snapshot_unbounded()).await
    }

    async fn snapshot_unbounded(&self) -> Result<Snapshot, LiveError> {
        match self.proxy.snapshot().await {
            Ok((json, values)) => {
                let schema = LiveSchema::from_json(&json)
                    .map_err(|why| LiveError::Failed(why.to_string()))?;
                let values = values
                    .iter()
                    .map(|(key, value)| Ok((KeyPath(key.clone()), from_wire(value)?)))
                    .collect::<Result<_, LiveError>>()?;
                Ok(Snapshot { schema, values })
            }
            Err(why) if is_unknown_method(&why) => self.parts_unbounded().await,
            Err(why) => Err(why),
        }
    }

    /// `describe` then a `get` per key: what [`LiveClient::snapshot`] does against a module
    /// without `Snapshot`.
    ///
    /// The whole read has one read deadline, not one per key.
    pub async fn snapshot_by_parts(&self) -> Result<Snapshot, LiveError> {
        bounded(self.timeouts.read, self.parts_unbounded()).await
    }

    async fn parts_unbounded(&self) -> Result<Snapshot, LiveError> {
        let schema = self.describe_unbounded().await?;
        let mut reads = Vec::with_capacity(schema.key.len());
        for spec in &schema.key {
            reads.push((spec.path.clone(), self.get_unbounded(&spec.path).await));
        }
        Snapshot::from_reads(schema, reads)
    }

    pub async fn set(&self, key: &KeyPath, value: &toml::Value) -> Result<(), LiveError> {
        let wire = to_wire(value)?;
        bounded(self.timeouts.action, async {
            self.proxy.set(&key.0, &wire).await
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
    pub async fn changes(&self) -> Result<Changes, LiveError> {
        let inner = bounded(self.timeouts.read, async {
            Ok(self.proxy.receive_changed().await?)
        })
        .await?;
        self.describe().await?;
        Ok(Changes { inner })
    }
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
                    value: from_wire(args.value())?,
                })
            })
        })
    }
}
