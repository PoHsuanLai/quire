//! The client side: what the Settings app (detent) uses to read and set a live module.

use std::pin::Pin;
use std::task::{Context, Poll};

use zbus::Connection;
use zbus::export::futures_core::Stream;
use zbus::zvariant::{OwnedValue, Value};

use super::value::{from_wire, to_wire};
use super::{LiveError, LiveSchema};
use crate::schema::KeyPath;

/// The raw proxy of `org.quire.SettingsModule1`: the frozen signatures, no conversion.
#[zbus::proxy(interface = "org.quire.SettingsModule1", gen_blocking = false)]
pub trait SettingsModule1 {
    fn describe(&self) -> Result<String, LiveError>;
    fn get(&self, key: &str) -> Result<OwnedValue, LiveError>;
    fn set(&self, key: &str, value: &Value<'_>) -> Result<(), LiveError>;
    #[zbus(signal)]
    fn changed(&self, key: &str, value: Value<'_>) -> zbus::Result<()>;
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
}

impl LiveClient {
    /// A client of the module at `path` on the peer named `destination`
    /// (`org.quire.Accounts1`, ...).
    pub async fn new(
        connection: &Connection,
        destination: &str,
        path: &str,
    ) -> Result<LiveClient, LiveError> {
        let proxy = SettingsModule1Proxy::builder(connection)
            .destination(destination.to_owned())?
            .path(path.to_owned())?
            .build()
            .await?;
        Ok(LiveClient { proxy })
    }

    /// The module's schema (checked: a schema that breaks the never-settable rule is refused here
    /// too, so a misbehaving daemon cannot hand the Settings app an agent-settable `accounts.` key).
    pub async fn describe(&self) -> Result<LiveSchema, LiveError> {
        let json = self.proxy.describe().await?;
        LiveSchema::from_json(&json).map_err(|why| LiveError::Failed(why.to_string()))
    }

    pub async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
        from_wire(&Value::from(self.proxy.get(&key.0).await?))
    }

    pub async fn set(&self, key: &KeyPath, value: &toml::Value) -> Result<(), LiveError> {
        self.proxy.set(&key.0, &to_wire(value)?).await
    }

    /// Runs an action key (`KeyKind::Live`): a `Set` of `true`.
    pub async fn invoke(&self, key: &KeyPath) -> Result<(), LiveError> {
        self.set(key, &toml::Value::Boolean(true)).await
    }

    /// The module's `Changed` signals, in order.
    pub async fn changes(&self) -> Result<Changes, LiveError> {
        Ok(Changes {
            inner: self.proxy.receive_changed().await?,
        })
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
