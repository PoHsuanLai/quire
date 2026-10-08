//! What the account sheets are given and what they report: plain values, no porter types. A host
//! maps its sheet view to these in one table-tested file (design/31 section 5.1).

use super::hidden::Hidden;
use ds::components::content::provider_mark::{MarkProvider, MarkStyle};
use ds_core::vocab::Check;
use ds_core::word::Word;

/// One account in a chooser, named by the host.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChoiceKey(pub String);

/// One provider in the list, named by the host.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProviderKey(pub String);

/// One service (a kind of capability) on the review, named by the host.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServiceKey(pub String);

/// An account the person may pick.
#[derive(Debug, Clone, PartialEq)]
pub struct AccountChoice {
    /// What the host calls it.
    pub key: ChoiceKey,
    /// What the person reads: the address or the name.
    pub label: String,
    /// Whose mark it wears.
    pub provider: MarkProvider,
}

/// A provider the person may sign in to.
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderEntry {
    /// What the host calls it.
    pub key: ProviderKey,
    /// What the person reads ("Nextcloud").
    pub label: String,
    /// Whose mark it wears.
    pub mark: MarkProvider,
    /// Its letter or its favicon: `MarkStyle::Letter` unless the host holds the provider's image.
    pub style: MarkStyle,
}

impl ProviderEntry {
    /// An entry that wears its provider's letter.
    pub fn new(key: ProviderKey, label: impl Into<String>, mark: MarkProvider) -> Self {
        ProviderEntry {
            key,
            label: label.into(),
            mark,
            style: MarkStyle::Letter,
        }
    }

    /// The same entry wearing `style`.
    pub fn styled(self, style: MarkStyle) -> Self {
        ProviderEntry { style, ..self }
    }
}

/// What the person picked in the provider list.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProviderPick {
    /// A provider of the list.
    Provider(ProviderKey),
    /// The generic "Other" row: any server that speaks the open protocols.
    Other,
}

/// What a field is for; the sheet words it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum FieldRole {
    /// The account's address.
    Address,
    /// A server the lookup did not find.
    Server,
    /// A user name that is not the address.
    Username,
    /// A password or an app password.
    Password,
    /// An API key.
    #[word(slug = "api-key")]
    ApiKey,
    /// An API token.
    Token,
    /// Which protocol the server speaks (a choice: IMAP, POP3, JMAP).
    Protocol,
    /// The incoming server's port; empty means the usual one.
    Port,
    /// How the incoming connection is secured (a choice: TLS, STARTTLS).
    Security,
    /// The server mail is sent through.
    #[word(slug = "outgoing-server")]
    OutgoingServer,
    /// The outgoing server's port; empty means the usual one.
    #[word(slug = "outgoing-port")]
    OutgoingPort,
    /// How the outgoing connection is secured (a choice: TLS, STARTTLS).
    #[word(slug = "outgoing-security")]
    OutgoingSecurity,
    /// The address of a JMAP session resource.
    #[word(slug = "session-url")]
    SessionUrl,
}

/// Whether the person may leave a field empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Requirement {
    /// The sign-in cannot go on without it.
    Required,
    /// May stay empty.
    Optional,
}

/// What a field holds. The variant says whether the field hides its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldText {
    /// Shown as typed.
    Plain(String),
    /// Hidden as typed; its `Debug` never prints it.
    Secret(Hidden),
}

impl FieldText {
    /// Whether nothing, or only blanks, is typed.
    pub fn is_blank(&self) -> bool {
        match self {
            FieldText::Plain(text) => text.trim().is_empty(),
            FieldText::Secret(text) => text.is_empty(),
        }
    }
}

/// One option of a choice field. The host words it; the slug is what the host's model knows it by.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// What the host calls it ("imap", "starttls"): exactly what an edit of the field reports.
    pub slug: String,
    /// What the person reads ("IMAP", "STARTTLS").
    pub label: String,
}

impl Choice {
    /// An option called `slug` and read as `label`.
    pub fn new(slug: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            slug: slug.into(),
            label: label.into(),
        }
    }
}

/// A titled part of a long form. Neighbouring fields of one part sit in one group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum FormPart {
    /// The server mail is read from.
    Incoming,
    /// The server mail is sent through.
    Outgoing,
    /// Who the person is to the server.
    #[word(slug = "sign-in")]
    SignIn,
}

/// One field of the sign-in form. The host owns what is typed and hands it back on every render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormField {
    /// What it is for.
    pub role: FieldRole,
    /// Whether it may stay empty.
    pub requirement: Requirement,
    /// What it holds now. A choice field holds the chosen slug as [`FieldText::Plain`].
    pub text: FieldText,
    /// The options, when the field is a choice (a pop-up button); `None` for a text entry.
    pub choices: Option<Vec<Choice>>,
    /// The hint in an empty entry (a port's usual number); `None` keeps the role's own words.
    pub hint: Option<String>,
    /// The group it sits in; a form where no field names one is a single group.
    pub part: Option<FormPart>,
}

impl FormField {
    /// A text entry with no choices, hint or group.
    pub fn new(role: FieldRole, requirement: Requirement, text: FieldText) -> Self {
        Self {
            role,
            requirement,
            text,
            choices: None,
            hint: None,
            part: None,
        }
    }

    /// The same field as a choice among `choices`.
    pub fn choosing(self, choices: Vec<Choice>) -> Self {
        Self {
            choices: Some(choices),
            ..self
        }
    }

    /// The same field with `hint` in its empty entry.
    pub fn hinted(self, hint: impl Into<String>) -> Self {
        Self {
            hint: Some(hint.into()),
            ..self
        }
    }

    /// The same field placed in `part`.
    pub fn in_part(self, part: FormPart) -> Self {
        Self {
            part: Some(part),
            ..self
        }
    }
}

/// What is wrong with a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum ProblemKind {
    /// A required field was left empty.
    Missing,
    /// The server refused what was typed.
    Refused,
    /// What was typed cannot be right (a port that is not a number).
    Invalid,
}

/// How many times the form has been submitted: a password field shakes once for each new
/// attempt that is refused, and not again for the same one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Attempt(pub u32);

/// A field to mark, and why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FieldProblem {
    /// Which field.
    pub role: FieldRole,
    /// What is wrong.
    pub kind: ProblemKind,
    /// The submit this answers.
    pub attempt: Attempt,
}

/// How often an allow lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum AllowScope {
    /// This time only.
    Once,
    /// From now on.
    Always,
    /// For this session only.
    Session,
}

/// Whether the consent alert offers "This Session Only" between Allow Once and Always Allow: a
/// host offers it when the service can be granted for one launcher session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum SessionOffer {
    /// Two allows: once and always.
    #[default]
    None,
    /// Three allows: once, this session only and always.
    Offered,
}

/// What the person answered on the consent alert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentAnswer {
    /// "Allow Once", "This Session Only" or "Always Allow", with the account picked.
    Allow {
        /// The account.
        account: ChoiceKey,
        /// For how long.
        scope: AllowScope,
    },
    /// "Don't Allow": the host stores a denial.
    Deny,
    /// Escape: the sheet was closed and nothing is stored.
    Dismiss,
    /// "Add Account...", offered when no account fits.
    AddAccount,
}

/// Why an account, a service or a sign-in is less than the provider's full product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Limitation {
    /// Uploads only; existing items are not readable.
    #[word(slug = "append-only")]
    AppendOnly,
    /// Existing items only through the provider's own picker.
    #[word(slug = "picker-only")]
    PickerOnly,
    /// Only files this desktop created.
    #[word(slug = "app-folder-only")]
    AppFolderOnly,
    /// The provider offers no access.
    #[word(slug = "provider-offers-none")]
    ProviderOffersNone,
    /// The organisation's administrator has to consent first.
    #[word(slug = "admin-consent")]
    AdminConsent,
    /// This build of the desktop is not verified by the provider yet.
    #[word(slug = "unverified-build")]
    UnverifiedBuild,
    /// The person turned it off for this account.
    #[word(slug = "turned-off")]
    TurnedOff,
    /// The server does not have it.
    #[word(slug = "not-on-server")]
    NotOnServer,
    /// The sign-in lasts seven days.
    #[word(slug = "seven-days")]
    SevenDays,
    /// The sign-in lasts until the provider password changes.
    #[word(slug = "until-password-change")]
    UntilPasswordChange,
}

/// Whether the account can use a service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceOffer {
    /// The provider offers it; the person may switch it.
    Offered(Check),
    /// The provider offers none, for this reason.
    Absent(Limitation),
}

/// One service row of the review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceLine {
    /// What the host calls it.
    pub key: ServiceKey,
    /// What the person reads ("Mail", "Calendar").
    pub name: String,
    /// On, off, or why it cannot be.
    pub offer: ServiceOffer,
    /// What limits it while it is on.
    pub limit: Option<Limitation>,
}

/// What the person did on the account chooser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickerChoice {
    /// One of the accounts.
    Account(ChoiceKey),
    /// "Add Account...".
    Add,
}

/// Why an app has no account to use (design/31 section 5.1, `Found::None`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum NoAccountWhy {
    /// There is none yet: the person may add one.
    #[word(slug = "needs-account")]
    NeedsAccount,
    /// The person said no: only Settings changes it.
    Denied,
    /// This provider cannot do what the app needs.
    Unsupported,
}

/// Whether the link or code has been copied, as the host knows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum CopyState {
    /// Nothing copied lately.
    #[default]
    Idle,
    /// The host just copied it: the button says so.
    Copied,
}

/// Why a sign-in ended without an account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum SignInFault {
    /// The provider refused what was typed.
    Refused,
    /// The server could not be reached.
    Unreachable,
    /// The server answered something unreadable.
    Unreadable,
    /// This build has no client registered with the provider.
    #[word(slug = "needs-client-id")]
    NeedsClientId,
    /// The person did not finish in time.
    #[word(slug = "timed-out")]
    TimedOut,
    /// The person closed the step.
    Cancelled,
    /// The organisation or the provider forbids it.
    Forbidden,
    /// It worked, but the account could not be kept.
    #[word(slug = "store-failed")]
    StoreFailed,
    /// An agent login has no app set up to run it.
    #[word(slug = "no-launcher")]
    NoLauncher,
    /// The agent the login runs through is not installed.
    #[word(slug = "not-installed")]
    NotInstalled,
}

/// Who draws a step's title.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StepTitle {
    /// The step, above its body.
    #[default]
    Own,
    /// The host's window, in its title bar: the step draws none.
    Host,
}
