//! `Desktop::watch`: follow name ownership.

use crate::Desktop;

/// A feed of the desktop's capabilities, updated as services appear and go. Await
/// [`Watch::changed`] in whatever task suits.
#[derive(Debug)]
pub struct Watch {
    #[cfg(feature = "dbus")]
    inner: live::Live,
}

impl Desktop {
    /// Start following the buses, or `None` without feature `dbus`. The first value
    /// [`Watch::current`] gives is a fresh probe; subscription happens before it, so no change
    /// between the two is lost.
    pub async fn watch() -> Option<Watch> {
        #[cfg(feature = "dbus")]
        {
            let (session, system) = crate::probe::bus::connect().await;
            Some(Desktop::watch_on(session.as_ref(), system.as_ref()).await)
        }
        #[cfg(not(feature = "dbus"))]
        {
            None
        }
    }
}

#[cfg(feature = "dbus")]
impl Desktop {
    /// Follow the given connections (a private bus in tests).
    pub async fn watch_on(
        session: Option<&zbus::Connection>,
        system: Option<&zbus::Connection>,
    ) -> Watch {
        Watch {
            inner: live::Live::start(session, system).await,
        }
    }
}

#[cfg(feature = "dbus")]
impl Watch {
    /// The capabilities as of the last change (or the first probe).
    pub fn current(&self) -> &Desktop {
        &self.inner.desktop
    }

    /// The desktop after the next change in any capability's presence, or `None` when no bus
    /// is being followed.
    pub async fn changed(&mut self) -> Option<Desktop> {
        self.inner.changed().await
    }
}

#[cfg(not(feature = "dbus"))]
impl Watch {
    /// Without feature `dbus` nothing changes: this is always `None`.
    pub async fn changed(&mut self) -> Option<Desktop> {
        None
    }
}

#[cfg(feature = "dbus")]
mod live {
    use crate::Desktop;
    use crate::capability::Bus;
    use crate::probe::bus::{Side, capabilities_named, fill};
    use std::future::{Future, poll_fn};
    use std::pin::pin;
    use std::task::Poll;
    use zbus::export::ordered_stream::OrderedStreamExt;
    use zbus::fdo::NameOwnerChangedStream;

    struct Followed {
        side: Side,
        changes: NameOwnerChangedStream,
    }

    impl Followed {
        async fn open(connection: Option<&zbus::Connection>) -> Option<Followed> {
            let side = Side::open(connection?).await?;
            let changes = side.proxy.receive_name_owner_changed().await.ok()?;
            Some(Followed { side, changes })
        }
    }

    pub(super) struct Live {
        pub(super) desktop: Desktop,
        session: Option<Followed>,
        system: Option<Followed>,
    }

    impl std::fmt::Debug for Live {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Live")
                .field("desktop", &self.desktop)
                .finish_non_exhaustive()
        }
    }

    /// A change of ownership of `name` on `bus`.
    struct Moved {
        bus: Bus,
        name: String,
        owned: bool,
    }

    impl Live {
        pub(super) async fn start(
            session: Option<&zbus::Connection>,
            system: Option<&zbus::Connection>,
        ) -> Live {
            let session = Followed::open(session).await;
            let system = Followed::open(system).await;
            let mut desktop = Desktop::default();
            fill(
                &mut desktop,
                session.as_ref().map(|f| &f.side),
                system.as_ref().map(|f| &f.side),
            )
            .await;
            Live {
                desktop,
                session,
                system,
            }
        }

        pub(super) async fn changed(&mut self) -> Option<Desktop> {
            if self.session.is_none() && self.system.is_none() {
                return None;
            }
            loop {
                let moved = self.next_move().await?;
                if self.apply(&moved) {
                    return Some(self.desktop.clone());
                }
            }
        }

        /// Fold `moved` into the desktop; whether any capability's presence changed.
        fn apply(&mut self, moved: &Moved) -> bool {
            let followed = match moved.bus {
                Bus::Session => self.session.as_ref(),
                Bus::System => self.system.as_ref(),
            };
            let Some(followed) = followed else {
                return false;
            };
            let presence = followed.side.presence(&moved.name, moved.owned);
            let mut changed = false;
            for capability in capabilities_named(moved.bus, &moved.name) {
                if self.desktop.get(capability) != presence {
                    self.desktop.set(capability, presence);
                    changed = true;
                }
            }
            changed
        }

        async fn next_move(&mut self) -> Option<Moved> {
            poll_fn(|cx| {
                for (bus, followed) in [
                    (Bus::Session, self.session.as_mut()),
                    (Bus::System, self.system.as_mut()),
                ] {
                    let Some(followed) = followed else { continue };
                    if let Poll::Ready(signal) = pin!(followed.changes.next()).poll(cx) {
                        let Some(signal) = signal else {
                            return Poll::Ready(None);
                        };
                        let Ok(args) = signal.args() else { continue };
                        return Poll::Ready(Some(Moved {
                            bus,
                            name: args.name().to_string(),
                            owned: args.new_owner().is_some(),
                        }));
                    }
                }
                Poll::Pending
            })
            .await
        }
    }
}
