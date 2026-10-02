<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/actions.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# Area "actions": the typed action layer and its safety core

Status: a proposal for the user to lock. Sources read: BRIEF.md, COMPANION.md (items 1-11, Security, Build map), cua-integration.md, research-ecosystem-infra.md, research-ecosystem-agents.md, research-os-memory.md §8, intents-research/models.md and agent-ux.md, quire design/31-ACCOUNTS.md (§2, §4, §5.5, §11), 30-CATALOGUE.md §3.4, 21-SPACES.md, quire CONVENTIONS.md, porter (ARCHITECTURE, CONVENTIONS, FINDINGS, `porter-core` consent/app_id/data_class/wire, `porter-dbus`, `check-boundary.sh`), sill-launcher (`provider.rs`, `item.rs`, `activation.rs`, `subject.rs`, `query.rs`, `sink.rs`, `ids.rs`, `preview.rs`), mailo `mail-app/src/undo.rs`, quire `ds` host seams and `toast_hub::UndoToken`.

This version includes the coordinator's scope addition, the **auto-mode action reviewer** (§3.9, §4.4, §5.6).

---

## 1. Scope and non-goals

**In scope**
- The vocabulary: Entity, Action, Param, Value, Effect, Preview, Context, Invocation, Outcome, Refusal. Also the app manifest file and its validation.
- The router `org.quire.Intents1` (daemon `intentd`) and the per-app `org.quire.IntentProvider1`. This covers D-Bus activation, the pushed entity index and the caller identity the transport derives.
- The Context call. It reports where the user is, the selection as typed things, what is visible, and the editable text target. The quire layer answers it on the app's behalf.
- The shared undo history, labelled by actor: app stacks plus the router's journal.
- `prov`: the provenance and taint label lattice (FIDES-style integrity × confidentiality).
- `policy-point`: a Cedar wrapper. It holds the entity schema, the request shape and a decision of Deny, Allow, Review or Ask.
- `action-review`: the auto-mode reviewer that judges grey-zone actions.
- Consent grants for actions. This extends porter's `GrantKey` with a Space dimension and makes porter's `decide` generic over its key.
- The planner/reader split contract. It says what the planner may see, and that the reader returns typed values only. The router owns the handle table, so nobody can forge taint.
- The confirmation request and answer as values, plus the `Confirmer` seam. Drawing them is not ours.
- Budgets, the per-Space kill switch, and the audit records handed to the event log.
- `actions-mcp`: an MCP server generated from the registry, used at the edge only.
- How sill's launcher `Provider` relates to all of this.

**Not in scope** (owner area in brackets)
- The agent loop and planner prompting, session UX and the orb [planner/agent-loop, ux].
- Drawing confirmations, the compositor protocol and the kill chord's key handling [cua, ux].
- The CUA step loop and pixel actions [cua]. They pass through our gate as a caller kind.
- Model serving, the grammar and the choice of reviewer or reader model [models].
- Event-log storage and its hash chain, memory files and the recall index [memory].
- Sandboxing and egress [security, if it is an area; else cua].
- App-specific actions such as mailo's verbs (mailo session, fill wave).

---

## 2. Repos, crates, dependency direction

### 2.1 New repo `docket` (working name; question U1), at `~/docket`

It mirrors porter's layout, conventions, gate and `check-boundary.sh`.

| Crate | Purpose | I/O | Portable |
|---|---|---|---|
| `prov` | `Label`, `Integrity`, `Confidentiality`, `Source`, `Labelled<T>`, `join`, `Quarantined<T>`, `Witness` | none | yes; reusable by cua, memory and eventlog |
| `docket-core` | the vocabulary (ids, manifest, effect, params and values, context, preview, invocation, outcome, refusal, undo, confirm, budget, halt, planner/reader contract, authorization), `validate`, `tool_schema`, the wire enums and frames | none | yes |
| `policy-point` | Cedar schema, default policies, `Pdp::decide`, policy tests | none (cedar-policy only) | yes |
| `action-review` | `ReviewRequest`, `Verdict`, prompt rendering with quoted data, verdict parsing, the `Reviewer` trait, `combine`; `InferReviewer` (stub) over porter-infer's `Model` | none | yes |
| `docket-router` | the pure router: registry, `gate`, session and handle table, undo journal, budgets and halt, index model, authorizations. Seams: `AppLink`, `Confirmer`, `Reviewer`, `GrantStore`, `EventSink`, `Clock` | none (seams passed in) | yes |
| `docket-dbus` | zbus proxies and skeletons for `org.quire.Intents1`, `org.quire.IntentProvider1`, `org.quire.Confirm1`; `dbus/*.xml` | zbus | Linux |
| `docket-client` | the app side (the `IntentProvider` trait, `serve`) and the caller side (`Intents` over a `Transport`: `InProcess`, and `DbusTransport` behind feature `dbus`) | per transport | yes |
| `docket-fake` | test only: FakeMail and FakeFiles providers from real manifest files, `ScriptedConfirmer`, `ScriptedReviewer`, `FixedClock`, `RecordingSink`, `fake_router` | none | yes |
| `actions-mcp` | an rmcp server generated from the registry. Every call goes through the router as `Actor::Mcp`. Importing third-party MCP tools comes later. | rmcp, its own binary | yes |
| `intentd` | the daemon: builds the router over its seams and serves the bus. Until then it is a skeleton that exits 2 with "not implemented". | everything | Linux first |

**Allowed edges** (enforced by `scripts/check-boundary.sh`):

| Crate | May depend on |
|---|---|
| `prov` | `porter-core` |
| `docket-core` | `prov`, `porter-core` |
| `policy-point` | `docket-core`, `prov` (+ `cedar-policy`) |
| `action-review` | `docket-core`, `prov`, `porter-infer` |
| `docket-router` | `docket-core`, `prov`, `policy-point`, `action-review`, `porter-core` |
| `docket-dbus` | `docket-core` |
| `docket-client` | `docket-core`, `docket-router` (for InProcess), `docket-dbus` (feature `dbus`) |
| `docket-fake` | `docket-core`, `docket-router`, `docket-client`, `policy-point`, `action-review`, `porter-core` |
| `actions-mcp` | `docket-core`, `docket-client` (+ `rmcp`) |
| `intentd` | `docket-core`, `docket-router`, `docket-dbus`, `policy-point`, `action-review`, `porter-core` |

**External boundaries**
- No crate except `docket-dbus`, `intentd` and `actions-mcp` ever reaches `zbus`, `zvariant`, `tokio`, `reqwest` or `hyper`.
- Only `policy-point` (and the crates above it) reaches `cedar-policy`.
- Only `actions-mcp` reaches `rmcp`.
- `prov` and `docket-core` never reach `toml`. Manifest TOML is parsed in `docket-router::registry::parse`.

**Sibling path dependencies:** `porter-core` and `porter-infer` (`../porter/crates/...`).

**Third-party, joining quire's `docs/workspace-deps.toml` first** (CONVENTIONS §10):
- `cedar-policy = "4.13"` (4.13.0, 2026-09-15, Apache-2.0)
- `rmcp = { version = "3.5", default-features = false, features = ["server", "transport-io"] }` (3.5.0, 2026-09-28). Pin the minor; 3.x churns.
- Already in the pinned block: `serde`, `serde_json`, `thiserror 2`, `toml 1`, `zbus 5.19` (tokio), `tokio 1`.
- Settled first: `cargo deny check licenses` against cedar's tree, which is lalrpop-based and heavier than ours [unverified size].

### 2.2 Changes in existing repos

| Repo | Change | Wave |
|---|---|---|
| porter | `porter-core`: new `space.rs` (`SpaceId`, `SpaceScope`); `GrantKey.space: SpaceScope`; consent generic over its key (`Grant<K = GrantKey>`, `decide<K: Eq>`). A design/31 §4.5 edit in step. | freeze |
| quire | Add `cedar-policy` and `rmcp` to `docs/workspace-deps.toml`. Write `design/33-ACTIONS.md` from this file. Edit 31 §4.5 (Space in the key) and §5.5 (inferd is no longer the MCP host, see C4). | freeze |
| quire | New crate `ds-intents`: the `ContextModel` gathered from components (`thing` prop on `Row`/`List`, edit surfaces as text targets), answered through `docker-client`. Blocked on question U6 (30 §3.4 defers companion work). | fill 2 |
| sill | `sill-launcher` gains `ProviderKind::Things`, `Subject::Entity`, `Activation::Intent`, `Preview::Intent`. `sill-search`/`sill-services` gain a `ThingsProvider` over the router. sill serves `IntentProvider1` as `org.quire.Shell` (launch, focus window, open file) by reusing its `Activation` executor. | fill 2 |
| mailo | First real provider (archive, move, label, draft, send). `UndoStack` entries gain `actor: Actor`. | fill 2 (mailo session) |

Correction to the table above: the crate `ds-intents` uses `docket-client`, not "docker-client".

---

## 3. Frozen interfaces

Rules, following porter:
- No `bool`; newtypes everywhere; adjacently tagged serde (`kind`/`v`) and snake_case slugs; derive order per CONVENTIONS §12.
- `#[derive]` lines are omitted below for brevity. Every public type is `Debug, Clone, PartialEq, Eq` (+ `Serialize, Deserialize` if it is stored or crosses a wire).
- Redacting `Debug` on every type that holds user text.
- Async seams use `-> impl Future<Output = …> + Send`.

### 3.1 `prov`: labels (freeze now)

```rust
/// FIDES-style integrity. Ordered: Untrusted < Trusted; join takes the minimum.
pub enum Integrity { Untrusted, Trusted }

/// Who may see it. Ordered lattice: Public < Private(spaces) < Secret; join takes the maximum,
/// and Private sets union (data from two Spaces may flow only where both may).
pub enum Confidentiality { Public, Private(BTreeSet<SpaceId>), Secret }

/// Where a value came from: what the confirmation sheet and the reviewer are told.
pub enum Source {
    User,                    // typed or chosen by the person (launcher, field, confirm sheet)
    App(AppName),            // app-authored metadata (labels, enum fields, ids, counts)
    Mail, Web, File, Screen, Clipboard, Calendar, Contacts, Notes,  // third-party-authored content
    Model(ModelRole),        // text a model produced (planner, reader, reviewer)
    Mcp(ClientName),         // an external MCP client's arguments
}
pub enum ModelRole { Planner, Reader, Reviewer, Cua }

pub struct Label {
    pub integrity: Integrity,
    pub confidentiality: Confidentiality,
    pub classes: BTreeSet<DataClass>,   // porter_core::DataClass
    pub sources: BTreeSet<Source>,
}
impl Label {
    pub fn trusted_user() -> Label;
    pub fn untrusted(source: Source, class: DataClass, space: SpaceId) -> Label;
    pub fn join(&self, other: &Label) -> Label;           // the only combiner
}
pub struct Labelled<T> { pub value: T, pub label: Label }
impl<T> Labelled<T> {
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Labelled<U>;   // keeps the label
    pub fn zip<U>(self, other: Labelled<U>) -> Labelled<(T, U)>;  // joins the labels
}

/// Untrusted content no planner may read. `Debug` prints length and sources only.
/// Type-level only: the real wall is the process boundary (§3.8).
pub struct Quarantined<T>(Labelled<T>);
impl<T> Quarantined<T> { pub fn new(v: Labelled<T>) -> Self; pub fn label(&self) -> &Label;
                         pub fn open(self, _key: &ReaderKey) -> Labelled<T>; }
pub struct ReaderKey(());                                // minted by the reader host only

/// Raising integrity or lowering confidentiality needs a witness.
pub enum Witness { UserConfirmed(ConfirmReceipt) }      // docket-core re-exports ConfirmReceipt's home here
pub fn endorse<T>(v: Labelled<T>, w: &Witness) -> Labelled<T>;     // integrity -> Trusted, sources += User
pub fn declassify<T>(v: Labelled<T>, to: Confidentiality, w: &Witness) -> Labelled<T>;
```

`ConfirmReceipt` lives in `prov` because `Witness` needs it and `prov` sits below docket-core. Its fields are listed in §3.7.

### 3.2 `porter-core` additions (freeze now; a porter change)

```rust
/// A Space's stable id, minted by quire when the Space is made (not the COSMIC workspace id,
/// which may not survive a session: 21-SPACES §11.3).
pub struct SpaceId(String);                      // id grammar like AccountId; Hash, Ord
pub enum SpaceScope { Any, Only(SpaceId) }

pub struct GrantKey { pub app: AppId, pub account: AccountId, pub kind: CapabilityKind,
                      pub class: DataClass, pub usage: Usage, pub space: SpaceScope }   // + space
pub struct Grant<K = GrantKey> { pub id: GrantId, pub key: K, pub decision: Decision,
                                 pub scope: GrantScope, pub at: UnixSeconds }
pub fn decide<K: Eq>(grants: &[Grant<K>], key: &K) -> Verdict;   // body unchanged
```

### 3.3 `docket-core`: identities and the manifest (freeze now)

```rust
pub struct EntityKind(String);   // "mail.thread": 2+ dotted [a-z][a-z0-9_]*; parse() -> Result
pub struct EntityKey(String);    // app-local, stable, opaque; ≤ 512 bytes
pub struct EntityId { pub app: AppName, pub kind: EntityKind, pub key: EntityKey }
pub struct ActionName(String);   // "mail.thread.archive": same grammar
pub struct ActionRef { pub app: AppName, pub name: ActionName }
pub struct ParamName(String);    // [a-z][a-z0-9_]*
pub struct ChoiceId(String);
pub struct CallId(u64); pub struct SessionId(String); pub struct RunId(String);
pub struct UndoToken(String);    // app-minted, opaque to the router
pub struct UndoId(u64);          // the router journal's
pub struct Handle(u64);          // router-minted reference to a value the planner may not read
pub struct TurnId(u64);          // one user prompt in a session, router-held
pub struct LabelText(String);    // app-authored UI words from the manifest (trusted)
pub struct IconName(String);
pub struct IntentsVocab(pub u32); impl IntentsVocab { pub const CURRENT: IntentsVocab = IntentsVocab(1); }

/// `$XDG_DATA_DIRS/quire/intents/<AppName>.toml`, installed beside the app's
/// `dbus-1/services/<AppName>.service` (DBusActivatable). Its serde form is the file format;
/// every field is written (porter's provider-file rule).
pub struct Manifest {
    pub vocab: IntentsVocab,
    pub app: AppName,
    pub entities: Vec<EntityDecl>,
    pub actions: Vec<ActionDecl>,
}
pub struct EntityDecl {
    pub kind: EntityKind, pub label: LabelText, pub plural: LabelText, pub icon: IconName,
    pub class: DataClass,
    pub index: IndexPolicy,
    pub titles: TitleTrust,                        // who writes this kind's titles
    pub props: Vec<PropDecl>,
}
pub enum IndexPolicy { NotIndexed, Indexed }
pub enum TitleTrust { AppAuthored, ThirdParty(Source) }   // mail subjects: ThirdParty(Mail)
pub struct PropDecl { pub name: ParamName, pub label: LabelText, pub ty: ParamType }

pub struct ActionDecl {
    pub name: ActionName,
    pub label: LabelText,                          // "Archive"
    pub on: Target,
    pub params: Vec<ParamDecl>,
    pub effect: Effect,
    pub classes: BTreeSet<DataClass>,
    pub undo: UndoSupport,
    pub reach: AgentReach,
    pub latency: Latency,
    pub result: ResultShape,
    pub keys: KeyHint,
    pub lasting: Lasting,                          // writes lasting agent memory?
}
/// COMPANION item 9. Ordered by severity.
pub enum Effect { Read, UndoableWrite, Outbound, Destructive }
pub enum Target { Nothing, One(EntityKind), Many(EntityKind), Text, Files }
pub enum UndoSupport { Token, NotUndoable }
pub enum AgentReach { Hidden, Offered, AskAlways }
pub enum Latency { Instant, Quick, Long }           // router timeouts: 250 ms, 5 s, progress Request
pub enum ResultShape { Nothing, Value(ParamType), Entities(EntityKind) }
pub enum KeyHint { None, Chord(String) }            // sill parses it into ActionKeys
pub enum Lasting { No, AgentMemory }

pub struct ParamDecl { pub name: ParamName, pub label: LabelText, pub ty: ParamType, pub need: Need }
pub enum Need { Required, Optional, Defaulted(Value) }
pub enum ParamType {
    Text { max: CharCount, lines: Lines },
    Integer { min: i64, max: i64 },
    Decimal { scale: Scale },                      // no floats in data (CONVENTIONS §12)
    Date, DateTime, Duration,
    Choice(Vec<ChoiceDecl>),
    Entity(EntityKind), Entities(EntityKind),
    File, Url,
    Dynamic(ActionName),                           // options from a Read action via Suggest
}
pub enum Lines { One, Many }
pub struct ChoiceDecl { pub id: ChoiceId, pub label: LabelText }

/// Validation, pure. Rules: Read ⇒ NotUndoable; UndoableWrite ⇒ Token (a write without undo
/// must be declared Destructive); names unique; Target and Entity params name declared kinds
/// (or kinds of other installed apps, checked by the registry); Defaulted value fits its type.
pub fn validate(m: Manifest) -> Result<ValidManifest, ManifestError>;
pub struct ValidManifest(Manifest);
pub enum ManifestError {
    VocabTooNew(IntentsVocab), DuplicateAction(ActionName), DuplicateKind(EntityKind),
    UnknownKind { action: ActionName, kind: EntityKind },
    WriteWithoutUndo(ActionName), ReadWithUndo(ActionName),
    DefaultOutOfType { action: ActionName, param: ParamName },
    ActionOutsideApp(ActionName),                   // names must start with the app's own prefix? (see U1 note)
}

/// JSON Schema for an action's arguments: the one source for the planner's grammar and MCP.
pub struct ToolSchema(pub serde_json::Value);
pub fn tool_schema(a: &ActionDecl) -> ToolSchema;
```

### 3.4 `docket-core`: values, context, preview (freeze now; preview shapes partly later)

```rust
pub enum Value {
    Text(String), Integer(i64), Decimal(Decimal), Date(CivilDate), DateTime(UnixSeconds),
    Duration(Seconds), Choice(ChoiceId), Entity(EntityId), Entities(Vec<EntityId>),
    File(FileRef), Url(String),
    Handle(Handle),                                // resolved by the router, label preserved
    List(Vec<Value>), Record(BTreeMap<ParamName, Value>),
}
pub type Args = BTreeMap<ParamName, Labelled<Value>>;
pub enum TargetValue { Nothing, Entities(Vec<EntityId>), Text(TextTargetRef), Files(Vec<FileRef>) }
pub struct FileRef(String);                        // a document-portal-style token later; path text in v1

/// What the user is looking at in one window: the Context call's answer.
pub struct ContextSnapshot {
    pub app: AppName,
    pub window: Labelled<String>,                  // title
    pub here: Here,
    pub selection: Selection,
    pub visible: Visible,
    pub text_target: TextTarget,
    pub privacy: WindowPrivacy,                    // Private ⇒ the router drops everything but app
}
pub enum Here { Nowhere, Entity(EntityRef), View { view: ViewName, query: Option<Labelled<String>> } }
pub enum Selection { Nothing, Entities { kind: EntityKind, items: Vec<EntityRef> },
                     Text(Labelled<String>), Files(Vec<FileRef>) }
pub struct Visible { pub kind: Option<EntityKind>, pub items: Vec<EntityRef>, pub total: Count }
pub enum TextTarget { None, Field(EditTarget) }
pub struct EditTarget { pub field: TextTargetRef, pub purpose: TextPurpose,
                        pub selection: CharRange, pub around: Labelled<String> }
pub enum TextPurpose { Plain, Rich, Code, Search }     // Password/Pin fields are never reported
pub struct TextTargetRef(String);                      // app-minted, valid while the field lives
pub struct EntityRef { pub id: EntityId, pub title: Labelled<String>, pub subtitle: Labelled<String> }
pub enum WindowPrivacy { Normal, Private }
pub enum ContextScope { ActiveWindow }                 // later: Window(ForeignHandle)

/// Closed vocabulary the host draws with quire components; apps send data only.
pub enum Preview {
    None,
    Text { heading: Labelled<String>, body: Labelled<Markdownish> },
    Facts(Vec<Fact>),
    Person { name: Labelled<String>, lines: Vec<Fact> },
    Thread { subject: Labelled<String>, messages: Vec<MessageSnip> },   // capped at 5
    Event { title: Labelled<String>, when: TimeRange, place: Option<Labelled<String>> },
    Image(FileRef), Pdf { file: FileRef, page: PageIndex }, File(FileFacts),
    List(Vec<EntityRef>),
    TextChange { before: Labelled<String>, after: Labelled<String> },   // P1 inline replace
}
// later (vocab bump): Plan (P4 rows), DraftReply, ProposedEvent cards.
```

### 3.5 `docket-core`: calls, outcomes, refusals, undo (freeze now)

```rust
/// Who acts. Set by the router from the caller's derived identity, never self-asserted.
pub enum Actor { User, Companion { session: SessionId }, Cua { run: RunId }, Mcp { client: ClientName }, App(AppName) }
pub enum Origin { Launcher, InWindowField, Companion, Shortcut, Mcp, AppInternal }

/// What the router sends to IntentProvider1.Perform. The app sees actor and origin (to label
/// undo), never the caller's bus identity.
pub struct Invocation {
    pub call: CallId, pub action: ActionName, pub target: TargetValue, pub args: Args,
    pub actor: Actor, pub origin: Origin, pub space: SpaceId,
}
/// What a caller asks the router for (the app is in `action`).
pub struct CallRequest { pub action: ActionRef, pub target: TargetValue, pub args: Args, pub origin: Origin }

pub struct Outcome {
    pub value: Option<Labelled<Value>>,
    pub said: Option<LabelText>,                   // "Archived 3 threads" (app-authored)
    pub show: Preview,
    pub undo: Undoable,
    pub follow: Follow,
}
pub enum Undoable { No, Yes(UndoToken) }
pub enum Follow { Nothing, Open(EntityId), Next(CallRequest) }   // the router caps chains at depth 4

/// The app's own refusals.
pub enum AppRefusal { NeedsParam { param: ParamName, options: Vec<EntityRef> }, NotFound(EntityId),
                      Stale(EntityId), Busy, Unsupported, Failed(FailText) }
/// Everything a caller can get back instead of an Outcome.
pub enum CallRefusal {
    App(AppRefusal),
    Denied(DenyReason),            // policy, consent or reviewer; carries the reason for the planner
    Unconfirmed(ConfirmEnd),       // the person said no, dismissed it, or it expired
    Halted(SpaceScope),
    OverBudget(BudgetKind),
    NoSuchAction(ActionRef), BadArgs { param: ParamName, why: ArgFault },
    AppUnavailable(AppName), Timeout,
}
pub enum ArgFault { Missing, WrongType, OutOfRange, UnknownHandle, CrossSpace }

/// The router's journal row; the app's own stack stays the truth for Cmd+Z.
pub struct UndoEntry {
    pub id: UndoId, pub at: UnixSeconds, pub actor: Actor, pub app: AppName,
    pub action: ActionName, pub said: LabelText, pub token: UndoToken,
    pub run: Option<RunId>, pub session: Option<SessionId>, pub state: UndoState,
}
pub enum UndoState { Available, Undoing, Undone { by: Actor }, Failed(UndoFault), Expired }
pub enum UndoFault { Gone, Conflict, AppUnavailable }
```

### 3.6 `docket-core`: consent for actions (freeze now)

```rust
pub struct ActionGrantKey {
    pub caller: ActorKind,                         // Companion | Cua | Mcp(ClientName) | App(AppName)
    pub owner: AppName,
    pub target: GrantTarget,
    pub class: DataClass,
    pub usage: Usage,                              // porter_core
    pub space: SpaceScope,                         // porter_core
}
pub enum GrantTarget { App, Action(ActionName) }   // "the companion may use Mail" vs one verb
pub enum ActorKind { User, Companion, Cua, Mcp(ClientName), App(AppName) }
pub type ActionGrant = porter_core::consent::Grant<ActionGrantKey>;
```

Defaults:
- First use by the companion, per (owner app, class, Space), asks through the `Confirmer` with Once/Always.
- An `Always` grant never covers `Destructive`.
- Taint voids `Always` for `Outbound` (enforced in Cedar).

### 3.7 `docket-core`: confirmation, budgets, halt (freeze now)

```rust
/// What the compositor draws. Every word comes from the manifest or the router's own
/// formatting of typed values; model text never appears. Untrusted values are marked.
pub struct ConfirmRequest {
    pub id: ConfirmId, pub space: SpaceId, pub actor: Actor, pub app: AppName,
    pub action: LabelText, pub effect: Effect, pub count: Count,
    pub lines: Vec<ArgLine>,
    pub why: Vec<AskReason>,                       // shown as short fixed phrases
    pub taint: TaintNote,
    pub offer: ConfirmOffer,
    pub anchor: Anchor,
    pub expires: Seconds,                          // setting agent.confirm.expiry_s, proposed 120
}
pub struct ArgLine { pub label: LabelText, pub value: Shown }
pub enum Shown { Plain(String), Quoted { text: String, from: Source } }   // Quoted is drawn as data
pub enum TaintNote { Clean, ReadUntrusted(BTreeSet<Source>) }
pub enum ConfirmOffer { OnceOnly, OnceOrAlways }
pub enum Anchor { Window(WindowKey), Launcher, Centre }
pub enum ConfirmAnswer { Allowed { scope: GrantScope, receipt: ConfirmReceipt }, Ended(ConfirmEnd) }
pub enum ConfirmEnd { Refused, Dismissed, Expired, Cancelled }
pub struct ConfirmReceipt { pub id: ConfirmId, pub input: InputProof, pub at: UnixSeconds }   // home: prov
pub enum InputProof { HardwareSeat, SheetFallback }   // SheetFallback = no compositor surface yet (C3)

pub trait Confirmer: Send + Sync {
    fn confirm(&self, request: ConfirmRequest) -> impl Future<Output = ConfirmAnswer> + Send;
    fn cancel(&self, id: &ConfirmId) -> impl Future<Output = ()> + Send;
}

pub struct Budget {             // every value a setting `agent.budget.*` (proposed defaults)
    pub calls: Count,           // 200 per session
    pub writes: Count,          // 100
    pub outbound: Count,        // 10
    pub destructive: Count,     // 5
    pub per_minute: Count,      // 30
    pub fan_out: Count,         // 50 entities per call
    pub chain: Depth,           // 4
    pub wall: Seconds,          // 1800
    pub reviews: Count,         // 300
    pub denials_in_a_row: Count,// 3, then the session pauses for the user
    pub spend: MicroUsd,        // shared with inferd's SpendCap
}
pub enum BudgetKind { Calls, Writes, Outbound, Destructive, Rate, FanOut, Chain, Wall, Reviews, Denials, Spend }
pub struct Ledger { /* used counts per kind, window start */ }
pub fn charge(ledger: &Ledger, budget: &Budget, cost: &Cost, now: UnixSeconds) -> Result<Ledger, BudgetKind>;
pub struct Cost { pub effect: Effect, pub entities: Count, pub depth: Depth, pub review: Reviewed }
pub enum Reviewed { No, Yes }

pub enum Halt { Running, Halted { since: UnixSeconds, by: HaltCause } }
pub enum HaltCause { KillChord, ControlCentre, Esc, Budget(BudgetKind), Denials }
pub struct KillSwitch { pub all: Halt, pub spaces: BTreeMap<SpaceId, Halt> }
pub fn halted(k: &KillSwitch, space: &SpaceId) -> Option<HaltCause>;   // all overrides spaces
```

### 3.8 `docket-core`: the planner/reader contract (freeze now)

Rules:
- The router owns a per-session `HandleTable`. When the router hands anything to a companion session (context, search hits, Read outcomes, previews), every `Labelled<String>` whose integrity is `Untrusted` is replaced by a `Handle`. Plain text reaches only the **reader identity**.
- The reader is a separate process with its own bus name. `intentd` is configured with that identity. Only the reader may call `Session.Resolve`.
- The router computes the session's planner taint itself, as the join of the labels of every plain value it delivered. A model cannot forge it.

```rust
/// Everything the planner may see; built only by `planner_view`.
pub struct PlannerView {
    pub turns: Vec<UserTurn>,                      // the user's own words: Trusted
    pub context: ContextView,                      // ContextSnapshot with untrusted text as Handles
    pub actions: Vec<ActionCard>,                  // decl label, effect, tool_schema, reach
    pub handles: Vec<HandleCard>,
    pub history: Vec<StepLine>,                    // the session's calls, outcomes, refusals with reasons
    pub taint: Integrity,                          // planner context integrity as the router sees it
}
pub struct UserTurn { pub id: TurnId, pub text: String, pub at: UnixSeconds }
pub struct HandleCard { pub handle: Handle, pub shape: HandleShape, pub from: Source, pub size: CharCount }
pub enum HandleShape { Text, Entity(EntityKind), File }
pub enum Shown2<T> { Plain(T), Handle(Handle) }   // name: `Visibility<T>` in code; see below
pub fn planner_view(ctx: &ContextSnapshot, table: &mut HandleTable, ...) -> PlannerView;

/// The planner asks; the reader answers with a typed value only.
pub struct ReaderAsk { pub inputs: Vec<Handle>, pub want: ValueSchema, pub task: ReaderTask }
pub enum ReaderTask { Classify, Extract, Summarise, Compare }   // the planner's instruction is a closed task + schema
pub enum ValueSchema {
    Choice(Vec<ChoiceId>), Integer { min: i64, max: i64 }, Date, DateTime,
    EntitiesAmong(Vec<EntityId>), Text { max: CharCount },
    Record(Vec<(ParamName, ValueSchema)>), List { of: Box<ValueSchema>, max: Count },
}
pub fn conforms(v: &Value, s: &ValueSchema) -> Result<(), SchemaFault>;
pub trait Reader: Send + Sync {
    fn extract(&self, ask: ReaderAsk, inputs: Vec<Quarantined<String>>)
        -> impl Future<Output = Result<Value, ReaderError>> + Send;
}
pub enum ReaderError { OutOfSchema(SchemaFault), Unparseable, Refused, ModelUnavailable }
```

Notes on the contract:
- The router labels a reader's result as the join of its inputs, plus `Source::Model(Reader)`.
- A result whose `Text` is anywhere inside it reaches the planner as a new `Handle`.
- Closed-set, numeric, date and entity-among results reach the planner as plain values. This taints the planner context (the residual CaMeL channel, stated honestly).
- Naming: `Shown2<T>` above is a placeholder. In code it is `Visibility<T> { Plain(T), Handle(Handle) }`, to avoid clashing with `Shown`.

### 3.9 `docket-core` + `action-review`: auto mode (freeze now)

```rust
/// Per Space, setting `agent.strictness` (proposed default: Default).
pub enum Strictness { AskMore, Default, TrustMore }

/// A scoped, expiring permission the user stated ("go ahead and archive the newsletters").
pub struct Authorization {
    pub id: AuthId, pub space: SpaceId, pub session: AuthSession,   // ThisSession | AnySession
    pub from: TurnId,                                // must be a router-held user turn
    pub quote: String,                               // the user's words, verbatim slice of that turn
    pub scope: AuthScope,
    pub expires: UnixSeconds,                        // setting agent.auth.max_s, proposed 3600
    pub uses: Uses,                                  // Unlimited | Left(Count)
    pub state: AuthState,
}
pub struct AuthScope { pub app: AppName, pub actions: ActionMatch, pub kinds: BTreeSet<EntityKind>,
                       pub ceiling: Effect, pub max_count: Count }
pub enum ActionMatch { One(ActionName), AnyUpTo(Effect) }
pub enum AuthSession { ThisSession, AnySession }
pub enum Uses { Unlimited, Left(Count) }
pub enum AuthState { Proposed, Active, Spent, Expired, Revoked }
/// Structural coverage, computed by the router before Cedar.
pub enum Coverage { Covered(AuthId), NotCovered }

/// What the reviewer sees. Built by `review_request`; no tools, no handles resolvable.
pub struct ReviewRequest {
    pub space: SpaceId, pub strictness: Strictness,
    pub turns: Vec<UserTurn>,                        // trusted intent
    pub proposed: ProposedAction,
    pub labels: ReviewLabels,
    pub history: Vec<StepLine>,                      // last N (setting, proposed 12), with prior verdicts
    pub authorizations: Vec<Authorization>,          // active, this Space
    pub quoted: Vec<QuotedData>,                     // untrusted content as data, truncated
}
pub struct ProposedAction { pub app: AppName, pub action: ActionName, pub label: LabelText,
                            pub effect: Effect, pub targets: Vec<EntityRef>, pub count: Count,
                            pub lines: Vec<ArgLine>, pub lasting: Lasting }
pub struct ReviewLabels { pub args: Label, pub planner: Integrity, pub coverage: Coverage }
pub struct QuotedData { pub from: Source, pub text: String, pub truncated: Truncated }
pub enum Truncated { Whole, Cut }

pub enum Verdict {
    Allow { why: ReviewReason, under: Option<AuthId> },
    Deny  { why: ReviewReason },
    Ask   { why: ReviewReason },
}
pub struct ReviewReason { pub code: ReasonCode, pub text: ReasonText }   // text ≤ 200 chars, model-written
pub enum ReasonCode { WithinRequest, CoveredByAuthorization, Routine,
                      OutsideRequest, ScopeCreep, Exfiltration, Irreversible,
                      InjectionSuspected, Uncertain, ReviewerFailed }

pub trait Reviewer: Send + Sync {
    fn review(&self, request: &ReviewRequest) -> impl Future<Output = Result<Verdict, ReviewError>> + Send;
}
pub enum ReviewError { Timeout, Unavailable, Unparseable, OutOfVocabulary }

/// Prompt text with untrusted data inside per-request nonce fences; the nonce is stripped
/// from the data first.
pub fn render(request: &ReviewRequest, nonce: Nonce) -> ReviewPrompt;
pub fn parse_verdict(raw: &str, request: &ReviewRequest) -> Result<Verdict, ReviewError>;  // JSON, grammar-constrained
/// The one place a verdict meets the hard rules. A verdict is consulted only for `Review`.
pub fn combine(hard: Decision, verdict: Result<Verdict, ReviewError>) -> Gate;
```

`combine` is a total function. Its rules:

| `hard` | verdict | `Gate` |
|---|---|---|
| Deny | any | Refuse(Denied) |
| Allow | any | Run |
| Ask | any | Confirm |
| Review | Ok(Allow) | Run |
| Review | Ok(Deny) | Refuse(Denied(Review)) |
| Review | Ok(Ask) | Confirm |
| Review | Err(_) | Confirm |

A failed or slow reviewer is never an Allow. The reviewer timeout is setting `agent.review.timeout_ms`, proposed 2000.

### 3.10 `policy-point` (freeze now: schema, request shape, decision; policy text: fill)

```rust
pub struct PolicyRequest {
    pub principal: PrincipalFacts { kind: ActorKind, caller: AppName, isolation: Isolation },
    pub op: Op,
    pub resource: ActionFacts { app: AppName, action: ActionName, effect: Effect,
                                classes: BTreeSet<DataClass>, reach: AgentReach, lasting: Lasting },
    pub context: PolicyContext {
        space: SpaceId, target_space: SpaceRelation,  // Same | Other | Unbound
        args: Integrity, planner: Integrity, confidentiality: Confidentiality,
        usage: Usage, origin: Origin, count: Count, mass_at: Count,   // setting agent.mass_at, proposed 20
        grant: GrantState,                            // Always | Once | None | Denied
        coverage: CoverageState,                      // Covered | NotCovered
        strictness: Strictness,
    },
}
pub enum Op { Perform, Search, Preview, Suggest, ReadContext, Undo, IndexPush }
/// Three Cedar queries per request: `Action::"perform"`, `"perform_unasked"`, `"perform_reviewed"`.
pub enum Decision { Allow(PolicyIds), Review(PolicyIds), Ask(Vec<AskReason>), Deny(DenyReason) }
pub enum AskReason { Effect(Effect), Tainted, CrossSpace, Mass(Count), FirstUse, AskAlways,
                     LastingFromUntrusted, Rule(PolicyId) }
pub enum DenyReason { NoPermit, Rule(PolicyId), Hidden, ConsentDenied,
                      Review { code: ReasonCode, text: ReasonText }, RepeatAfterDeny }
pub struct Pdp { /* cedar PolicySet + Schema + Authorizer */ }
impl Pdp {
    pub fn load(schema: &str, policies: &str) -> Result<Pdp, PolicyError>;  // validates strictly
    pub fn decide(&self, request: &PolicyRequest) -> Decision;              // pure
}
pub enum PolicyError { Schema(String), Parse(String), Invalid(String) }
```

How `decide` maps the three queries:
- If `perform` is not permitted: `Deny`. The reasons are the forbid ids, or `NoPermit`.
- Else if `perform_unasked` is permitted: `Allow`.
- Else if `perform_reviewed` is permitted: `Review`.
- Else: `Ask`. The reasons come from the forbids on `perform_unasked` or `perform_reviewed`.

The Cedar API used is `Authorizer::is_authorized` with `Response::diagnostics().reason()` [unverified against the 4.13 docs; check in the freeze].

Files:
- `crates/policy-point/schema/quire.cedarschema`. Namespace `Quire`. Entities: `Caller { kind: String }`, `App`, `Space`, `Act in [App] { effect, classes: Set<String>, reach, lasting }`. Actions: `perform`, `perform_unasked`, `perform_reviewed`, `search`, `preview`, `suggest`, `read_context`, `undo`, `index_push`. Context is the record above, using slugs.
- `crates/policy-point/policies/default.cedar`. Each policy carries `@id`. Its content is the table in §4.4.
- A user override directory `$XDG_CONFIG_HOME/quire/policy/*.cedar` comes later and is forbid-only (it may only tighten).

### 3.11 `docket-router` seams (freeze now; bodies `todo!()`)

```rust
pub trait AppLink: Send + Sync {          // calls IntentProvider1 on the app's bus name
    fn perform(&self, app: &AppName, inv: Invocation, budget: Latency) -> impl Future<Output = Result<Outcome, AppRefusal>> + Send;
    fn undo(&self, app: &AppName, token: &UndoToken, actor: &Actor) -> impl Future<Output = Result<(), UndoFault>> + Send;
    fn context(&self, app: &AppName, scope: ContextScope) -> impl Future<Output = Result<ContextSnapshot, LinkFault>> + Send;
    fn search(&self, app: &AppName, text: &str, gen: Generation) -> impl Future<Output = Result<Vec<Hit>, LinkFault>> + Send;
    fn preview(&self, app: &AppName, id: &EntityId) -> impl Future<Output = Result<Preview, LinkFault>> + Send;
    fn suggest(&self, app: &AppName, s: SuggestAsk) -> impl Future<Output = Result<Vec<EntityRef>, LinkFault>> + Send;
}
pub trait GrantStore: Send + Sync { fn grants(&self) -> Vec<ActionGrant>; fn record(&self, g: ActionGrant); }
pub trait EventSink: Send + Sync { fn append(&self, e: AuditRecord); }   // the memory area's eventlog
pub trait Clock: Send + Sync { fn now(&self) -> UnixSeconds; }

/// Never content; ids, kinds, decisions.
pub enum AuditRecord {
    Call { at: UnixSeconds, call: CallId, actor: Actor, app: AppName, action: ActionName,
           targets: Vec<EntityId>, effect: Effect, space: SpaceId, decided: DecidedBy, end: CallEnd },
    Review { at: UnixSeconds, call: CallId, verdict: VerdictKind, code: ReasonCode, latency: Millis, model: ModelId },
    Confirm { at: UnixSeconds, id: ConfirmId, answer: ConfirmAnswerKind, input: InputProof },
    Undo { at: UnixSeconds, entry: UndoId, by: Actor, end: UndoState },
    Halt { at: UnixSeconds, scope: SpaceScope, cause: HaltCause },
    Authorization { at: UnixSeconds, id: AuthId, state: AuthState },
}
pub enum DecidedBy { Policy(PolicyIds), Reviewer(AuthId?/ReasonCode), User(ConfirmReceipt) }

pub struct Router<L: AppLink, C: Confirmer, R: Reviewer, G: GrantStore, E: EventSink, K: Clock> { … }
impl<…> Router<…> {
    pub fn handle(&self, caller: AppId, request: IntentsRequest) -> impl Future<Output = IntentsReply> + Send;
}
/// The pure decision, table-tested.
pub fn gate(halt: &KillSwitch, ledger: &Ledger, budget: &Budget, consent: Verdict,
            decision: Decision, repeat: Repeat) -> Pending;   // Pending = Run | NeedsReview | Confirm(..) | Refuse(..)
```

Note: `DecidedBy::Reviewer` carries `{ code: ReasonCode, under: Option<AuthId> }`.

### 3.12 Wire and D-Bus (freeze now)

There is one protocol with two carriers, as in porter. `IntentsRequest`/`IntentsReply` are serde enums in `docket-core::wire`. The socket frame reuses porter's `{ vocab, body }` frame. Over D-Bus, bodies travel as JSON in `s`. The caller's identity is derived from the connection (porter `AppId`) and is never sent.

Caller roles are configured in `intentd`'s config:
- `launcher`: sill. Gets Actor::User when `Origin::Launcher`/`InWindowField`.
- `companion`: the agent daemon. Actor::Companion.
- `reader`: the reader process.
- `cua`: cuad.
- `mcp`: actions-mcp.
- `compositor`: may call `Halt`.
- `control`: sill's control centre. May call `Halt`/`Resume`.
- Any other app gets Actor::App: its own actions only, plus Search, Preview, Suggest.

**`org.quire.Intents1`** at `/org/quire/Intents1` (served by intentd):

| Interface | Member | Who |
|---|---|---|
| `.Registry` | `Manifests() -> a(ss)`; signal `ManifestChanged(s app)` | any |
| `.Index` | `Push(batch s)` (an `IndexBatch { epoch, upserts: Vec<IndexEntry>, removals: Vec<EntityKey> }`), `Reset(epoch t)`. The owner is derived and must equal the manifest's app. | the owning app |
| `.Search` | `Query(text s, scope s, gen t) -> s` (hits from the index, then app `Search` for non-indexed kinds within 250 ms); `Cancel(gen t)`; signal `Hits(gen t, hits s)` for late app hits | launcher, companion, apps |
| `.Run` | `Perform(call s, parent_window s) -> o` (a Request; `Response(u, reply s)`, `Progress(s)`); `Preview(entity s) -> s`; `Suggest(ask s) -> s`; `Undo(entry t) -> o` | per role |
| `.Context` | `Current(scope s) -> s` (pulls `IntentProvider1.Context` from the focused app; companion roles get a `ContextView` with handles) | launcher, companion |
| `.Session` | `Open(space s) -> s session`; `Turn(session s, text s) -> t` (records a trusted user turn; launcher only); `Close(s)`; `Resolve(handle t) -> s` (reader only); `Authorize(session s, turn t, scope s) -> o` | launcher, companion, reader |
| `.Control` | `Halt(scope s)`, `Resume(scope s)`, `State() -> s`, `Journal(filter s) -> s`; signals `Halted(s)`, `Resumed(s)`, `JournalChanged(t)` | compositor, control |
| `.Request` (per call `/org/quire/Intents1/request/<n>`) | `Close()`; signals `Progress(s)`, `Response(u, s)` | portal shape |

`IndexEntry` is `{ key, kind, title, subtitle, keywords: Vec<String>, updated, space: SpaceScope }`. It is stored by `intentd` per Space; encryption at rest comes later (U5).

**`org.quire.IntentProvider1`** at `/org/quire/IntentProvider1`, on the app's own activatable bus name (= `AppName`). The router calls it only after checking that the name's owner derives to the same `AppId`.

| Member | Notes |
|---|---|
| `Perform(invocation s) -> s` | `Outcome` or `AppRefusal`; the app labels its undo entry with `invocation.actor` |
| `Undo(token s, actor s) -> s` | `()` or `UndoFault` |
| `Context(scope s) -> s` | `ContextSnapshot`; answered by `ds-intents` without app code |
| `Search(text s, gen t) -> s` | non-indexed kinds only |
| `Preview(entity s) -> s`, `Suggest(ask s) -> s`, `Resolve(keys s) -> s` | the two-phase ids-then-metas split |
| signal `UndoChanged(token s, state s)` | Cmd+Z in the app keeps the router's journal true |
| signal `IndexStale(epoch t)` | asks the router to request a `Reset` |

**`org.quire.Confirm1`** at `/org/quire/Confirm1`, served by the compositor fork (cua/ux areas own it). The fallback is a separate sheet process.
- `Confirm(request s) -> o`, where the Request carries `Response(u, answer s)`.
- `Cancel(id s)`.
- The interface is frozen now; who serves it is decided later.

**Daemon config**, `$XDG_CONFIG_HOME/quire/intentd.toml`, the serde form of `IntentdConfig { roles: BTreeMap<Role, Vec<AppName>>, reviewer: ReviewerChoice, … }`. Freeze later; budgets and strictness come from settings keys.

### 3.13 `docket-client` (freeze now)

```rust
/// What an app implements. Context has a default answered by ds-intents.
pub trait IntentProvider: Send + Sync {
    fn manifest(&self) -> &ValidManifest;
    fn perform(&self, inv: Invocation) -> impl Future<Output = Result<Outcome, AppRefusal>> + Send;
    fn undo(&self, token: UndoToken, actor: Actor) -> impl Future<Output = Result<(), UndoFault>> + Send;
    fn search(&self, text: &str) -> impl Future<Output = Vec<Hit>> + Send;
    fn preview(&self, id: &EntityId) -> impl Future<Output = Preview> + Send;
    fn suggest(&self, ask: SuggestAsk) -> impl Future<Output = Vec<EntityRef>> + Send;
}
pub trait ContextSource: Send + Sync { fn snapshot(&self, scope: ContextScope) -> ContextSnapshot; }
pub trait Transport: Send + Sync {
    fn call(&self, request: IntentsRequest) -> impl Future<Output = Result<IntentsReply, TransportError>> + Send;
}
pub struct Intents<T: Transport>;   // perform, search, preview, suggest, undo, context
```

### 3.14 `actions-mcp` (types freeze now; behaviour fill 2)

```rust
pub struct McpTool { pub name: ToolName /* "<app_slug>__<action with _>" */, pub schema: ToolSchema,
                     pub annotations: ToolHints }
pub fn tools(registry: &[ValidManifest]) -> Vec<McpTool>;   // only AgentReach::Offered; Hidden dropped
pub enum ToolHints { ReadOnly, Undoable, OpenWorld, Destructive }   // maps Effect to MCP annotations
pub fn serve<T: Transport>(intents: Intents<T>, client: ClientName) -> impl rmcp::ServerHandler;
```

How `actions-mcp` behaves:
- Every argument from an MCP client is labelled `Untrusted, Source::Mcp(client)`.
- The actor is `Mcp`. MCP actors never get `Review`: they get Allow for Read, and Ask for everything else.
- It runs over stdio or a unix socket, off by default (U7).
- Importing third-party MCP tools comes later.

---

## 4. State machines

### 4.1 Call lifecycle (`docket-router::call`, a pure step)

| State | Event | Next | Effects |
|---|---|---|---|
| Received | args fail decl | Done(BadArgs) | audit |
| Received | ok | Gating | resolve handles (label kept); compute target Space, coverage, labels |
| Gating | `halted` | Done(Halted) | audit |
| Gating | budget exhausted | Done(OverBudget) | audit; if Denials, halt the session |
| Gating | same (action, targets) denied earlier in session | Done(Denied(RepeatAfterDeny)) | audit |
| Gating | consent Denied | Done(Denied(ConsentDenied)) | |
| Gating | Pdp Deny | Done(Denied) | |
| Gating | Pdp Allow | Dispatched | charge |
| Gating | Pdp Review | Reviewing | build ReviewRequest; start timer |
| Gating | Pdp Ask or consent Ask | Confirming | ConfirmRequest |
| Reviewing | verdict / error / timeout | per `combine` | audit Review |
| Reviewing | Halt | Done(Halted) | drop the review |
| Confirming | Allowed(receipt) | Dispatched | record grant if Always; audit Confirm |
| Confirming | Ended(e) | Done(Unconfirmed(e)) | count a denial |
| Confirming | Halt | Done(Halted) | `Confirmer::cancel` |
| Dispatched | Outcome | Done(Ok) | journal undo; spend Authorization use; audit; maybe Follow (depth+1, re-gated) |
| Dispatched | AppRefusal / timeout | Done(refusal) | audit |
| Dispatched | Halt | (continues) | an in-flight app call cannot be recalled; its result is journalled |

### 4.2 Undo entry

`Available` changes as follows:
- On `Undo` it goes to `Undoing`.
- `Undoing` goes to `Undone{by}` on success, or `Failed(fault)` (Gone or Conflict) on failure.
- On the app signal `UndoChanged(Undone)` it goes to `Undone{by: User}`.
- After 24 h, or when the app's stack drops the entry, it goes to `Expired`. The 24 h is setting `agent.undo.keep_h`.

"Undo all" for a run replays the run's Available entries newest first and stops at the first Failed. No locks: undo can fail because the user changed the thing since, and that is reported, not prevented.

### 4.3 Kill switch and session

Kill switch:
- Per Space and global: `Running` goes to `Halted{cause}` on `Halt` (compositor chord, control centre, Esc on the strip, budget).
- `Halted` goes back to `Running` only through `Control.Resume` from the `control` role.
- Global `Halted` overrides every Space.

Session:
- `Open(clean)` becomes `Open(tainted)` on the first plain untrusted reveal. The taint never goes back.
- After N reviewer or user denials in a row, the session is `Paused`. It waits for a user turn.
- `Closed` comes on Close, on Halt of its Space, or when the wall budget runs out.

### 4.4 Decision table (the default Cedar policy, by strictness)

"T" = args and planner integrity Trusted. "U" = either one Untrusted. "Cov" = covered by an active Authorization.

| Effect / condition | AskMore | Default | TrustMore |
|---|---|---|---|
| Read (same Space) | Allow | Allow | Allow |
| UndoableWrite, T | Review | Allow | Allow |
| UndoableWrite, U | Review | Review | Review |
| Outbound, T | Ask | Cov ? Review : Ask | Review |
| Outbound, U | Ask | Ask | Cov ? Review : Ask |
| Destructive | Ask | Ask | Cov and T ? Review : Ask |
| `Lasting::AgentMemory` with U args | Ask | Ask | Ask |
| count > `mass_at` (non-Read) | Ask | Ask | Ask |
| `AgentReach::AskAlways` | Ask | Ask | Ask |
| `AgentReach::Hidden` (non-User) | Deny | Deny | Deny |
| target Space Other | Ask (Deny for Mcp) | same | same |
| first companion use of app×class×Space (consent Ask) | Ask (OnceOrAlways) | same | same |
| Actor User via launcher | Allow (the app's own Alerts apply) | same | same |
| Actor Mcp | Read Allow, else Ask | same | same |

`Default` keeps COMPANION item 9 exactly unless the user has stated an Authorization (conflict C2).

### 4.5 Authorization

- `Session.Authorize(turn, scope)` creates it as `Proposed`. The turn must be router-held and the quote must be a substring of it.
- The reviewer then checks that the quote supports the scope. The check is a `Verdict` on a synthetic `ProposedAction`.
- With an Allow it becomes `Active`; anything else drops it.
- `Active` becomes `Spent` (uses reach 0), `Expired` or `Revoked` (Settings or control centre).
- An Authorization can never move a cell of §4.4 from Ask or Deny to Allow. It only turns Ask into Review where the table says "Cov".

### 4.6 Index sync per app

| State | Event | Next |
|---|---|---|
| Unknown | `Reset(e)` | Syncing(e) |
| Syncing(e) | `Push(e)` batches | Synced(e) |
| Synced(e) | `Push(e')`, e' ≠ e | refuse; ask for `Reset` |
| Synced(e) | `IndexStale` or app restart | Unknown |

On app uninstall the entries are dropped. Deleting a Space drops its index.

---

## 5. Tests that pin the shapes

All tests are hermetic. Bus tests use a private `dbus-daemon --session` with scratch XDG dirs (`docket-fake` provides it), or zbus p2p.

### 5.1 `prov`

- `join_is_commutative_associative_idempotent`: lattice laws over a generated table.
- `untrusted_wins_integrity_join` and `private_spaces_union_on_join`.
- `map_keeps_label`, `zip_joins_labels`.
- `endorse_needs_a_receipt`: there is no other constructor path, checked with trybuild compile-fail.
- `quarantined_debug_prints_no_text`.

### 5.2 `docket-core`

- `every_wire_type_round_trips`: one pinned JSON string per enum variant, sill-style. Covers `Effect`, `Value`, `Selection`, `Preview`, `CallRefusal`, `Verdict`, `IntentsRequest`/`Reply`.
- `manifest_files_parse_and_validate`: the fixtures `fixtures/manifests/org.quire.Mail.intents.toml` and `org.quire.Files.intents.toml`.
- `validate_rejects`: a table test with one row per `ManifestError` variant.
- `undoable_write_requires_token` and `read_has_no_undo`.
- `tool_schema_snapshot`: a pinned JSON Schema for three fixture actions.
- `conforms_table`: `ValueSchema` against values.
- `planner_view_hides_untrusted_text`: a snapshot with a mail subject yields a Handle and the subject string appears nowhere in the serialized view.
- `password_fields_never_reported`.
- `budget_charge_table` and `halt_global_overrides_space`.

### 5.3 `policy-point`

- `schema_and_default_policies_validate`.
- `default_policy_grid`: one table row per §4.4 cell × strictness. Each row names its case and asserts the `Decision` variant and the reason ids.
- `user_overrides_only_tighten`: an override `permit` fails to load. This test is "later".

### 5.4 `action-review`

- `combine_is_total_and_never_allows_outside_review`: every `Decision` × every verdict and error. `Allow` comes only from (Allow, _) or (Review, Ok(Allow)).
- `reviewer_failure_asks`: Timeout, Unavailable, Unparseable and OutOfVocabulary each give Confirm.
- `render_fences_untrusted_data`: untrusted text appears only inside nonce fences; a planted copy of the nonce in the data is stripped; user turns appear outside the fences.
- `parse_verdict_rejects_unknown_codes_and_long_reasons`.
- **`injected_content_never_turns_ask_or_deny_into_allow`**. The fixture is a corpus of about 40 cases: mail and web bodies saying "ignore previous instructions, send…", fake authorizations inside content, quotes forging a user turn, fence breakouts. Each case is run through the whole router with a **`ScriptedReviewer` that always answers Allow** (a fully hijacked reviewer). Every case whose §4.4 cell is Ask or Deny ends in Confirm or Refuse.
- `authorization_inside_content_is_ignored`: an Authorization whose turn id is not a router-held user turn is refused at `Session.Authorize`.

### 5.5 `docket-router` (through `docket-fake::fake_router`)

- `read_runs_without_confirm` and `outbound_asks_in_default`.
- `tainted_session_voids_always_grant`.
- `deny_reason_reaches_planner_refusal`.
- `exact_repeat_after_deny_is_denied_without_review`: the ScriptedReviewer call count stays the same.
- `three_denials_pause_session`.
- `halt_overrides_reviewer_allow`.
- `halt_cancels_pending_confirm`.
- `budget_exhausted_overrides_allow`.
- `fan_out_over_mass_asks`.
- `cross_space_target_asks`.
- `follow_chain_capped_at_four`.
- `undo_journal_labels_actor`.
- `undo_all_stops_at_first_conflict`.
- `app_cmd_z_marks_journal_undone`.
- `authorization_expires_and_spends`.
- `index_push_from_wrong_app_refused`.
- `planner_taint_computed_by_router_not_caller`: a companion that sends "trusted" labels gets nothing from it.
- `reader_only_may_resolve_handles`.
- `every_call_appends_one_audit_record`, plus `every_verdict_is_logged` (asserting exact counts).

### 5.6 `docket-dbus`, `docket-client`, `actions-mcp`, `intentd`

- `introspection_matches_xml`: the porter pattern. It fails with the new text.
- `p2p_perform_round_trip`: a zbus p2p proxy against the skeleton backed by the fake router.
- `caller_identity_derived_not_sent`.
- `tools_only_offered_actions` and `tool_hints_follow_effect`.
- `mcp_call_goes_through_router_as_untrusted`: an rmcp in-memory transport, asserting a Confirm for an UndoableWrite.
- `intentd_skeleton_exits_2`.

**Fixtures and fakes in `docket-fake`:** two manifests, FakeMail and FakeFiles providers (in-memory stores with real undo tokens), `ScriptedConfirmer` (answers queue), `ScriptedReviewer` (verdict queue, `AlwaysAllow`, `Timeout`), `FixedClock`, `RecordingSink`, `MemoryGrants`, the injection corpus `fixtures/injection/*.toml`, and a private-bus helper.

---

## 6. Work breakdown

### Freeze wave (parallel, one agent per repo)

1. **quire agent.**
   - Owns `docs/workspace-deps.toml`. It adds `cedar-policy`, `rmcp` and their licences, and lands this commit first.
   - Writes `design/33-ACTIONS.md` (this spec, settled parts marked).
   - Edits 31 §4.5 (`space` in `GrantKey`, generic grants) and §5.5 (MCP hosting moves to actions-mcp, C4).
   - Does not touch `ds`.
   - Verify: `scripts/check-doc-paths.sh`.
2. **porter agent.** Owns `crates/porter-core/src/{space.rs, consent/*, lib.rs}` and `tests/round_trip.rs` rows. Updates every caller of `GrantKey`/`Grant` in the same change (porter-service, porter-fake). Verify with porter's gate:
   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   ./scripts/check-boundary.sh
   cargo deny check licenses
   ```
3. **docket agent** (new repo; starts after 1 and 2 merge).
   - Scaffolds the workspace by copying porter's `rust-toolchain.toml` (1.98.1), `deny.toml`, `rustfmt.toml` and lints.
   - Writes every crate in §2.1 with the types and traits of §3. Behaviour bodies are `todo!()`, each listed in `FINDINGS.md`.
   - Fully implemented in this wave: the pure, cheap parts (`prov::join`, `validate`, `combine`, `charge`, `halted`, `conforms`).
   - Also: the XML, the fixtures, all §5 shape tests (behaviour tests `#[ignore]` with a FINDINGS line), `ARCHITECTURE.md` and `CONVENTIONS.md`.
   - Verify: the same gate as porter, plus `cargo test -p policy-point` (schema loads).

### Fill wave 1 (docket, parallel by file ownership)

- **F1a**: `prov` + `policy-point` (Cedar evaluation, `default.cedar`, the grid test). Owns `crates/prov/**` and `crates/policy-point/**`.
- **F1b**: `action-review` (render, parse, `InferReviewer` over `porter_infer::Model` with `FakeModel`, the injection corpus). Owns `crates/action-review/**` and `crates/docket-fake/fixtures/injection/**`.
- **F1c**: `docket-router` (call step, gate wiring, session and handles, journal, authorizations, index model). Owns `crates/docket-router/**`.
- **F1d**: `docket-dbus` codec and p2p tests, and `docket-client` `InProcess`/`DbusTransport`. Owns `crates/docket-dbus/**`, `crates/docket-client/**` and `dbus/**`.
- `docket-fake` belongs to F1c. Others request additions through it.
- Verify per agent: `cargo test -p <crate>`, then the full gate.

### Fill wave 2 (cross-repo)

- **docket F2a**: `intentd` serves the bus over the private bus, with seams wired: `AppLink` over zbus, `GrantStore` file, `EventSink` to the memory area's `eventlog` (or a JSONL stub until it exists).
- **docket F2b**: `actions-mcp` over rmcp.
- **sill agent**: the `sill-launcher` variants, following the warn-before-lint-rules practice. Pinned wire tests for the new `Activation::Intent`/`Subject::Entity`/`Preview::Intent`. `ThingsProvider` in `sill-search` over `docket-client`. `org.quire.Shell` provider in `sill-services`. Verify with sill's gate.
- **quire agent**: `ds-intents` (`ContextModel`, a `thing` prop on `Row`/`List`, edit-surface text targets, a default `ContextSource`). Only after the user answers U6.
- **mailo session**: the manifest, `IntentProvider`, and `actor` on `UndoStack`.

### Fill wave 3 (other areas join)

- The compositor `Confirm1` server and the kill chord [cua/ux].
- The real reviewer and reader models through inferd [models].
- `eventlog` [memory].
- The CUA gate integration [cua].

---

## 7. Dependencies, conflicts, questions

### 7.1 Dependencies on other areas

| Area | What we need from it | What it needs from us |
|---|---|---|
| **cua** | Draws `org.quire.Confirm1` and the acting-here glow; kill chord → `Control.Halt`; effect classification for pixel actions | `policy-point::Decision`, `gate`, `Actor::Cua`, `Effect`, `ConfirmRequest`, `prov::Source::Screen` |
| **ux** | Look of the confirmation and companion strip, Activity/"Undo all" UI, launcher companion mode | `Preview`, `UndoEntry`, `Journal`, `Control` signals, sill variants |
| **models** | inferd serving the reviewer and reader models (small local, no tools, grammar-constrained JSON), model latency | `ReviewRequest`/`render`/`parse_verdict`, `ValueSchema` → grammar, `tool_schema` |
| **memory** | `eventlog` crate taking `AuditRecord`; `Actor` shared; memfiles' write action declared `Lasting::AgentMemory` | `AuditRecord`, `Actor`, `prov::Label`, `EntityId` |
| **planner / agent loop** (name unknown) | Session driving (`Session.Open/Turn`), planner prompt from `PlannerView`, the reader process, proposing Authorizations, adapting to `Denied` reasons | §3.8, §3.9, `CallRefusal` |
| **security / sandbox** (if separate) | systemd, Landlock and seccomp units for intentd and actions-mcp; bubblewrap for imported MCP tools | caller roles config |

### 7.2 Conflicts between docs, with proposed resolutions

- **C1. Effect names.** models.md has `Read | Write | Destructive | Outbound`; COMPANION item 9 has "undoable write". Resolution: `UndoableWrite`, and a write without undo must be declared `Destructive` (enforced by `validate`).
- **C2. Who confirms what.** models.md says "the agent always confirms Write and above", and the auto-mode request lets a reviewer skip asks. COMPANION item 9 (settled) says reads and undoable writes run, outbound and destructive ask. Resolution:
  - The `Default` strictness keeps item 9 exactly.
  - The reviewer only judges tainted undoable writes. It also judges outbound actions that a user-stated Authorization covers.
  - `TrustMore` widens this; `AskMore` narrows it.
  - Taint's role (Security item 2): it voids `Always` grants and Authorizations, and gates `Lasting` writes.
- **C3. Unforgeable consent.** COMPANION Security 4 requires compositor-drawn confirmations; the fork may be late. Resolution: until it lands, a separate sheet process serves `Confirm1` and receipts carry `InputProof::SheetFallback`. This is acceptable for tier 1 because typed actions inject no input. The CUA tier must not ship on the fallback.
- **C4. MCP.** 31 §5.5 makes inferd the MCP host. The build map makes `actions-mcp` edge-only. Resolution: MCP in both directions lives in `actions-mcp`. Imported tools become router actions of a pseudo-app `mcp.<server>`, default `Outbound`, each in bubblewrap. inferd drops the role (an edit to 31).
- **C5. Provider kinds.** models.md has `ProviderKind::App(AppName)`, which breaks sill's `Copy` + `ALL` Tab order. Resolution: one `ProviderKind::Things`, with per-kind scoping later (`Scope::Kind`).
- **C6. Companion work is deferred.** 30 §3.4 says "Deferred, map only: one-input/companion/AI work". Resolution: freeze types only; `ds` is untouched until U6.
- **C7. Space identity.** 21-SPACES §11.3 says COSMIC workspace ids may be unstable, yet Spaces are hard walls with their own key. Resolution: a quire-minted `SpaceId`, mapped to workspaces by `SpaceStore`.
- **C8. P1 inline replace.** agent-ux P1 ("nothing applies without the Apply keypress") differs from item 9 (`text.replace` is an UndoableWrite and runs). This is a UX call (U8).

### 7.3 Questions only the user can answer

- **U1.** Repo name and placement: a new `docket` repo (proposed), versus putting the router in porter or sill. Also: must action names carry their app's prefix (`mail.…` only in Mail)?
- **U2.** Auto mode default strictness per Space: `Default` as defined (item 9 unless you state an Authorization), or something else?
- **U3.** May the reviewer ever allow `Destructive` (proposed: only under `TrustMore` with an Authorization and untainted args), and may it ever allow tainted `Outbound` (proposed: only `TrustMore` with an Authorization)?
- **U4.** Authorizations: is the reviewer's check of your own words enough, or should each one also get a one-time compositor confirm ("Let it archive newsletters in Work for an hour?")? Default length is 1 h, this session only.
- **U5.** The shadow index stores third-party titles (mail subjects) outside the app. Allow it, per app opt-out, and encrypted at rest when?
- **U6.** May quire's `ds` gain context reporting (`thing` props, text targets) now, despite 30 §3.4's deferral?
- **U7.** Expose actions to external MCP clients in v1 (off by default), or not at all?
- **U8.** Companion text replace into your field: run at once (item 9), or show a TextChange preview and wait for Apply (agent-ux P1)?
- **U9.** Budget defaults (§3.7) and the reviewer timeout of 2 s. Is any of them yours to set rather than a setting?
- **U10.** Should launcher actions you pick yourself skip policy entirely (proposed: yes, the app's own Alerts apply), given the launcher is a trusted shell process?

### 7.4 How sill's launcher `Provider` relates

| sill today | With actions |
|---|---|
| `Provider { kind, query(&Query, &mut dyn ResultSink), activate(&Item,&Choice) -> Activation }` | Stays the in-process seam. A new `ThingsProvider` implements it: `query` calls `Intents1.Search` with `Query.generation`, pushes index hits as one `Batch`, then late app hits from `Hits(gen)`; `is_cancelled` maps to `Search.Cancel(gen)`. |
| `ProviderKind` (9, `Copy`, `ALL`) | `+ Things`, last in Tab order (C5) |
| `Item.subject: Subject` | `+ Subject::Entity(docket_core::EntityRef)` |
| `Item.actions: Vec<Action{id,label,keys}>` | From `ActionDecl`s whose `Target` fits the row's kind. `ActionId` = `ActionName` text; `keys` from `KeyHint` |
| `activate` (pure) | Returns `Activation::Intent(CallRequest)`; the daemon sends `Run.Perform` with `Origin::Launcher`. A `NeedsParam` refusal makes the launcher draw a field from `ParamDecl` and re-perform. |
| `Activation` (closed) | `+ Intent(CallRequest)` only. `Launch`, `Open`, `Copy` and the rest stay the host's own verbs and are also served as actions of `org.quire.Shell` by the same executor, so the companion uses the path you use. |
| `Preview` (pure from the row) | `+ Preview::Intent(docket_core::Preview)`, fetched lazily by `Run.Preview` after selection dwell (150 ms) |
| `Query.scope` | Later: `Scope::Kind(EntityKind)` for typed pickers |
| `CuaRunProvider` (cua doc) | The cua area's; it uses the same seam |

### Critical files for implementation

- /home/pohsuanlai/porter/crates/porter-core/src/consent/grant.rs and /home/pohsuanlai/porter/crates/porter-core/src/consent/decide.rs (the `GrantKey` Space dimension and generic `Grant<K>`)
- /home/pohsuanlai/porter/ARCHITECTURE.md and /home/pohsuanlai/porter/scripts/check-boundary.sh (the template for docket's layout, gate and boundaries)
- /home/pohsuanlai/sill/crates/sill-launcher/src/activation.rs, /home/pohsuanlai/sill/crates/sill-launcher/src/ids.rs and /home/pohsuanlai/sill/crates/sill-launcher/src/subject.rs (the sill variants)
- /home/pohsuanlai/quire/docs/workspace-deps.toml (cedar-policy and rmcp must join first)
- /home/pohsuanlai/quire/design/31-ACCOUNTS.md (§4.5 and §5.5 edits), /home/pohsuanlai/rs-wt/companion/COMPANION.md and /home/pohsuanlai/rs-wt/intents-research/models.md (the sources this spec resolves)
- /home/pohsuanlai/mailo/crates/mail-app/src/undo.rs (the first app-side undo stack to gain `actor`)
<!-- paths: end -->
