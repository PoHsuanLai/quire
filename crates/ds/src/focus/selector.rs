//! Focusing an element named by a CSS selector (mailo gaps, G8): an app's panel opens and its
//! field, or the window's own `.app`, should have the keyboard, and the caller holds no mounted
//! handle for it. The host finds the element in its document ([`HostFind`], ds-native's), the
//! focus and the select-all go through the same `HostFocus`/`HostSelect` writes a field's own
//! focus does, and a `TextInput` found this way hears its `onfocus` once, as through
//! `Focus::OnMount`.

use crate::focus::host::{Focused, focus_selecting};
use crate::focus::select::Select;
use crate::focus::targets::FocusTargets;
use crate::time::{FRAME_SLACK, sleep};
use dioxus::prelude::*;
use std::rc::Rc;

/// Frames a selector waits for its element to be drawn: a field asked for in the handler that
/// opens its panel mounts a frame or two later (mailo's webview scripts waited twenty).
const FIND_FRAMES: usize = 20;

/// One attempt at finding an element.
#[derive(Clone)]
pub enum Found {
    /// The first element that matches, as a handle the host's focus writes accept.
    Element(Rc<MountedData>),
    /// Nothing matches (yet).
    Missing,
    /// The document is busy (rendering); try again next frame.
    Busy,
    /// Not a selector the host's document can read.
    BadSelector,
}

/// The host's selector lookup, provided as root context by `ds-native` (`ds_native::launch` and
/// its harness). `same` says whether two handles are the same node, since an element found by
/// selector is a different handle from the one its component mounted with.
#[derive(Clone)]
pub struct HostFind {
    /// The first element in the document matching a selector.
    pub find: Rc<dyn Fn(&str) -> Found>,
    /// Whether two mounted handles name the same node.
    pub same: fn(&MountedData, &MountedData) -> bool,
}

impl std::fmt::Debug for Found {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Found::Element(_) => "Element",
            Found::Missing => "Missing",
            Found::Busy => "Busy",
            Found::BadSelector => "BadSelector",
        })
    }
}

impl std::fmt::Debug for HostFind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostFind").finish_non_exhaustive()
    }
}

/// Why [`focus_by_selector`] did not move the focus.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FocusError {
    /// No host can find elements here: a webview has no [`HostFind`] (its app focuses by
    /// script).
    #[error("no host finds elements by selector here")]
    NoHost,
    /// The host's document cannot parse the selector.
    #[error("{selector:?} is not a selector the host can read")]
    BadSelector {
        /// The selector as given.
        selector: String,
    },
    /// Nothing matched, for as long as an element asked for is waited for.
    #[error("no element matches {selector:?}")]
    NoSuchElement {
        /// The selector as given.
        selector: String,
    },
    /// The document stayed busy.
    #[error("the document stayed busy")]
    Busy,
    /// The host found the element but could not focus it.
    #[error("the element {selector:?} matched cannot take the focus")]
    Refused {
        /// The selector as given.
        selector: String,
    },
}

/// Give the first element matching `selector` the keyboard, then do `select` with its text,
/// waiting for up to twenty frames for it to be drawn and out a busy document. A `TextInput`
/// found this way hears its `onfocus` once.
///
/// Await it from a task of a scope that outlives the ask (the window's shell, when the panel
/// asking is closing): `spawn(async move { let _ = ds::focus_by_selector(".find input",
/// Select::All).await; })`.
pub async fn focus_by_selector(
    selector: impl Into<String>,
    select: Select,
) -> Result<(), FocusError> {
    let selector = selector.into();
    let host = try_consume_context::<HostFind>().ok_or(FocusError::NoHost)?;
    let element = find(&host, &selector).await?;
    let told = try_consume_context::<FocusTargets>()
        .and_then(|targets| targets.told_at(&element, host.same));
    match focus_selecting(&element, select).await {
        Focused::Done => {
            if let Some(told) = told {
                told.focus.call(());
            }
            Ok(())
        }
        Focused::Busy => Err(FocusError::Busy),
        Focused::Unknown => Err(FocusError::Refused { selector }),
    }
}

/// The first element matching `selector`, once it is drawn and the document is free.
async fn find(host: &HostFind, selector: &str) -> Result<Rc<MountedData>, FocusError> {
    let mut last = FocusError::NoSuchElement {
        selector: selector.to_owned(),
    };
    for _ in 0..FIND_FRAMES {
        match (host.find)(selector) {
            Found::Element(element) => return Ok(element),
            Found::BadSelector => {
                return Err(FocusError::BadSelector {
                    selector: selector.to_owned(),
                });
            }
            Found::Busy => last = FocusError::Busy,
            Found::Missing => {
                last = FocusError::NoSuchElement {
                    selector: selector.to_owned(),
                }
            }
        }
        sleep(FRAME_SLACK).await;
    }
    Err(last)
}
