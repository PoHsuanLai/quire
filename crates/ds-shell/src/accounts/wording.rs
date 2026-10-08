//! Every sentence the account sheets say, and the small decisions that read from them: which
//! providers a search keeps, whether a form may continue. Pure, so a table can test each one.

use ds::components::content::provider_mark::MarkProvider;

use super::model::{
    AccountChoice, AllowScope, ChoiceKey, FieldProblem, FieldRole, FormField, FormPart, Limitation,
    NoAccountWhy, ProblemKind, ProviderEntry, ProviderPick, Requirement, SignInFault,
};

/// "Mail wants to keep its library in your files": `request` finishes the sentence.
pub(crate) fn consent_title(app: &str, request: &str) -> String {
    format!("{app} wants to {request}")
}

/// The line under the consent title: which account it would use, or to choose one.
pub(crate) fn consent_message(choices: &[AccountChoice]) -> String {
    match choices {
        [] => "None of your accounts can do this yet.".to_owned(),
        [only] => format!("It would use {}.", only.label),
        _ => "Choose the account it may use.".to_owned(),
    }
}

/// The button that allows for `scope`: "Allow Once", "This Session Only" or "Always Allow".
pub(crate) fn allow_label(scope: AllowScope) -> &'static str {
    match scope {
        AllowScope::Once => "Allow Once",
        AllowScope::Session => "This Session Only",
        AllowScope::Always => "Always Allow",
    }
}

/// The label a form field carries.
pub(crate) fn field_label(role: FieldRole) -> &'static str {
    match role {
        FieldRole::Address => "Email address",
        FieldRole::Server => "Server",
        FieldRole::Username => "User name",
        FieldRole::Password => "Password",
        FieldRole::AppPassword => "App password",
        FieldRole::ApiKey => "API key",
        FieldRole::Token => "Access token",
        FieldRole::Protocol => "Server type",
        FieldRole::Port => "Port",
        FieldRole::Security => "Security",
        FieldRole::OutgoingServer => "Outgoing server",
        FieldRole::OutgoingPort => "Outgoing port",
        FieldRole::OutgoingSecurity => "Outgoing security",
        FieldRole::SessionUrl => "Server web address",
    }
}

/// The hint inside an empty form field. An optional login name says what an empty one means; it
/// never says "Required".
pub(crate) fn field_placeholder(role: FieldRole, requirement: Requirement) -> &'static str {
    match role {
        FieldRole::Address => "name@example.org",
        FieldRole::Server => "mail.example.org",
        FieldRole::Username => match requirement {
            Requirement::Required => "Required",
            Requirement::Optional => "Same as your address",
        },
        FieldRole::Password => "Password or app password",
        FieldRole::AppPassword => "Paste your app password",
        FieldRole::ApiKey => "Paste your key",
        FieldRole::Token => "Paste your token",
        FieldRole::Protocol | FieldRole::Security | FieldRole::OutgoingSecurity => "Choose",
        FieldRole::Port | FieldRole::OutgoingPort => "Usual port",
        FieldRole::OutgoingServer => "smtp.example.org",
        FieldRole::SessionUrl => "https://mail.example.org/session",
    }
}

/// The title over a group of a long form.
pub(crate) fn part_words(part: FormPart) -> &'static str {
    match part {
        FormPart::Incoming => "Incoming",
        FormPart::Outgoing => "Outgoing",
        FormPart::SignIn => "Sign in",
    }
}

/// What is said under a field that is marked.
pub(crate) fn problem_text(problem: FieldProblem) -> String {
    let label = field_label(problem.role).to_lowercase();
    match problem.kind {
        ProblemKind::Missing => format!("Enter the {label}."),
        ProblemKind::Refused => format!("The {label} was not accepted. Check it and try again."),
        ProblemKind::Invalid => format!("The {label} is not valid. Check it and try again."),
    }
}

/// Whether the form may continue: every required field has something in it.
/// Whether Continue is live: every required field holds something, and no field is known to be
/// invalid. A refusal does not hold Continue back: the person corrects the field and tries again.
pub(crate) fn form_ready(fields: &[FormField], problem: Option<FieldProblem>) -> bool {
    let invalid = problem.is_some_and(|problem| problem.kind == ProblemKind::Invalid);
    !invalid
        && fields
            .iter()
            .all(|field| field.requirement == Requirement::Optional || !field.text.is_blank())
}

/// The review's last button: a plain "Done", or the one-grant "Add, and allow Mail to use it"
/// when an app's chooser found no account and sent the person here (design/31 section 4.5).
pub(crate) fn confirm_label(allow: Option<&str>) -> String {
    match allow {
        Some(app) => format!("Add, and allow {app} to use it"),
        None => "Done".to_owned(),
    }
}

/// The secondary line for a limitation.
pub(crate) fn limitation(limit: Limitation) -> &'static str {
    match limit {
        Limitation::AppendOnly => "Can add items, but not read the ones already there.",
        Limitation::PickerOnly => "Existing items only through the provider's own picker.",
        Limitation::AppFolderOnly => "Only files this desktop created.",
        Limitation::ProviderOffersNone => "The provider offers no access to this.",
        Limitation::AdminConsent => "Your organisation's administrator has to allow this first.",
        Limitation::UnverifiedBuild => "The provider has not verified this app yet.",
        Limitation::TurnedOff => "Turned off for this account.",
        Limitation::NotOnServer => "This server does not have it.",
        Limitation::SevenDays => "Signing in lasts seven days; you will be asked again.",
        Limitation::UntilPasswordChange => "Signing in lasts until the password changes.",
    }
}

/// What is said when a sign-in ended without an account. `label` names the account being added;
/// `provider` is its mark, when the host knows it.
pub(crate) fn fault(fault: SignInFault, label: &str, provider: Option<MarkProvider>) -> String {
    let text = match fault {
        SignInFault::Refused => "The provider did not accept what was entered.",
        SignInFault::Unreachable => {
            "The server could not be reached. Check the address and your connection."
        }
        SignInFault::Unreadable => "The server answered in a way this desktop cannot read.",
        SignInFault::NeedsClientId => match provider {
            Some(MarkProvider::Google) => "Google sign-in needs a client id: see docs/google.md",
            _ => {
                "This build is not registered with the provider. Add a client id in Settings, under Accounts."
            }
        },
        SignInFault::TimedOut => "Signing in took too long and was stopped.",
        SignInFault::Cancelled => "Signing in was cancelled.",
        SignInFault::Forbidden => "Your organisation or the provider does not allow this.",
        SignInFault::StoreFailed => "Signed in, but the account could not be saved.",
        SignInFault::AlreadyAdded => "This account is already added.",
        SignInFault::NoLauncher => return format!("No app is set up to sign {label} in"),
        SignInFault::NotInstalled => return format!("{label} isn't installed"),
    };
    text.to_owned()
}

/// The title and the line of the empty state for an app with no account.
pub(crate) fn no_account(why: NoAccountWhy) -> (&'static str, &'static str) {
    match why {
        NoAccountWhy::NeedsAccount => ("No account yet", "Add an account to use this."),
        NoAccountWhy::Denied => (
            "Access was turned down",
            "Change this in Settings, under Accounts.",
        ),
        NoAccountWhy::Unsupported => (
            "None of your providers can do this",
            "Add an account with a provider that can.",
        ),
    }
}

/// The providers whose label holds `query` (ignoring case and the blanks around it), in their
/// order. An empty query keeps all of them.
pub(crate) fn matching<'a>(entries: &'a [ProviderEntry], query: &str) -> Vec<&'a ProviderEntry> {
    let wanted = query.trim().to_lowercase();
    entries
        .iter()
        .filter(|entry| entry.label.to_lowercase().contains(&wanted))
        .collect()
}

/// The account the consent alert would use: the one the host chose if it still fits, else the
/// first.
pub(crate) fn effective_choice<'a>(
    choices: &'a [AccountChoice],
    chosen: Option<&ChoiceKey>,
) -> Option<&'a AccountChoice> {
    chosen
        .and_then(|key| choices.iter().find(|choice| &choice.key == key))
        .or_else(|| choices.first())
}

/// The row Return picks: the one under the cursor if it is still listed, else the first match,
/// else "Other".
pub(crate) fn pick_for_enter(
    matches: &[&ProviderEntry],
    cursor: Option<&ProviderPick>,
) -> ProviderPick {
    let listed = |pick: &ProviderPick| match pick {
        ProviderPick::Provider(key) => matches.iter().any(|entry| &entry.key == key),
        ProviderPick::Other => true,
    };
    match cursor.filter(|pick| listed(pick)) {
        Some(pick) => pick.clone(),
        None => matches.first().map_or(ProviderPick::Other, |entry| {
            ProviderPick::Provider(entry.key.clone())
        }),
    }
}

/// Which way the arrow keys move the provider cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CursorStep {
    /// Down.
    Next,
    /// Up.
    Previous,
}

/// The row the cursor moves to from `cursor` among `rows`, in their order: from no cursor (or a
/// row no longer listed) Down lands on the first and Up on the last; at either end it stays.
pub(crate) fn step_cursor(
    rows: &[ProviderPick],
    cursor: Option<&ProviderPick>,
    step: CursorStep,
) -> Option<ProviderPick> {
    let at = cursor.and_then(|pick| rows.iter().position(|row| row == pick));
    let to = match (at, step) {
        (None, CursorStep::Next) => 0,
        (None, CursorStep::Previous) => rows.len().checked_sub(1)?,
        (Some(at), CursorStep::Next) => (at + 1).min(rows.len().checked_sub(1)?),
        (Some(at), CursorStep::Previous) => at.saturating_sub(1),
    };
    rows.get(to).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::hidden::Hidden;
    use crate::accounts::model::{Attempt, ChoiceKey, FieldText, ProviderKey};
    use ds_core::word::Word;

    fn entry(key: &str, label: &str) -> ProviderEntry {
        ProviderEntry::new(ProviderKey(key.to_owned()), label, MarkProvider::Imap)
    }

    fn field(requirement: Requirement, text: FieldText) -> FormField {
        FormField::new(FieldRole::Address, requirement, text)
    }

    fn choice(label: &str) -> AccountChoice {
        AccountChoice {
            key: ChoiceKey(label.to_owned()),
            label: label.to_owned(),
            provider: MarkProvider::Google,
        }
    }

    #[test]
    fn a_form_continues_only_when_every_required_field_has_text() {
        let plain = |text: &str| FieldText::Plain(text.to_owned());
        let secret = |text: &str| FieldText::Secret(Hidden::new(text));
        let cases: &[(&str, Vec<FormField>, bool)] = &[
            ("no fields", vec![], true),
            (
                "required and filled",
                vec![field(Requirement::Required, plain("ada@example.org"))],
                true,
            ),
            (
                "required and empty",
                vec![field(Requirement::Required, plain(""))],
                false,
            ),
            (
                "required and only blanks",
                vec![field(Requirement::Required, plain("  \t"))],
                false,
            ),
            (
                "optional and empty",
                vec![field(Requirement::Optional, plain(""))],
                true,
            ),
            (
                "a secret still empty",
                vec![
                    field(Requirement::Required, plain("ada@example.org")),
                    field(Requirement::Required, secret("")),
                ],
                false,
            ),
            (
                "a secret typed",
                vec![
                    field(Requirement::Required, plain("ada@example.org")),
                    field(Requirement::Required, secret("pw")),
                ],
                true,
            ),
        ];
        for (name, fields, want) in cases {
            assert_eq!(form_ready(fields, None), *want, "{name}");
        }
        // A field known to be invalid holds Continue back; a refusal does not (retry after editing).
        let filled = vec![field(Requirement::Required, plain("ada@example.org"))];
        let problem = |kind| FieldProblem {
            role: FieldRole::Port,
            kind,
            attempt: Attempt(1),
        };
        assert!(!form_ready(&filled, Some(problem(ProblemKind::Invalid))));
        assert!(form_ready(&filled, Some(problem(ProblemKind::Refused))));
    }

    #[test]
    fn an_optional_login_name_never_says_required() {
        assert_eq!(
            field_placeholder(FieldRole::Username, Requirement::Optional),
            "Same as your address"
        );
        assert_eq!(
            field_placeholder(FieldRole::Username, Requirement::Required),
            "Required"
        );
    }

    #[test]
    fn the_search_keeps_labels_that_hold_the_query_in_any_case() {
        let entries = [
            entry("nc", "Nextcloud"),
            entry("fm", "Fastmail"),
            entry("ms", "Microsoft 365"),
        ];
        let cases: &[(&str, &[&str])] = &[
            ("", &["nc", "fm", "ms"]),
            ("  ", &["nc", "fm", "ms"]),
            ("CLOUD", &["nc"]),
            (" mail ", &["fm"]),
            ("soft", &["ms"]),
            ("zzz", &[]),
        ];
        for (query, want) in cases {
            let got: Vec<&str> = matching(&entries, query)
                .iter()
                .map(|entry| entry.key.0.as_str())
                .collect();
            assert_eq!(&got, want, "{query:?}");
        }
    }

    #[test]
    fn return_picks_the_cursor_row_else_the_first_match_else_other() {
        let entries = [entry("nc", "Nextcloud"), entry("fm", "Fastmail")];
        let all: Vec<&ProviderEntry> = entries.iter().collect();
        let fm = ProviderPick::Provider(ProviderKey("fm".to_owned()));
        let nc = ProviderPick::Provider(ProviderKey("nc".to_owned()));
        let cases: &[(
            &str,
            &[&ProviderEntry],
            Option<&ProviderPick>,
            &ProviderPick,
        )] = &[
            ("the cursor row", &all, Some(&fm), &fm),
            ("no cursor: the first", &all, None, &nc),
            ("a cursor row the search dropped", &all[..1], Some(&fm), &nc),
            ("nothing matches", &[], None, &ProviderPick::Other),
            (
                "Other under the cursor",
                &all,
                Some(&ProviderPick::Other),
                &ProviderPick::Other,
            ),
        ];
        for (name, matches, cursor, want) in cases {
            assert_eq!(&pick_for_enter(matches, *cursor), *want, "{name}");
        }
    }

    #[test]
    fn the_alert_uses_the_chosen_account_while_it_fits_else_the_first() {
        let choices = [choice("a"), choice("b")];
        let key = |label: &str| ChoiceKey(label.to_owned());
        let cases: &[(&str, Option<ChoiceKey>, Option<&str>)] = &[
            ("chosen", Some(key("b")), Some("b")),
            ("none chosen", None, Some("a")),
            ("chosen one is gone", Some(key("z")), Some("a")),
        ];
        for (name, chosen, want) in cases {
            let got = effective_choice(&choices, chosen.as_ref()).map(|c| c.label.as_str());
            assert_eq!(got, *want, "{name}");
        }
        assert!(effective_choice(&[], Some(&key("a"))).is_none());
    }

    #[test]
    fn the_review_names_the_app_only_when_one_sent_the_person() {
        assert_eq!(confirm_label(None), "Done");
        assert_eq!(confirm_label(Some("Mail")), "Add, and allow Mail to use it");
    }

    #[test]
    fn consent_words_follow_how_many_accounts_fit() {
        assert_eq!(
            consent_title("Photos", "keep its library in your files"),
            "Photos wants to keep its library in your files"
        );
        let cases: &[(&[AccountChoice], &str)] = &[
            (&[], "None of your accounts can do this yet."),
            (
                &[choice("ada@example.org")],
                "It would use ada@example.org.",
            ),
        ];
        for (choices, want) in cases {
            assert_eq!(consent_message(choices), *want);
        }
        assert_eq!(
            consent_message(&[choice("a"), choice("b")]),
            "Choose the account it may use."
        );
    }

    #[test]
    fn every_fault_has_its_own_sentence() {
        let sentences: Vec<String> = SignInFault::ALL
            .iter()
            .map(|f| fault(*f, "Ada", None))
            .collect();
        let mut unique = sentences.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), sentences.len(), "{sentences:?}");
        assert!(
            sentences
                .iter()
                .filter(|s| !s.contains("Ada"))
                .all(|s| s.ends_with('.')),
            "{sentences:?}"
        );
    }

    #[test]
    fn google_needs_a_client_id_says_where_to_look_and_others_keep_their_text() {
        let google = fault(
            SignInFault::NeedsClientId,
            "Ada",
            Some(MarkProvider::Google),
        );
        assert_eq!(
            google,
            "Google sign-in needs a client id: see docs/google.md"
        );
        for provider in [None, Some(MarkProvider::Imap)] {
            let other = fault(SignInFault::NeedsClientId, "Ada", provider);
            assert!(other.starts_with("This build is not registered"), "{other}");
        }
    }

    #[test]
    fn the_agent_faults_name_the_account() {
        assert_eq!(
            fault(SignInFault::NoLauncher, "Claude", None),
            "No app is set up to sign Claude in"
        );
        assert_eq!(
            fault(SignInFault::NotInstalled, "Claude", None),
            "Claude isn't installed"
        );
    }

    #[test]
    fn a_marked_field_says_which_and_why() {
        let cases = [
            (
                FieldRole::Password,
                ProblemKind::Refused,
                "The password was not accepted. Check it and try again.",
            ),
            (
                FieldRole::Address,
                ProblemKind::Missing,
                "Enter the email address.",
            ),
            (
                FieldRole::AppPassword,
                ProblemKind::Refused,
                "The app password was not accepted. Check it and try again.",
            ),
            (
                FieldRole::OutgoingPort,
                ProblemKind::Invalid,
                "The outgoing port is not valid. Check it and try again.",
            ),
        ];
        for (role, kind, want) in cases {
            assert_eq!(
                problem_text(FieldProblem {
                    role,
                    kind,
                    attempt: Attempt::default(),
                }),
                want
            );
        }
    }

    #[test]
    fn the_arrows_walk_the_listed_rows_and_stop_at_the_ends() {
        let rows = [
            ProviderPick::Provider(ProviderKey("a".to_owned())),
            ProviderPick::Provider(ProviderKey("b".to_owned())),
            ProviderPick::Other,
        ];
        let go = |from: Option<&ProviderPick>, step| step_cursor(&rows, from, step);
        assert_eq!(go(None, CursorStep::Next).as_ref(), rows.first());
        assert_eq!(go(None, CursorStep::Previous).as_ref(), rows.last());
        assert_eq!(go(rows.first(), CursorStep::Next).as_ref(), rows.get(1));
        assert_eq!(go(rows.last(), CursorStep::Next).as_ref(), rows.last());
        assert_eq!(
            go(rows.first(), CursorStep::Previous).as_ref(),
            rows.first()
        );
        assert_eq!(step_cursor(&[], None, CursorStep::Next), None);
    }
}
