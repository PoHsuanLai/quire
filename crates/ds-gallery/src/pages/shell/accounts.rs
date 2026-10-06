//! The Accounts page: the consent alert and every step of the add-account sheet in light and
//! dark, each in a Sheet root of its own over the wallpaper, and the small parts an app shows
//! (account picker, no-account states, badge, limited notes). The values are the page's own:
//! nothing here talks to an account service.

use crate::axes::Axes;
use crate::pages::Section;
use crate::wallpaper;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::content::provider_mark::MarkProvider;
use ds::prelude::*;
use ds_shell::accounts::model::{
    AccountChoice, Attempt, ChoiceKey, CopyState, FieldProblem, FieldRole, FieldText, FormField,
    Limitation, NoAccountWhy, ProblemKind, ProviderEntry, ProviderKey, Requirement, ServiceKey,
    ServiceLine, ServiceOffer, SignInFault,
};
use ds_shell::prelude::*;

fn account(label: &str, provider: MarkProvider) -> AccountChoice {
    AccountChoice {
        key: ChoiceKey(label.to_owned()),
        label: label.to_owned(),
        provider,
    }
}

fn accounts() -> Vec<AccountChoice> {
    vec![
        account("ada@example.org", MarkProvider::Fastmail),
        account("ada@work.example", MarkProvider::Microsoft),
    ]
}

fn providers() -> Vec<ProviderEntry> {
    [
        ("nextcloud", "Nextcloud", MarkProvider::Imap),
        ("fastmail", "Fastmail", MarkProvider::Fastmail),
        ("google", "Google", MarkProvider::Google),
        ("microsoft", "Microsoft 365", MarkProvider::Microsoft),
    ]
    .map(|(key, label, mark)| ProviderEntry {
        key: ProviderKey(key.to_owned()),
        label: label.to_owned(),
        mark,
    })
    .to_vec()
}

fn field(role: FieldRole, text: FieldText) -> FormField {
    FormField {
        role,
        requirement: Requirement::Required,
        text,
    }
}

fn service(key: &str, name: &str, offer: ServiceOffer, limit: Option<Limitation>) -> ServiceLine {
    ServiceLine {
        key: ServiceKey(key.to_owned()),
        name: name.to_owned(),
        offer,
        limit,
    }
}

fn services() -> Vec<ServiceLine> {
    vec![
        service("mail", "Mail", ServiceOffer::Offered(Check::On), None),
        service(
            "calendar",
            "Calendar",
            ServiceOffer::Offered(Check::On),
            None,
        ),
        service(
            "files",
            "Files",
            ServiceOffer::Offered(Check::Off),
            Some(Limitation::AppFolderOnly),
        ),
        service(
            "notes",
            "Notes",
            ServiceOffer::Absent(Limitation::NotOnServer),
            None,
        ),
    ]
}

/// The page.
#[component]
pub fn AccountsPage() -> Element {
    rsx! {
        Section { title: "Consent", note: "ConsentAlert: an app asks to use an account. The alert's own column (icon slot, title, message), the account pop-up when several fit, then Allow Once (the default, Return), Always Allow and Don't Allow stacked. Escape dismisses and stores nothing; only Don't Allow denies. With no account that fits it offers Add Account instead.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: consent(vec![account("ada@example.org", MarkProvider::Fastmail)]) }
                Stage { theme: Theme::Dark, body: consent(accounts()) }
                Stage { theme: Theme::Light, body: consent(vec![]) }
            }
        }
        Section { title: "Choose a provider", note: "ProviderList: a search field over the providers' rows with their marks, and a generic Other row. The keyboard starts in the search; Return picks the row under the cursor, else the first match; Escape cancels.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: step(rsx! {
                    ProviderList { providers: providers(), query: String::new(), on_query: |_| {}, on_cursor: |_| {}, on_pick: |_| {}, on_cancel: |_| {} }
                }) }
                Stage { theme: Theme::Dark, body: step(rsx! {
                    ProviderList { providers: providers(), query: "mail".to_owned(), on_query: |_| {}, on_cursor: |_| {}, on_pick: |_| {}, on_cancel: |_| {} }
                }) }
            }
        }
        Section { title: "Sign in", note: "SignInForm: the fields the step asks for, words from the role. Continue stays disabled until every required field has text; a refused password marks its field with a sentence under it and shakes once per refused attempt. The password is a hidden field: its text reaches the host as secret text and never the markup.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: sign_in(vec![
                    field(FieldRole::Address, FieldText::Plain(String::new())),
                    field(FieldRole::Password, FieldText::Secret(Hidden::default())),
                ], None) }
                Stage { theme: Theme::Dark, body: sign_in(vec![
                    field(FieldRole::Address, FieldText::Plain("ada@example.org".to_owned())),
                    field(FieldRole::Password, FieldText::Secret(Hidden::new("not shown"))),
                ], Some(FieldProblem { role: FieldRole::Password, kind: ProblemKind::Refused, attempt: Attempt(1) })) }
            }
        }
        Section { title: "Browser and device code", note: "BrowserWait: the page that was opened, Copy Link (the host copies; Copied once it says so), Open Again and Cancel. ShowCode: the device code large, the page to type it at, Copy Code.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: step(rsx! {
                    BrowserWait { provider: "Nextcloud", url: "https://cloud.example.org/login/v2/flow/abc123", on_open_again: |_| {}, on_copy: |_| {}, on_cancel: |_| {} }
                }) }
                Stage { theme: Theme::Dark, body: step(rsx! {
                    ShowCode { provider: "Microsoft 365", code: "BQKD-4MZP", url: "https://microsoft.com/devicelogin", copied: CopyState::Copied, on_copy: |_| {}, on_cancel: |_| {} }
                }) }
            }
        }
        Section { title: "Working and failed", note: "SignInWorking: nothing can be pressed but Cancel. SignInFailed: why it ended without an account in a sentence, with Back, Cancel and Try Again (the default).",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: step(rsx! {
                    SignInWorking { provider: "Fastmail", on_cancel: |_| {} }
                }) }
                Stage { theme: Theme::Dark, body: step(rsx! {
                    SignInFailed { provider: "Fastmail", why: SignInFault::Unreachable, on_retry: |_| {}, on_back: |_| {}, on_cancel: |_| {} }
                }) }
            }
        }
        Section { title: "Review", note: "ReviewServices: each service the provider offers with a switch, a limited one with its reason, and one the provider lacks with why. The last button reads Done, or Add, and allow Mail to use it when an app's chooser sent the person here.",
            div { class: "g-row g-row-top",
                Stage { theme: Theme::Light, body: step(rsx! {
                    ReviewServices { account: "ada@example.org", services: services(), on_toggle: |_| {}, on_done: |_| {}, on_back: |_| {}, on_cancel: |_| {} }
                }) }
                Stage { theme: Theme::Dark, body: step(rsx! {
                    ReviewServices { account: "ada@example.org", services: services(), allow: Some("Mail".to_owned()), on_toggle: |_| {}, on_done: |_| {}, on_back: |_| {}, on_cancel: |_| {} }
                }) }
            }
        }
        Section { title: "In an app", note: "AccountPicker: a pop-up of the accounts an app may use with Add Account under them. NoAccount: the empty state for each reason, with the one action that can help. AccountBadge: the mark and the name. LimitedNote: the one secondary line.",
            div { class: "g-row g-row-top",
                AccountPicker { accounts: accounts(), chosen: Some(ChoiceKey("ada@example.org".to_owned())), on_pick: |_| {} }
                AccountBadge { provider: MarkProvider::Fastmail, label: "ada@example.org" }
                LimitedNote { limit: Limitation::SevenDays }
            }
            div { class: "g-row g-row-top",
                for why in [NoAccountWhy::NeedsAccount, NoAccountWhy::Denied, NoAccountWhy::Unsupported] {
                    div { class: "g-col",
                        NoAccount { why, on_add: |_| {}, on_settings: |_| {} }
                    }
                }
            }
        }
    }
}

fn consent(choices: Vec<AccountChoice>) -> Element {
    rsx! {
        ConsentAlert { app: "Photos", request: "keep its library in your files", choices, on_choose: |_| {}, on_answer: |_| {} }
    }
}

/// A step in the standalone panel, as a host without a window of its own would show it.
fn step(body: Element) -> Element {
    rsx! {
        AccountSheet { label: "Add Account", on_dismiss: |_| {}, {body} }
    }
}

fn sign_in(fields: Vec<FormField>, problem: Option<FieldProblem>) -> Element {
    step(rsx! {
        SignInForm { provider: "Fastmail", mark: MarkProvider::Fastmail, fields, problem, on_input: |_| {}, on_submit: |_| {}, on_back: |_| {}, on_cancel: |_| {} }
    })
}

/// A sheet root of `theme` over the wallpaper holding `body`.
#[component]
fn Stage(theme: Theme, body: Element) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion) = {
        let axes = axes.read();
        (axes.accent, axes.motion)
    };
    rsx! {
        div { class: "g-modal g-accounts", style: "background-image:url(\"{wallpaper::uri()}\")",
            Ds {
                appearance: Appearance { theme, accent, motion },
                material: Material::Sheet,
                stylesheet: Inject::Host,
                div { class: "g-accounts-stage" }
                {body}
            }
        }
    }
}
