//! The serving side: exports a [`LiveModule`] as `org.quire.SettingsModule1` on a connection.

use zbus::message::Header;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, Value};
use zbus::{Connection, interface};

use super::value::{from_wire, to_wire};
use super::{Access, Caller, INTERFACE, LiveError, LiveModule, SenderName, Verdict};
use crate::schema::KeyPath;

/// The object `org.quire.SettingsModule1` is exported from: the module and nothing else (the
/// daemon keeps its state behind the module).
struct Skeleton<M> {
    module: M,
}

impl<M: LiveModule> Skeleton<M> {
    /// Asks the module's hook; a call with no sender (a peer-to-peer message) is refused.
    async fn authorise(
        &self,
        header: &Header<'_>,
        connection: &Connection,
        access: Access,
    ) -> Result<(), LiveError> {
        let sender = header.sender().map(|name| SenderName(name.to_string()));
        let verdict = match sender {
            Some(sender) => {
                self.module
                    .permit(
                        &Caller {
                            sender,
                            connection: connection.clone(),
                        },
                        access,
                    )
                    .await
            }
            None => Verdict::Refuse,
        };
        match verdict {
            Verdict::Allow => Ok(()),
            Verdict::Refuse => Err(LiveError::NotPermitted(match access {
                Access::Read => "this caller may not read the settings module".to_owned(),
                Access::Write => "this caller may not change the settings module".to_owned(),
            })),
        }
    }

    /// Refuses a `Set` of a key to a choice its current schema lists as unavailable, with the
    /// reason. A key the schema does not list is left to the module (`UnknownKey` is its call).
    async fn refuse_unavailable(
        &self,
        key: &KeyPath,
        value: &toml::Value,
    ) -> Result<(), LiveError> {
        let schema = self.module.describe().await;
        let spec = schema.key.iter().find(|spec| spec.path == *key);
        match spec.map(|spec| spec.check_available(value)) {
            Some(Err(refused)) => Err(LiveError::Unavailable(refused.reason.0)),
            _ => Ok(()),
        }
    }
}

#[interface(name = "org.quire.SettingsModule1")]
impl<M: LiveModule> Skeleton<M> {
    /// The module's schema as JSON (`LiveSchema`), refused if it breaks the never-settable rule.
    #[zbus(out_args("schema"))]
    async fn describe(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> Result<String, LiveError> {
        self.authorise(&header, connection, Access::Read).await?;
        let schema = self.module.describe().await;
        schema
            .check()
            .map_err(|why| LiveError::Failed(why.to_string()))?;
        Ok(schema.to_json())
    }

    /// A key's current value.
    #[zbus(out_args("value"))]
    async fn get(
        &self,
        key: String,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> Result<Value<'static>, LiveError> {
        self.authorise(&header, connection, Access::Read).await?;
        let value = self.module.get(&KeyPath(key)).await?;
        to_wire(&value)
    }

    /// Sets a key (or runs an action key), then broadcasts `Changed`.
    async fn set(
        &self,
        key: String,
        value: Value<'_>,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> Result<(), LiveError> {
        self.authorise(&header, connection, Access::Write).await?;
        let key = KeyPath(key);
        let wire = from_wire(&value)?;
        self.refuse_unavailable(&key, &wire).await?;
        self.module.set(&key, wire.clone()).await?;
        // An action key holds no value: its `Changed` carries what was sent.
        let now = self.module.get(&key).await.unwrap_or(wire);
        Self::changed(&emitter, &key.0, to_wire(&now)?).await?;
        Ok(())
    }

    /// A key's new value.
    #[zbus(signal)]
    async fn changed(emitter: &SignalEmitter<'_>, key: &str, value: Value<'_>) -> zbus::Result<()>;
}

/// A module that is being served: lets the daemon announce changes it did not get through `Set`
/// (an account that went `NeedsReauth`, a runtime that came online).
#[derive(Debug, Clone)]
pub struct Served {
    connection: Connection,
    path: ObjectPath<'static>,
}

impl Served {
    /// Broadcasts `Changed(key, value)` from the module's object path.
    pub async fn changed(&self, key: &KeyPath, value: &toml::Value) -> Result<(), LiveError> {
        let emitter = SignalEmitter::new(&self.connection, self.path.clone())?;
        emitter
            .emit(INTERFACE, "Changed", &(key.0.as_str(), to_wire(value)?))
            .await?;
        Ok(())
    }

    /// The object path the module is served at.
    pub fn path(&self) -> &ObjectPath<'static> {
        &self.path
    }
}

/// Exports `module` as `org.quire.SettingsModule1` at `path` on `connection`.
///
/// The module's schema is checked first ([`LiveSchema::check`](super::LiveSchema::check)): a
/// module that marks a never-settable key agent-settable is not exported. The caller still
/// requests its well-known bus name.
pub async fn serve<M: LiveModule>(
    connection: &Connection,
    path: &str,
    module: M,
) -> Result<Served, LiveError> {
    module
        .describe()
        .await
        .check()
        .map_err(|why| LiveError::Failed(why.to_string()))?;
    let path = ObjectPath::try_from(path.to_owned())
        .map_err(|why| LiveError::Failed(format!("bad object path {path}: {why}")))?;
    connection
        .object_server()
        .at(path.clone(), Skeleton { module })
        .await?;
    Ok(Served {
        connection: connection.clone(),
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::Skeleton;
    use crate::live::{
        Access, Caller, INTROSPECTION_XML, LiveError, LiveModule, LiveSchema, Verdict,
    };
    use crate::schema::KeyPath;
    use zbus::object_server::Interface;

    struct Nothing;
    impl LiveModule for Nothing {
        async fn permit(&self, _: &Caller, _: Access) -> Verdict {
            Verdict::Refuse
        }
        async fn describe(&self) -> LiveSchema {
            LiveSchema {
                version: 1,
                key: Vec::new(),
            }
        }
        async fn get(&self, key: &KeyPath) -> Result<toml::Value, LiveError> {
            Err(LiveError::UnknownKey(key.0.clone()))
        }
        async fn set(&self, key: &KeyPath, _: toml::Value) -> Result<(), LiveError> {
            Err(LiveError::UnknownKey(key.0.clone()))
        }
    }

    #[test]
    fn the_skeleton_matches_the_checked_in_introspection_xml() {
        let mut xml = String::new();
        Skeleton { module: Nothing }.introspect_to_writer(&mut xml, 0);
        assert_eq!(xml.trim(), INTROSPECTION_XML.trim());
    }
}
