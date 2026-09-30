//! Focusing an element named by a CSS selector: an app's panel opens and its
//! field, or the window's own `.app`, should have the keyboard, and the caller holds no mounted
//! handle for it. The host finds the element in its document
//! ([`GeometryHost::find`](crate::GeometryHost::find)), the focus and the select-all go through the
//! same writes a field's own focus does, and a `TextInput` found this way hears its `onfocus` once, as through
//! `Focus::OnMount`.

use crate::focus::select::Select;
use crate::focus::soon::focus_selecting;
use crate::focus::targets::FocusTargets;
use crate::host::document::use_document_host;
use crate::host::focused::Focused;
use crate::host::found::Found;
use crate::host::parts::GeometryHost;
use dioxus::prelude::*;
use ds_core::time::{FRAME_SLACK, clock::sleep};
use std::rc::Rc;

/// Frames a selector waits for its element to be drawn: a field asked for in the handler that
/// opens its panel mounts a frame or two later.
const FIND_FRAMES: usize = 20;

/// Why [`focus_by_selector`] did not move the focus.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum FocusError {
    /// The host has no document to search by selector.
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
    let host = use_document_host();
    let element = find(host.geometry(), &selector).await?;
    let told = try_consume_context::<FocusTargets>()
        .and_then(|targets| targets.told_at(&element, host.geometry()));
    match focus_selecting(&element, select.into()).await {
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
async fn find(host: &dyn GeometryHost, selector: &str) -> Result<Rc<MountedData>, FocusError> {
    let mut last = FocusError::NoSuchElement {
        selector: selector.to_owned(),
    };
    for _ in 0..FIND_FRAMES {
        match host.find(selector) {
            Found::Element(element) => return Ok(element),
            Found::BadSelector => {
                return Err(FocusError::BadSelector {
                    selector: selector.to_owned(),
                });
            }
            Found::Unreachable => return Err(FocusError::NoHost),
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
