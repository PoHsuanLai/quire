//! Who may drive the menu: the policy a consumer supplies, and the caller it is asked about.
//!
//! The session bus lets any program send `Event` to `com.canonical.dbusmenu`, and an event runs the
//! app's command. Before one does, the export asks a [`MenuPermit`] about the sender. The
//! desktop's answer is `ShellOnly` (feature `peephole`): only the shell, proved
//! by its image and unit, may press an item. There is no default: `MenuExport::start` takes the
//! permit as an argument, so an app cannot export a menu anyone may press by leaving it out.

use std::future::Future;

use zbus::fdo;
use zbus::message::Header;

/// What a refused caller is told: the same words whatever the reason, so a program learns nothing
/// about which check it failed.
const DENIED: &str = "this caller may not use the menu";

/// The unique bus name of the peer that made a call (`:1.42`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SenderName(pub String);

/// Who is calling: the sender's unique bus name and the connection that received the call, so the
/// permit can resolve the peer its own way.
#[derive(Debug, Clone)]
pub struct Caller {
    pub sender: SenderName,
    pub connection: zbus::Connection,
}

/// The answer to [`MenuPermit::permit`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    /// The caller may not press items. The call fails with `AccessDenied`.
    Refuse,
}

/// The policy for who may press the exported menu's items. Asked before every `Event` and
/// `EventGroup`; a caller whose message names no sender is refused without asking.
pub trait MenuPermit: Send + Sync + 'static {
    /// Whether `caller` may press menu items. An implementation that cannot tell (the bus did not
    /// answer, the process is gone) answers [`Verdict::Refuse`].
    fn permit(&self, caller: &Caller) -> impl Future<Output = Verdict> + Send;
}

/// A permit that lets every caller press every item. For tests and scenarios only (feature
/// `testing`).
#[cfg(feature = "testing")]
#[derive(Debug, Clone, Copy, Default)]
pub struct AllowAnyCaller;

#[cfg(feature = "testing")]
impl MenuPermit for AllowAnyCaller {
    async fn permit(&self, _caller: &Caller) -> Verdict {
        Verdict::Allow
    }
}

/// The refusal a method returns.
pub(super) fn denied() -> fdo::Error {
    fdo::Error::AccessDenied(DENIED.to_owned())
}

/// The sender the message names, or the refusal for a message that names none (a peer-to-peer
/// message: nothing says who sent it).
pub(super) fn named(header: &Header<'_>) -> fdo::Result<SenderName> {
    header
        .sender()
        .map(|name| SenderName(name.to_string()))
        .ok_or_else(denied)
}

#[cfg(test)]
mod tests {
    use super::named;

    #[test]
    fn a_message_without_a_sender_is_refused() {
        let message = zbus::Message::method_call("/", "Event")
            .and_then(|call| call.build(&()))
            .expect("a message with no sender");
        assert!(matches!(
            named(&message.header()),
            Err(zbus::fdo::Error::AccessDenied(_))
        ));
    }
}
