//! PackageKit over the system bus: `org.freedesktop.PackageKit`'s `CreateTransaction`, then on
//! the transaction `SetHints`, `Resolve` and `InstallPackages`, read through the transaction's
//! `Package`, `ErrorCode` and `Finished` signals (the interface XML PackageKit ships in
//! `/usr/share/dbus-1/interfaces`). The crate docs say why this route and not `Modify2`.

use super::Request;
use crate::outcome::{Missing, Outcome};
use std::pin::Pin;
use zbus::export::futures_core::Stream;
use zbus::message::Type;
use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, MatchRule, Message, MessageStream};

const SERVICE: &str = "org.freedesktop.PackageKit";
const ROOT_PATH: &str = "/org/freedesktop/PackageKit";
const ROOT_INTERFACE: &str = "org.freedesktop.PackageKit";
const TRANSACTION_INTERFACE: &str = "org.freedesktop.PackageKit.Transaction";

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
}

/// What a transaction reported before it finished.
#[derive(Debug, Default)]
struct Report {
    packages: Vec<String>,
    error: Option<(u32, String)>,
    exit: u32,
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
        PackageKit { bus: Bus::System }
    }

    /// A PackageKit (or a stand-in) reached over `connection`.
    pub fn on(connection: Connection) -> PackageKit {
        PackageKit {
            bus: Bus::Given(connection),
        }
    }

    pub(super) async fn install(&self, request: &Request) -> Outcome {
        let connection = match &self.bus {
            Bus::Given(connection) => connection.clone(),
            Bus::System => match Connection::system().await {
                Ok(connection) => connection,
                Err(_) => return Outcome::Unsupported(request.missing.clone()),
            },
        };
        match run(&connection, request).await {
            Ok(outcome) => outcome,
            Err(error) => classify(&error, &request.missing),
        }
    }
}

async fn run(connection: &Connection, request: &Request) -> zbus::Result<Outcome> {
    for name in &request.candidates {
        match resolve(connection, name.as_str()).await? {
            Found::Installed => return Ok(Outcome::Installed),
            Found::Available(id) => return install_id(connection, id).await,
            Found::Absent => {}
        }
    }
    Ok(Outcome::NotFound(request.missing.clone()))
}

async fn resolve(connection: &Connection, name: &str) -> zbus::Result<Found> {
    let report = transact(connection, "Resolve", &(FILTER_NONE, vec![name])).await?;
    resolve_report(&report).map_err(zbus::Error::Failure)
}

/// What a `Resolve` report says. The package id is `name;version;arch;data`; an installed
/// package's data starts with `installed`. Read that, not the `info` enum.
fn resolve_report(report: &Report) -> Result<Found, String> {
    match &report.error {
        Some((code, _)) if *code == ERROR_PACKAGE_NOT_FOUND => return Ok(Found::Absent),
        Some((_, details)) => return Err(details.clone()),
        None => {}
    }
    let installed = |id: &String| {
        id.rsplit(';')
            .next()
            .is_some_and(|d| d.starts_with("installed"))
    };
    Ok(match report.packages.iter().find(|id| !installed(id)) {
        _ if report.packages.iter().any(installed) => Found::Installed,
        Some(id) => Found::Available(id.clone()),
        None => Found::Absent,
    })
}

async fn install_id(connection: &Connection, id: String) -> zbus::Result<Outcome> {
    let report = transact(
        connection,
        "InstallPackages",
        &(FLAG_ONLY_TRUSTED, vec![id]),
    )
    .await?;
    Ok(match (report.exit, report.error) {
        (EXIT_SUCCESS, _) => Outcome::Installed,
        (EXIT_CANCELLED, _)
        | (_, Some((ERROR_TRANSACTION_CANCELLED | ERROR_NOT_AUTHORIZED, _))) => Outcome::Declined,
        (_, Some((_, details))) if !details.is_empty() => Outcome::Failed(details),
        _ => Outcome::Failed("The package manager did not finish the installation.".to_owned()),
    })
}

/// One transaction: create it, subscribe to its signals before calling `method` (so none is
/// missed), then read until `Finished`. Polkit is asked inside the method call, because the
/// transaction is told the caller is interactive.
async fn transact<B>(connection: &Connection, method: &str, body: &B) -> zbus::Result<Report>
where
    B: serde::Serialize + zbus::zvariant::DynamicType,
{
    let created = connection
        .call_method(
            Some(SERVICE),
            ROOT_PATH,
            Some(ROOT_INTERFACE),
            "CreateTransaction",
            &(),
        )
        .await?;
    let path: OwnedObjectPath = created.body().deserialize()?;
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .path(path.as_str())?
        .interface(TRANSACTION_INTERFACE)?
        .build();
    let mut signals = MessageStream::for_match_rule(rule, connection, None).await?;
    let path = path.as_str();
    let hints = (vec!["interactive=true"],);
    connection
        .call_method(
            Some(SERVICE),
            path,
            Some(TRANSACTION_INTERFACE),
            "SetHints",
            &hints,
        )
        .await?;
    connection
        .call_method(
            Some(SERVICE),
            path,
            Some(TRANSACTION_INTERFACE),
            method,
            body,
        )
        .await?;
    read_until_finished(&mut signals).await
}

async fn read_until_finished(signals: &mut MessageStream) -> zbus::Result<Report> {
    let mut report = Report::default();
    loop {
        let next = std::future::poll_fn(|cx| Pin::new(&mut *signals).poll_next(cx)).await;
        let message = next.ok_or_else(|| zbus::Error::Failure("the bus closed".to_owned()))??;
        if apply(&mut report, &message)? == Step::Done {
            return Ok(report);
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Step {
    More,
    Done,
}

fn apply(report: &mut Report, message: &Message) -> zbus::Result<Step> {
    let header = message.header();
    let member = header
        .member()
        .map(|m| m.as_str().to_owned())
        .unwrap_or_default();
    let body = message.body();
    match member.as_str() {
        "Package" => {
            let (_info, id, _summary): (u32, String, String) = body.deserialize()?;
            report.packages.push(id);
        }
        "ErrorCode" => report.error = Some(body.deserialize()?),
        "Finished" => {
            let (exit, _runtime): (u32, u32) = body.deserialize()?;
            report.exit = exit;
            return Ok(Step::Done);
        }
        _ => {}
    }
    Ok(Step::More)
}

/// A bus-level failure as an outcome: policy refusal is a decline, no PackageKit is
/// unsupported, anything else is a failure with the daemon's words.
fn classify(error: &zbus::Error, missing: &Missing) -> Outcome {
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
            assert_eq!(resolve_report(&input), want, "{name}");
        }
    }
}
