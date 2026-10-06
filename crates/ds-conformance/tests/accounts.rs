//! The account sheets on a real Blitz document: the keyboard starts where the person can type or
//! answer; Return does what the default button does and Escape what Cancel does, once; Continue
//! waits for every required field and Return waits with it; a typed password reaches the host
//! and never the markup; Escape on the consent alert dismisses and never denies; the provider
//! search narrows the host's list and Return picks its first match; the review's switches and
//! the copy buttons report to the host.

use dioxus::prelude::*;
use ds::components::content::provider_mark::MarkProvider;
use ds::prelude::*;
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use ds_shell::accounts::model::{
    AccountChoice, AllowScope, ChoiceKey, ConsentAnswer, CopyState, FieldRole, FieldText,
    FormField, Limitation, ProviderEntry, ProviderKey, ProviderPick, Requirement, ServiceKey,
    ServiceLine, ServiceOffer, SignInFault, StepTitle,
};
use ds_shell::prelude::*;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 800,
    height: 640,
    scale_percent: 100,
};

const SECRET: &str = "hunter2-hunter2";

static LOG: GlobalSignal<Vec<String>> = Signal::global(Vec::new);
static FIELDS: GlobalSignal<Vec<FormField>> = Signal::global(Vec::new);
static QUERY: GlobalSignal<String> = Signal::global(String::new);
static CURSOR: GlobalSignal<Option<ProviderPick>> = Signal::global(|| None);
static COPIED: GlobalSignal<CopyState> = Signal::global(CopyState::default);
static SERVICES: GlobalSignal<Vec<Check>> = Signal::global(Vec::new);

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn log(harness: &mut Harness) -> Vec<String> {
    harness.within(|| LOG.peek().clone())
}

fn field(role: FieldRole, text: FieldText) -> FormField {
    FormField {
        role,
        requirement: Requirement::Required,
        text,
    }
}

fn account(label: &str) -> AccountChoice {
    AccountChoice {
        key: ChoiceKey(label.to_owned()),
        label: label.to_owned(),
        provider: MarkProvider::Fastmail,
    }
}

fn provider(key: &str, label: &str) -> ProviderEntry {
    ProviderEntry {
        key: ProviderKey(key.to_owned()),
        label: label.to_owned(),
        mark: MarkProvider::Imap,
    }
}

#[allow(non_snake_case)]
fn Stage(body: Element) -> Element {
    rsx! {
        Ds { sheet: Some(ds_shell::stylesheet()), appearance: Appearance::default(), material: Material::Sheet,
            extent: RootExtent::Viewport,
            {body}
            p { class: "log", style: "position:absolute; left:0; bottom:0", {LOG().join(",")} }
        }
    }
}

#[allow(non_snake_case)]
fn SignIn() -> Element {
    Stage(rsx! {
        SignInForm {
            provider: "Fastmail",
            mark: MarkProvider::Fastmail,
                fields: FIELDS(),
                on_input: move |(role, text): (FieldRole, FieldText)| {
                    let mut fields = FIELDS.write();
                    if let Some(field) = fields.iter_mut().find(|field| field.role == role) {
                        field.text = text;
                    }
                },
                on_submit: move |()| LOG.write().push("submit".to_owned()),
                on_back: move |()| LOG.write().push("back".to_owned()),
                on_cancel: move |()| LOG.write().push("cancel".to_owned()),
        }
    })
}

#[allow(non_snake_case)]
fn HostTitled() -> Element {
    Stage(rsx! {
        ProviderList {
            title: StepTitle::Host,
            providers: vec![provider("fastmail", "Fastmail")],
            query: QUERY(),
            cursor: CURSOR(),
            on_query: move |query: String| *QUERY.write() = query,
            on_cursor: move |pick: ProviderPick| *CURSOR.write() = Some(pick),
            on_pick: move |pick: ProviderPick| LOG.write().push(format!("pick {pick:?}")),
            on_cancel: move |()| LOG.write().push("cancel".to_owned()),
        }
    })
}

#[allow(non_snake_case)]
fn ConsentInWindow() -> Element {
    Stage(rsx! {
        ConsentBody {
            app: "Photos",
            request: "keep its library in your files",
            choices: vec![account("ada@example.org")],
            on_choose: move |key: ChoiceKey| LOG.write().push(format!("choose {}", key.0)),
            on_answer: move |answer: ConsentAnswer| LOG.write().push(format!("{answer:?}")),
        }
    })
}

#[allow(non_snake_case)]
fn Consent() -> Element {
    Stage(rsx! {
        ConsentAlert {
            app: "Photos",
            request: "keep its library in your files",
            choices: vec![account("ada@example.org"), account("work@example.org")],
            on_choose: move |key: ChoiceKey| LOG.write().push(format!("choose {}", key.0)),
            on_answer: move |answer: ConsentAnswer| LOG.write().push(format!("{answer:?}")),
        }
    })
}

#[allow(non_snake_case)]
fn Providers() -> Element {
    Stage(rsx! {
        ProviderList {
            providers: vec![provider("nextcloud", "Nextcloud"), provider("fastmail", "Fastmail"), provider("fruit", "Fruitmail")],
            query: QUERY(),
            cursor: CURSOR(),
            on_query: move |query: String| *QUERY.write() = query,
            on_cursor: move |pick: ProviderPick| *CURSOR.write() = Some(pick),
            on_pick: move |pick: ProviderPick| LOG.write().push(format!("pick {pick:?}")),
            on_cancel: move |()| LOG.write().push("cancel".to_owned()),
        }
    })
}

#[allow(non_snake_case)]
fn Review() -> Element {
    let names = ["Mail", "Calendar"];
    let services: Vec<ServiceLine> = SERVICES()
        .into_iter()
        .zip(names)
        .map(|(value, name)| ServiceLine {
            key: ServiceKey(name.to_owned()),
            name: name.to_owned(),
            offer: ServiceOffer::Offered(value),
            limit: (name == "Calendar").then_some(Limitation::AppFolderOnly),
        })
        .collect();
    Stage(rsx! {
        ReviewServices {
            account: "ada@example.org",
            services,
            allow: Some("Mail".to_owned()),
            on_toggle: move |(key, value): (ServiceKey, Check)| {
                LOG.write().push(format!("toggle {} {value:?}", key.0));
                let index = if key.0 == "Mail" { 0 } else { 1 };
                SERVICES.write()[index] = value;
            },
            on_done: move |()| LOG.write().push("done".to_owned()),
            on_back: move |()| LOG.write().push("back".to_owned()),
            on_cancel: move |()| LOG.write().push("cancel".to_owned()),
        }
    })
}

#[allow(non_snake_case)]
fn Browser() -> Element {
    Stage(rsx! {
        BrowserWait {
            provider: "Nextcloud",
            url: "https://cloud.example.org/login/v2/flow/abc",
            copied: COPIED(),
            on_open_again: move |()| LOG.write().push("open".to_owned()),
            on_copy: move |text: String| {
                LOG.write().push(format!("copy {text}"));
                *COPIED.write() = CopyState::Copied;
            },
            on_cancel: move |()| LOG.write().push("cancel".to_owned()),
        }
    })
}

#[allow(non_snake_case)]
fn Failed() -> Element {
    Stage(rsx! {
        SignInFailed {
            provider: "Fastmail",
            why: SignInFault::Unreachable,
            on_retry: move |()| LOG.write().push("retry".to_owned()),
            on_back: move |()| LOG.write().push("back".to_owned()),
            on_cancel: move |()| LOG.write().push("cancel".to_owned()),
        }
    })
}

#[allow(non_snake_case)]
fn Working() -> Element {
    Stage(rsx! {
        SignInWorking { provider: "Fastmail", on_cancel: move |()| LOG.write().push("cancel".to_owned()) }
    })
}

fn start(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.within(|| {
        LOG.write().clear();
        *FIELDS.write() = vec![
            field(FieldRole::Address, FieldText::Plain(String::new())),
            field(FieldRole::Password, FieldText::Secret(Hidden::default())),
        ];
        QUERY.write().clear();
        *CURSOR.write() = None;
        *COPIED.write() = CopyState::Idle;
        *SERVICES.write() = vec![Check::On, Check::Off];
    });
    // Past the entrance and the focus that waits for the document.
    harness.advance(ms(600));
    harness
}

fn type_text(harness: &mut Harness, text: &str) {
    for c in text.chars() {
        harness.send(Input::key(ShortcutKey::Char(c)));
        harness.advance(ms(20));
    }
}

fn press(harness: &mut Harness, key: ShortcutKey) {
    harness.send(Input::key(key));
    harness.advance(ms(40));
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} in {}", harness.html()));
    harness.send(Input::click(at));
    harness.advance(ms(40));
}

#[test]
fn the_form_opens_with_the_keyboard_on_the_first_field() {
    let harness = start(SignIn);
    assert_eq!(
        harness.focus_of(".ds-acc-step input"),
        FocusState::Focused,
        "{}",
        harness.html()
    );
}

#[test]
fn continue_and_return_wait_for_every_required_field() {
    let mut harness = start(SignIn);
    press(&mut harness, ShortcutKey::Enter);
    assert!(log(&mut harness).is_empty(), "Return with nothing typed");

    type_text(&mut harness, "ada@example.org");
    press(&mut harness, ShortcutKey::Enter);
    assert!(
        log(&mut harness).is_empty(),
        "an address alone is not enough: the password is required"
    );
    assert_eq!(
        harness
            .attr(
                ".ds-acc-actions .ds-button[data-answers=\"return\"]",
                "aria-disabled"
            )
            .as_deref(),
        Some("true"),
        "Continue is disabled"
    );

    press(&mut harness, ShortcutKey::Tab);
    type_text(&mut harness, SECRET);
    assert_eq!(
        harness.attr(
            ".ds-acc-actions .ds-button[data-answers=\"return\"]",
            "aria-disabled"
        ),
        None,
        "Continue is enabled once the password is typed"
    );
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&mut harness), ["submit"], "Return submits once");
}

#[test]
fn what_is_typed_reaches_the_host_and_a_password_never_reaches_the_markup() {
    let mut harness = start(SignIn);
    type_text(&mut harness, "ada@example.org");
    press(&mut harness, ShortcutKey::Tab);
    type_text(&mut harness, SECRET);
    let (address, secret) = harness.within(|| {
        let fields = FIELDS.peek();
        (fields[0].text.clone(), fields[1].text.clone())
    });
    assert_eq!(address, FieldText::Plain("ada@example.org".to_owned()));
    match secret {
        FieldText::Secret(text) => assert_eq!(text.reveal(), SECRET, "the host hears the password"),
        other => panic!("the password field reports {other:?}"),
    }
    assert!(
        !harness.html().contains(SECRET),
        "the password is in the markup"
    );
}

#[test]
fn escape_cancels_once() {
    let mut harness = start(SignIn);
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(log(&mut harness), ["cancel"], "{}", harness.html());
}

#[test]
fn a_host_that_empties_the_password_empties_the_field() {
    let mut harness = start(SignIn);
    press(&mut harness, ShortcutKey::Tab);
    type_text(&mut harness, "abc");
    assert!(
        harness.text_of(".ds-acc-step .ds-input-mask").is_some(),
        "the typed dots show"
    );
    harness.within(|| {
        FIELDS.write()[1].text = FieldText::Secret(Hidden::default());
    });
    harness.advance(ms(40));
    assert_eq!(
        harness.text_of(".ds-acc-step .ds-input-mask"),
        None,
        "the field is empty again"
    );
}

#[test]
fn the_consent_alert_starts_on_allow_once_and_return_allows_the_first_account_once() {
    let mut harness = start(Consent);
    assert_eq!(
        harness.focus_of(".ds-alert-slot:nth-child(1) > .ds-button"),
        FocusState::Focused,
        "Allow Once is the default"
    );
    press(&mut harness, ShortcutKey::Enter);
    let want = format!(
        "{:?}",
        ConsentAnswer::Allow {
            account: ChoiceKey("ada@example.org".to_owned()),
            scope: AllowScope::Once
        }
    );
    assert_eq!(log(&mut harness), [want]);
}

#[test]
fn escape_on_the_consent_alert_dismisses_and_never_denies() {
    let mut harness = start(Consent);
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(log(&mut harness), ["Dismiss"]);
}

#[test]
fn dont_allow_denies_and_always_allow_is_always() {
    let mut harness = start(Consent);
    click(&mut harness, ".ds-alert-slot:nth-child(3) > .ds-button");
    assert_eq!(log(&mut harness), ["Deny"]);
    let mut harness = start(Consent);
    click(&mut harness, ".ds-alert-slot:nth-child(2) > .ds-button");
    let want = format!(
        "{:?}",
        ConsentAnswer::Allow {
            account: ChoiceKey("ada@example.org".to_owned()),
            scope: AllowScope::Always
        }
    );
    assert_eq!(log(&mut harness), [want]);
}

#[test]
fn the_search_narrows_the_list_and_return_picks_the_first_match() {
    let mut harness = start(Providers);
    assert_eq!(
        harness.focus_of(".ds-acc-step input"),
        FocusState::Focused,
        "the search takes the keyboard"
    );
    let rows = |harness: &Harness| harness.count(".ds-acc-step .ds-avatar");
    assert_eq!(rows(&harness), 4, "three providers and Other");
    type_text(&mut harness, "fast");
    harness.advance(ms(800));
    assert_eq!(rows(&harness), 2, "Fastmail and Other: {}", harness.html());
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(
        log(&mut harness),
        [format!(
            "pick {:?}",
            ProviderPick::Provider(ProviderKey("fastmail".to_owned()))
        )]
    );
}

#[test]
fn escape_cancels_the_provider_list() {
    let mut harness = start(Providers);
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(log(&mut harness), ["cancel"]);
}

#[test]
fn the_review_opens_on_its_last_button_and_return_presses_it() {
    let mut harness = start(Review);
    assert_eq!(
        harness
            .text_of(".ds-acc-actions .ds-button[data-answers=\"return\"]")
            .as_deref(),
        Some("Add, and allow Mail to use it")
    );
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&mut harness), ["done"]);
}

#[test]
fn the_review_reports_each_switch_from_the_host_value() {
    let mut harness = start(Review);
    click(&mut harness, ".ds-acc-step .ds-toggle");
    assert_eq!(
        log(&mut harness),
        ["toggle Mail Off"],
        "the first switch was on"
    );
    click(&mut harness, ".ds-acc-step .ds-toggle");
    assert_eq!(
        log(&mut harness),
        ["toggle Mail Off", "toggle Mail On"],
        "the host's value came back, so the next flip is the other way"
    );
}

#[test]
fn return_still_finishes_the_review_after_a_switch_was_pressed() {
    let mut harness = start(Review);
    click(&mut harness, ".ds-acc-step .ds-toggle");
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&mut harness), ["toggle Mail Off", "done"]);
}

#[test]
fn copy_link_reports_the_link_and_shows_the_hosts_word() {
    let mut harness = start(Browser);
    click(&mut harness, ".ds-acc-actions .ds-button:nth-child(1)");
    assert_eq!(
        log(&mut harness),
        ["copy https://cloud.example.org/login/v2/flow/abc"]
    );
    assert_eq!(
        harness
            .text_of(".ds-acc-actions .ds-button:nth-child(1)")
            .as_deref(),
        Some("Copied")
    );
}

#[test]
fn a_password_the_host_hands_back_shows_in_the_field() {
    let mut harness = start(SignIn);
    harness.within(|| {
        FIELDS.write()[1].text = FieldText::Secret(Hidden::new("abc".to_owned()));
    });
    harness.advance(ms(40));
    assert!(
        harness.text_of(".ds-acc-step .ds-input-mask").is_some(),
        "a form drawn again with a typed password shows its dots: {}",
        harness.html()
    );
}

#[test]
fn down_in_the_search_moves_the_cursor_through_the_rows_and_return_picks_it() {
    let mut harness = start(Providers);
    press(&mut harness, ShortcutKey::Down);
    press(&mut harness, ShortcutKey::Down);
    assert_eq!(
        harness.focus_of(".ds-acc-step input"),
        FocusState::Focused,
        "the keyboard stays in the search"
    );
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(
        log(&mut harness),
        [format!(
            "pick {:?}",
            ProviderPick::Provider(ProviderKey("fastmail".to_owned()))
        )],
        "the second row"
    );
}

#[test]
fn return_opens_the_browser_page_again() {
    let mut harness = start(Browser);
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&mut harness), ["open"]);
}

#[test]
fn return_tries_a_failed_sign_in_again_and_escape_gives_up() {
    let mut harness = start(Failed);
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(log(&mut harness), ["retry"]);
    let mut harness = start(Failed);
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(log(&mut harness), ["cancel"]);
}

#[test]
fn nothing_but_cancel_answers_while_working() {
    let mut harness = start(Working);
    press(&mut harness, ShortcutKey::Enter);
    assert!(log(&mut harness).is_empty(), "Return answers nothing");
    press(&mut harness, ShortcutKey::Escape);
    assert_eq!(log(&mut harness), ["cancel"]);
}

#[test]
fn a_step_in_a_host_window_draws_no_title_and_no_attached_sheet() {
    let mut harness = start(HostTitled);
    assert_eq!(harness.count(".ds-acc-title"), 0, "{}", harness.html());
    assert_eq!(harness.count(".ds-sheet"), 0, "the step hangs from nothing");
    assert_eq!(harness.count(".ds-acc-step"), 1);
    press(&mut harness, ShortcutKey::Enter);
    assert_eq!(
        log(&mut harness),
        [format!(
            "pick {:?}",
            ProviderPick::Provider(ProviderKey("fastmail".to_owned()))
        )],
        "its keys work without a sheet around it"
    );
}

#[test]
fn a_step_draws_its_own_title_unless_the_host_does() {
    let harness = start(Providers);
    assert_eq!(harness.count(".ds-acc-title"), 1, "{}", harness.html());
}

#[test]
fn the_consent_body_stands_in_a_host_window_with_its_keys() {
    let mut harness = start(ConsentInWindow);
    assert_eq!(harness.count(".ds-sheet"), 0, "no sheet around the body");
    assert_eq!(harness.count(".ds-alert"), 1);
    press(&mut harness, ShortcutKey::Enter);
    press(&mut harness, ShortcutKey::Escape);
    let want = format!(
        "{:?}",
        ConsentAnswer::Allow {
            account: ChoiceKey("ada@example.org".to_owned()),
            scope: AllowScope::Once
        }
    );
    assert_eq!(log(&mut harness), [want, "Dismiss".to_owned()]);
}
