//! The client's hold on a module: the name's current owner, once the owner check let it pass.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use zbus::Connection;
use zbus::fdo::DBusProxy;
use zbus::names::BusName;
use zbus::zvariant::ObjectPath;

use super::{LiveError, OwnerCheck, SenderName, SettingsModule1Proxy, Verdict};

/// What a refused owner is told to the person, in plain words.
const NOT_THE_SERVICE: &str =
    "This service could not be confirmed, so nothing was read from it or sent to it.";

/// An owner that passed, and the proxy that speaks to it by its unique name.
#[derive(Debug, Clone)]
struct Held {
    owner: SenderName,
    proxy: SettingsModule1Proxy<'static>,
}

/// The module's name and path, the check, and the owner that last passed it. Cheap to clone; the
/// clones share what passed.
#[derive(Debug, Clone)]
pub(super) struct Link {
    connection: Connection,
    destination: String,
    path: String,
    check: Arc<dyn OwnerCheck>,
    held: Arc<Mutex<Option<Held>>>,
}

impl Link {
    /// A link to `destination` at `path`; both are checked to be names now, not on first use.
    pub(super) fn new(
        connection: &Connection,
        destination: &str,
        path: &str,
        check: Arc<dyn OwnerCheck>,
    ) -> Result<Link, LiveError> {
        BusName::try_from(destination)
            .map_err(|why| LiveError::Failed(format!("bad bus name {destination}: {why}")))?;
        ObjectPath::try_from(path)
            .map_err(|why| LiveError::Failed(format!("bad object path {path}: {why}")))?;
        Ok(Link {
            connection: connection.clone(),
            destination: destination.to_owned(),
            path: path.to_owned(),
            check,
            held: Arc::default(),
        })
    }

    /// A proxy to the name's owner right now, when that owner holds the module's role. The owner
    /// is looked up on every call; the check is asked only when it is a different connection than
    /// the one that last passed. The proxy's destination is the owner's unique name, so the name
    /// cannot change hands between this check and the call made on the proxy.
    pub(super) async fn proxy(&self) -> Result<SettingsModule1Proxy<'static>, LiveError> {
        let owner = self.owner().await?;
        if let Some(proxy) = self.passed(&owner) {
            return Ok(proxy);
        }
        match self.check.holds_role(&self.destination, &owner).await {
            Verdict::Refuse => {
                *self.held() = None;
                Err(LiveError::NotTheService(NOT_THE_SERVICE.to_owned()))
            }
            Verdict::Allow => {
                let proxy = SettingsModule1Proxy::builder(&self.connection)
                    .destination(owner.0.clone())?
                    .path(self.path.clone())?
                    .build()
                    .await?;
                *self.held() = Some(Held {
                    owner,
                    proxy: proxy.clone(),
                });
                Ok(proxy)
            }
        }
    }

    fn held(&self) -> MutexGuard<'_, Option<Held>> {
        // A poisoned lock still holds a whole value: the only writers replace it.
        self.held.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn passed(&self, owner: &SenderName) -> Option<SettingsModule1Proxy<'static>> {
        self.held()
            .as_ref()
            .filter(|held| held.owner == *owner)
            .map(|held| held.proxy.clone())
    }

    /// The unique name that owns the module's well-known name now.
    async fn owner(&self) -> Result<SenderName, LiveError> {
        let name = BusName::try_from(self.destination.as_str())
            .map_err(|why| LiveError::Failed(why.to_string()))?;
        let bus = DBusProxy::new(&self.connection).await?;
        let owner = bus.get_name_owner(name).await.map_err(zbus::Error::from)?;
        Ok(SenderName(owner.as_str().to_owned()))
    }
}
