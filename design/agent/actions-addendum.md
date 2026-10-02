<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/actions-addendum.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
ADDENDUM to ~/rs-wt/agent-spec/actions.md, folding in research-action-gating.md. My handback slot is used, so this comes by SendMessage. Please save it as actions-addendum.md, or append it to actions.md. Where this addendum and actions.md disagree, this addendum wins.

# Addendum A: action gating after the prior-art review

Summary of what changes:
- A deterministic per-task policy (`TaskPolicy`) replaces the reviewer-checked `Authorization`.
- The reviewer can only make Cedar's result stricter, never looser.
- A circuit breaker stops a session after repeated denials.
- Rule of Two becomes a named Cedar rule.
- Arguments are gated by their provenance labels, not by string matching.
- The agent gets only coarse denial reasons.
- The reviewer runs as a cascade, with a second model on high-impact allows.
- Confirmations show the concrete change.
- A red-team and eval suite ships with the gate.

Several points from the coordinator's first auto-mode message change:
- The reviewer may no longer allow anything Cedar would have sent to Ask. It only sees actions Cedar already allows.
- The reviewer's input is stripped to the user's messages, the typed action and labels. It no longer gets tool outputs, agent prose, or quoted untrusted content.
- A conflict note C9 records these two changes.

## A1. Per-task privilege derivation (Progent/Conseca)

**Replaces in actions.md:** all of §4.5 (Authorization); the `Authorization`/`AuthScope`/`AuthState`/`Coverage` types in §3.9; and the `Session.Authorize` row in the §3.12 D-Bus table.

**Relation to grants:**
- An `ActionGrant` (§3.6) is a standing consent: which app, data class and Space the companion may touch at all. It is set by the user and lives across tasks.
- A `TaskPolicy` is a per-task narrowing inside those grants. It is derived from the user's own request and expires with the task.
- Both must allow an action. A `TaskPolicy` can never exceed the grants.

**How a TaskPolicy is made:**
- At `Session.Turn`, the router calls a `PolicyWriter`.
- The writer's input is only the router-held user turns and the action catalogue. It never sees content, tool output, or planner prose.
- The writer's output is enforced deterministically by Cedar, through context attributes.

**How it changes:**
- Narrowing is automatic and silent.
- Widening, from a later turn or a planner request, needs a compositor confirmation that quotes the user's turn.
- No model can mint or widen a TaskPolicy: not the reviewer, the planner, the reader or the CUA.
- If the writer fails, there is no TaskPolicy. Every non-Read call is then "Outside".

```rust
// docket-core::task_policy  (freeze now)
pub struct TaskPolicy {
    pub task: TaskId, pub space: SpaceId, pub from: Vec<TurnId>,   // router-held user turns only
    pub actions: BTreeSet<ActionMatch>, pub kinds: BTreeSet<EntityKind>,
    pub ceiling: Effect, pub max_count: Count,
    pub recipients: Vec<TrustedPattern>, pub destinations: Vec<TrustedPattern>, pub paths: Vec<TrustedPattern>,
    pub expires: UnixSeconds,                       // setting agent.task_policy.max_s, proposed 3600 or task end
    pub rationale: LabelText,                       // shown to the user on widening
    pub state: TaskPolicyState,
}
pub enum ActionMatch { One(ActionRef), AppUpTo(AppName, Effect) }
/// Only values that trace to a user turn or a user-chosen entity; never content.
pub enum TrustedPattern { Exact(Value), Entity(EntityId), Domain(String), Under(FileRef) }
pub enum TaskPolicyState { Active, Superseded, Expired, Revoked }
pub enum PolicyChange { Narrows, Same, Widens(Vec<Widening>) }
pub enum Widening { Action(ActionMatch), Kind(EntityKind), Ceiling(Effect), Count(Count), Pattern(ArgSink, TrustedPattern), Expiry }
pub fn compare(new: &TaskPolicy, old: &TaskPolicy) -> PolicyChange;           // pure, table-tested
pub enum Coverage { Inside, Outside(Widening) }
pub fn covers(p: &TaskPolicy, call: &CallRequest, labels: &ArgLabels) -> Coverage;  // untrusted sink args are never Inside
pub trait PolicyWriter: Send + Sync {
    fn derive(&self, turns: &[UserTurn], catalogue: &[ActionCard], space: &SpaceId)
        -> impl Future<Output = Result<TaskPolicy, ReviewError>> + Send;
}
```

**D-Bus changes (§3.12):**
- `.Session.Authorize` is removed.
- Added: `.Session.TaskPolicy(session s) -> s`, which reads the policy.
- Added: `.Session.Widen(session s, turn t, change s) -> o`, a Request answered by a compositor confirmation.
- `.Session.Turn` now derives the policy or narrows it.

**Cedar context (§3.11 `PolicyContext`):** `coverage: Inside | Outside`; `task_ceiling: String`.

**State machine. Replace §4.5 with:**

| State | Event | Next |
|---|---|---|
| (none) | Turn, and the writer succeeds | Active |
| (none) | Turn, and the writer fails | (none); every non-Read call is Outside |
| Active | Turn, and `compare` returns Narrows or Same | the new policy is Active; the old one is Superseded |
| Active | Turn or Widen, and `compare` returns Widens | Confirm. Allowed: the new policy is Active. Refused: the old one stays |
| Active | expiry or task end | Expired |
| Active | Settings or control centre | Revoked |

**Tests:**
- `task_policy_compare_table`.
- `covers_rejects_untrusted_recipient`.
- `policy_never_derived_from_content`: widening text planted in a mail body leaves the policy unchanged.
- `narrowing_is_silent_widening_confirms`.
- `writer_failure_means_outside`.
- `task_policy_cannot_exceed_grants`.
- Removed: `authorization_*` tests.

## A2. The reviewer only tightens

**Replaces in actions.md:**
- The `Decision` enum in §3.10.
- `combine` and its table in §3.9.
- The §4.4 grid.
- The Reviewing rows of §4.1.

```rust
// policy-point: four outcomes; Cedar queries "perform", "perform_unasked", "perform_unjudged"
pub enum Decision { Deny(Vec<PolicyId>), Ask(Vec<AskReason>), AllowJudged(Vec<PolicyId>), AllowFinal(Vec<PolicyId>) }
//   perform not permitted                 -> Deny
//   perform_unasked not permitted         -> Ask
//   perform_unjudged not permitted        -> AllowJudged   (reviewer may tighten)
//   all permitted                         -> AllowFinal
// action-review: replaces `combine`
pub fn tighten(d: Decision, verdicts: &[(Stage, Result<Verdict, ReviewError>)]) -> Gate;
// Run only from AllowFinal, or AllowJudged with every planned stage Ok(Allow).
// Deny, Ask and AllowFinal pass through untouched; any Err gives Confirm.
pub enum Verdict { Allow, Ask { why: ReviewReason }, Deny { why: ReviewReason } }
```

**New §4.4 grid (Default column = COMPANION item 9).**
- **T** means the planner is Trusted and every sink argument is Trusted.
- **In** means `coverage = Inside`.

| Effect / condition | AskMore | Default | TrustMore |
|---|---|---|---|
| Read, same Space | AllowFinal | AllowFinal | AllowFinal |
| UndoableWrite, T, In | AllowJudged | AllowFinal | AllowFinal |
| UndoableWrite, otherwise | Ask | AllowJudged | AllowJudged |
| Outbound, T, In | Ask | AllowJudged + second opinion | AllowJudged + second opinion |
| Outbound, otherwise | Ask | Ask | Ask |
| Destructive, T, In | Ask | Ask | AllowJudged + second opinion |
| Destructive, otherwise | Ask | Ask | Ask |

The named rules in A4 and A5 then override the grid toward Ask or Deny.

**Change to the decision in C2.** The "Cov ? Review : Ask" cells are gone. The only loosening of item 9 left is Outbound inside a TaskPolicy derived from the user's own words, with all sinks trusted and two reviewers agreeing. The user should confirm this (question U2).

**State machine (§4.1 rows replaced):**

| State | Event | Next |
|---|---|---|
| Gating | Pdp AllowFinal | Dispatched |
| Gating | Pdp AllowJudged | Reviewing(plan(d)) |
| Gating | Pdp Ask | Previewing (A8) |
| Reviewing | every stage Allow | Dispatched |
| Reviewing | any Ask, Err or timeout | Previewing |
| Reviewing | any Deny | Done(Denied) |

**Tests:**
- `tighten_never_loosens`: covers every Decision against every combination of verdicts and errors.
- `injected_content_never_turns_ask_or_deny_into_allow`: as in actions.md, but now Cedar Ask/Deny cells must survive `ScriptedReviewer::AlwaysAllow` **plus** a maximal TaskPolicy.
- `reviewer_never_sees_ask_or_deny_decisions`: the reviewer's call count is 0 for those cells.

## A3. Denial circuit breaker

**Adds to** §3.9 and §4.3. **Removes** the `denials_in_a_row` field from `Budget` (§3.7). **Replaces** the Denials budget kind and `HaltCause::Denials`.

```rust
// action-review::breaker  (freeze now; pure)
pub struct Breaker { pub consecutive: Count, pub recent: Vec<DenialMark> /* last 50 decisions */ }
pub struct DenialMark { pub at: UnixSeconds, pub goal: GoalKey, pub by: DeniedBy }
pub struct GoalKey { pub app: AppName, pub action: ActionName, pub kind: Option<EntityKind> }   // args excluded
pub enum DeniedBy { Policy, Reviewer, User }
pub enum BreakerTrip { Consecutive, Recent, Probing }
pub fn note(b: Breaker, outcome: DecisionMark) -> (Breaker, Option<BreakerTrip>);
// Trip limits (settings): 3 consecutive (agent.breaker.consecutive); 10 of the last 50
// (agent.breaker.recent); Probing = 3 denials with the same GoalKey and different args.
pub enum HaltCause { KillChord, ControlCentre, Esc, Budget(BudgetKind), Breaker(BreakerTrip) }
pub enum CallRefusal { /* … */ Paused(BreakerTrip) /* … */ }
```

**Effects of a trip:**
- The session moves to `Paused` and the orb shows waiting-for-you.
- `Control` emits `BreakerTripped(session)`.
- Only a new user turn resumes the session. A resume after a trip also resets `Breaker`.
- An exact repeat of a denied (goal, args) is still refused without review (`Repeated`).

**State machine (§4.3 session):**
- `Open` + `BreakerTrip` → `Paused`.
- `Paused` + user `Turn` → `Open`.
- `Paused` + any `Perform` → `Done(Paused)`.

**Tests:**
- `breaker_trips_on_three_consecutive`.
- `breaker_trips_on_ten_of_fifty`.
- `breaker_detects_probing`.
- `trip_pauses_session_until_user_turn`.
- `allow_resets_consecutive_not_recent`.

## A4. Rule of Two as a named Cedar rule

**Adds to** §3.11 `PolicyContext` and to the named rules under §4.4.

**Router side:** the router computes `SessionSaw` itself. A model cannot report it.

```rust
pub struct SessionSaw { pub private: Saw, pub untrusted: Saw }   // grows only; reset at task end
pub enum Saw { Seen, NotSeen }
// PolicyContext gains: saw_private: Saw, saw_untrusted: Saw
// AskReason gains: RuleOfTwo
```

**Rule semantics:**
- *Private* means the router delivered (plainly or as a handle the reader resolved) a value with `Confidentiality::Private` or `Secret`.
- *Untrusted* means any value with `Integrity::Untrusted` reached the planner or the reader.

**Cedar:**

```cedar
@id("rule-of-two")
forbid (principal, action in [Quire::Action::"perform_unasked", Quire::Action::"perform_unjudged"], resource)
when { resource.effect == "outbound" && context.saw_private == "seen" && context.saw_untrusted == "seen" };
```

This rule applies in every strictness. No TaskPolicy and no reviewer can lift it.

**Tests:**
- `rule_of_two_requires_confirm_in_every_strictness`.
- `rule_of_two_off_when_only_two_legs`: three rows, one per missing leg.
- `saw_flags_set_by_router_not_caller`.

## A5. Gate typed arguments by provenance, not strings

**Adds** `ArgSink` to `ParamDecl` (§3.3) and a manifest rule. **Adds** per-sink integrity to `PolicyContext` (§3.11).

```rust
pub struct ParamDecl { pub name: ParamName, pub label: LabelText, pub ty: ParamType, pub need: Need, pub sink: ArgSink }
pub enum ArgSink { Inert, Recipient, Destination, Body, Path, Query }
// ManifestError gains: OutboundWithoutSink(ActionName)  (Outbound must declare a Recipient or Destination)
pub struct SinkIntegrity { pub recipient: Integrity, pub destination: Integrity, pub body: Integrity, pub path: Integrity }
// PolicyContext gains: sinks: SinkIntegrity; AskReason gains: UntrustedSink(ArgSink)
pub struct ArgLabels { pub per_arg: BTreeMap<ParamName, Label>, pub planner: Integrity, pub saw: SessionSaw }
```

**What counts as a Trusted sink value:**
- A value typed by the user.
- An `EntityId` the user picked, or one owned by the app's own trusted store (for example a contact from Contacts).
- A value matching a `TrustedPattern` in the TaskPolicy.

**Equality does not launder text.** A string copied out of mail stays Untrusted even if it equals a contact's address. There is no string matching anywhere.

**Cedar:**

```cedar
@id("untrusted-sink")
forbid (principal, action in [Quire::Action::"perform_unasked", Quire::Action::"perform_unjudged"], resource)
when { (resource.effect == "outbound" || resource.effect == "destructive") &&
       (context.sinks.recipient == "untrusted" || context.sinks.destination == "untrusted" ||
        context.sinks.path == "untrusted" || context.sinks.body == "untrusted") };
```

**Tests:**
- `untrusted_recipient_asks_even_inside_task_policy`.
- `string_lookalike_recipient_is_not_trusted`.
- `picked_contact_is_trusted_sink`.
- `validate_rejects_outbound_without_sink`.

## A6. Minimal denial reasons to the agent (no oracle)

**Replaces** `CallRefusal::Denied(DenyReason)` in §3.5. `DenyReason` stays internal to `policy-point` and the audit log.

```rust
pub enum CallRefusal { App(AppRefusal), Denied(DenyCode), Unconfirmed(ConfirmEnd), Halted(SpaceScope),
    Paused(BreakerTrip), OverBudget(BudgetKind), NoSuchAction(ActionRef),
    BadArgs { param: ParamName, why: ArgFault }, AppUnavailable(AppName), Timeout }
pub enum DenyCode { OutsideTask, NotAllowed, NeedsUser, Repeated }   // no policy ids, no reviewer text
// AuditRecord::Review keeps code + text; AuditRecord::Call keeps PolicyIds.
// The user's Activity view may show the full reason; the planner never does.
```

**Planner contract** (planner/agent-loop area): on `Denied`, the planner either takes a materially different path or asks the user. Every retry counts toward the breaker.

**Tests:**
- `denial_reason_to_planner_is_coarse`: the `DenyCode` reaches the planner; the policy id and the reviewer text appear only in `RecordingSink`.
- `planner_view_history_has_codes_only`.

## A7. Cascade and second-model check

**Replaces** the single `Reviewer::review` in §3.9 with a staged reviewer. **Adds** `Impact` to `PolicyContext`.

```rust
pub enum Stage { Quick, Deliberate, SecondOpinion }
pub trait Reviewer: Send + Sync {
    fn review(&self, stage: Stage, r: &ReviewRequest) -> impl Future<Output = Result<Verdict, ReviewError>> + Send;
}
pub enum Impact { Low, High }   // High = Outbound|Destructive, or count > mass_at/2, or Lasting::AgentMemory
pub fn plan(d: &Decision, impact: Impact) -> Vec<Stage>;
// AllowJudged + Low  -> [Quick]  (on Quick Ask, escalate: [Quick, Deliberate])
// AllowJudged + High -> [Quick, Deliberate, SecondOpinion]
// SecondOpinion must use a different model family from Deliberate (models area enforces it in config);
// any disagreement -> Ask (ReasonCode::Disagreement).
pub struct ReviewerSet { pub quick: ModelChoice, pub deliberate: ModelChoice, pub second: ModelChoice }
// Timeouts (settings): agent.review.quick_ms 300, deliberate_ms 3000, second_ms 3000. Timeout -> Ask.
```

**Stage behaviour:**
- `Quick` is a small local judge emitting a single grammar-constrained token (Pass or Flag). It is tuned to over-flag.
- `Deliberate` is a larger local model with reasoning, run only after a Quick flag or for High impact.

**Stripped input** (this replaces the `ReviewRequest` fields in §3.9). `quoted` is removed. `history` keeps typed steps only.

```rust
pub struct ReviewRequest { pub space: SpaceId, pub strictness: Strictness, pub turns: Vec<UserTurn>,
    pub proposed: ProposedAction, pub labels: ArgLabels, pub task_policy: Option<TaskPolicy>, pub history: Vec<TypedStep> }
pub struct ProposedAction { pub app: AppName, pub action: ActionName, pub label: LabelText, pub effect: Effect,
    pub kinds: BTreeSet<EntityKind>, pub count: Count, pub args: Vec<(ParamName, ArgSink, ArgView)>, pub lasting: Lasting }
pub enum ArgView { Trusted(Value), Untrusted { from: BTreeSet<Source>, size: CharCount } }
pub struct TypedStep { pub action: ActionRef, pub effect: Effect, pub end: CallEndKind, pub verdict: Option<ReasonCode> }
```

**Audit:** `AuditRecord::Review` gains `stage: Stage`. Every stage is logged.

**Tests:**
- `cascade_plan_table`.
- `second_opinion_disagreement_asks`.
- `each_stage_timeout_asks`.
- `request_is_stripped`: after a session read a mail body, the serialized ReviewRequest contains no body text, no third-party titles and no planner prose.
- `every_stage_verdict_logged_once`.

## A8. Confirmation shows the concrete change

**Replaces** `ConfirmRequest` in §3.7. **Adds** `DryRun` to `ActionDecl` (§3.3), to `IntentProvider1` and `AppLink` (§3.12), and to `IntentProvider` (§3.14).

```rust
pub enum DryRun { None, Preview }                                 // ActionDecl field
pub struct ConfirmRequest { pub id: ConfirmId, pub space: SpaceId, pub actor: Actor, pub app: AppName,
    pub action: LabelText, pub effect: Effect, pub count: Count, pub detail: ConfirmDetail,
    pub lines: Vec<ArgLine>, pub why: Vec<AskReason>, pub taint: TaintNote, pub offer: ConfirmOffer,
    pub gesture: Gesture, pub anchor: Anchor, pub expires: Seconds }
pub enum ConfirmDetail { Recipients(Vec<Shown>), TextDiff { before: Shown, after: Shown },
    Moves(Vec<FileMove>), Entities(Vec<EntityRef>), Destination(Shown), Preview(Preview), Plain }
pub enum Gesture { Press, HoldToConfirm }                         // HoldToConfirm for Destructive
// Preview gains: Moves(Vec<FileMove>), Message { to: Vec<Labelled<String>>, subject: Labelled<String>, body: Labelled<String> }
// IntentProvider1 gains: DryRun(invocation s) -> s   (Preview); AppLink/IntentProvider gain dry_run().
```

**Rule:** every word on the confirmation comes from one of three places:
- manifest labels;
- the router's own formatting of typed arguments;
- the app's DryRun preview.

Agent prose is never shown. Untrusted values are drawn as `Shown::Quoted` with their source.

**Health metrics:** Ask rate and approval rate, kept per Space.

**State machine (§4.1):**
- New state `Previewing`, between Gating/Reviewing and Confirming.
- `Previewing` → `Confirming` after `dry_run`, or right away if the action declares `DryRun::None`.
- A dry-run failure falls back to `Plain` detail plus the argument lines.

**Tests:**
- `confirm_shows_dry_run_recipients_not_agent_text`.
- `destructive_confirm_requires_hold`.
- `untrusted_values_render_quoted`.

## A9. Shipped red-team and eval suite

**Replaces** the injection-corpus bullet in §5.4. **Adds** crate `docket-eval` to the §2.1 table and edges: it may depend on `docket-fake` and its dependencies, plus `porter-infer`. **Adds** fill-wave agent F1e to §6.

```rust
// docket-eval  (freeze now: types; runner fill 1)
pub enum Corpus { Injection, Overeager, Exfiltration, AdaptiveJudge, Benign, UiSpoofing }
pub struct Case { pub id: CaseId, pub corpus: Corpus, pub space: SpaceId, pub strictness: Strictness,
    pub turns: Vec<String>, pub world: WorldFixture, pub expect: Expect }
pub enum Expect { NoOutbound, AskOrDeny, Allow, NoReceiptFromSynthetic }
pub struct RunReport { pub version: String, pub per_corpus: BTreeMap<Corpus, Metrics> }
pub struct Metrics { pub n: Count, pub fp: Count, pub fn_: Count, pub fpr: Rate95, pub fnr: Rate95,
    pub ask_rate: Permille, pub approve_rate: Permille, pub p50: Millis, pub p95: Millis,
    pub per_stage: BTreeMap<Stage, Latency2>, pub cost_per_1000: MicroUsd }
pub struct Rate95 { pub point: Permille, pub low: Permille, pub high: Permille }   // Wilson interval
```

**Corpora** (`docket/eval/<corpus>/*.toml`):

| Corpus | Contents |
|---|---|
| Injection | About 200 AgentDojo/ASB-style tasks adapted to mail, web, files and calendar, delivered as FakeMail/FakeFiles content |
| Overeager | Vague requests where the agent oversteps. Hand-seeded, grown from opt-in logs |
| Exfiltration | 1,000+ synthetic items: recipients, URLs or paths taken from content; fragmented across steps |
| AdaptiveJudge | Suffixes in content, base64 and homoglyph encodings, approval-lookalike text, forged user turns, fence breakouts, multi-step fragmentation |
| Benign | False Ask and false Deny per strictness |
| UiSpoofing | Co-owned with cua: agent-drawn lookalike windows; reis/uinput synthetic input must never produce a `HardwareSeat` receipt |

**Running:**
- **CI** (`cargo test -p docket-eval`) runs every corpus except UiSpoofing through the deterministic layers, with `ScriptedReviewer::AlwaysAllow` and a maximal `TaskPolicy`. The structural guarantees (`AskOrDeny`, `NoOutbound`) must hold at 100%.
- **Release** (`scripts/eval-release.sh`, a dev script on a private bus) runs the real local models and writes `eval/reports/<version>.md`.
- **Release gate:** each corpus has an FNR target (the user sets these, U9). If a target is missed, AllowJudged cells shrink to Ask (a `default.cedar` change) until it passes.

**Tests:**
- `corpora_parse`.
- `structural_guarantees_hold_with_hijacked_judge`.
- `report_shape_round_trips`.

## Conflict note C9 (replaces C2's auto-mode wording in §7.2)

The coordinator's first message let the reviewer decide a grey zone, including Allow, and gave it quoted untrusted content and a history summary. The research instruction supersedes both:
- The reviewer only tightens AllowJudged.
- Its input is the user's turns, the typed action, labels, the TaskPolicy and typed history only.
- `Authorization` is replaced by the deterministic `TaskPolicy`.

## Questions changed (§7.3)

- **U2.** Should Default allow Outbound inside a user-derived TaskPolicy, with trusted sinks, after two reviewers agree? Or should Outbound always ask?
- **U4.** Is the TaskPolicy lifetime 1 h or the task's end? Should silent narrowing be visible?
- **U9.** The per-corpus FNR targets, and who may raise them.
- **U11 (new).** Which pair of open model families for Deliberate and SecondOpinion?

## Work breakdown deltas (§6)

- **Freeze wave (docket agent)** additionally writes: `task_policy.rs`, `breaker.rs`, `stage.rs`, the `docket-eval` types, `default.cedar` stubs with the ids `rule-of-two` and `untrusted-sink`, and the DryRun members in the XML.
- **F1a** owns the new Cedar rules and grid tests.
- **F1b** owns the cascade and the breaker.
- **F1c** owns `covers`, `compare`, `SessionSaw` and Previewing.
- **F1e (new)** owns `crates/docket-eval/**` and `eval/**`.
- The verify commands are unchanged, plus `cargo test -p docket-eval`.
<!-- paths: end -->
