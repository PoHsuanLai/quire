//! The desktop portal's print dialog: `PreparePrint` shows it and answers the chosen settings
//! and a token, then `Print` hands over the PDF as a file descriptor with that token. Each call
//! answers on a Request object whose path is known in advance from the `handle_token` option,
//! so the answer is subscribed to before the call is made and cannot be missed.

use super::{PrintError, PrintOutcome};
use std::collections::HashMap;
use std::io::{Seek, SeekFrom, Write};
use std::os::fd::OwnedFd;
use zbus::blocking::{Connection, Proxy};
use zbus::zvariant::{Fd, OwnedValue, Value};

const DESKTOP: &str = "org.freedesktop.portal.Desktop";
const DESKTOP_PATH: &str = "/org/freedesktop/portal/desktop";
const PRINT: &str = "org.freedesktop.portal.Print";
const REQUEST: &str = "org.freedesktop.portal.Request";
/// No parent window: winit exports no xdg-foreign handle to name one with.
const UNPARENTED: &str = "";

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

/// Print `pdf` through the portal's dialog.
pub(super) fn print(pdf: &[u8], title: &str) -> Attempt {
    let Ok(connection) = Connection::session() else {
        return Attempt::Unavailable;
    };
    let prepared = match prepare(&connection, title) {
        Ok(Some(prepared)) => prepared,
        Ok(None) => return Attempt::Answered(Ok(PrintOutcome::Cancelled)),
        Err(Stage::Before) => return Attempt::Unavailable,
        Err(Stage::After(error)) => return Attempt::Answered(Err(error)),
    };
    Attempt::Answered(submit(&connection, title, pdf, prepared))
}

/// Where a portal call failed: before anything was shown (no portal, no print backend: fall
/// back to the viewer), or after, when the person has seen a dialog and must hear why.
enum Stage {
    Before,
    After(PrintError),
}

/// The token `PreparePrint` answered with, which `Print` must carry.
struct Prepared {
    token: u32,
}

/// Show the dialog; the token if the person accepted, `None` if they cancelled.
fn prepare(connection: &Connection, title: &str) -> Result<Option<Prepared>, Stage> {
    let handle = handle_token("prepare");
    let mut answers = answers(connection, &handle).map_err(|_| Stage::Before)?;
    let empty: HashMap<&str, Value<'_>> = HashMap::new();
    let options: HashMap<&str, Value<'_>> = HashMap::from([
        ("handle_token", Value::from(handle.as_str())),
        ("modal", Value::from(true)),
    ]);
    portal(connection)
        .and_then(|print| {
            print.call_method(
                "PreparePrint",
                &(UNPARENTED, title, &empty, &empty, options),
            )
        })
        .map_err(|_| Stage::Before)?;
    let (response, results) = answers.next_answer().map_err(Stage::After)?;
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
fn submit(
    connection: &Connection,
    title: &str,
    pdf: &[u8],
    prepared: Prepared,
) -> Result<PrintOutcome, PrintError> {
    let portal_error = |error: zbus::Error| PrintError::Portal(error.to_string());
    let file = sealed_file(pdf)?;
    let handle = handle_token("print");
    let mut answers = answers(connection, &handle).map_err(portal_error)?;
    let options: HashMap<&str, Value<'_>> = HashMap::from([
        ("handle_token", Value::from(handle.as_str())),
        ("token", Value::from(prepared.token)),
        ("modal", Value::from(true)),
    ]);
    portal(connection)
        .and_then(|print| {
            print.call_method("Print", &(UNPARENTED, title, Fd::from(&file), options))
        })
        .map_err(portal_error)?;
    match answers.next_answer()?.0 {
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

fn portal(connection: &Connection) -> zbus::Result<Proxy<'static>> {
    Proxy::new(connection, DESKTOP, DESKTOP_PATH, PRINT)
}

/// The answers to the request that `handle` will name, subscribed before the call.
fn answers(connection: &Connection, handle: &str) -> zbus::Result<Answers> {
    let sender = connection
        .unique_name()
        .map(|name| name.as_str().to_owned())
        .unwrap_or_default();
    let request = Proxy::new(connection, DESKTOP, request_path(&sender, handle), REQUEST)?;
    let signals = request.receive_signal("Response")?;
    Ok(Answers { signals })
}

/// A request's `Response` signals.
struct Answers {
    signals: zbus::blocking::proxy::SignalIterator<'static>,
}

impl Answers {
    /// The next answer: its code and results.
    fn next_answer(&mut self) -> Result<(Response, HashMap<String, OwnedValue>), PrintError> {
        let message = self
            .signals
            .next()
            .ok_or_else(|| PrintError::Portal("the portal went away before answering".into()))?;
        let (code, results): (u32, HashMap<String, OwnedValue>) = message
            .body()
            .deserialize()
            .map_err(|error| PrintError::Portal(error.to_string()))?;
        Ok((Response::of(code), results))
    }
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
