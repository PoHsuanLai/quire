//! The account sheets as markup: every step in light and dark matches its golden under
//! `tests/snapshots/accounts/`, lints clean, and uses only `ds-` classes the stylesheet styles.
//! The markup carries what the props say, and no secret reaches it.
//!
//! `DS_BLESS=1 cargo test -p ds-shell --test accounts_ssr` rewrites the goldens.

#[path = "../../ds/tests/support/golden.rs"]
mod golden;

use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::content::provider_mark::MarkProvider;
use ds::prelude::*;
use ds_lint::{LintConfig, markup};
use ds_shell::accounts::model::{
    AccountChoice, Attempt, ChoiceKey, CopyState, FieldProblem, FieldRole, FieldText, FormField,
    Limitation, NoAccountWhy, ProblemKind, ProviderEntry, ProviderKey, ProviderPick, Requirement,
    ServiceKey, ServiceLine, ServiceOffer, SignInFault,
};
use ds_shell::prelude::*;

const PASSWORD: &str = "correct horse battery staple";

#[derive(Clone, PartialEq)]
struct StageProps {
    theme: Theme,
    /// Which of `SPECIMENS` it holds.
    specimen: usize,
}

#[allow(non_snake_case)]
fn Stage(props: StageProps) -> Element {
    let body = (SPECIMENS[props.specimen].1)();
    rsx! {
        Ds {
            appearance: Appearance { theme: props.theme, ..Appearance::default() },
            material: Material::Sheet,
            extent: RootExtent::Viewport,
            stylesheet: Inject::Host,
            {body}
        }
    }
}

fn account(label: &str) -> AccountChoice {
    AccountChoice {
        key: ChoiceKey(label.to_owned()),
        label: label.to_owned(),
        provider: MarkProvider::Fastmail,
    }
}

fn provider(key: &str, label: &str, mark: MarkProvider) -> ProviderEntry {
    ProviderEntry {
        key: ProviderKey(key.to_owned()),
        label: label.to_owned(),
        mark,
    }
}

fn providers() -> Vec<ProviderEntry> {
    vec![
        provider("nextcloud", "Nextcloud", MarkProvider::Imap),
        provider("fastmail", "Fastmail", MarkProvider::Fastmail),
        provider("microsoft", "Microsoft 365", MarkProvider::Microsoft),
    ]
}

fn field(role: FieldRole, text: FieldText) -> FormField {
    FormField {
        role,
        requirement: Requirement::Required,
        text,
    }
}

fn address(text: &str) -> FormField {
    field(FieldRole::Address, FieldText::Plain(text.to_owned()))
}

fn password(text: &str) -> FormField {
    field(FieldRole::Password, FieldText::Secret(Hidden::new(text)))
}

fn sign_in(fields: Vec<FormField>, problem: Option<FieldProblem>) -> Element {
    rsx! {
        SignInForm {
            provider: "Fastmail",
            fields,
            problem,
            on_input: |_| {},
            on_submit: |_| {},
            on_back: |_| {},
            on_cancel: |_| {},
        }
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

fn review(allow: Option<&str>) -> Element {
    let services = vec![
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
    ];
    rsx! {
        ReviewServices {
            account: "ada@example.org",
            services,
            allow: allow.map(str::to_owned),
            on_toggle: |_| {},
            on_done: |_| {},
            on_back: |_| {},
            on_cancel: |_| {},
        }
    }
}

fn consent(choices: Vec<AccountChoice>) -> Element {
    rsx! {
        ConsentAlert {
            app: "Photos",
            request: "keep its library in your files",
            choices,
            on_choose: |_| {},
            on_answer: |_| {},
        }
    }
}

fn provider_list(query: &str, cursor: Option<ProviderPick>) -> Element {
    rsx! {
        ProviderList {
            providers: providers(),
            query: query.to_owned(),
            cursor,
            on_query: |_| {},
            on_cursor: |_| {},
            on_pick: |_| {},
            on_cancel: |_| {},
        }
    }
}

fn failed(why: SignInFault) -> Element {
    rsx! {
        SignInFailed { provider: "Fastmail", why, on_retry: |_| {}, on_back: |_| {}, on_cancel: |_| {} }
    }
}

fn no_account(why: NoAccountWhy) -> Element {
    rsx! { NoAccount { why, on_add: |_| {}, on_settings: |_| {} } }
}

type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("consent-one", || consent(vec![account("ada@example.org")])),
    ("consent-several", || {
        consent(vec![
            account("ada@example.org"),
            account("work@example.org"),
        ])
    }),
    ("consent-none", || consent(vec![])),
    ("providers", || provider_list("", None)),
    ("providers-search", || provider_list("mail", None)),
    ("providers-cursor", || {
        provider_list(
            "",
            Some(ProviderPick::Provider(ProviderKey("fastmail".to_owned()))),
        )
    }),
    ("sign-in-empty", || {
        sign_in(vec![address(""), password("")], None)
    }),
    ("sign-in-ready", || {
        sign_in(vec![address("ada@example.org"), password(PASSWORD)], None)
    }),
    ("sign-in-refused", || {
        sign_in(
            vec![address("ada@example.org"), password(PASSWORD)],
            Some(FieldProblem {
                role: FieldRole::Password,
                kind: ProblemKind::Refused,
                attempt: Attempt(1),
            }),
        )
    }),
    ("sign-in-server", || {
        sign_in(
            vec![
                address("ada@example.org"),
                field(FieldRole::Server, FieldText::Plain(String::new())),
                password(PASSWORD),
            ],
            None,
        )
    }),
    ("browser", || {
        rsx! {
            BrowserWait {
                provider: "Nextcloud",
                url: "https://cloud.example.org/login/v2/flow/abc123",
                on_open_again: |_| {},
                on_copy: |_| {},
                on_cancel: |_| {},
            }
        }
    }),
    ("browser-copied", || {
        rsx! {
            BrowserWait {
                provider: "Nextcloud",
                url: "https://cloud.example.org/login/v2/flow/abc123",
                copied: CopyState::Copied,
                on_open_again: |_| {},
                on_copy: |_| {},
                on_cancel: |_| {},
            }
        }
    }),
    ("code", || {
        rsx! {
            ShowCode {
                provider: "Microsoft 365",
                code: "BQKD-4MZP",
                url: "https://microsoft.com/devicelogin",
                on_copy: |_| {},
                on_cancel: |_| {},
            }
        }
    }),
    ("working", || {
        rsx! { SignInWorking { provider: "Fastmail", on_cancel: |_| {} } }
    }),
    ("failed-unreachable", || failed(SignInFault::Unreachable)),
    ("failed-forbidden", || failed(SignInFault::Forbidden)),
    ("review", || review(None)),
    ("review-allow", || review(Some("Mail"))),
    ("picker", || {
        rsx! {
            AccountPicker {
                accounts: vec![account("ada@example.org"), account("work@example.org")],
                chosen: Some(ChoiceKey("work@example.org".to_owned())),
                on_pick: |_| {},
            }
        }
    }),
    ("no-account-needs", || {
        no_account(NoAccountWhy::NeedsAccount)
    }),
    ("no-account-denied", || no_account(NoAccountWhy::Denied)),
    ("no-account-unsupported", || {
        no_account(NoAccountWhy::Unsupported)
    }),
    ("badge", || {
        rsx! { AccountBadge { provider: MarkProvider::Fastmail, label: "ada@example.org" } }
    }),
    ("limited", || {
        rsx! { LimitedNote { limit: Limitation::SevenDays } }
    }),
];

/// Build the dom and flush the effects that register overlays, so a sheet is in the markup.
fn render(theme: Theme, specimen: usize) -> String {
    let mut dom = VirtualDom::new_with_props(Stage, StageProps { theme, specimen });
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

fn themes() -> [(&'static str, Theme); 2] {
    [("light", Theme::Light), ("dark", Theme::Dark)]
}

/// Where the specimen called `name` stands in `SPECIMENS`.
fn specimen(name: &str) -> usize {
    SPECIMENS
        .iter()
        .position(|(known, _)| *known == name)
        .unwrap_or_else(|| panic!("no specimen {name}"))
}

fn light(name: &str) -> String {
    render(Theme::Light, specimen(name))
}

#[test]
fn every_specimen_matches_its_golden_in_light_and_dark() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .enumerate()
        .flat_map(|(at, (name, _))| {
            themes().into_iter().filter_map(move |(look, theme)| {
                golden::check(&format!("accounts/{name}-{look}.html"), &render(theme, at)).err()
            })
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds_shell::stylesheet();
    let mut failures = Vec::new();
    for (at, (name, _)) in SPECIMENS.iter().enumerate() {
        let html = render(Theme::Light, at);
        for offence in markup(&html, sheet, &LintConfig::new(&ds_shell::kits())) {
            failures.push(format!("{name}: {:?} {}", offence.rule, offence.text));
        }
        for class in html
            .split("class=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .filter(|class| class.starts_with("ds-"))
        {
            let needle = format!(".{class}");
            let styled = sheet.match_indices(&needle).any(|(at, _)| {
                !sheet[at + needle.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            });
            if !styled {
                failures.push(format!("{name}: .{class} is not styled"));
            }
        }
    }
    failures.dedup();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A password never reaches the markup, whatever the step: not as a value, not as text.
#[test]
fn a_typed_password_is_not_in_the_markup() {
    for (name, _) in SPECIMENS
        .iter()
        .filter(|(name, _)| name.starts_with("sign-in"))
    {
        let html = light(name);
        assert!(
            !html.contains(PASSWORD) && !html.contains("value=\"c"),
            "{name}: the password is in {html}"
        );
    }
    let ready = light("sign-in-ready");
    assert!(ready.contains("data-kind=\"secure\""), "{ready}");
}

#[test]
fn continue_waits_for_every_required_field() {
    let empty = light("sign-in-empty");
    let ready = light("sign-in-ready");
    let continue_button = |html: &str| {
        html.split("<button")
            .find(|button| {
                button.contains("aria-label=\"Continue\"") || button.contains(">Continue<")
            })
            .unwrap_or_else(|| panic!("a Continue button in {html}"))
            .split('>')
            .next()
            .unwrap_or_default()
            .to_owned()
    };
    assert!(
        continue_button(&empty).contains("aria-disabled=\"true\""),
        "{}",
        continue_button(&empty)
    );
    assert!(
        !continue_button(&ready).contains("aria-disabled"),
        "{}",
        continue_button(&ready)
    );
}

#[test]
fn the_steps_say_what_the_props_say() {
    let cases: &[(&str, &[&str])] = &[
        (
            "consent-one",
            &[
                "Photos wants to keep its library in your files",
                "It would use ada@example.org.",
                "Allow Once",
                "Always Allow",
                "Don&#39;t Allow",
                "data-layout=\"stack\"",
            ],
        ),
        (
            "consent-several",
            &["Choose the account it may use.", "ds-popup"],
        ),
        (
            "consent-none",
            &[
                "Add Account\u{2026}",
                "None of your accounts can do this yet.",
            ],
        ),
        ("providers", &["Nextcloud", "Fastmail", "Other\u{2026}"]),
        (
            "sign-in-refused",
            &["The password was not accepted. Check it and try again."],
        ),
        ("code", &["BQKD-4MZP", "https://microsoft.com/devicelogin"]),
        ("browser", &["Copy Link", "Open Again", "Cancel"]),
        ("browser-copied", &["Copied"]),
        (
            "working",
            &["Signing in to Fastmail", "This takes a moment."],
        ),
        (
            "failed-unreachable",
            &[
                "Could not add Fastmail",
                "The server could not be reached.",
                "Try Again",
            ],
        ),
        (
            "failed-forbidden",
            &["Your organisation or the provider does not allow this."],
        ),
        (
            "review",
            &[
                "Mail",
                "Calendar",
                "Only files this desktop created.",
                "This server does not have it.",
                ">Done<",
            ],
        ),
        ("review-allow", &["Add, and allow Mail to use it"]),
        (
            "no-account-needs",
            &["No account yet", "Add Account\u{2026}"],
        ),
        ("no-account-denied", &["Open Settings"]),
        ("limited", &["Signing in lasts seven days"]),
    ];
    for (name, wants) in cases {
        let html = light(name);
        for want in *wants {
            assert!(html.contains(want), "{name}: {want} in {html}");
        }
    }
    let narrowed = light("providers-search");
    assert!(
        !narrowed.contains("Nextcloud"),
        "the search drops Nextcloud"
    );
    assert!(narrowed.contains("Fastmail"), "{narrowed}");
    assert!(narrowed.contains("Other"), "Other stays");
}
