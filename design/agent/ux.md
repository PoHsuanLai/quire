<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/ux.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# Area "ux": the companion's user-facing surfaces (quire components, sill surfaces)

Status: plan for the implementation spec. Grounded in COMPANION.md settled items 1-11, cua-integration.md §4.3-§5 and §7, agent-ux.md P1-P8, models.md (Intents1 proposal), design/30 (§0 rules, §1.5 vocabulary, VoiceOrb row, §3.4), design/22 §3/§9, design/31 §2.3/§4.5/§5.5/§5.6, and the code in quire `ds`/`ds-core`/`ds-settings`, sill `sill-launcher`/`sill-launcher-ui`/`sill-ipc`/`sill-settings`/`sill-bar`, and detent.

Other area names below ("actions", "context", "agent", "cua", "memory", "models", "security", "compositor") are my best guess at the sibling spec files. Map them to the real names when you merge.

---

## 1. Scope and non-goals

**In scope**
- **Summon.** Double-tap ⌘. Toshy handles it now and the compositor later. In a quire field, the field becomes the prompt. Anywhere else, the launcher opens in companion mode. Double-tap again or Esc returns.
- **Launcher companion mode** and its coupling with app actions: headless runs, previews, compact forms, promote into the app window.
- **Presence.** One `CompanionPresence` vocabulary. `CompanionOrb` built on `VoiceOrb`. A bar item and a control-centre module. The acting-here glow is data that the compositor draws.
- **Answer surfaces.** Native cards: text answer, draft reply, proposed event, plan list with run/edit, inline replace with undo, compact form, refusal. Every card has a footer showing which model served it, the sources and the scope.
- **CUA run rows**: Watch, Take over, Resume, Hand back, Stop, Replay, Undo all.
- **Compositor-drawn confirmation**: the visuals and the arming behaviour. The request type comes from "actions"/"security".
- **Memory control UI**: timeline, forget (with cascade preview), keep or discard pending facts, the consolidation diff, export.
- **Activity / undo strip.**
- **Model picker**: the session chip in the launcher. detent's per-tier pickers come from schema and live modules.
- **Settings**: the `companion` domain keys, a new `Page::Intelligence`, the control-centre module, and the page layout.

**Non-goals**
- The planner, router policy, typed-action registry, Context call, memory store, inferd routing, the CUA loop, and the compositor protocols. Those belong to other areas. UX states only what it consumes from them.
- Voice ("hold ⌘"). The prop slot is reserved; nothing else.
- Threads as windows (agent-ux P6), ambient suggestions (P9), ask-the-screen (P10).
- Restyling the command pill or list search. design/30 §3.4 says "command pill and list search stay as is". Companion mode is purely additive: a field in `Input` mode is byte-for-byte what it is today.
- Visual values. Every look below marked **[design review]** is a placeholder for the user's design pass. Only the interfaces are frozen.

---

## 2. Repos and crates

Dependency direction (unchanged rules): `ds-core < ds-style < ds-motion < ds < ds-shell`. The pure sill crates sit below the sill surface crates. The daemons' wire crates are leaves that sill depends on, the same way it depends on `keycap-protocol` today. quire never names a companion daemon crate: its types are presentational, and sill converts wire types into props.

| Piece | Lives in | New / existing | Notes |
|---|---|---|---|
| `CompanionPresence` (5 states) | quire `ds-core::vocab` | new enum | the one home; sill-model and ds both name it |
| `CompanionOrb`, `ContextChips`, prompt mode + `CompanionPort` seam, answer cards, `PlanList`, `ReplaceBar`, `CompactForm`, `RunRow`, `ActivityStrip`, `MemoryTimeline`, `ServedByChip` | quire `ds::components::companion/` | new group, layer order `… < editor < chrome < companion < app` | generic: both apps and the launcher draw them |
| `TextField` / `CommandPalette` prompt props | quire `ds::components::fields`, `menus::palette` | existing, gains props | `mode: FieldMode` and `chips` only; default `Input` = today |
| `EditSurface` proposal marks | quire `ds::editor` | existing, gains `proposals` prop | generic strike/insert spans (like `spell::marks`) |
| `ConfirmCard` (content of the trusted surface), `GlowState`/`GlowSpec`/`glow_spec` | quire `ds-shell::{confirm, tokens::glow}` | new | shell-only |
| Orb period tokens `OrbListen`, `OrbWork`, `OrbAct` | quire `ds-style::tokens::duration` | new tokens | lint `RawDuration`. Needs design/30 §1.2 rows (user) |
| `Page::Intelligence` | quire `ds-settings::schema::key::Page` | new variant | then detent `page.rs` row + icon |
| Launcher verbs: `ProviderKind::{Companion, Runs, Memory}`, `Subject`/`Preview` variants, `Activation::Companion(CompanionAct)` | sill `sill-launcher` | existing closed enums gain variants | warn-before-lint practice; wire-pin test extended |
| Companion service model: summon machine, double-tap machine, presence, runs, activity, answers | sill `sill-model::companion/` | new service dir (`model.rs`, `command.rs`, `step.rs`, `summon.rs`, `tap.rs`, `tests.rs`) | depends on the areas' serde-only wire crates |
| Companion service effects (D-Bus client to agent/cua/memory daemons; app `Summon` call) | sill `sill-services::companion/{mod.rs, backend/}` | new | fake backend for tests |
| `sill companion summon\|tap\|dismiss\|stop-all` | sill `sill-ipc::show` (`ShowRequest::Companion`), `sill/src/cli` | new request | |
| Providers `CompanionProvider`, `RunsProvider`, `MemoryProvider` | sill `sill-search::providers/{companion,runs,memory}/` | new | rows from the companion service state |
| Wire to props conversions (`view_of_*`) | sill `sill-ui-kit::companion/` | new module | the only place wire types meet ds props |
| Launcher companion mode | sill `sill-launcher-ui::launcher/{mode.rs, companion_keys.rs, companion_pane.rs}` | new files; `keys.rs`/`session.rs`/`panel.rs` gain a mode | |
| Bar item + control-centre module | sill `sill-bar::control_center::modules::companion`, `sill-settings::control_center::ControlCenterModule::Companion` | new variant + module | `InMenuBar::Show` gives the bar orb |
| Confirmation surface, phase A (no fork) | sill `sill-overlays::confirm/` | new surface | not unforgeable; see §7 Q7 |
| Glow, phase A (no fork) | sill `sill-overlays::glow/` | new input-transparent layer surface | phase B: the compositor draws it |
| `companion` settings domain | sill `sill-settings::companion.rs` | new domain | `Page::Intelligence` |
| COSMIC binding + Toshy snippet | sill `dist/cosmic/.../custom`, `dist/toshy/companion_tap.py` | new rows/files | |

Adopted crates: nothing new. Existing pins only: dioxus, serde, `ds-core::word::Word` derive, zbus (sill-services only).

---

## 3. Frozen interfaces

### 3.1 quire `ds-core::vocab` — freeze now

```rust
/// The companion's one presence (COMPANION 8): a look of the orb and the window glow, never a panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanionPresence {
    /// Still; zero frames anywhere.
    #[default]
    Idle,
    /// Taking a prompt (field focused in prompt mode; later: voice).
    Listening,
    /// Thinking or running typed actions out of sight.
    Working,
    /// Acting in a window: the compositor glows that window (and spot).
    Acting,
    /// Blocked on the person: a question, a confirmation, a taken-over run. Still (zero frames).
    Waiting,
}
```

### 3.2 quire `ds::components::companion::orb` — types freeze now, view later

```rust
/// The orb's size ladder [design review: px values]. Inline 14, Bar 16, Field 20, Module 40, Hero 192.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum OrbSize { Inline, Bar, #[default] Field, Module, Hero }

/// The period a moving orb turns at; each is a DurationToken (OrbListen, OrbWork, OrbAct).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum OrbPeriod { Listen, Work, Act }

/// Tone for the stylesheet (`data-tone`) [design review].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum OrbTone { #[default] Calm, Bright, Held }

pub struct OrbLook { pub activity: Activity, pub period: Option<OrbPeriod>, pub tone: OrbTone }

/// Pure, table-tested. Idle and Waiting are always Inactive (zero frames); Reduced motion makes all Inactive.
pub fn orb_look(presence: CompanionPresence, motion: MotionLevel) -> OrbLook;

#[component]
pub fn CompanionOrb(
    presence: CompanionPresence,
    #[props(default)] size: OrbSize,
    #[props(default)] common: Common,   // aria_label defaults to the presence's words ("Companion is working")
) -> Element; // draws VoiceOrb { size, activity, period } + data-presence/data-tone
```

`VoiceOrb` keeps its catalogue contract unchanged. `CompanionOrb` is a wrapper, so R3 holds.

### 3.3 Context chips and prompt mode — freeze now

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum ChipKind { Query, Results, Selection, Window, App, Space, Mention, Text }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Removal { #[default] Removable, Fixed }

/// What the prompt carries, shown as chips; the chips are the consent surface (agent-ux P2/P7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextChip { pub kind: ChipKind, pub label: String, pub count: Option<Count>, pub removal: Removal }

#[component]
pub fn ContextChips(chips: Vec<ContextChip>, #[props(default)] on_remove: EventHandler<usize>) -> Element;

/// A field's mode. `Input` is today's field, unchanged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum FieldMode { #[default] Input, Prompt }

/// Where a summon in a field of this kind puts the prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PromptHost {
    /// Plain / Search single-line fields and the palette: the field itself becomes the prompt.
    Field,
    /// Multiline fields and EditSurface: a prompt anchored at the selection; answer = inline replace.
    Anchored,
    /// Secure fields: never; the summon falls through to the launcher, with no field context.
    Refused,
}
pub fn prompt_host(kind: FieldKind) -> PromptHost; // total, table-tested

/// What the field gives back when prompt mode ends: the query verbatim and its caret.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept { pub query: String, pub caret: CaretSpan }

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum PromptState { #[default] Off, On { kept: Kept, chips: Vec<ContextChip>, text: String, phase: PromptPhase } }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PromptPhase { Composing, Sent, Answered }

pub enum PromptIn { Summon { kept: Kept, chips: Vec<ContextChip> }, Typed(String), Send, Answered, Escape, SummonAgain, ChipRemoved(usize) }
pub enum PromptOut { Enter, Submit { prompt: String, chips: Vec<ContextChip> }, Restore(Kept), CancelAsk }
pub fn step(state: PromptState, input: PromptIn) -> (PromptState, Vec<PromptOut>); // pure
```

`TextField` gains `#[props(default)] mode: FieldMode` and `#[props(default)] chips: Vec<ContextChip>`. In `Prompt` mode:
- the leading magnifier becomes `CompanionOrb { size: Field }`;
- the query becomes a `Query` chip and the field empties for the prompt **[design review: chip vs keep text]**;
- the placeholder is short Mac copy, e.g. "Ask about these results" **[design review]**.

`CommandPalette` gains the same two props and passes them to its field.

**The app seam.** Apps do nothing; ds wires it.

```rust
/// Installed by the app's companion client (context area's crate); `NoPort` default = feature absent.
pub trait CompanionPort: 'static {
    /// The field ds found focused answers a summon synchronously.
    fn on_summon(&self, handler: Callback<SummonSerial, SummonAnswer>);
    /// Send the prompt with the chips the person kept.
    fn ask(&self, serial: SummonSerial, prompt: String, chips: Vec<ContextChip>);
    /// Answers for that serial, as props (the port converts wire -> AnswerView).
    fn answers(&self) -> ReadSignal<Option<(SummonSerial, AnswerView)>>;
    fn cancel(&self, serial: SummonSerial);
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SummonSerial(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word, Serialize, Deserialize)]
pub enum SummonAnswer { TookField, TookAnchored, Restored, Declined }
/// Registers a field as promptable; TextField and CommandPalette call it themselves.
pub fn use_prompt_target(handle: FieldHandle, kind: FieldKind) -> PromptBinding; // mode(), chips(), state()
```

### 3.4 Answer cards — freeze now

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct AnswerId(pub u64);
#[derive(Debug, Clone, PartialEq, Eq, Hash)] pub struct CardActionId(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum AnswerView {
    Text(TextAnswer),
    DraftReply(DraftReply),
    ProposedEvent(ProposedEvent),
    Plan(PlanView),
    Replace(ReplaceProposal),
    Form(CompactForm),
    Refused(RefusedView),
}

/// Every card's footer (agent-ux §3: served-by, sources, scope).
pub struct CardFooter { pub served_by: ServedByView, pub sources: Vec<SourceChip>, pub scope: Vec<ContextChip> }
pub struct ServedByView { pub model: String, pub place: ServedPlace }
#[derive(Word)] pub enum ServedPlaceKind { ThisComputer, LocalNetwork, Cloud }
pub struct ServedPlace { pub kind: ServedPlaceKind, pub provider: Option<String> }
pub struct SourceChip { pub label: String, pub icon: IconSource, pub key: SourceKey } // key opaque, opened by sill
pub struct SourceKey(pub String);

/// Mirrors the actions area's Effect for drawing only.
#[derive(Word)] pub enum EffectMark { Read, UndoableWrite, Outbound, Destructive }
#[derive(Word)] pub enum ActionRole { Primary, Secondary, Destructive }
pub struct CardAction {
    pub id: CardActionId, pub label: String, pub role: ActionRole, pub effect: EffectMark,
    pub keys: Option<Shortcut>, pub availability: Availability,
}

#[derive(Word)] pub enum Streaming { Arriving, Complete }
pub struct TextAnswer { pub body: Vec<TextLine>, pub streaming: Streaming, pub actions: Vec<CardAction>, pub footer: CardFooter }

pub struct PersonLine { pub name: String, pub address: String, pub avatar: AvatarSource }
pub struct DraftReply {
    pub to: Vec<PersonLine>, pub subject: String, pub quoted: Option<String>,
    pub body: String,                  // editable in place (TextField Multiline) [design review]
    pub actions: Vec<CardAction>,      // typically Send (Outbound), Save Draft (UndoableWrite), Open in Mail (promote)
    pub footer: CardFooter,
}

/// Minutes from midnight; a day strip, not a calendar.
pub struct Span { pub from: u16, pub to: u16 }
#[derive(Word)] pub enum BusyKind { Existing, Proposed, Conflict }
pub struct DaySlot { pub span: Span, pub kind: BusyKind, pub label: String }
pub struct ProposedEvent {
    pub title: String, pub when: String, pub place: Option<String>, pub people: Vec<PersonLine>,
    pub day: Vec<DaySlot>, pub actions: Vec<CardAction>, pub footer: CardFooter,
}

pub enum RefusedView {
    NeedsCloud { class: String },      // porter RequiresCloud; action "Allow Once…" -> accounts-ui
    NotAllowed { what: String },       // policy Denied / grant missing
    NoWay { app: String },             // no typed action, CUA off for app
    OverBudget,
    Failed { why: String },
}

#[component] pub fn AnswerCard(view: AnswerView, #[props(default)] focused: Option<usize>, on_action: EventHandler<CardActionId>, #[props(default)] on_edit: EventHandler<(FieldKeyId, FormValue)>, #[props(default)] common: Common) -> Element;
```

**Plan list (agent-ux P4).**

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)] pub struct StepId(pub u32);
#[derive(Word)] pub enum Inclusion { Included, Excluded }
pub enum StepState { Pending, Running, Done, Failed(String), Skipped, Undone }
pub struct PlanStep { pub id: StepId, pub label: String, pub detail: Option<String>, pub app: Option<IconSource>, pub effect: EffectMark, pub inclusion: Inclusion, pub state: StepState }
pub struct PlanGroup { pub label: String, pub effect: EffectMark, pub steps: Vec<PlanStep> }
#[derive(Word)] pub enum StopWhy { You, Failed, NeedsYou }
#[derive(Word)] pub enum Finish { AllDone, Partly }
pub enum PlanPhase { Draft, Running { at: StepId }, Stopped { at: StepId, why: StopWhy }, Finished(Finish), Undoing, Undone }
pub struct PlanView { pub title: String, pub groups: Vec<PlanGroup>, pub phase: PlanPhase, pub footer: CardFooter }

pub enum PlanIn { Toggle(StepId), Run, Edit, Stop, Started(StepId), Done(StepId), Failed(StepId, String), Asked(StepId), Resume, UndoAll, AllUndone }
pub enum PlanOut { Run(Vec<StepId>), Edit, Stop, Resume, UndoAll }
pub fn plan_step(view: PlanView, input: PlanIn) -> (PlanView, Vec<PlanOut>); // pure
```

**Inline replace (P1).**

```rust
pub struct UndoKey(pub String);
pub enum ReplacePhase { Proposed, Applied { undo: UndoKey }, Undone, Discarded }
pub struct ReplaceProposal { pub original: String, pub proposed: String, pub phase: ReplacePhase, pub footer: CardFooter }
pub enum ReplaceIn { Apply, Discard, Applied(UndoKey), Undo, Undone, Expired }
pub enum ReplaceOut { Apply, Discard, Undo(UndoKey) }
pub fn replace_step(p: ReplacePhase, i: ReplaceIn) -> (ReplacePhase, Vec<ReplaceOut>);
// ds::editor: EditSurface gains `proposals: Vec<ProposalSpan>`; ProposalSpan { range: TextRange, kind: ProposalKind {Removed, Inserted} } [design review: marks]
```

**Compact form.** Rendered from the actions area's `NeedsParam` / `ParamDecl`.

```rust
pub struct FieldKeyId(pub String);
#[derive(Word)] pub enum Need { Required, Optional }
pub enum FormInput { Text, Lines, Number, Date, Time, Choice(Vec<(String, String)>), Entity { suggestions: Vec<EntityOption> } }
pub struct EntityOption { pub key: String, pub title: String, pub subtitle: String, pub icon: IconSource }
pub enum FormValue { Empty, Text(String), Number(String), Choice(String), Entity(String) }
pub struct FormField { pub key: FieldKeyId, pub label: String, pub input: FormInput, pub need: Need, pub value: FormValue, pub validity: Validity }
pub struct CompactForm { pub title: String, pub fields: Vec<FormField>, pub submit: CardAction, pub footer: CardFooter }
```

### 3.5 Run rows, activity strip, memory, served-by chip — freeze now

```rust
#[derive(Word)] pub enum RunPlace { InPlace, AgentWorkspace, NestedSession }
#[derive(Word)] pub enum RunState { Starting, Running, Paused, NeedsYou, TakenOver, Done, Failed, Stopped, BudgetOut }
pub struct AppMark { pub icon: IconSource, pub name: String }
pub struct RunRowView {
    pub goal: String, pub app: AppMark, pub step: Count, pub budget: Option<Count>,
    pub thought: Option<String>, pub place: RunPlace, pub state: RunState, pub served_by: ServedByView,
    pub actions: Vec<CardAction>,   // labels and keys from sill (verbs are sill's, §3.7)
}
#[component] pub fn RunRow(view: RunRowView, #[props(default)] selection: Selection, on_action: EventHandler<CardActionId>) -> Element;

#[derive(Word)] pub enum Actor { You, Companion }
pub enum ActivityState { Running(Option<Fraction>), NeedsYou, Done, Failed, Undone, Expired }
pub enum Undoable { Undo, UndoAll(Count), Not }
pub struct ActivityEntry { pub key: String, pub title: String, pub actor: Actor, pub app: Option<AppMark>, pub when: String, pub state: ActivityState, pub undo: Undoable }
#[component] pub fn ActivityStrip(entries: Vec<ActivityEntry>, on_undo: EventHandler<String>, on_open: EventHandler<String>) -> Element;

#[derive(Word)] pub enum FactOrigin { YouSaid, Companion, Consolidated }
#[derive(Word)] pub enum FactStanding { Kept, Pending, Superseded } // Pending = from untrusted text, awaits you (COMPANION 11)
pub struct MemoryRowView { pub key: String, pub fact: String, pub learned: String, pub space: String, pub sources: Vec<SourceChip>, pub origin: FactOrigin, pub standing: FactStanding }
pub struct MemoryDay { pub label: String, pub rows: Vec<MemoryRowView> }
#[derive(Word)] pub enum MemoryVerb { Forget, Keep, Discard, OpenSource, Export }
pub struct ForgetPreview { pub fact: String, pub derived: Count, pub procedures: Count } // "Also forgets 3 facts and 1 routine."
pub enum DiffLine { Added(String), Merged { from: Vec<String>, into: String }, Dropped(String) }
pub struct ConsolidationDiff { pub night: String, pub lines: Vec<DiffLine> }
#[component] pub fn MemoryTimeline(days: Vec<MemoryDay>, on_verb: EventHandler<(String, MemoryVerb)>) -> Element;
#[component] pub fn ConsolidationView(diff: ConsolidationDiff) -> Element;

/// The launcher's model chip; click opens a quire Menu of models (no new picker component).
pub struct ModelOption { pub key: String, pub label: String, pub place: ServedPlace, pub state: ModelState }
pub enum ModelState { Ready, Loading, NeedsDownload { size: String }, Unavailable(String) }
#[component] pub fn ServedByChip(current: ServedByView, options: Vec<ModelOption>, on_choose: EventHandler<String>) -> Element;
```

### 3.6 quire `ds-shell` — GlowState freeze now, GlowSpec values later

```rust
pub enum GlowState { None, Working, Acting { spot: Option<GlowSpot> }, Waiting }
pub struct GlowSpot { pub x: Px, pub y: Px, pub radius: Px }           // logical px inside the window
pub struct GlowSpec { pub colours: [Srgb; 3], pub width: Px, pub blur: Px, pub period: Option<Duration>, pub strength: Fraction }
/// From the orb tokens, so the compositor never links quire. None and Waiting have period None (zero frames).
pub fn glow_spec(state: &GlowState, scheme: Scheme) -> GlowSpec;      // [design review: values]

/// The content of the trusted confirmation surface.
pub struct ConfirmView {
    pub asker: AppMark,             // "Companion"
    pub target: AppMark,            // the app/window it acts in
    pub title: String,              // router-generated, never model text ("Send this email?")
    pub facts: Vec<Fact>,           // To, Subject… (FactList)
    pub verb: String,               // the default button ("Send")
    pub effect: EffectMark,
    pub taint: TaintNote,
    pub offer: ScopeOffer,
    pub arm: ArmState,
}
pub enum TaintNote { Clean, ReadUntrusted { source: String } } // "It read a message from a sender you don't know."
#[derive(Word)] pub enum ScopeOffer { OnceOnly, OnceOrAlways }
#[derive(Word)] pub enum ArmState { Arming, Armed }
pub enum ConfirmAnswer { Allow { always: AlwaysChoice }, Deny }
#[derive(Word)] pub enum AlwaysChoice { Once, Always }
#[component] pub fn ConfirmCard(view: ConfirmView, on_answer: EventHandler<ConfirmAnswer>) -> Element;
```

### 3.7 sill — freeze now

**`sill-launcher`** (pure; extends the pinned wire test):

```rust
// ids.rs: ProviderKind gains, at the end of Tab order (ALL becomes [_; 12]):
Companion, Runs, Memory,

// keys (own newtypes, transparent serde)
pub struct AnswerKey(pub u64); pub struct RunKey(pub u64); pub struct ActivityKey(pub u64); pub struct MemoryKey(pub String);

// subject.rs
Subject::Answer(AnswerKey), Subject::Run(RunKey), Subject::Activity(ActivityKey), Subject::Memory(MemoryKey),
// preview.rs (the pane reads live state from the companion service by key)
Preview::Answer(AnswerKey), Preview::Run(RunKey), Preview::Memory(MemoryKey),

// activation.rs
Activation::Companion(CompanionAct),

#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum CompanionAct {
    /// Send the prompt with this opening's context chips.
    Ask { prompt: String },
    /// A card button.
    Card { answer: AnswerKey, action: String },
    /// Open the answer's object in its app (actions' Follow::Open) and close.
    Promote { answer: AnswerKey },
    Run { run: RunKey, control: RunControl },
    Undo { entry: ActivityKey },
    Memory { item: MemoryKey, verb: MemoryVerb },
    /// Switch the panel's mode in place.
    Mode(LauncherMode),
}
pub enum RunControl { Watch, TakeOver, HandBack, Pause, Resume, Stop, Replay, UndoAll }
pub enum MemoryVerb { Forget, Keep, Discard, OpenSource, Export }   // launcher's verb, sill-ui-kit maps ds's
pub enum LauncherMode { Search, Companion }
```

`Activation::Intent { action, args }` belongs to "actions". It is listed here only because launcher coupling needs it.

**`sill-ipc::show`**:

```rust
ShowRequest::Companion(CompanionRequest),
pub enum CompanionRequest {
    /// The double-tap fired (Toshy route B: a single tap; sill's DoubleTap machine decides).
    Tap,
    /// A detected double-tap (Toshy route A, the compositor later).
    Summon,
    Dismiss,
    /// Pause every run in the focused Space (the kill switch's soft form; the hard chord is the compositor's).
    StopAll,
}
```

CLI: `sill companion tap|summon|dismiss|stop-all`.

**`sill-model::companion`**:

```rust
// tap.rs: the double-tap machine (also the compositor's reference implementation)
pub enum Tap { Rest, Armed { since: Stamp } }
pub enum TapIn { Tap, Elapsed }  impl From<Elapsed> for TapIn
pub enum TapOut { Summon }
pub struct TapParams { pub window: Ms }      // companion.double_tap_ms
impl Machine for Tap { /* wake = Armed.since + window */ }

// summon.rs: where a summon goes
pub struct FocusFacts { pub window: Option<WindowKey>, pub app: Option<AppId>, pub launcher: LauncherFacts, pub serves_companion: Serves }
pub enum LauncherFacts { Closed, Open(LauncherMode) }
pub enum Serves { Yes, No }                   // the app owns the companion client name on the bus
pub enum Summon {
    Rest,
    AskingApp { app: AppId, window: WindowKey, serial: SummonSerial, until: Stamp },
    InApp { app: AppId, serial: SummonSerial },
    InLauncher { origin: LauncherOrigin },
}
pub enum LauncherOrigin { Fresh { window: Option<WindowKey> }, FromSearch }
pub enum SummonIn { Summon(FocusFacts), AppAnswered(SummonAnswer), Elapsed, LauncherClosed, Dismiss }
pub enum SummonOut { AskApp { app: AppId, serial: SummonSerial }, OpenLauncher { mode: LauncherMode, window: Option<WindowKey> }, SwitchLauncher(LauncherMode), CloseLauncher }
pub struct SummonParams { pub app_wait: Ms, pub in_field: InField }    // app_wait proposed 80 ms
impl Machine for Summon { … }

// model.rs / command.rs: the service (State/Command/Event/Effect + step), wire types from the areas' crates
pub struct CompanionState { pub presence: CompanionPresence, pub summon: Summon, pub tap: Tap,
    pub answers: BTreeMap<AnswerKey, AnswerFacts>, pub runs: BTreeMap<RunKey, RunFacts>,
    pub activity: Vec<ActivityFacts>, pub glow: BTreeMap<WindowKey, GlowFacts> }
pub fn presence_of(state: &CompanionState) -> CompanionPresence; // priority Waiting > Acting > Working > Listening > Idle
pub fn run_controls(state: RunStateWire, place: RunPlaceWire) -> Vec<RunControl>; // table, §4.6
```

**`sill-settings::companion`** (data; full now):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ds_settings::SettingsSchema)]
#[serde(default)]
#[settings(file = "sill/settings.toml", domain = "companion", page = Page::Intelligence)]
pub struct CompanionSettings {
    #[settings(label = "Press ⌘ twice to ask", section = "Companion")]           pub summon: SummonKey,       // DoubleTapCommand | Off (toggle)
    #[settings(label = "In a text field", section = "Companion")]                pub in_field: InField,       // Prompt | Launcher
    #[settings(label = "Double-tap speed", range = "150..=600", unit = "ms", section = "Companion", advanced)] pub double_tap_ms: Ms, // 350, conf L
    #[settings(label = "Notify when it needs you", section = "Activity")]        pub notify: NotifyWhen,      // NeedsYou | Never (toggle)
    #[settings(label = "Keep undo for", range = "1..=168", unit = "h", section = "Activity", advanced)] pub undo_keep_h: Count, // 24
    #[settings(label = "Pause when you touch its window", section = "Computer use")] pub touch: TouchPause,  // Pause | Never (toggle)
    #[settings(label = "Long tasks run", section = "Computer use")]              pub run_place: RunPlaceDefault, // Auto | Here | Aside
    #[settings(label = "Confirmation delay", range = "300..=1500", unit = "ms", section = "Confirmations", advanced)] pub confirm_arm_ms: Ms, // 500
}
```

`ControlCenterModule::Companion` gets its own `InMenuBar` (default `Show` while AI is on) **[design review]**.

Keys on `Page::Intelligence` owned by other areas (UX fixes only the section order: Companion, Models, Memory, Computer use, Privacy):
- "models": `ai.local_only`, `ai.floor.<class>`, tiers via the inferd live module.
- "memory": `memory.*`, live module.
- "cua"/"security": `cua.apps` rows, budgets.

**D-Bus that UX needs from other areas** (freeze together with them; suggested shapes):

| Owner | Member | UX use |
|---|---|---|
| context (app side, the app's own name) | `Summon(serial t, origin s) -> s` (`SummonAnswer` slug) | §4.1. ds answers via `CompanionPort` |
| agent | `org.quire.Companion1`: property `Presence s`; `Ask(prompt s, context s, origin s, parent_window s) -> o` Answer; Answer `.Updated(answer s)` (serde `AnswerWire`), `.Act(action s) -> o`, `.Cancel()`; signal `ActivityAppended(s)`, `ActivityChanged(s)`; `Undo(entry s) -> o`; `PauseSpace(space s)` | launcher, ds port, bar |
| cua | `org.quire.Cua1` as in cua-integration §3.4 (`State`, `Step`, `Window`, `StepStarted`, `ThoughtDelta`, `AskUser`, `Finished`, `Pause/Resume/TakeOver/HandBack/Cancel`) plus `Replay(run)` and `UndoAll(run)` | `RunRow` |
| memory | `Timeline(space s, before t, limit u) -> s`, `ForgetPreview(item s) -> s`, `Forget(item s) -> o`, `Keep(item s)`, `Discard(item s)`, `Diff(night s) -> s`, `Export(space s, fd h) -> o` | `MemoryTimeline` |
| models | `Inference1.Usage`, and a `Models(tier s) -> s` list with state; `Choose(session, model)` | `ServedByChip` |
| actions / security | `ConfirmRequest` type (router-generated text, effect, taint, offer) and the compositor delivery | `ConfirmCard` |
| compositor | `quire_agent_glow_v1.set(toplevel, GlowState, GlowSpec)`; `quire_trusted_surface_v1` (role, hardware-only input, no agent seat, no capture, compositor frame) | glow, confirm |

Each area ships a **serde-only wire crate** (no zbus), the same way keycap ships `keycap-protocol`. `sill-model` depends on those crates.

---

## 4. State machines

### 4.1 Summon (sill-model `Summon`, effects in the daemon)

| State | Event | Next | Out |
|---|---|---|---|
| Rest | Summon, launcher Open(Search) | InLauncher{FromSearch} | SwitchLauncher(Companion): the palette field becomes the prompt, keeping the query, rows and selection |
| Rest | Summon, launcher Open(Companion) | Rest | SwitchLauncher(Search) if origin FromSearch, else CloseLauncher |
| Rest | Summon, focused app Serves::Yes, in_field = Prompt | AskingApp{until = now + app_wait} | AskApp |
| Rest | Summon, otherwise | InLauncher{Fresh{window}} | OpenLauncher(Companion, window) |
| AskingApp | AppAnswered(TookField\|TookAnchored) | InApp | — |
| AskingApp | AppAnswered(Restored) | Rest | — |
| AskingApp | AppAnswered(Declined) or Elapsed | InLauncher{Fresh} | OpenLauncher(Companion, window) |
| InApp | Summon (same app focused) | AskingApp | AskApp (the app's prompt machine reads it as SummonAgain and restores) |
| InLauncher | Dismiss / LauncherClosed | Rest | CloseLauncher (or SwitchLauncher(Search) for FromSearch) |

### 4.2 Double-tap (`Tap`)

| State | Event | Next | Out |
|---|---|---|---|
| Rest | Tap@t | Armed{t} (wake t + window) | — |
| Armed{s} | Tap@t, t − s ≤ window | Rest | Summon |
| Armed{s} | Elapsed | Rest | — |

The machine is idle in Rest and runs no timer there.

### 4.3 Prompt field (ds `PromptState`)

| State | Event | Next | Out |
|---|---|---|---|
| Off | Summon{kept, chips} | On{Composing, text ""} | Enter |
| On(Composing) | Typed(s) | text = s | — |
| On(Composing), text non-empty | Send | On(Sent) | Submit |
| On(Sent) | Answered | On(Answered) | — |
| On(Answered) | Typed | On(Composing) (follow-up) | — |
| On(Answered\|Sent) | Escape | On(Composing) | CancelAsk if Sent |
| On(Composing) | Escape / SummonAgain | Off | Restore(kept) |
| On(any) | SummonAgain | Off | Restore(kept), CancelAsk if Sent |
| On | ChipRemoved(i) | chip removed (Fixed chips ignore it) | — |

### 4.4 Launcher companion mode (sill-launcher-ui; extends `LauncherState`)

`LauncherState` gains `mode: Mode`, where `Mode = Search | Companion(Pane)` and `Pane = Composing | Thinking(AnswerKey) | Showing(AnswerKey) | Form(AnswerKey)`.

Rows in companion mode, in order:
1. typed interpretations (actions area, before any model call);
2. the "Ask" row;
3. the live answer;
4. active runs;
5. recent activity.

The selected answer's card is the palette `aside` (an Element slot; `PaneContent` is not touched) **[design review: card in pane vs replacing the list]**.

| Key | Composing | Showing / Form |
|---|---|---|
| Enter | Ask row: `Ask`; other rows: as today | the card's Primary action (Outbound asks through the compositor) |
| Tab / Shift-Tab | Tab cycles the provider filter (Companion, Runs, Memory are filters) | moves through card actions (pane_keys) |
| ⌘Enter | — | Promote |
| ⌘. | Stop the selected run | Cancel the streaming answer |
| Esc | origin FromSearch: back to Search with the query restored; Fresh: close | back to Composing |
| double-tap ⌘ | same as Esc to origin | same |

### 4.5 Answer action lifecycle (per card action)

`Idle → Running` (button Busy) → one of:
- `Done{undo}`: the card shows "Undo" for `ToastHold`, and the entry joins the activity strip;
- `NeedsConfirm`: presence becomes Waiting and the compositor shows the confirm → Allow returns to Running, Deny returns to Idle with a quiet note;
- `NeedsParam`: Form;
- `Refused(RefusedView)`.

### 4.6 Run controls (`run_controls`, table-tested)

| RunState | InPlace | AgentWorkspace / Nested |
|---|---|---|
| Starting, Running | Pause, TakeOver, Stop | Watch, Pause, TakeOver, Stop |
| Paused | Resume, TakeOver, Stop | Watch, Resume, TakeOver, Stop |
| NeedsYou | Stop (the answer is in its form / confirm) | Watch, Stop |
| TakenOver | HandBack, Stop | HandBack, Stop |
| Done | Replay, UndoAll | Replay, UndoAll |
| Failed, Stopped, BudgetOut | Replay, UndoAll | Watch (last frame), Replay, UndoAll |

### 4.7 Confirmation surface

`Arming{until = shown + confirm_arm_ms}`: buttons are drawn but disabled and Enter is ignored. Then `Armed`, followed by `Answered(Allow{Once|Always}|Deny)` or `Withdrawn` (the run was cancelled).

- Esc means Deny in both Arming and Armed.
- Only hardware input counts. That filter is the compositor's; in phase A it is best effort.
- One confirmation at a time; others queue (FIFO per Space).
- "Always" shows only when `offer = OnceOrAlways`. The router never offers it with Taint or Destructive.

### 4.8 Activity entry

`Running(p) → NeedsYou → Running`; `Running → Done | Failed`; `Done → Undone` (on Undo) `| Expired` (after `undo_keep_h`). Only NeedsYou notifies, and only when `notify = NeedsYou`.

### 4.9 Memory item

`Pending → Kept | Discarded`. `Kept → Forgetting` (after the ForgetPreview Alert) `→ Gone`. The row leaves with a roster exit.

### 4.10 Presence and glow derivation (pure)

`presence_of`:
- Waiting if any confirm, ask, NeedsYou or TakenOver;
- otherwise Acting if any run is Running InPlace or a typed action targets a visible window;
- otherwise Working if a prompt is Sent or something runs aside;
- otherwise Listening if a prompt is Composing;
- otherwise Idle.

Glow per window: `None` unless leased. A leased window glows Acting while a step is executing (spot = the proposed point or node bounds), Working between steps, Waiting while paused on it.

---

## 5. Tests that pin the shapes

quire:
- `ds-core`: `companion_presence_words_round_trip` (Word ALL / slug / serde).
- `ds/companion/orb`:
  - `orb_look_table`: every presence × motion level;
  - `idle_and_waiting_never_move`;
  - SSR `companion_orb_ssr` (data-presence, aria label);
  - conformance `idle_orb_schedules_no_frame` (Harness virtual clock: no wake after mount in Idle or Waiting);
  - `presence_change_keeps_turn`.
- `prompt`: `prompt_host_is_total`; `prompt_step_table` (every row of §4.3); conformance `summon_in_search_field_keeps_query_and_restores_verbatim`, `secure_field_declines`, `input_mode_markup_unchanged` (golden equals today's TextField and CommandPalette goldens).
- `card`:
  - `answer_view_variants_render` (SSR golden per variant);
  - `outbound_action_never_primary_without_effect_mark`;
  - `footer_always_present` (every variant draws served-by);
  - lint `ds_lint::markup` clean.
- `plan`: `plan_step_table`; `excluded_steps_not_run`; `stop_keeps_done_steps`; `undo_all_only_after_run`.
- `replace`: `replace_step_table`; editor conformance `proposal_marks_do_not_enter_text`.
- `run_row`, `activity`, `memory`: SSR goldens; `forget_preview_names_counts`.
- `ds-shell`:
  - `glow_spec_idle_and_waiting_have_no_period`;
  - `confirm_card_title_is_given_text` (no model text slot);
  - `arm_state_disables_buttons`.
- `ds-settings`: `page_intelligence_label_and_order`.

sill:
- `sill-launcher`: `every_activation_has_a_pinned_wire_form_and_round_trips` (extended with all `CompanionAct` forms); `provider_kind_all_order` (12); preview_for over the new subjects.
- `sill-ipc`: request round-trip for `CompanionRequest`.
- `sill-model::companion`:
  - `tap_machine_table` (inside window, outside window, triple tap = one summon plus an armed state);
  - `tap_rest_has_no_wake`;
  - `summon_table` (every §4.1 row);
  - `presence_priority_table`;
  - `run_controls_table` (§4.6);
  - `wire_to_facts_total` (exhaustive match over each area's wire enums).
- `sill-settings`: `companion_schema_round_trip`, `every_key_in_catalogue_has_a_spec` with the new design/22 §3.x rows; `toggle_words` (summon, notify, touch become switches).
- `sill-ui-kit::companion`: `view_of_*_total`.
- `sill-launcher-ui`:
  - `companion_keys_table` (§4.4);
  - `escape_returns_to_search_with_query`;
  - `double_tap_in_open_launcher_switches_mode_in_place` (same serial, same rows, same selection);
  - surface test `answer_card_in_aside`.
- `sill-bar`: `companion_item_idle_zero_frames` (shell-host stats: no frames over 2 s of virtual time in Idle and Waiting).
- `sill` daemon (PrivateBus, scratch XDG): `summon_falls_back_to_launcher_after_app_wait`; `summon_routes_to_serving_app`; `stop_all_pauses_space`.

Fakes and fixtures:
- `FakeCompanionPort` (ds `testing` feature): scripted answers.
- `FakeCompanionBackend` (sill-services env seam): presence, answers, runs, activity, memory.
- One fixture JSON per `AnswerView` variant, run, memory day and diff, shared by goldens and gallery pages.
- A stand-in app in `sill-testkit/examples/companion_app.rs` that owns a bus name and answers `Summon`.
- detent fixture regenerated with `scripts/gen-fixtures.sh`.

---

## 6. Work breakdown

**Freeze wave** (parallel, one agent per repo; bodies `todo!()`, shape tests pass):

- **F1 quire.** Owns:
  - `ds-core/src/vocab.rs` (+ test);
  - `ds/src/components/companion/**` (all `model.rs` types complete; `step` functions `todo!()`; views render a placeholder root `div.ds-<name>` so the lint passes);
  - `ds/src/components/companion/mod.rs` and the `components/mod.rs` line;
  - `ds/src/assembly/sheets.rs` (empty sheets registered);
  - `ds-shell/src/{confirm,tokens/glow.rs}`;
  - `ds-settings/src/schema/key.rs` (`Page::Intelligence`).

  It does **not** touch TextField or CommandPalette yet. Verify: the quire gate (`cargo fmt --all --check; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --all-features; ./scripts/check-boundary.sh; ./scripts/check-consumer.sh; cargo deny check licenses`).
- **F2 sill** (after F1's `Page` and vocab land, since these are path deps). Owns:
  - `sill-launcher/src/{ids,subject,preview,activation}.rs` and the new `companion.rs`;
  - `sill-ipc/src/show.rs`;
  - `sill-model/src/companion/**`;
  - `sill-settings/src/{companion.rs, control_center.rs (variant), lib.rs}`;
  - `sill-ui-kit/src/companion/**` (signatures);
  - `sill-launcher-ui/src/launcher/{mode.rs, companion_keys.rs}` (types only);
  - `dist/cosmic/.../custom` (one row: unique chord to `sill companion tap`);
  - `dist/toshy/companion_tap.py`;
  - design/22 §3 rows (doc edit in quire, coordinated).

  Exhaustive matches that gain arms in the daemon return `Effect::Nothing` until W4. Verify: `cargo fmt --all --check; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace --features sill/debug; ./scripts/check-boundary.sh; cargo deny check licenses`.
- **F3 detent** (after F1). Owns `detent-model/src/page.rs`, `detent-ui/src/icons.rs`, fixtures. Verify: the detent gate plus `scripts/gen-fixtures.sh`.

**Fill waves** (file ownership is disjoint):

- **W1 quire, prompt and orb.**
  - companion `orb*`, `chips*`, `prompt/**`;
  - `fields/text_field.rs` + `menus/palette/command_palette.rs` (prop pass-through only);
  - `ds-style/src/tokens/duration.rs` (OrbListen/OrbWork/OrbAct; values need user approval);
  - conformance and gallery pages.
- **W2 quire, cards.** companion `card/**`, `plan/**`, `replace/**`, `form/**`; `editor/` proposal marks (`editor/proposals.rs` plus one prop in `surface.rs`).
- **W3 quire, rows.** companion `run_row/**`, `activity/**`, `memory/**`, `served_by.rs`; `ds-shell` confirm view, `glow_spec`.
- **W4 sill.** Can start on fakes after F2; needs W1 for the field.
  - `sill-model/companion` step bodies;
  - `sill-services/companion/**` (fake plus zbus backend behind the areas' wire crates);
  - `sill-search/providers/{companion,runs,memory}/**`;
  - `sill-launcher-ui/launcher/{companion_pane.rs, keys.rs, session.rs, panel.rs}`;
  - `sill/src/{cli,ipc,daemon}` companion arms.
- **W5 sill.**
  - `sill-bar/control_center/modules/companion/**` and its bar item;
  - `sill-overlays/{confirm,glow}/**` (phase A);
  - `sill-notify-ui` (NeedsYou notification only).
- **W6, integration** (after the agent, cua, memory and models areas fill their daemons): replace fakes; end-to-end in a nested compositor with a private bus and scratch HOME.

---

## 7. Dependencies, conflicts, and questions for the user

**Depends on**
- **actions**: Effect, Preview/Outcome/Refusal/ParamDecl (→ `CompactForm`), `Activation::Intent`, Follow::Open (promote), shared undo labels and keys, `ConfirmRequest`.
- **context**: the app-side `Summon` method, typed Context to chips, the client crate that implements `CompanionPort`.
- **agent**: `Companion1` (presence, Ask/Answer stream as `AnswerWire`, activity, undo, PauseSpace).
- **cua**: `Cua1` plus Replay and UndoAll; RunPlace.
- **memory**: timeline, forget preview and cascade, pending facts, diff, export.
- **models**: model list and state, session choice, `ServedBy`.
- **security**: router-generated confirm text, the taint flag, rules for the "Always" offer, the kill chord.
- **compositor**: double-tap detection (the `Tap` machine is the reference), the glow protocol, trusted surfaces, agent-seat exclusion.

**Conflicts found, with proposed resolutions**
1. **Esc on a run.** cua-integration §5 says Esc pauses a run and a second Esc cancels. The launcher rule is that Esc closes one level. Proposal: Esc keeps the launcher's meaning (the run continues, as on the Mac); stopping uses ⌘. or the Stop button, plus the compositor's hardware kill chord.
2. **"In a field the field becomes the prompt"** does not fit multiline and editor fields, where it would replace the body being edited. Proposal: an anchored prompt there (`PromptHost::Anchored`, Writing-Tools style); Secure fields always decline.
3. **R2 "one infinite loop".** design/30 allows the spinner and the Active orb. The compositor glow pulse while Working or Acting is a third loop, outside quire. Proposal: allow it only while a run is live, never in Idle or Waiting.
4. **Only the user changes design/30.** New catalogue rows (CompanionOrb, ContextChips, AnswerCard family, PlanList, RunRow, ActivityStrip, MemoryTimeline, ConfirmCard) and the three orb duration tokens need user approval. Until then they are marked "proposed" in DESIGN.md.
5. **agent-ux P6** (threads as windows) vs COMPANION ("no chat panel"). Promote opens the owning app's window. Threads stay deferred.
6. **One home vs three layers.** A wire enum (daemon), a verb (sill-launcher) and a view (ds) each own a separate concept. The `*_total` tests keep them in step.

**Questions only the user can answer**
1. In prompt mode, does the query collapse into a chip with the field emptied (my proposal), or stay as editable text?
2. Is the anchored prompt the right answer for multiline and editor fields?
3. Where does the orb live with the launcher closed? A bar item from the control-centre module ("Show in Menu Bar", on by default?) or hidden while Idle?
4. What do the Waiting look and the orb ladder sizes look like? Should the orb tint be the orb colours or the accent?
5. Does the answer card go in the launcher's preview pane or replace the results list? What does the "headless action done" feedback look like (toast with Undo vs a line in the strip)?
6. Glow look: width, colours, the acting spot. Is phase A (a sill overlay around the window rect, not unforgeable) acceptable before the compositor fork exists?
7. Confirmation: a sheet attached to the target window or a centred alert? Arm delay of 500 ms? Phase A (sill overlay, forgeable by uinput) only for development, or acceptable to ship?
8. Memory UI home: a launcher "Memory" filter for the timeline and forget, plus the detent Intelligence page for export and the diff? Or one dedicated window?
9. Settings page name: "Intelligence" (as in design/31) or "Companion"?
10. Undo wording for agent actions in app menus: "Undo Archive (Companion)", or a plain "Undo Archive" with the actor shown only in the activity strip?
11. Toshy route:
    - (A) Toshy detects the double-tap in Python and emits one unique chord to `sill companion summon`, which is the settled wording;
    - (B) Toshy maps a lone ⌘ tap to a unique key, and sill's tested `Tap` machine detects the double. This needs a spike: does a tap-vs-hold remap on ⌘ disturb ⌘-click and ⌘-drag?

    I recommend B if the spike passes, otherwise A.

### Critical Files for Implementation
- /home/pohsuanlai/quire/crates/ds/src/components/content/voice_orb/view.rs
- /home/pohsuanlai/quire/crates/ds/src/components/menus/palette/command_palette.rs
- /home/pohsuanlai/sill/crates/sill-launcher/src/activation.rs
- /home/pohsuanlai/sill/crates/sill-launcher-ui/src/launcher/keys.rs
- /home/pohsuanlai/quire/crates/ds-settings/src/schema/key.rs
<!-- paths: end -->
