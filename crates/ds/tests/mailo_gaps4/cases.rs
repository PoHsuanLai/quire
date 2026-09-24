//! The mailo gaps 4 states, as data: the golden each renders to and how to make it.

use dioxus::prelude::*;
use ds::components::vocab::Switch;
use ds::{
    AccountFace, AccountTile, Colour, Hex, MarkSize, MarkStyle, Provider, ProviderMark,
};

/// An account colour.
const SLATE: Colour = Colour::Solid(Hex([0x2f, 0x7f, 0x6e]));

/// A local-folders account: no provider, so the neutral folder mark.
fn local() -> AccountFace {
    AccountFace::One {
        initial: 'L',
        colour: SLATE,
        provider: Provider::Local,
        address: None,
    }
}

/// One state and its golden.
pub struct Case {
    pub golden: &'static str,
    pub make: fn() -> Element,
}

pub const CASES: &[Case] = &[
    Case {
        golden: "lists/account_tile/one-local.html",
        make: || rsx! { AccountTile { account: local(), pressed: Switch::On, unread: 2, onclick: |_| {} } },
    },
    Case {
        golden: "lists/provider_mark/local-row.html",
        make: || rsx! { ProviderMark { provider: Provider::Local, size: MarkSize::Row, style: MarkStyle::Letter } },
    },
    Case {
        golden: "lists/provider_mark/local-image-ignored.html",
        make: || rsx! { ProviderMark { provider: Provider::Local, size: MarkSize::Inline, style: MarkStyle::Image(ds::ImageSource("data:image/png;base64,iVBORw0KGgo=".to_string())) } },
    },
];
