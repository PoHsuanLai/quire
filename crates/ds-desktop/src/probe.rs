//! `Desktop::probe`: ask the bus who answers.

use crate::Desktop;

impl Desktop {
    /// Ask the buses which capabilities are here. Without feature `dbus`, or when no bus can be
    /// reached, everything is `Absent`; this never fails.
    pub async fn probe() -> Desktop {
        #[cfg(feature = "dbus")]
        {
            let (session, system) = bus::connect().await;
            Desktop::probe_on(session.as_ref(), system.as_ref()).await
        }
        #[cfg(not(feature = "dbus"))]
        {
            Desktop::default()
        }
    }
}

#[cfg(feature = "dbus")]
pub(crate) mod bus {
    use crate::capability::{Bus, Capability};
    use crate::{Desktop, Presence};
    use std::collections::HashSet;
    use zbus::Connection;
    use zbus::fdo::DBusProxy;

    /// The session and system connections; either may be missing (a headless box, a sandbox).
    pub(crate) async fn connect() -> (Option<Connection>, Option<Connection>) {
        (
            Connection::session().await.ok(),
            Connection::system().await.ok(),
        )
    }

    /// One bus: its daemon proxy and the names it can activate.
    pub(crate) struct Side {
        pub(crate) proxy: DBusProxy<'static>,
        pub(crate) activatable: HashSet<String>,
    }

    impl Side {
        pub(crate) async fn open(connection: &Connection) -> Option<Side> {
            let proxy = DBusProxy::new(connection).await.ok()?;
            let activatable = proxy
                .list_activatable_names()
                .await
                .map(|names| names.into_iter().map(|n| n.to_string()).collect())
                .unwrap_or_default();
            Some(Side { proxy, activatable })
        }

        /// A name with an owner, or one the bus would start, is here.
        pub(crate) fn presence(&self, name: &str, owned: bool) -> Presence {
            if owned || self.activatable.contains(name) {
                Presence::Here
            } else {
                Presence::Absent
            }
        }

        async fn owned(&self, name: &str) -> bool {
            let Ok(name) = zbus::names::BusName::try_from(name) else {
                return false;
            };
            self.proxy.name_has_owner(name).await.unwrap_or(false)
        }
    }

    /// The capabilities whose service is `name` on `bus`.
    pub(crate) fn capabilities_named(bus: Bus, name: &str) -> impl Iterator<Item = Capability> {
        let name = name.to_owned();
        Capability::ALL
            .into_iter()
            .filter(move |c| c.service().is_some_and(|s| s.bus == bus && s.name == name))
    }

    pub(crate) async fn fill(desktop: &mut Desktop, session: Option<&Side>, system: Option<&Side>) {
        for capability in Capability::ALL {
            let Some(service) = capability.service() else {
                continue;
            };
            let side = match service.bus {
                Bus::Session => session,
                Bus::System => system,
            };
            let presence = match side {
                Some(side) => side.presence(service.name, side.owned(service.name).await),
                None => Presence::Absent,
            };
            desktop.set(capability, presence);
        }
    }

    impl Desktop {
        /// Probe over the given connections (a private bus in tests); a missing bus leaves its
        /// capabilities `Absent`.
        pub async fn probe_on(
            session: Option<&Connection>,
            system: Option<&Connection>,
        ) -> Desktop {
            let session = match session {
                Some(c) => Side::open(c).await,
                None => None,
            };
            let system = match system {
                Some(c) => Side::open(c).await,
                None => None,
            };
            let mut desktop = Desktop::default();
            fill(&mut desktop, session.as_ref(), system.as_ref()).await;
            desktop
        }
    }
}
