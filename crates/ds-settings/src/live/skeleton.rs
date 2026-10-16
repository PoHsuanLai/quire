//! The serving side: exports a [`LiveModule`] as `org.quire.SettingsModule1` on a connection.

use std::collections::HashMap;

use zbus::message::Header;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{ObjectPath, Value};
use zbus::{Connection, interface};

use super::readers::{Readers, addressed};
use super::value::{from_wire, to_wire};
use super::{Access, Caller, INTERFACE, LiveError, LiveModule, SenderName, Verdict};
use crate::schema::KeyPath;

/// The object `org.quire.SettingsModule1` is exported from: the module and nothing else (the
/// daemon keeps its state behind the module).
struct Skeleton<M> {
    module: M,
    /// The peers `permit` let read: the only ones `Changed` is sent to.
    readers: Readers,
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
        let verdict = match &sender {
            Some(sender) => {
                self.module
                    .permit(
                        &Caller {
                            sender: sender.clone(),
                            connection: connection.clone(),
                        },
                        access,
                    )
                    .await
            }
            None => Verdict::Refuse,
        };
        match (verdict, sender) {
            (Verdict::Allow, Some(reader)) if access == Access::Read => {
                self.readers.add(connection, &reader).await;
                Ok(())
            }
            (Verdict::Allow, _) => Ok(()),
            (Verdict::Refuse, _) => Err(LiveError::NotPermitted(match access {
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

    /// The schema (as `Describe`) and every key's value (as `Get`), read once.
    #[zbus(out_args("schema", "values"))]
    async fn snapshot(
        &self,
        #[zbus(header)] header: Header<'_>,
        #[zbus(connection)] connection: &Connection,
    ) -> Result<(String, HashMap<String, Value<'static>>), LiveError> {
        self.authorise(&header, connection, Access::Read).await?;
        let snapshot = self.module.snapshot().await?;
        snapshot
            .schema
            .check()
            .map_err(|why| LiveError::Failed(why.to_string()))?;
        let values = snapshot
            .values
            .iter()
            .map(|(key, value)| Ok((key.0.clone(), to_wire(value)?)))
            .collect::<Result<_, LiveError>>()?;
        Ok((snapshot.schema.to_json(), values))
    }

    /// Sets a key (or runs an action key), then sends `Changed` to the peers that may read.
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
        for reader in self.readers.present(connection).await {
            if let Some(to_reader) = addressed(&emitter, reader) {
                // A reader that left a moment ago loses its signal and no one else does.
                let _ = Self::changed(&to_reader, &key.0, to_wire(&now)?).await;
            }
        }
        Ok(())
    }

    /// A key's new value, sent to the peers that may read (never broadcast).
    #[zbus(signal)]
    async fn changed(emitter: &SignalEmitter<'_>, key: &str, value: Value<'_>) -> zbus::Result<()>;
}

/// A module that is being served: lets the daemon announce changes it did not get through `Set`
/// (an account that went `NeedsReauth`, a runtime that came online).
#[derive(Debug, Clone)]
pub struct Served {
    connection: Connection,
    path: ObjectPath<'static>,
    readers: Readers,
}

impl Served {
    /// Sends `Changed(key, value)` from the module's object path to the peers `permit` let read,
    /// and to no one else.
    pub async fn changed(&self, key: &KeyPath, value: &toml::Value) -> Result<(), LiveError> {
        let emitter = SignalEmitter::new(&self.connection, self.path.clone())?;
        for reader in self.readers.present(&self.connection).await {
            if let Some(to_reader) = addressed(&emitter, reader) {
                let body = (key.0.as_str(), to_wire(value)?);
                // A reader that left a moment ago loses its signal and no one else does.
                let _ = to_reader.emit(INTERFACE, "Changed", &body).await;
            }
        }
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
    let readers = Readers::default();
    connection
        .object_server()
        .at(
            path.clone(),
            Skeleton {
                module,
                readers: readers.clone(),
            },
        )
        .await?;
    Ok(Served {
        connection: connection.clone(),
        path,
        readers,
    })
}

#[cfg(test)]
mod tests {
    use super::{Readers, Skeleton};
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
        Skeleton {
            module: Nothing,
            readers: Readers::default(),
        }
        .introspect_to_writer(&mut xml, 0);
        assert_eq!(xml.trim(), INTROSPECTION_XML.trim());
    }
}
