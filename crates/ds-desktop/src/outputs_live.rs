//! Reading `org.quire.Outputs1` and following its changes.

use crate::outputs::{Outputs, Wire};
use std::future::poll_fn;
use std::pin::Pin;
use zbus::export::futures_core::Stream;
use zbus::proxy::PropertyStream;
use zbus::{Connection, proxy};

#[proxy(
    interface = "org.quire.Outputs1",
    default_service = "org.quire.Outputs1",
    default_path = "/org/quire/Outputs1",
    gen_blocking = false
)]
trait Outputs1 {
    #[zbus(property)]
    fn work_areas(&self) -> zbus::Result<Vec<Wire>>;
}

impl Outputs {
    /// The shell's outputs over the session bus; empty when no bus or no shell answers.
    pub async fn read() -> Outputs {
        match Connection::session().await {
            Ok(connection) => Outputs::read_on(&connection).await,
            Err(_) => Outputs::default(),
        }
    }

    /// The shell's outputs over `connection` (a private bus in tests); empty when nothing
    /// answers.
    pub async fn read_on(connection: &Connection) -> Outputs {
        let Ok(proxy) = Outputs1Proxy::new(connection).await else {
            return Outputs::default();
        };
        proxy
            .work_areas()
            .await
            .map(Outputs::of_wire)
            .unwrap_or_default()
    }

    /// Follow the outputs on the session bus, or `None` when there is no bus.
    pub async fn watch() -> Option<OutputsWatch> {
        let connection = Connection::session().await.ok()?;
        OutputsWatch::on(&connection).await
    }
}

/// The outputs as of now, and each change after.
pub struct OutputsWatch {
    proxy: Outputs1Proxy<'static>,
    changes: PropertyStream<'static, Vec<Wire>>,
    /// What the last read or change said, so a repeat is not reported as a change (the stream
    /// first hands over the value it already holds).
    seen: Outputs,
}

impl std::fmt::Debug for OutputsWatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputsWatch").finish_non_exhaustive()
    }
}

impl OutputsWatch {
    /// Follow `connection`. The subscription is made before anything is read, so no change
    /// after [`current`](OutputsWatch::current) is lost.
    pub async fn on(connection: &Connection) -> Option<OutputsWatch> {
        let proxy = Outputs1Proxy::new(connection).await.ok()?;
        let changes = proxy.receive_work_areas_changed().await;
        let seen = Outputs::read_on(connection).await;
        Some(OutputsWatch {
            proxy,
            changes,
            seen,
        })
    }

    /// The outputs now; empty when the shell is not there.
    pub async fn current(&self) -> Outputs {
        self.proxy
            .work_areas()
            .await
            .map(Outputs::of_wire)
            .unwrap_or_default()
    }

    /// The outputs after the next change (one that differs from the last seen), or `None` when
    /// the bus has gone.
    pub async fn changed(&mut self) -> Option<Outputs> {
        loop {
            let changed = poll_fn(|cx| Pin::new(&mut self.changes).poll_next(cx)).await?;
            let now = changed
                .get()
                .await
                .map(Outputs::of_wire)
                .unwrap_or_default();
            if now != self.seen {
                self.seen.clone_from(&now);
                return Some(now);
            }
        }
    }
}
