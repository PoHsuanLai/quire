//! PackageKit over the system bus: `org.freedesktop.PackageKit`'s `CreateTransaction`, then on
//! the transaction `SetHints`, `Resolve` and `InstallPackages`, read through the transaction's
//! `Package`, `ErrorCode` and `Finished` signals (the interface XML PackageKit ships in
//! `/usr/share/dbus-1/interfaces`). The crate docs say why this route and not `Modify2`.

use super::transaction::{Budget, Fault, PEER, Report, transact};
use super::{Request, Timeouts};
use crate::outcome::{Missing, Outcome};
use ds_peer::within;
use zbus::Connection;

/// `PK_FILTER_ENUM_NONE` as a bitfield value (`1 << 1`; `UNKNOWN` is bit 0).
const FILTER_NONE: u64 = 1 << 1;
/// `PK_TRANSACTION_FLAG_ENUM_ONLY_TRUSTED` as a bitfield value (`NONE` is bit 0): the polkit
/// action for trusted installs, not the "untrusted" one.
const FLAG_ONLY_TRUSTED: u64 = 1 << 1;
/// `PK_EXIT_ENUM_SUCCESS` and `PK_EXIT_ENUM_CANCELLED`.
const EXIT_SUCCESS: u32 = 1;
const EXIT_CANCELLED: u32 = 3;
/// `PK_ERROR_ENUM_PACKAGE_NOT_FOUND`, `PK_ERROR_ENUM_TRANSACTION_CANCELLED` and
/// `PK_ERROR_ENUM_NOT_AUTHORIZED` (checked against PackageKitGlib-1.0's typelib: 8, 17, 48).
const ERROR_PACKAGE_NOT_FOUND: u32 = 8;
const ERROR_TRANSACTION_CANCELLED: u32 = 17;
const ERROR_NOT_AUTHORIZED: u32 = 48;

/// Where the system's PackageKit is reached.
#[derive(Debug)]
enum Bus {
    /// The system bus, connected at the first request (so building the backend never touches
    /// a bus).
    System,
    /// A connection the caller made (the tests' private bus).
    Given(Connection),
}

/// The PackageKit backend. It never runs a package manager and never uses sudo.
#[derive(Debug)]
pub struct PackageKit {
    bus: Bus,
    timeouts: Timeouts,
}

/// What resolving one candidate name found.
#[derive(Debug, PartialEq, Eq)]
enum Found {
    Installed,
    Available(String),
    Absent,
}

impl PackageKit {
    /// The system's PackageKit, over the system bus.
    pub fn system() -> PackageKit {
        PackageKit {
            bus: Bus::System,
            timeouts: Timeouts::default(),
        }
    }

    /// A PackageKit (or a stand-in) reached over `connection`.
    pub fn on(connection: Connection) -> PackageKit {
        PackageKit {
            bus: Bus::Given(connection),
            timeouts: Timeouts::default(),
        }
    }

    /// This backend with `timeouts` for each wait on PackageKit.
    pub fn with_timeouts(mut self, timeouts: Timeouts) -> PackageKit {
        self.timeouts = timeouts;
        self
    }

    pub(super) async fn install(&self, request: &Request) -> Outcome {
        let connection = match &self.bus {
            Bus::Given(connection) => connection.clone(),
            Bus::System => match within(self.timeouts.step, PEER, Connection::system()).await {
                Ok(Ok(connection)) => connection,
                Ok(Err(_)) => return Outcome::Unsupported(request.missing.clone()),
                Err(silent) => return Outcome::Failed(silent.to_string()),
            },
        };
        match run(&connection, request, self.timeouts).await {
            Ok(outcome) => outcome,
            Err(fault) => classify(&fault, &request.missing),
        }
    }
}

async fn run(
    connection: &Connection,
    request: &Request,
    timeouts: Timeouts,
) -> Result<Outcome, Fault> {
    for name in &request.candidates {
        match resolve(connection, name.as_str(), timeouts).await? {
            Found::Installed => return Ok(Outcome::Installed),
            Found::Available(id) => return install_id(connection, id, timeouts).await,
            Found::Absent => {}
        }
    }
    Ok(Outcome::NotFound(request.missing.clone()))
}

async fn resolve(connection: &Connection, name: &str, timeouts: Timeouts) -> Result<Found, Fault> {
    let budget = Budget {
        quick: timeouts.step,
        method: timeouts.step,
    };
    let report = transact(connection, "Resolve", &(FILTER_NONE, vec![name]), budget).await?;
    resolve_report(&report, name).map_err(|details| zbus::Error::Failure(details).into())
}

/// What a `Resolve` report says about `name`. The package id is `name;version;arch;data`; an
/// installed package's data starts with `installed`. Read that, not the `info` enum. An id for any
/// other name is not an answer to this request and is ignored, so only a package that was asked
/// for can reach `InstallPackages`.
fn resolve_report(report: &Report, name: &str) -> Result<Found, String> {
    match &report.error {
        Some((code, _)) if *code == ERROR_PACKAGE_NOT_FOUND => return Ok(Found::Absent),
        Some((_, details)) => return Err(details.clone()),
        None => {}
    }
    let (installed, available): (Vec<&String>, Vec<&String>) = report
        .packages
        .iter()
        .filter(|id| id.split(';').next() == Some(name))
        .partition(|id| is_installed(id));
    Ok(match (installed.first(), available.first()) {
        (Some(_), _) => Found::Installed,
        (None, Some(id)) => Found::Available((*id).clone()),
        (None, None) => Found::Absent,
    })
}

/// Whether a package id's data field says the package is installed.
fn is_installed(id: &str) -> bool {
    id.rsplit(';')
        .next()
        .is_some_and(|data| data.starts_with("installed"))
}

async fn install_id(
    connection: &Connection,
    id: String,
    timeouts: Timeouts,
) -> Result<Outcome, Fault> {
    let budget = Budget {
        quick: timeouts.step,
        method: timeouts.install,
    };
    let report = transact(
        connection,
        "InstallPackages",
        &(FLAG_ONLY_TRUSTED, vec![id]),
        budget,
    )
    .await?;
    Ok(match (report.exit, report.error) {
        (EXIT_SUCCESS, _) => Outcome::Installed,
        (EXIT_CANCELLED, _)
        | (_, Some((ERROR_TRANSACTION_CANCELLED | ERROR_NOT_AUTHORIZED, _))) => Outcome::Declined,
        (_, Some((_, details))) if !details.is_empty() => Outcome::Failed(details),
        _ => Outcome::Failed("The installation didn’t finish.".to_owned()),
    })
}

/// A failed transaction as an outcome: a PackageKit that did not answer or went away is a failure
/// in plain words, a bus-level failure is [`classify_bus`].
fn classify(fault: &Fault, missing: &Missing) -> Outcome {
    match fault {
        Fault::Bus(error) => classify_bus(error, missing),
        Fault::Silent(_) | Fault::Gone => Outcome::Failed(fault.to_string()),
    }
}

/// A bus-level failure as an outcome: policy refusal is a decline, no PackageKit is
/// unsupported, anything else is a failure with the daemon's words.
fn classify_bus(error: &zbus::Error, missing: &Missing) -> Outcome {
    use zbus::fdo::Error as Fdo;
    match error {
        zbus::Error::MethodError(name, details, _) => {
            let name = name.as_str();
            match name {
                _ if name.ends_with(".RefusedByPolicy") || name.ends_with(".Denied") => {
                    Outcome::Declined
                }
                "org.freedesktop.DBus.Error.AccessDenied" => Outcome::Declined,
                "org.freedesktop.DBus.Error.ServiceUnknown"
                | "org.freedesktop.DBus.Error.NameHasNoOwner" => {
                    Outcome::Unsupported(missing.clone())
                }
                _ => Outcome::Failed(details.clone().unwrap_or_else(|| name.to_owned())),
            }
        }
        zbus::Error::FDO(fdo) => match **fdo {
            Fdo::ServiceUnknown(_) | Fdo::NameHasNoOwner(_) => {
                Outcome::Unsupported(missing.clone())
            }
            Fdo::AccessDenied(_) => Outcome::Declined,
            _ => Outcome::Failed(fdo.to_string()),
        },
        other => Outcome::Failed(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::{Found, Report, resolve_report};

    fn report(ids: &[&str], error: Option<(u32, &str)>) -> Report {
        Report {
            packages: ids.iter().map(|id| (*id).to_owned()).collect(),
            error: error.map(|(code, text)| (code, text.to_owned())),
            exit: 1,
        }
    }

    #[test]
    fn a_resolve_report_reads_as_installed_available_or_absent() {
        let table = [
            (
                "available",
                report(&["mpv;1;x86_64;fedora"], None),
                Ok(Found::Available("mpv;1;x86_64;fedora".into())),
            ),
            (
                "installed",
                report(&["mpv;1;x86_64;installed:fedora"], None),
                Ok(Found::Installed),
            ),
            (
                "both",
                report(&["mpv;2;x86_64;fedora", "mpv;1;x86_64;installed"], None),
                Ok(Found::Installed),
            ),
            ("none", report(&[], None), Ok(Found::Absent)),
            (
                "not found error",
                report(&[], Some((8, "no such package"))),
                Ok(Found::Absent),
            ),
            (
                "other error",
                report(&[], Some((2, "no network"))),
                Err("no network".to_owned()),
            ),
        ];
        for (name, input, want) in table {
            assert_eq!(resolve_report(&input, "mpv"), want, "{name}");
        }
    }

    #[test]
    fn a_package_that_was_not_asked_for_is_never_chosen() {
        let table = [
            (
                "only another name",
                vec!["evil;1;x86_64;repo"],
                Found::Absent,
            ),
            (
                "another name beside the asked one",
                vec!["evil;1;x86_64;repo", "mpv;2;x86_64;repo"],
                Found::Available("mpv;2;x86_64;repo".into()),
            ),
            (
                "a longer name that starts the same",
                vec!["mpv-evil;1;x86_64;repo"],
                Found::Absent,
            ),
            (
                "another name that is installed",
                vec!["other;1;x86_64;installed"],
                Found::Absent,
            ),
        ];
        for (name, ids, want) in table {
            assert_eq!(
                resolve_report(&report(&ids, None), "mpv"),
                Ok(want),
                "{name}"
            );
        }
    }
}
