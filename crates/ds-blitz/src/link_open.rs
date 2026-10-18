//! A top-level link the document navigates to: the desktop's browser (or mail client) opens it.
//!
//! Opening runs on a worker thread. Handing a link to the desktop can wait on a handler that never
//! returns (`xdg-open` on a hung browser), and on the UI thread that is a frozen window. A link
//! that cannot be opened is reported to the app ([`LinkFailureHandler`]) instead of dropped, so it
//! can say so. A `mailto:` link loses any `attach` or `attachment` field: it is a way to make a
//! mail client attach a file from the person's disk, not something a link in a message should be
//! able to ask for.

use blitz_traits::navigation::{NavigationOptions, NavigationProvider};
use blitz_traits::net::Method;
use std::io;
use std::sync::Arc;
use std::thread;

/// How many links may be on their way to the desktop at once. A person clicks a few; more means a
/// handler is stuck, and another thread each click would only add to it.
const MAX_OPENING: usize = 4;

/// A clicked link that could not be opened.
#[derive(Debug, thiserror::Error)]
#[error("the {scheme} link could not be opened: {source}")]
#[non_exhaustive]
pub struct LinkOpenError {
    /// The link's scheme (`https`, `mailto`).
    pub scheme: String,
    /// Why: no browser set up, a handler that failed, too many links still opening.
    #[source]
    pub source: io::Error,
}

/// What the app does when a clicked link cannot be opened (default: write it to stderr). It is
/// called on the worker that tried, not on the UI thread.
#[derive(Clone)]
pub struct LinkFailureHandler(Arc<dyn Fn(LinkOpenError) + Send + Sync>);

impl LinkFailureHandler {
    /// A handler that calls `handler` for each link that could not be opened.
    pub fn new(handler: impl Fn(LinkOpenError) + Send + Sync + 'static) -> Self {
        LinkFailureHandler(Arc::new(handler))
    }

    fn report(&self, error: LinkOpenError) {
        (self.0)(error);
    }
}

impl Default for LinkFailureHandler {
    fn default() -> Self {
        LinkFailureHandler::new(|error| eprintln!("ds-blitz: {error}"))
    }
}

impl std::fmt::Debug for LinkFailureHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LinkFailureHandler")
    }
}

/// Hands a link to the desktop; `webbrowser::open` in the app, a stand-in in a test.
type Open = fn(&str) -> io::Result<()>;

/// The window's navigation provider.
pub(crate) struct LinkOpener {
    failed: LinkFailureHandler,
    open: Open,
    /// One strong reference here, one more in each worker: how many links are still being opened.
    opening: Arc<()>,
}

impl LinkOpener {
    pub(crate) fn new(failed: LinkFailureHandler) -> LinkOpener {
        LinkOpener::with(failed, webbrowser::open)
    }

    fn with(failed: LinkFailureHandler, open: Open) -> LinkOpener {
        LinkOpener {
            failed,
            open,
            opening: Arc::new(()),
        }
    }

    /// Open `href` (a link of `scheme`) on a worker, or report why it was not started.
    fn start(&self, scheme: &str, href: &str) {
        let scheme = scheme.to_owned();
        let href = if scheme == "mailto" {
            without_attachments(href)
        } else {
            href.to_owned()
        };
        let failed = self.failed.clone();
        if Arc::strong_count(&self.opening) > MAX_OPENING {
            let source = io::Error::new(io::ErrorKind::WouldBlock, "other links are still opening");
            return failed.report(LinkOpenError { scheme, source });
        }
        let (open, claim) = (self.open, Arc::clone(&self.opening));
        let worker = {
            let (scheme, failed) = (scheme.clone(), failed.clone());
            thread::Builder::new()
                .name("quire-open-link".to_owned())
                .spawn(move || {
                    let opened = open(&href);
                    drop(claim);
                    if let Err(source) = opened {
                        failed.report(LinkOpenError { scheme, source });
                    }
                })
        };
        if let Err(source) = worker {
            failed.report(LinkOpenError { scheme, source });
        }
    }
}

impl NavigationProvider for LinkOpener {
    fn navigate_to(&self, options: NavigationOptions) {
        let scheme = options.url.scheme();
        if opens(&options.method, scheme) {
            self.start(scheme, options.url.as_str());
        }
    }
}

/// Whether a navigation is one the desktop opens: a GET of a web or mail link.
fn opens(method: &Method, scheme: &str) -> bool {
    *method == Method::GET && matches!(scheme, "http" | "https" | "mailto")
}

/// `href` without any `attach` or `attachment` field of its query (the field's name matched
/// without regard to case or percent-encoding). The rest, and the fragment, are untouched.
fn without_attachments(href: &str) -> String {
    let Some((head, rest)) = href.split_once('?') else {
        return href.to_owned();
    };
    let (query, fragment) = match rest.split_once('#') {
        Some((query, fragment)) => (query, Some(fragment)),
        None => (rest, None),
    };
    let kept: Vec<&str> = query
        .split('&')
        .filter(|field| !is_attachment(field))
        .collect();
    let mut opened = head.to_owned();
    if !kept.is_empty() {
        opened.push('?');
        opened.push_str(&kept.join("&"));
    }
    if let Some(fragment) = fragment {
        opened.push('#');
        opened.push_str(fragment);
    }
    opened
}

/// Whether a `name=value` query field is named `attach` or `attachment`.
fn is_attachment(field: &str) -> bool {
    let name = field.split('=').next().unwrap_or_default();
    matches!(
        decoded(name).to_ascii_lowercase().as_str(),
        "attach" | "attachment"
    )
}

/// `text` with its `%xx` escapes undone (bytes that are not valid UTF-8 become U+FFFD).
fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let hex = |at: usize| {
        bytes
            .get(at)
            .and_then(|byte| char::from(*byte).to_digit(16))
    };
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let escaped = match (bytes[at], hex(at + 1), hex(at + 2)) {
            (b'%', Some(high), Some(low)) => u8::try_from(high * 16 + low).ok(),
            _ => None,
        };
        if let Some(byte) = escaped {
            out.push(byte);
            at += 3;
        } else {
            out.push(bytes[at]);
            at += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::{
        LinkFailureHandler, LinkOpenError, LinkOpener, Method, opens, without_attachments,
    };
    use std::io;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[test]
    fn a_mailto_link_loses_its_attach_fields_and_nothing_else() {
        let table = [
            ("mailto:a@b.c", "mailto:a@b.c"),
            (
                "mailto:a@b.c?subject=Hi&attach=/home/me/.ssh/id_ed25519",
                "mailto:a@b.c?subject=Hi",
            ),
            ("mailto:a@b.c?attach=/x", "mailto:a@b.c"),
            ("mailto:a@b.c?Attachment=/x&body=hi", "mailto:a@b.c?body=hi"),
            ("mailto:a@b.c?att%61ch=/x", "mailto:a@b.c"),
            ("mailto:a@b.c?subject=attach", "mailto:a@b.c?subject=attach"),
            ("mailto:a@b.c?attach=/x#f", "mailto:a@b.c#f"),
            (
                "mailto:a@b.c?cc=d%40e.f&body=a%2Fb",
                "mailto:a@b.c?cc=d%40e.f&body=a%2Fb",
            ),
        ];
        for (given, want) in table {
            assert_eq!(without_attachments(given), want, "{given}");
        }
    }

    #[test]
    fn only_a_get_of_a_web_or_mail_link_is_opened() {
        let table = [
            (Method::GET, "https", true),
            (Method::GET, "http", true),
            (Method::GET, "mailto", true),
            (Method::POST, "https", false),
            (Method::GET, "file", false),
            (Method::GET, "ssh", false),
        ];
        for (method, scheme, want) in table {
            assert_eq!(opens(&method, scheme), want, "{method} {scheme}");
        }
    }

    fn collecting() -> (LinkFailureHandler, mpsc::Receiver<LinkOpenError>) {
        let (send, receive) = mpsc::channel();
        let send = std::sync::Mutex::new(send);
        let handler = LinkFailureHandler::new(move |error| {
            if let Ok(send) = send.lock() {
                let _ = send.send(error);
            }
        });
        (handler, receive)
    }

    #[test]
    fn a_link_that_cannot_be_opened_is_reported_with_its_scheme() {
        let (failed, reports) = collecting();
        let opener = LinkOpener::with(failed, |_| {
            Err(io::Error::new(io::ErrorKind::NotFound, "no browser"))
        });
        opener.start("https", "https://example.org/");
        let report = reports
            .recv_timeout(Duration::from_secs(5))
            .expect("the failure is reported");
        assert_eq!(report.scheme, "https");
        assert_eq!(report.source.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn a_handler_that_never_returns_does_not_hold_the_caller() {
        let (failed, reports) = collecting();
        let opener = LinkOpener::with(failed, |_| {
            loop {
                std::thread::park();
            }
        });
        let started = Instant::now();
        opener.start("https", "https://example.org/");
        assert!(started.elapsed() < Duration::from_secs(1));
        assert!(reports.recv_timeout(Duration::from_millis(100)).is_err());
    }
}
