//! Who may own a module's bus name: the check a consumer supplies to [`LiveClient`](super::LiveClient).
//!
//! A well-known name on the session bus goes to whoever asks first, so a client that trusts the
//! name sends its typed secrets to a squatter. The client therefore asks an [`OwnerCheck`] about
//! the current owner before it reads or sets anything. This crate knows nothing of how a program is
//! recognised: the consumer supplies that (the desktop answers from its caller table), and a test
//! supplies [`AnyOwner`].

use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use super::{SenderName, Verdict};

/// What an [`OwnerCheck`] answers, boxed so a client holds a check without a type parameter.
pub type OwnerAnswer<'a> = Pin<Box<dyn Future<Output = Verdict> + Send + 'a>>;

/// Whether the owner of a module's bus name is the program the module belongs to.
///
/// Asked before the first `Snapshot`, `Get`, `Describe` or `Set`, and again each time the name's
/// owner is a different connection than the last one that passed. An implementation that cannot
/// tell (the bus did not answer, the process is gone) answers [`Verdict::Refuse`].
pub trait OwnerCheck: Debug + Send + Sync + 'static {
    /// Whether `owner`, the unique name that owns the well-known `name`, holds the role of the
    /// module served under `name`.
    fn holds_role<'a>(&'a self, name: &'a str, owner: &'a SenderName) -> OwnerAnswer<'a>;
}

/// A check that lets every owner pass. For tests and scenarios only (feature `testing`).
#[cfg(feature = "testing")]
#[derive(Debug, Clone, Copy, Default)]
pub struct AnyOwner;

#[cfg(feature = "testing")]
impl OwnerCheck for AnyOwner {
    fn holds_role<'a>(&'a self, _name: &'a str, _owner: &'a SenderName) -> OwnerAnswer<'a> {
        Box::pin(async { Verdict::Allow })
    }
}
