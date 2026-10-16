//! One PackageKit transaction on the system bus: create it, subscribe to its signals before the
//! method is called (so none is missed), then read until `Finished`.
//!
//! Every wait has a deadline ([`Timeouts`](super::Timeouts)), and the transaction is believed
//! only when the signals come from the PackageKit that created it: the match rule names its
//! unique name and `apply` checks the sender again, so another process on the bus cannot forge a
//! `Package` or a `Finished`. A PackageKit that vanishes mid-transaction ends the wait at once,
//! and one that goes quiet is asked to cancel.

use ds_peer::{Deadline, NotAnswering, within};
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::task::Poll;
use zbus::export::futures_core::Stream;
use zbus::message::Type;
use zbus::names::UniqueName;
use zbus::zvariant::OwnedObjectPath;
use zbus::{Connection, MatchRule, Message, MessageStream};

const SERVICE: &str = "org.freedesktop.PackageKit";
const ROOT_PATH: &str = "/org/freedesktop/PackageKit";
const ROOT_INTERFACE: &str = "org.freedesktop.PackageKit";
const TRANSACTION_INTERFACE: &str = "org.freedesktop.PackageKit.Transaction";
const BUS_SERVICE: &str = "org.freedesktop.DBus";
/// Who a screen says is not answering.
pub(super) const PEER: &str = "The package manager";

/// What a transaction reported before it finished.
#[derive(Debug, Default)]
pub(super) struct Report {
    pub(super) packages: Vec<String>,
    pub(super) error: Option<(u32, String)>,
    pub(super) exit: u32,
}

/// Why a transaction did not finish.
#[derive(Debug, thiserror::Error)]
pub(super) enum Fault {
    #[error(transparent)]
    Bus(#[from] zbus::Error),
    #[error(transparent)]
    Silent(#[from] NotAnswering),
    #[error("The package manager stopped before it finished.")]
    Gone,
}

/// How long each part of one transaction may take: the quick calls around it, and the method
/// with the wait for `Finished`.
#[derive(Debug, Clone, Copy)]
pub(super) struct Budget {
    pub(super) quick: Deadline,
    pub(super) method: Deadline,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Step {
    More,
    Done,
}

/// A call to PackageKit under `deadline`.
async fn call<T>(
    deadline: Deadline,
    work: impl Future<Output = zbus::Result<T>>,
) -> Result<T, Fault> {
    Ok(within(deadline, PEER, work).await??)
}

pub(super) async fn transact<B>(
    connection: &Connection,
    method: &str,
    body: &B,
    budget: Budget,
) -> Result<Report, Fault>
where
    B: serde::Serialize + zbus::zvariant::DynamicType,
{
    let created = call(
        budget.quick,
        connection.call_method(
            Some(SERVICE),
            ROOT_PATH,
            Some(ROOT_INTERFACE),
            "CreateTransaction",
            &(),
        ),
    )
    .await?;
    let path: OwnedObjectPath = created.body().deserialize()?;
    let owner = created
        .header()
        .sender()
        .map(UniqueName::to_owned)
        .ok_or(Fault::Gone)?;
    let rule = MatchRule::builder()
        .msg_type(Type::Signal)
        .sender(owner.clone())?
        .path(path.as_str())?
        .interface(TRANSACTION_INTERFACE)?
        .build();
    let mut signals = MessageStream::for_match_rule(rule, connection, None).await?;
    let mut gone = MessageStream::for_match_rule(vanished_rule(&owner)?, connection, None).await?;
    let path = path.as_str();
    let hints = (vec!["interactive=true"],);
    call(
        budget.quick,
        transaction_call(connection, path, "SetHints", &hints),
    )
    .await?;
    let finished = within(budget.method, PEER, async {
        transaction_call(connection, path, method, body).await?;
        read_until_finished(&mut signals, &mut gone, &owner).await
    })
    .await;
    match finished {
        Ok(read) => read,
        Err(silent) => {
            cancel(connection, path, budget.quick).await;
            Err(silent.into())
        }
    }
}

async fn transaction_call<B>(
    connection: &Connection,
    path: &str,
    method: &str,
    body: &B,
) -> zbus::Result<Message>
where
    B: serde::Serialize + zbus::zvariant::DynamicType,
{
    connection
        .call_method(
            Some(SERVICE),
            path,
            Some(TRANSACTION_INTERFACE),
            method,
            body,
        )
        .await
}

/// Ask PackageKit to drop a transaction that went quiet. It may be gone, so the answer does not
/// matter.
async fn cancel(connection: &Connection, path: &str, deadline: Deadline) {
    let _ = within(
        deadline,
        PEER,
        transaction_call(connection, path, "Cancel", &()),
    )
    .await;
}

/// The bus telling us `owner` (a unique name) has disconnected.
fn vanished_rule<'a>(owner: &'a UniqueName<'a>) -> zbus::Result<MatchRule<'a>> {
    Ok(MatchRule::builder()
        .msg_type(Type::Signal)
        .sender(BUS_SERVICE)?
        .interface(BUS_SERVICE)?
        .member("NameOwnerChanged")?
        .add_arg(owner.as_str())?
        .build())
}

/// What happened next on the bus.
enum Event {
    Signal(Option<zbus::Result<Message>>),
    Gone,
}

async fn next_event(signals: &mut MessageStream, gone: &mut MessageStream) -> Event {
    poll_fn(|cx| {
        if let Poll::Ready(next) = Pin::new(&mut *signals).poll_next(cx) {
            return Poll::Ready(Event::Signal(next));
        }
        match Pin::new(&mut *gone).poll_next(cx) {
            Poll::Ready(_) => Poll::Ready(Event::Gone),
            Poll::Pending => Poll::Pending,
        }
    })
    .await
}

async fn read_until_finished(
    signals: &mut MessageStream,
    gone: &mut MessageStream,
    owner: &UniqueName<'_>,
) -> Result<Report, Fault> {
    let mut report = Report::default();
    loop {
        let message = match next_event(signals, gone).await {
            Event::Gone => return Err(Fault::Gone),
            Event::Signal(None) => {
                return Err(zbus::Error::Failure("the bus closed".to_owned()).into());
            }
            Event::Signal(Some(message)) => message?,
        };
        if apply(&mut report, &message, owner)? == Step::Done {
            return Ok(report);
        }
    }
}

/// Fold one signal into `report`; a signal from anyone but `owner` is not PackageKit's and
/// changes nothing.
pub(super) fn apply(
    report: &mut Report,
    message: &Message,
    owner: &UniqueName<'_>,
) -> zbus::Result<Step> {
    let header = message.header();
    if header.sender() != Some(owner) {
        return Ok(Step::More);
    }
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

#[cfg(test)]
mod tests {
    use super::{Report, Step, apply};
    use zbus::Message;
    use zbus::names::UniqueName;

    const PATH: &str = "/1";
    const INTERFACE: &str = "org.freedesktop.PackageKit.Transaction";

    fn signal<B>(sender: &str, member: &str, body: &B) -> Message
    where
        B: serde::Serialize + zbus::zvariant::DynamicType,
    {
        Message::signal(PATH, INTERFACE, member)
            .and_then(|builder| builder.sender(sender))
            .and_then(|builder| builder.build(body))
            .expect("a signal message")
    }

    #[test]
    fn a_signal_from_anyone_but_the_transactions_owner_changes_nothing() {
        let owner = UniqueName::try_from(":1.10").expect("a unique name");
        let mut report = Report::default();
        let forged = [
            signal(":1.99", "Package", &(2u32, "evil;1;x86_64;repo", "")),
            signal(":1.99", "ErrorCode", &(8u32, "forged")),
            signal(":1.99", "Finished", &(1u32, 1u32)),
        ];
        for message in &forged {
            assert_eq!(apply(&mut report, message, &owner).ok(), Some(Step::More));
        }
        assert!(report.packages.is_empty() && report.error.is_none() && report.exit == 0);

        let real = signal(":1.10", "Package", &(2u32, "mpv;1;x86_64;repo", ""));
        assert_eq!(apply(&mut report, &real, &owner).ok(), Some(Step::More));
        assert_eq!(report.packages, ["mpv;1;x86_64;repo"]);
        let done = signal(":1.10", "Finished", &(1u32, 1u32));
        assert_eq!(apply(&mut report, &done, &owner).ok(), Some(Step::Done));
        assert_eq!(report.exit, 1);
    }
}
