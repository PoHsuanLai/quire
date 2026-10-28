//! Where one request goes, as a pure function of whether a frame asks, the URL's scheme and the
//! app's policy, so the whole table is tested without a document.

use crate::net_policy::NetPolicy;

/// What happens to a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Route {
    /// Decoded in place.
    Data,
    /// Read from the disk.
    File,
    /// Handed to the provider the document had before ds-blitz's.
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

/// Where a request for a `scheme` URL goes under `policy`, from a frame's sub-document
/// (`framed`) or the app's own.
pub(crate) fn route(framed: bool, scheme: &str, policy: Policy) -> Route {
    match (scheme, framed, policy) {
        ("data", _, _) => Route::Data,
        // `about:blank` names an empty document: there is nothing to fetch.
        ("about", _, _) | (_, _, Policy::Sealed) | (_, true, Policy::Local) => Route::Drop,
        (_, true, Policy::Custom) => Route::App,
        ("file", false, _) => Route::File,
        (_, false, Policy::Local) => Route::Fallback,
        (_, false, Policy::Custom) => Route::App,
    }
}

#[cfg(test)]
mod tests {
    use super::{Policy, Route, route};

    type Case = ((bool, &'static str, Policy), Route);

    const CASES: &[Case] = &[
        ((false, "data", Policy::Sealed), Route::Data),
        ((true, "data", Policy::Local), Route::Data),
        ((false, "about", Policy::Local), Route::Drop),
        ((false, "file", Policy::Local), Route::File),
        ((false, "file", Policy::Custom), Route::File),
        ((false, "file", Policy::Sealed), Route::Drop),
        ((true, "file", Policy::Local), Route::Drop),
        ((true, "file", Policy::Sealed), Route::Drop),
        ((true, "file", Policy::Custom), Route::App),
        ((false, "https", Policy::Local), Route::Fallback),
        ((false, "dioxus", Policy::Local), Route::Fallback),
        ((false, "https", Policy::Custom), Route::App),
        ((false, "https", Policy::Sealed), Route::Drop),
        ((true, "https", Policy::Local), Route::Drop),
        ((true, "https", Policy::Custom), Route::App),
        ((true, "cid", Policy::Custom), Route::App),
    ];

    #[test]
    fn requests_go_where_the_table_says() {
        for &((framed, scheme, policy), want) in CASES {
            assert_eq!(
                route(framed, scheme, policy),
                want,
                "framed {framed} {scheme} {policy:?}"
            );
        }
    }
}
