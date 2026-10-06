//! A stand-in for PackageKit's system service, claimed on the private bus under the real name.

use std::sync::{Arc, Mutex};
use zbus::object_server::SignalEmitter;
use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, ObjectServer, interface};

const NAME: &str = "org.freedesktop.PackageKit";

/// What the fake does.
#[derive(Debug, Clone, Default)]
pub struct Behaviour {
    available: Vec<String>,
    installed: Vec<String>,
    refuse_policy: bool,
    cancel_transaction: bool,
    resolve_error: Option<(u32, String)>,
    install_error: Option<(u32, String)>,
}

impl Behaviour {
    pub fn available(mut self, names: &[&str]) -> Self {
        self.available = names.iter().map(|n| (*n).to_owned()).collect();
        self
    }
    pub fn already(mut self, names: &[&str]) -> Self {
        self.installed = names.iter().map(|n| (*n).to_owned()).collect();
        self
    }
    /// Polkit says no: the InstallPackages call fails with RefusedByPolicy.
    pub fn refuse_policy(mut self) -> Self {
        self.refuse_policy = true;
        self
    }
    /// The transaction finishes `cancelled`.
    pub fn cancel_transaction(mut self) -> Self {
        self.cancel_transaction = true;
        self
    }
    pub fn resolve_errors(mut self, code: u32, text: &str) -> Self {
        self.resolve_error = Some((code, text.to_owned()));
        self
    }
    pub fn fail_install(self, text: &str) -> Self {
        self.install_errors(2, text)
    }
    /// InstallPackages reports `ErrorCode(code, text)` and finishes `failed`.
    pub fn install_errors(mut self, code: u32, text: &str) -> Self {
        self.install_error = Some((code, text.to_owned()));
        self
    }
}

#[derive(Debug, Default)]
struct Record {
    resolved: Vec<String>,
    installed: Vec<String>,
    hints: Vec<String>,
    flags: Vec<u64>,
    transactions: u32,
}

/// What the fake was asked.
#[derive(Debug, Clone, Default)]
pub struct Calls(Arc<Mutex<Record>>);

impl Calls {
    fn with<T>(&self, f: impl FnOnce(&mut Record) -> T) -> T {
        f(&mut self.0.lock().expect("record"))
    }
    pub fn resolved(&self) -> Vec<String> {
        self.with(|r| r.resolved.clone())
    }
    pub fn installed(&self) -> Vec<String> {
        self.with(|r| r.installed.clone())
    }
    pub fn hints(&self) -> Vec<String> {
        self.with(|r| r.hints.clone())
    }
    pub fn flags(&self) -> Vec<u64> {
        self.with(|r| r.flags.clone())
    }
}

#[derive(Debug, zbus::DBusError)]
#[zbus(prefix = "org.freedesktop.PackageKit.Transaction")]
enum TransactionError {
    #[zbus(error)]
    ZBus(zbus::Error),
    RefusedByPolicy(String),
}

struct Root {
    behaviour: Behaviour,
    calls: Calls,
}

struct Transaction {
    behaviour: Behaviour,
    calls: Calls,
}

#[interface(name = "org.freedesktop.PackageKit")]
impl Root {
    async fn create_transaction(
        &self,
        #[zbus(object_server)] server: &ObjectServer,
    ) -> zbus::fdo::Result<OwnedObjectPath> {
        let n = self.calls.with(|r| {
            r.transactions += 1;
            r.transactions
        });
        let path = format!("/{n}");
        let transaction = Transaction {
            behaviour: self.behaviour.clone(),
            calls: self.calls.clone(),
        };
        server.at(path.as_str(), transaction).await?;
        Ok(OwnedObjectPath::try_from(path).map_err(zbus::Error::from)?)
    }
}

#[interface(name = "org.freedesktop.PackageKit.Transaction")]
impl Transaction {
    fn set_hints(&self, hints: Vec<String>) {
        self.calls.with(|r| r.hints.extend(hints));
    }

    async fn resolve(
        &self,
        _filter: u64,
        packages: Vec<String>,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> zbus::fdo::Result<()> {
        self.calls.with(|r| r.resolved.extend(packages.clone()));
        if let Some((code, text)) = &self.behaviour.resolve_error {
            Self::error_code(&emitter, *code, text).await?;
            Self::finished(&emitter, 2, 1).await?;
            return Ok(());
        }
        for name in &packages {
            if self.behaviour.installed.contains(name) {
                let id = format!("{name};1.0;x86_64;installed:fedora");
                Self::package(&emitter, 1, &id, "").await?;
            } else if self.behaviour.available.contains(name) {
                let id = format!("{name};1.0;x86_64;fedora");
                Self::package(&emitter, 2, &id, "").await?;
            }
        }
        Self::finished(&emitter, 1, 1).await?;
        Ok(())
    }

    async fn install_packages(
        &self,
        flags: u64,
        package_ids: Vec<String>,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> Result<(), TransactionError> {
        if self.behaviour.refuse_policy {
            return Err(TransactionError::RefusedByPolicy(
                "Failed to obtain authentication.".to_owned(),
            ));
        }
        self.calls.with(|r| {
            r.flags.push(flags);
            r.installed.extend(package_ids);
        });
        let to_zbus = |e: zbus::Error| TransactionError::ZBus(e);
        match (
            &self.behaviour.install_error,
            self.behaviour.cancel_transaction,
        ) {
            (Some((code, text)), _) => {
                Self::error_code(&emitter, *code, text)
                    .await
                    .map_err(to_zbus)?;
                Self::finished(&emitter, 2, 1).await.map_err(to_zbus)?;
            }
            (None, true) => Self::finished(&emitter, 3, 1).await.map_err(to_zbus)?,
            (None, false) => Self::finished(&emitter, 1, 1).await.map_err(to_zbus)?,
        }
        Ok(())
    }

    #[zbus(signal)]
    async fn package(
        emitter: &SignalEmitter<'_>,
        info: u32,
        package_id: &str,
        summary: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn error_code(emitter: &SignalEmitter<'_>, code: u32, details: &str) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn finished(emitter: &SignalEmitter<'_>, exit: u32, runtime: u32) -> zbus::Result<()>;
}

/// Claim PackageKit's name on `connection` and serve `behaviour`.
pub async fn serve(connection: &Connection, behaviour: Behaviour) -> Calls {
    let calls = Calls::default();
    let root = Root {
        behaviour,
        calls: calls.clone(),
    };
    connection
        .object_server()
        .at("/org/freedesktop/PackageKit", root)
        .await
        .expect("root served");
    connection.request_name(NAME).await.expect("name claimed");
    calls
}
