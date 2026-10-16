//! The desktop portal's print dialog: `PreparePrint` shows it and answers the chosen settings
//! and a token, then `Print` hands over the PDF as a file descriptor with that token. Each call
//! answers on a Request object whose path is known in advance from the `handle_token` option,
//! so the answer is subscribed to before the call is made and cannot be missed.
//!
//! Every wait has a deadline (`ds-peer`): the calls themselves are short, the person's answer to
//! the dialog gets the long prompt bound, and a portal that vanishes while the dialog is up ends
//! the wait at once. The answers are taken only from the portal's own unique name (the proxy
//! filters on it), so another process cannot answer for the dialog.

use super::{PrintError, PrintOutcome};
use ds_peer::{Deadline, NotAnswering, within};
use std::collections::HashMap;
use std::future::poll_fn;
use std::io::{Seek, SeekFrom, Write};
use std::os::fd::OwnedFd;
use std::pin::Pin;
use std::task::Poll;
use zbus::export::futures_core::Stream;
use zbus::proxy::{OwnerChangedStream, SignalStream};
use zbus::zvariant::{Fd, OwnedValue, Value};
use zbus::{Connection, Message, Proxy};

const DESKTOP: &str = "org.freedesktop.portal.Desktop";
const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";
const PRINT: &str = "org.freedesktop.portal.Print";
const REQUEST: &str = "org.freedesktop.portal.Request";
/// No parent window: winit exports no xdg-foreign handle to name one with.
const UNPARENTED: &str = "";
/// Who a screen says is not answering.
const PEER: &str = "The print service";

/// How far the portal got.
pub(super) enum Attempt {
    /// The dialog was shown; this is how it ended.
    Answered(Result<PrintOutcome, PrintError>),
    /// There is no portal, or it cannot print: nothing was shown.
    Unavailable,
}

/// How a portal request ended (its `Response` code).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Response {
    Success,
    Cancelled,
    Ended,
}

impl Response {
    fn of(code: u32) -> Self {
        match code {
            0 => Response::Success,
            1 => Response::Cancelled,
            _ => Response::Ended,
        }
    }
}

/// Print `pdf` through the portal's dialog. Blocks until the person has answered it or a
/// deadline has passed; call it off the UI thread.
pub(super) fn print(pdf: &[u8], title: &str) -> Attempt {
    zbus::block_on(print_async(pdf, title))
}

async fn print_async(pdf: &[u8], title: &str) -> Attempt {
    let Ok(Ok(connection)) = within(Deadline::READ, PEER, Connection::session()).await else {
        return Attempt::Unavailable;
    };
    let prepared = match prepare(&connection, title).await {
        Ok(Some(prepared)) => prepared,
        Ok(None) => return Attempt::Answered(Ok(PrintOutcome::Cancelled)),
        Err(Stage::Before) => return Attempt::Unavailable,
        Err(Stage::After(error)) => return Attempt::Answered(Err(error)),
    };
    Attempt::Answered(submit(&connection, title, pdf, prepared).await)
}

/// Where a portal call failed: before anything was shown (no portal, no print backend, a portal
/// that does not answer: fall back to the viewer), or after, when the person has seen a dialog and
/// must hear why.
enum Stage {
    Before,
    After(PrintError),
}

/// The token `PreparePrint` answered with, which `Print` must carry.
struct Prepared {
    token: u32,
}

/// Show the dialog; the token if the person accepted, `None` if they cancelled.
async fn prepare(connection: &Connection, title: &str) -> Result<Option<Prepared>, Stage> {
    let handle = handle_token("prepare");
    let mut answers = answers(connection, &handle)
        .await
        .map_err(|_| Stage::Before)?;
    let empty: HashMap<&str, Value<'_>> = HashMap::new();
    let options: HashMap<&str, Value<'_>> = HashMap::from([
        ("handle_token", Value::from(handle.as_str())),
        ("modal", Value::from(true)),
    ]);
    let portal = portal(connection).await.map_err(|_| Stage::Before)?;
    let shown = within(
        Deadline::ACTION,
        PEER,
        portal.call_method(
            "PreparePrint",
            &(UNPARENTED, title, &empty, &empty, options),
        ),
    )
    .await;
    if !matches!(shown, Ok(Ok(_))) {
        return Err(Stage::Before);
    }
    let (response, results) = answers
        .next_answer(Deadline::PROMPT)
        .await
        .map_err(Stage::After)?;
    match response {
        Response::Success => {
            let token = results
                .get("token")
                .and_then(|value| u32::try_from(value).ok())
                .ok_or_else(|| Stage::After(PrintError::Portal("no token in the answer".into())))?;
            Ok(Some(Prepared { token }))
        }
        Response::Cancelled => Ok(None),
        Response::Ended => Err(Stage::After(PrintError::Portal(
            "the dialog ended without an answer".into(),
        ))),
    }
}

/// Hand `pdf` to the portal with the dialog's token.
async fn submit(
    connection: &Connection,
    title: &str,
    pdf: &[u8],
    prepared: Prepared,
) -> Result<PrintOutcome, PrintError> {
    let portal_error = |error: zbus::Error| PrintError::Portal(error.to_string());
    let silent = |error: NotAnswering| PrintError::Portal(error.to_string());
    let file = sealed_file(pdf)?;
    let handle = handle_token("print");
    let mut answers = answers(connection, &handle).await.map_err(portal_error)?;
    let options: HashMap<&str, Value<'_>> = HashMap::from([
        ("handle_token", Value::from(handle.as_str())),
        ("token", Value::from(prepared.token)),
        ("modal", Value::from(true)),
    ]);
    let portal = portal(connection).await.map_err(portal_error)?;
    within(
        Deadline::ACTION,
        PEER,
        portal.call_method("Print", &(UNPARENTED, title, Fd::from(&file), options)),
    )
    .await
    .map_err(silent)?
    .map_err(portal_error)?;
    match answers.next_answer(Deadline::ACTION).await?.0 {
        Response::Success => Ok(PrintOutcome::Printed),
        Response::Cancelled => Ok(PrintOutcome::Cancelled),
        Response::Ended => Err(PrintError::Portal("printing ended without success".into())),
    }
}

/// `pdf` in an anonymous in-memory file, rewound and sealed against change: the portal reads it
/// from the start, and nothing can alter what it reads.
fn sealed_file(pdf: &[u8]) -> Result<OwnedFd, PrintError> {
    let memfd = memfd::MemfdOptions::default()
        .allow_sealing(true)
        .create("quire-print")
        .map_err(|error| PrintError::Portal(error.to_string()))?;
    let mut file = memfd.as_file();
    file.write_all(pdf)?;
    file.seek(SeekFrom::Start(0))?;
    memfd
        .add_seals(&[
            memfd::FileSeal::SealShrink,
            memfd::FileSeal::SealGrow,
            memfd::FileSeal::SealWrite,
            memfd::FileSeal::SealSeal,
        ])
        .map_err(|error| PrintError::Portal(error.to_string()))?;
    Ok(OwnedFd::from(memfd.into_file()))
}

async fn portal(connection: &Connection) -> zbus::Result<Proxy<'static>> {
    Proxy::new(connection, DESKTOP, DESKTOP_PATH, PRINT).await
}

/// The answers to the request that `handle` will name, subscribed before the call, and a watch on
/// the portal's name so a portal that goes away is noticed.
async fn answers(connection: &Connection, handle: &str) -> zbus::Result<Answers> {
    let sender = connection
        .unique_name()
        .map(|name| name.as_str().to_owned())
        .unwrap_or_default();
    let request = Proxy::new(connection, DESKTOP, request_path(&sender, handle), REQUEST).await?;
    let signals = request.receive_signal("Response").await?;
    let owner = request.receive_owner_changed().await?;
    Ok(Answers {
        signals: Box::pin(signals),
        owner: Box::pin(owner),
    })
}

/// A request's `Response` signals, and the portal's ownership of its name.
struct Answers {
    signals: Pin<Box<SignalStream<'static>>>,
    owner: Pin<Box<OwnerChangedStream<'static>>>,
}

/// What happened next while waiting for an answer.
enum Heard {
    Signal(Option<Message>),
    PortalGone,
}

impl Answers {
    /// The next answer, its code and results, within `deadline`.
    async fn next_answer(
        &mut self,
        deadline: Deadline,
    ) -> Result<(Response, HashMap<String, OwnedValue>), PrintError> {
        match within(deadline, PEER, self.hear()).await {
            Ok(heard) => heard,
            Err(silent) => Err(PrintError::Portal(silent.to_string())),
        }
    }

    async fn hear(&mut self) -> Result<(Response, HashMap<String, OwnedValue>), PrintError> {
        let heard = poll_fn(|cx| {
            loop {
                if let Poll::Ready(next) = self.signals.as_mut().poll_next(cx) {
                    return Poll::Ready(Heard::Signal(next));
                }
                match self.owner.as_mut().poll_next(cx) {
                    // The portal has, or has again, an owner: keep waiting for its answer.
                    Poll::Ready(Some(Some(_))) => continue,
                    Poll::Ready(Some(None) | None) => return Poll::Ready(Heard::PortalGone),
                    Poll::Pending => return Poll::Pending,
                }
            }
        })
        .await;
        match heard {
            Heard::Signal(Some(message)) => answer_of(&message),
            Heard::Signal(None) | Heard::PortalGone => Err(PrintError::Portal(
                "the print service went away before answering".into(),
            )),
        }
    }
}

/// The code and results in a `Response` signal.
fn answer_of(message: &Message) -> Result<(Response, HashMap<String, OwnedValue>), PrintError> {
    let (code, results): (u32, HashMap<String, OwnedValue>) = message
        .body()
        .deserialize()
        .map_err(|error| PrintError::Portal(error.to_string()))?;
    Ok((Response::of(code), results))
}

/// The Request object a call with `handle_token` = `handle` answers on: the caller's unique
/// name without its colon, dots as underscores (the portal's documented rule).
fn request_path(sender: &str, handle: &str) -> String {
    let sender = sender.trim_start_matches(':').replace('.', "_");
    format!("{DESKTOP_PATH}/request/{sender}/{handle}")
}

/// A handle token unique in this process: letters, digits and underscores only.
fn handle_token(step: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "quire_{step}_{}_{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

#[cfg(test)]
mod tests {
    use super::{Response, handle_token, request_path};

    #[test]
    fn the_request_path_follows_the_portal_rule() {
        assert_eq!(
            request_path(":1.42", "quire_prepare_7_0"),
            "/org/freedesktop/portal/desktop/request/1_42/quire_prepare_7_0"
        );
    }

    #[test]
    fn handle_tokens_are_unique_and_plain() {
        let (a, b) = (handle_token("print"), handle_token("print"));
        assert_ne!(a, b);
        assert!(
            a.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
            "{a}"
        );
    }

    #[test]
    fn response_codes_mean_what_the_portal_says() {
        const CASES: &[(u32, Response)] = &[
            (0, Response::Success),
            (1, Response::Cancelled),
            (2, Response::Ended),
        ];
        for &(code, want) in CASES {
            assert_eq!(Response::of(code), want, "{code}");
        }
    }
}
