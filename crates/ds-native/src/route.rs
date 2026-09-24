//! Where one request goes, as a pure function of the asking document, the URL's scheme and the
//! app's policy, so the whole table is tested without a document.

use crate::net_policy::NetPolicy;

/// Which document a provider serves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Document {
    /// The app's own.
    Top,
    /// A frame's sub-document.
    Frame,
}

/// What happens to a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Route {
    /// Decoded in place.
    Data,
    /// Read from the disk.
    File,
    /// Handed to the provider the document had before ds-native's.
    Fallback,
    /// Put to the app's `AppNet`.
    App,
    /// Never answered.
    Drop,
}

/// The policy's shape, without the app's handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Policy {
    Local,
    Custom,
    Sealed,
}

impl Policy {
    pub(crate) fn of(policy: &NetPolicy) -> Self {
        match policy {
            NetPolicy::Local => Policy::Local,
            NetPolicy::Custom(_) => Policy::Custom,
            NetPolicy::Sealed => Policy::Sealed,
        }
    }
}

/// Where a request for a `scheme` URL from `document` goes under `policy`.
pub(crate) fn route(document: Document, scheme: &str, policy: Policy) -> Route {
    match (scheme, document, policy) {
        ("data", _, _) => Route::Data,
        // `about:blank` names an empty document: there is nothing to fetch.
        ("about", _, _) => Route::Drop,
        (_, _, Policy::Sealed) => Route::Drop,
        (_, Document::Frame, Policy::Local) => Route::Drop,
        (_, Document::Frame, Policy::Custom) => Route::App,
        ("file", Document::Top, _) => Route::File,
        (_, Document::Top, Policy::Local) => Route::Fallback,
        (_, Document::Top, Policy::Custom) => Route::App,
    }
}

#[cfg(test)]
mod tests {
    use super::{Document, Policy, Route, route};

    type Case = ((Document, &'static str, Policy), Route);

    const CASES: &[Case] = &[
        ((Document::Top, "data", Policy::Sealed), Route::Data),
        ((Document::Frame, "data", Policy::Local), Route::Data),
        ((Document::Top, "about", Policy::Local), Route::Drop),
        ((Document::Top, "file", Policy::Local), Route::File),
        ((Document::Top, "file", Policy::Custom), Route::File),
        ((Document::Top, "file", Policy::Sealed), Route::Drop),
        ((Document::Frame, "file", Policy::Local), Route::Drop),
        ((Document::Frame, "file", Policy::Sealed), Route::Drop),
        ((Document::Frame, "file", Policy::Custom), Route::App),
        ((Document::Top, "https", Policy::Local), Route::Fallback),
        ((Document::Top, "dioxus", Policy::Local), Route::Fallback),
        ((Document::Top, "https", Policy::Custom), Route::App),
        ((Document::Top, "https", Policy::Sealed), Route::Drop),
        ((Document::Frame, "https", Policy::Local), Route::Drop),
        ((Document::Frame, "https", Policy::Custom), Route::App),
        ((Document::Frame, "cid", Policy::Custom), Route::App),
    ];

    #[test]
    fn requests_go_where_the_table_says() {
        for &((document, scheme, policy), want) in CASES {
            assert_eq!(
                route(document, scheme, policy),
                want,
                "{document:?} {scheme} {policy:?}"
            );
        }
    }
}
