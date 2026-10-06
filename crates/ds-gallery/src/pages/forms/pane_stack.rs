//! A two-level settings pane: a Form of accounts with a chevron on every row; choosing one pushes
//! that account's detail Form into the same pane under a header that names the page to go back
//! to (System Settings > Internet Accounts).

use dioxus::prelude::*;
use ds::components::fields::field_row::FieldRow;
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds::root::common::Common;
use ds::style::tokens::hex::Hex;

/// The pages of the specimen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Page {
    /// The list of accounts.
    Accounts,
    /// One account's detail.
    Account(Account),
}

/// An account the specimen lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Account {
    name: &'static str,
    icon: Icon,
    colour: Hex,
}

const ACCOUNTS: [Account; 3] = [
    Account {
        name: "iCloud",
        icon: Icon::Globe,
        colour: Hex([0x0a, 0x84, 0xff]),
    },
    Account {
        name: "Gmail",
        icon: Icon::Mail,
        colour: Hex([0xff, 0x3b, 0x30]),
    },
    Account {
        name: "Fastmail",
        icon: Icon::Mail,
        colour: Hex([0x30, 0xb0, 0x50]),
    },
];

fn title(page: Page) -> String {
    match page {
        Page::Accounts => "Internet Accounts".to_owned(),
        Page::Account(account) => account.name.to_owned(),
    }
}

/// The element id of the row that opens `account` in the stack named `stack`.
fn row_id(stack: &str, account: Account) -> String {
    format!("{stack}-open-{}", account.name.to_lowercase())
}

/// The accounts, each a chevron row that `open`s it.
fn accounts(stack: String, open: EventHandler<Account>) -> Element {
    let items = ACCOUNTS
        .into_iter()
        .map(|account| {
            let id = row_id(&stack, account);
            ListItem::row(
                account.name,
                account.name,
                rsx! {
                    Row {
                        title: account.name,
                        size: RowSize::Settings,
                        leading: RowLeading::Tile(TileFace::Glyph(account.icon, account.colour)),
                        accessory: Accessory::Chevron,
                        common: Common { id: Some(id), ..Common::default() },
                        onclick: move |_| open.call(account),
                    }
                },
            )
        })
        .collect();
    rsx! {
        Form {
            FormSection { footer: "Choose an account to change what it syncs.",
                List::<&'static str> {
                    label: "Accounts",
                    style: ListStyle::Grouped,
                    items,
                    onpick: move |name: &'static str| {
                        if let Some(account) = ACCOUNTS.into_iter().find(|a| a.name == name) {
                            open.call(account);
                        }
                    },
                }
            }
        }
    }
}

/// One account's detail.
fn detail(account: Account) -> Element {
    rsx! {
        Form {
            FormSection { title: "Use with",
                FieldRow { label: TextLine::from("Mail"),
                    Toggle { label: "Mail", value: Check::On, onchange: |_| {} }
                }
                FieldRow { label: TextLine::from("Contacts"),
                    Toggle { label: "Contacts", value: Check::On, onchange: |_| {} }
                }
                FieldRow { label: TextLine::from("Calendars"),
                    Toggle { label: "Calendars", value: Check::Off, onchange: |_| {} }
                }
            }
            FormSection { footer: "{account.name} stays signed in until it is removed.",
                FieldRow { label: TextLine::from("Notes"),
                    Toggle { label: "Notes", value: Check::Off, onchange: |_| {} }
                }
            }
        }
    }
}

/// The stack named `id`, opening on `start` (its root, or an account pushed on it).
#[component]
pub fn AccountsStack(id: String, start: Option<Account>) -> Element {
    let mut path = use_signal(|| match start {
        Some(account) => PanePath::new(Page::Accounts).pushed(Page::Account(account)),
        None => PanePath::new(Page::Accounts),
    });
    let stack = id.clone();
    let page = Callback::new(move |page: Page| match page {
        Page::Accounts => accounts(
            stack.clone(),
            EventHandler::new(move |account| {
                let next = path.peek().pushed(Page::Account(account));
                path.set(next);
            }),
        ),
        Page::Account(account) => detail(account),
    });
    let stack = id.clone();
    rsx! {
        PaneStack::<Page> {
            path: path(),
            title: Callback::new(title),
            page,
            opener: Callback::new(move |page: Page| match page {
                Page::Accounts => None,
                Page::Account(account) => Some(row_id(&stack, account)),
            }),
            on_back: move |()| {
                let next = path.peek().popped();
                path.set(next);
            },
            common: Common { id: Some(id), ..Common::default() },
        }
    }
}

/// The two specimens: the root, and an account pushed.
pub(crate) fn specimens() -> Element {
    rsx! {
        div { class: "g-row g-row-top",
            div { class: "g-stage-pad", style: "width:420px;background:var(--surface-2)",
                AccountsStack { id: "pane-root", start: None }
            }
            div { class: "g-stage-pad", style: "width:420px;background:var(--surface-2)",
                AccountsStack { id: "pane-pushed", start: Some(ACCOUNTS[1]) }
            }
        }
    }
}
