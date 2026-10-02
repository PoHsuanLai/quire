<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/SPEC.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# The companion: locked implementation spec (consolidation of the five area specs)

Status: LOCKED for the freeze wave, except the items that point at QUESTIONS.md. Written 2026-10-02.
Inputs: BRIEF.md, models.md, actions.md + actions-addendum.md (the addendum wins), memory.md, cua.md,
ux.md, research-action-gating.md; context ~/rs-wt/companion/COMPANION.md, ~/quire/design/31-ACCOUNTS.md,
~/porter, quire CONVENTIONS.md.

How to read this file:
- The **area specs stay the detail** (state tables, test lists, fill-wave splits). This file overrides
  them wherever it renames, moves or changes something, and it holds **every signature that crosses
  an area or repo boundary**. Where an area spec and this file disagree, this file wins.
- "Area" names used by the area specs that have no spec of their own map as follows:

| Name used in a spec | Where it lives now |
|---|---|
| "context" | docket (`docket-core::context`, `IntentProvider1.Context/Summon`), quire `ds-intents` (view shapes, seams), docket `docket-ds` (the adapter) |
| "agent", "planner", "agent-loop", "companion daemon" | docket: `agent-loop`, `companion-wire`, `companiond`, `readerd` (§3.6, §4) |
| "security" | labels: porter `prov`; policy, review, breaker, budgets: docket; sandbox units: each daemon's `dist/` in its own repo; egress proxy and `secret-cell`: deferred until a cloud backend is turned on (stoker `model-http` already takes a proxy `HttpTarget`) |
| "compositor" | the cosmic-comp fork (cua repo owns the protocol crate; the fork branch is shared with double-tap ⌘ later) and sill (`sill-overlays` phase A) |
| "launcher", "presence", "settings" | ux (sill, quire ds, detent; design/22 rows by the quire agent) |
| "intents" | docket |

---

## 1. Repos and crates

### 1.1 Repos

Four new library repos plus one GPL fork. Every new repo copies porter's shape: `rust-toolchain.toml`
(1.98.1), quire's `deny.toml`, `rustfmt.toml`, CONVENTIONS pointer + porter additions, ARCHITECTURE.md
(crate map, one-home table, recipes), FINDINGS.md (every `todo!()`), `scripts/check-boundary.sh`,
`dbus/*.xml` checked by an introspection test. Licence MIT OR Apache-2.0 except the fork.

| Repo | New? | Holds | Portable |
|---|---|---|---|
| `~/stoker` | new (name: Q-P1) | the model layer: provider trait, backends, CUA action/parse/session, vision prep, catalog, engine supervisor | yes, no porter dependency |
| `~/porter` | existing | + `prov` (labels and shared who/what vocabulary), `SpaceId`, ComputerUse vocabulary, streaming inference, inferd engines | yes |
| `~/almanac` | new | memory: eventlog, memfiles, recall, memoryd | yes (memoryd Linux) |
| `~/docket` | new | typed actions, router, policy, review, eval, MCP edge, **and the companion agent** (agent-loop, companiond, readerd), the ds adapter | core yes; daemons Linux |
| `~/cua` | new (name: Q-P1) | computer-use runtime: a11y tree, agent seat, masked capture, desktop leases, run machine, cuad, `quire-agent-v1` protocol | core yes; cuad Linux |
| `~/cosmic-comp` | new fork, GPL-3.0-only | `src/quire/**` only (agent seat, masked source, trusted surface, glow, kill chord, later double-tap) | no; nothing of ours depends on it |
| `~/quire` | existing | + `ds-intents` crate; companion components and vocab; design docs | yes |
| `~/sill` | existing | launcher companion mode, companion service, providers, overlays (confirm and glow phase A), settings domain | no |
| `~/detent` | existing | `Page::Intelligence` row and icon | no |
| `~/mailo` | existing | first real provider (fill wave only) | no |
| `~/shell-host` | existing | nothing in the freeze; later the `quire_sensitive_v1` client | no |

Why the companion agent lives in docket and not its own repo: the planner never links cua or almanac
(it reaches CUA and memory as router actions, §4.2), so docket can host it without a repo cycle, and
the planner/reader contract types are already docket's. This saves a repo.

Why `prov` lives in porter: memory (almanac) needs labels, `Actor`, `Effect` and `EntityId`; docket's
intentd needs almanac's client. With `prov` in docket the repos would form a cycle. porter is already
the pure, portable foundation that owns `AppName`, `DataClass` and now `SpaceId`.

### 1.2 Repo dependency direction (acyclic; enforced per repo by `check-boundary.sh` rows)

```
stoker  ←  porter  ←  almanac  ←  docket  ←  cua  ←  sill
   ↑          ↑          ↑          ↑   ↖quire(ds-intents only)
   └──────────┴──────────┴──────────┴── mailo, detent (clients only)
quire depends on none of them.  cosmic-comp fork → cua's quire-agent-protocol only.
```

### 1.3 Every crate (final names)

| # | Crate | Repo | Kind | Depends on (ours, cross-repo in **bold**) |
|---|---|---|---|---|
| 1 | `cua-action` | stoker | pure | — |
| 2 | `vision-prep` | stoker | pure (`pixels` feature: CPU) | cua-action |
| 3 | `model-provider` | stoker | pure | cua-action, vision-prep |
| 4 | `cua-parse` | stoker | pure, fuzzed | cua-action, model-provider |
| 5 | `cua-vendors` (was models' `cua-wire`) | stoker | pure | cua-action, model-provider, vision-prep |
| 6 | `cua-session` | stoker | pure | 1-5 |
| 7 | `model-replay` | stoker | pure (+ injected sink) | model-provider |
| 8 | `model-catalog` | stoker | pure | model-provider, vision-prep, cua-action |
| 9 | `engine-supervisor` | stoker | pure core; io features | model-catalog |
| 10 | `model-http` | stoker | io | — |
| 11 | `model-openai-compat` | stoker | io | model-provider, model-http |
| 12 | `prov` | porter | pure | porter-core |
| — | porter-core, porter-provider, porter-infer, porter-dbus, porter-client, porter-fake, inferd | porter | existing, changed (§3.2) | porter-infer → **cua-action**; inferd → **stoker 1-3, 6, 8-11** |
| 13 | `almanac-core` | almanac | pure, serde-only | **porter-core, prov** |
| 14 | `almanac-seal` | almanac | pure | almanac-core |
| 15 | `eventlog` | almanac | sqlite | almanac-core, almanac-seal |
| 16 | `memfiles` | almanac | fs via `Vault` | almanac-core, almanac-seal |
| 17 | `recall` | almanac | sqlite; generic | — |
| 18 | `recall-fastembed` | almanac | ort (excluded from gate) | recall |
| 19 | `almanac-service` | almanac | pure over seams | 13-17 |
| 20 | `almanac-watch` | almanac | inotify | almanac-core |
| 21 | `almanac-dbus` | almanac | zbus | almanac-core |
| 22 | `almanac-client` | almanac | per transport | almanac-core, almanac-service, almanac-dbus (feature) |
| 23 | `almanac-fake` | almanac | test | 13-19 |
| 24 | `memoryd` | almanac | daemon | all almanac, **porter-client** |
| 25 | `docket-core` | docket | pure, serde-only | **porter-core, prov, cua-action** |
| 26 | `policy-point` | docket | pure (+ cedar-policy) | docket-core, **prov** |
| 27 | `action-review` | docket | pure (+ an `InferReviewer` over a seam) | docket-core, **prov, porter-infer** |
| 28 | `docket-router` | docket | pure over seams | docket-core, policy-point, action-review, **prov, porter-core** |
| 29 | `docket-dbus` | docket | zbus | docket-core |
| 30 | `docket-client` | docket | per transport | docket-core, docket-router (InProcess), docket-dbus (feature) |
| 31 | `docket-fake` | docket | test | 25-30 |
| 32 | `docket-eval` | docket | test + release script | docket-fake and its deps, **porter-infer** |
| 33 | `actions-mcp` | docket | rmcp edge binary, off by default | docket-core, docket-client |
| 34 | `intentd` | docket | daemon (also hosts the built-in `org.quire.Memory` provider) | 25-30, **almanac-core, almanac-client, porter-core** |
| 35 | `companion-wire` | docket | pure, serde-only | docket-core, **prov, porter-infer** (`ServedBy`) |
| 36 | `agent-loop` | docket | pure step machine | docket-core, companion-wire, **prov** |
| 37 | `companiond` | docket | daemon | agent-loop, companion-wire, docket-client, **porter-client** |
| 38 | `readerd` | docket | daemon, separate process | docket-core, docket-client, **porter-client** |
| 39 | `docket-ds` | docket | adapter used by quire apps | docket-client, companion-wire, **ds-intents** |
| 40 | `a11y-tree` | cua | pure; `atspi` feature | **cua-action, prov** |
| 41 | `seat-input` | cua | pure; `ei` feature | **cua-action** |
| 42 | `maskcap` | cua | pure; `wayland` feature | **cua-action** |
| 43 | `cua-desktop` | cua | pure; `cosmic` feature | **cua-action, docket-core** (`RunMode`, `WindowTrust`), quire-agent-protocol (feature) |
| 44 | `quire-agent-protocol` | cua | wayland-scanner | — |
| 45 | `cua-bus` (new split) | cua | pure, serde-only (`Cua1` bodies, `CuaRecord`) | **prov, docket-core, porter-infer** |
| 46 | `cua-run` | cua | pure run machine | 40-43, 45, **docket-core, prov, porter-core, porter-infer** |
| 47 | `cua-dbus` | cua | zbus | cua-bus |
| 48 | `cua-fake` | cua | test | 40-47 default features, reis, **model-replay** |
| 49 | `cuad` | cua | daemon | all cua, **porter-client, docket-client, almanac-client** |
| 50 | `ds-intents` | quire | pure view shapes + seams | ds-core |
| — | ds-core, ds, ds-shell, ds-style, ds-settings | quire | existing, gain types (§3.7) | — |
| — | sill-launcher, sill-ipc, sill-model, sill-services, sill-search, sill-ui-kit, sill-launcher-ui, sill-bar, sill-settings, sill-overlays, sill-notify-ui | sill | existing, gain variants/modules (§3.8) | **docket-core, prov, almanac-core, companion-wire, cua-bus**; sill-services also **docket-dbus, almanac-dbus, cua-dbus, porter-dbus** |

Later (named now, not frozen): `model-anthropic`, `model-openai-responses`, `model-gemini`,
`recall-sqlite-vec` (Q-P5), `egress`, `secret-cell`, procedures in `cua-run`.

Boundary rules per repo (each repo's `check-boundary.sh`): pure crates never reach `tokio hyper
hyper-util rustls zbus zvariant reqwest wayland-client wayland-backend reis atspi oo7 ort fastembed
rusqlite notify cedar-policy rmcp` except where the table says so (`policy-point`→cedar;
`actions-mcp`→rmcp; `eventlog`/`recall`→rusqlite; `*-dbus`, daemons, feature backends → io).

### 1.4 Adopted external crates (join `quire/docs/workspace-deps.toml` in wave 0)

| Crate | Version | For | Note |
|---|---|---|---|
| cedar-policy | 4.13 | policy-point | `cargo deny` against its lalrpop tree first |
| rmcp | 3.5, `default-features=false, ["server","transport-io"]` | actions-mcp | pin the minor |
| hyper / hyper-util / http-body-util | 1 / 0.1.21 / 0.1 | model-http | one HTTP stack, Unix sockets |
| hyper-rustls + rustls | 0.27 / 0.23 | model-http cloud (later) | no OpenSSL |
| base64 | 0.22 | porter-infer inline images | |
| fast_image_resize | 6.1.0 | vision-prep `pixels` | |
| blake3 | 1 | model-replay, eventlog, almanac-seal | |
| chacha20poly1305 / zeroize / getrandom | 0.10 / 1 / 0.3 | almanac-seal | |
| rusqlite | 0.40, `bundled-sqlcipher-vendored-openssl` | eventlog, recall | after spike S-SQL (FTS5 + deny) |
| tar | 0.4 | almanac-service export | |
| oo7 | 0.6 | almanac-seal `Oo7Keys`, porter secrets | |
| fastembed | 7.1 (ort 2.0.0-rc.13) | recall-fastembed only | excluded from gate |
| reis | 0.7 (the minor cosmic-comp's lock resolves) | seat-input `ei`, cua-fake | |
| atspi | 0.30 | a11y-tree `atspi` | |
| proptest | 1 (dev) | cua-parse, vision-prep, model-http | |
| tempfile | 3 (dev) | almanac | |
| keyboard-types | 0.7 + **`serde` feature** | cua-action `Chord` | pinned line changes |
| tokio | 1 + **`net`, `io-util`, `process`** | io crates and daemons | pinned line changes |
| already pinned, unchanged | serde, serde_json, thiserror 2, toml 1, zbus 5.19, image 0.25.6, jiff 0.2, notify 8.2, wayland-* 0.31/0.3/0.32.13, cosmic-client-toolkit, xkbcommon 0.8, memfd 0.6, rustix 1.1 | | |
| later | ashpd 0.13, pipewire, nix 0.30 (fanotify), sqlite-vec | | |

Not adopted: async-openai, genai, rig (reqwest cannot reach Unix-socket engines; our trait is
smaller), llguidance/tokenizers (server flags suffice until an in-process engine).

---

## 2. One home per type (every duplicate found, and its resolution)

"View" types are allowed beside a wire type only in quire/sill, where quire may not name agent
crates; each pair gets a `*_total` conversion test in `sill-ui-kit::companion`.

| Concept | Was defined in | Single owner | Others |
|---|---|---|---|
| `Effect {Read, UndoableWrite, Outbound, Destructive}` | docket-core, almanac-core (re-export), cua `EffectClass`, ux `EffectMark` | **prov** | cua uses `prov::Effect` (EffectClass deleted); ds-core `EffectMark` is the view |
| `SpaceId`, `SpaceScope` | porter-core (actions), prov (memory), almanac | **porter-core** | prov re-exports |
| Coordinate-space trait `Space` | cua-action | **cua-action**, renamed `CoordSpace` | frees "Space" for desktop Spaces |
| `Actor` | docket-core, almanac-core, cua (`Companion(CuaRun)`), ux view `Actor{You,Companion}` | **prov** (merged, §3.1) | ds-core `ActorMark` view; docket `GrantCaller` for grant keys |
| `AppName`, `AppId` | porter-core | **porter-core** | ux `AppMark` is a view (icon + name) |
| `EntityKind/Key/Id`, `ActionName` | docket-core; almanac `ThingRef/ThingKind/ThingKey/ActionName` | **prov** | almanac `pub type ThingRef = prov::EntityId` (and ThingKind/Key aliases) |
| `SessionId`, `RunId` | docket (String), almanac (String), cua `RunId(u64)` | **prov**, String slugs | cuad mints `r-<n>` |
| `TaskId`, `TurnId`, `UndoId`, `Handle`, `CallId` | docket-core | **docket-core** | ux `AnswerId`, sill `AnswerKey`→ `TaskId`; `RunKey`→`RunId`; `ActivityKey`→`UndoId`; `MemoryKey`→`almanac_core::MemoryItem` |
| Tool-call id | model-provider `CallId(String)`, porter-infer `CallId` | **model-provider** `ToolCallId` | porter-infer wire mirrors (`ToolCallId`, bridged in `inferd::bridge`) |
| `Label`, `Integrity`, `Confidentiality`, `Source`, `Labelled`, `Quarantined` | prov (actions), almanac uses | **prov** | |
| `ConfirmReceipt`, `ConfirmId`, `InputProof`, `Witness` | prov/docket-core; almanac `Confirmation` | **prov** | almanac `Settlement::Keep(ConfirmReceipt)` |
| `ConfirmRequest`, `ConfirmAnswer`, `TaintNote`, `ConfirmOffer`, `Gesture` | docket-core; ds-shell `ConfirmView/ConfirmAnswer/TaintNote/ScopeOffer` | **docket-core** (wire) | ds-shell: `ConfirmView`, `ConfirmChoice` (renamed), `TaintLine` (renamed), `ScopeOffer`, + `gesture: GestureMark` |
| `Preview` (content vocabulary) | docket-core; sill-launcher `Preview` (pane selector) | **docket-core** for content | sill `Preview::Intent(docket_core::Preview)`; ux `AnswerView` is a view |
| `Fact` (label line) vs memory `Fact` | docket-core `Fact`, almanac `Fact`, ds `FactList` | almanac `Fact` = a memory fact; docket's renamed **`FactLine`** | ds-shell `ConfirmView.facts` uses quire's FactList items |
| `Count` | porter-core `Count(u32)`; ds-settings `Count(u16)` | **porter-core** on wires | ds-core gains view `Tally(u32)`; ds-settings Count stays settings-only |
| `Fraction` | ds-core | **ds-core** (view) | |
| `ServedBy` | porter-infer; ux `ServedByView`; cua `ServedByNote` | **porter-infer** | cua records `ServedBy`; ux view `ServedByView` |
| Undo journal | docket `UndoEntry/UndoToken/UndoId`; ux `UndoKey`, Companion1 `Undo`, `ActivityAppended`; ds `toast_hub::UndoToken(u64)` | **docket-core** journal + `Intents1.Run.Undo/UndoAll`, `Control.Journal/JournalChanged` | ux `UndoKey` deleted: views use ds `UndoToken(u64)` = `UndoId.0`; Companion1 has no undo or activity members |
| Run state | cua `RunPhase`/`RunStateSlug`; ux `RunState`, `RunPlace` | **cua-bus** `RunStateSlug`, `CuaOutcome`; **docket-core** `RunMode` | ux `RunState`/`RunPlace` are views; `sill-model::run_state_of(slug, outcome)` total |
| `RunMode`, `WindowTrust` | cua-desktop | **docket-core** (the gate contract needs them) | cua-desktop re-exports |
| `GlowState` | cua-desktop, ds-shell, ux protocol sketch | **cua-desktop** (protocol values) | ds-shell renames to `GlowLook` (view input to `glow_spec`) |
| Presence | ux `CompanionPresence`; Companion1 `Presence` property | **ds-core::vocab::CompanionPresence**, derived only by `sill-model::companion::presence_of` | Companion1 has no Presence property; apps derive in-field presence from their own `PromptState` |
| `MemoryVerb` | ds and sill-launcher (twice) | **ds-core::vocab::MemoryVerb** | sill-launcher uses it |
| `CuaAction` | cua-action (models), cua.md §3.0 | **cua-action** (merged, §3.3) | `TargetedAction`, `WindowRef` → cua-run |
| Frame types | vision-prep `RawFrame`, cua `RawFrame/PixelBuf/PxSize`, porter-infer `FrameLayout` | `PixelFormat`, `DeviceSize`, `Scale120` → **cua-action**; `PixelBuf` → maskcap; `RawFrame<'a>` → vision-prep | `PxSize` deleted (= `DeviceSize`) |
| CUA model step | cua `CuaTurn/StepReply/StepEvent`, models `CuaStepRequest/Reply` | **porter-infer::cua** | cua-run `turn(obs) -> CuaStepRequest` |
| Event payloads (policy, consent, CUA, session) | almanac re-declared each | **their owners** (`docket-core::AuditRecord`, `cua-bus::CuaRecord`, `companion-wire::SessionRecord`) | almanac stores them as `EventBody::Area` (§3.4) |
| Budgets | docket `Budget/BudgetKind`, cua `Budget/BudgetKind` | session budget **docket-core**; per-run budget **cua-run `RunBudget/RunBudgetKind`** | cua `denials` field removed: the router's breaker counts |
| Denial reasons | docket `DenyReason` (internal), `DenyCode` (to agent); cua `DenyReason` | **docket-core** `DenyCode`; `DenyReason` internal to policy-point/audit | cua keeps only local pre-checks `LocalRefusal {OutsideLease, SecretField}` |
| Verdict names | porter consent `Verdict`, action-review `Verdict`, almanac `Verdict` | porter keeps `Verdict` | action-review → `ReviewVerdict`; almanac → `Settlement {Keep(ConfirmReceipt), Discard}` |
| Decision names | porter consent `Decision`; policy-point `Decision` | porter keeps `Decision` | policy-point → `Ruling {Deny, Ask, AllowJudged, AllowFinal}` |
| `Need` | porter-core `Need`; docket `Need` (param); ds `Need` (form) | porter-core | docket → `ParamNeed`; ds → `FieldNeed` |
| `Visibility<T>` (plain or handle) | docket | docket-core, renamed **`Reveal<T>`** | cua-run's window `Visibility` keeps its name |
| `Usage` | porter consent `Usage`; model-provider `Usage` (tokens) | porter keeps `Usage` | model-provider → `TurnUsage`; porter-infer `TokenUsage` unchanged |
| `Role` | model-provider message `Role`; catalog `roles`; intentd caller roles | model-provider `Role` | catalog `roles: BTreeSet<AiKind>`; intentd `CallerRole` |
| `Latency` | docket `Latency` (decl), addendum `Latency2` | docket `Latency` | → `StageLatency` |
| `Repeat` | cua-action `Repeat` (key repeat), docket gate param | cua-action | docket gate → `RepeatState` |
| `Need::Cua` vs `Need::ComputerUse` | cua.md, models.md | **porter-core `Need::ComputerUse(CuaNeed)`** | |
| Authorization | actions §3.9 | **deleted** → docket-core `TaskPolicy` (addendum A1) | |
| `Summon` app method | ux ("context area") | **`IntentProvider1.Summon`**, answered by ds via `docket-ds` | |
| Settings keys | `companion.undo_keep_h` vs `agent.undo.keep_h`; `companion.touch` vs `cua.autopause`; `companion.run_place` vs `cua.default_mode` | `agent.undo.keep_h`, `cua.autopause`, `cua.default_mode {auto, in_place, agent_workspace}` | sill's Intelligence page shows them; `CompanionSettings` drops those three fields |

---

## 3. Frozen interfaces that cross a boundary

Shared rules (all repos): no `bool`; data is `Eq` (embedding vectors excepted); newtypes per unit;
serde `#[serde(tag="kind", content="v", rename_all="snake_case")]` for data enums, slugs for unit
enums; standard derive order; redacting `Debug` on user text; async seams `-> impl Future + Send`;
closed implementation sets are enums, never `dyn`. Derives and doc comments are omitted below.
Everything here is **freeze now** unless marked later. Area-internal types stay in the area specs.

### 3.1 porter `prov` (new crate) and porter-core additions

```rust
// porter-core/src/space.rs
pub struct SpaceId(String);                       // slug grammar of porter ids; minted by sill's Space store (Q-P6); reserved "desktop"
pub enum SpaceScope { Any, Only(SpaceId) }
// porter-core consent: GrantKey gains `space: SpaceScope`; Grant<K = GrantKey>; decide<K: Eq>(&[Grant<K>], &K) -> Verdict
// porter-core vocabulary: VocabVersion(2); Capability::ComputerUse(CuaCap); CapabilityKind::ComputerUse;
//   Need::ComputerUse(CuaNeed); CuaCap { environments: BTreeSet<CuaEnv>, batching: CuaBatching, zoom: Offered, max_image: Px, wire: LlmWire }
//   CuaEnv { Desktop, Browser, Mobile }; CuaBatching { One, Many }; CuaNeed { environments: BTreeSet<CuaEnv> }

// prov/src/ids.rs
pub use porter_core::{AppName, DataClass, SpaceId, SpaceScope, UnixSeconds};
pub struct SessionId(String);  pub struct RunId(String);  pub struct ClientName(String);
pub struct EntityKind(String);                    // "mail.thread": 2+ dotted [a-z][a-z0-9_]*
pub struct EntityKey(String);                     // app-local, opaque, <= 512 bytes, no control chars
pub struct EntityId { pub app: AppName, pub kind: EntityKind, pub key: EntityKey }
pub struct ActionName(String);                    // "mail.thread.archive"

// prov/src/effect.rs  (COMPANION item 9; ordered by severity)
pub enum Effect { Read, UndoableWrite, Outbound, Destructive }

// prov/src/actor.rs  (set by the transport/daemon, never self-asserted)
pub enum Actor {
    User { via: AppName },                        // via = the app or org.quire.Shell (launcher)
    Companion { session: SessionId, role: AgentRole },
    Mcp { client: ClientName },
    App { app: AppName },                         // an app on its own (sync, rules)
    ThirdParty { app: AppName, channel: Channel },
    System { part: SystemPart },
    Unknown,                                      // an unexplained file change
}
pub enum AgentRole { Planner, Reader, Cua { run: RunId } }
pub enum Channel { Atspi, Dbus, Portal, Mpris, FileWatch }
pub enum SystemPart { Memory, Router, Watcher, Consolidation, Retention, Cua }
pub enum ActorKind { User, Companion, Cua, Mcp, App, ThirdParty, System, Unknown }
impl Actor { pub fn kind(&self) -> ActorKind; }

// prov/src/label.rs  (FIDES lattice; actions.md §3.1 semantics)
pub enum Integrity { Untrusted, Trusted }                         // join = min
pub enum Confidentiality { Public, Private(BTreeSet<SpaceId>), Secret } // join = max, Private unions
pub enum Source { User, App(AppName), Mail, Web, File, Screen { app: AppName }, Clipboard,
                  Calendar, Contacts, Notes, Model(ModelRole), Mcp(ClientName) }
pub enum ModelRole { Planner, Reader, Reviewer, PolicyWriter, Cua, Consolidator }
pub struct Label { pub integrity: Integrity, pub confidentiality: Confidentiality,
                   pub classes: BTreeSet<DataClass>, pub sources: BTreeSet<Source> }
impl Label { pub fn trusted_user() -> Label; pub fn untrusted(s: Source, c: DataClass, space: SpaceId) -> Label;
             pub fn join(&self, o: &Label) -> Label; }
pub struct Labelled<T> { pub value: T, pub label: Label }        // map keeps, zip joins
pub struct Quarantined<T>(Labelled<T>);                           // Debug prints length + sources only
pub struct ReaderKey(());                                         // minted by readerd's host only

// prov/src/consent.rs
pub struct ConfirmId(String);
pub enum InputProof { HardwareSeat, SheetFallback, ShellCaller }  // ShellCaller: memory Keep from sill's own UI
pub struct ConfirmReceipt { pub id: ConfirmId, pub input: InputProof, pub at: UnixSeconds }
pub enum Witness { UserConfirmed(ConfirmReceipt) }
pub fn endorse<T>(v: Labelled<T>, w: &Witness) -> Labelled<T>;
pub fn declassify<T>(v: Labelled<T>, to: Confidentiality, w: &Witness) -> Labelled<T>;
```

### 3.2 porter-infer, porter-client, Inference1 (models.md §3.8, with these renames)

```rust
// request.rs (changes a frozen type; design/31 edited in step)
pub struct ChatRequest { /* frozen fields */ pub tools: Vec<ToolDecl> }
pub struct ToolDecl { pub name: ToolName, pub description: String, pub params: JsonSchemaText } // from docket tool_schema
pub enum MessagePart { Text(String), Image(ImagePart), ToolCall(ToolCallPart), ToolResult(ToolResultPart) }
pub struct ToolCallPart { pub id: ToolCallId, pub name: ToolName, pub args: JsonText }
pub struct ToolResultPart { pub id: ToolCallId, pub status: ToolStatus, pub parts: Vec<MessagePart> }
pub struct ImagePart { pub media_type: String, pub source: ImageSource }
pub enum ImageSource { Inline(Base64Bytes), Attached(AttachIndex) }   // memfd via SCM_RIGHTS on the Open fd
pub enum InferRequest { Chat(ChatRequest), Embed(EmbedRequest), Task(TaskRequest), CuaBegin(CuaBegin), CuaStep(CuaStepRequest) }

// cua.rs — the cuad <-> inferd contract; all points in WindowSpace
pub struct CuaBegin { pub goal: String, pub hints: Vec<String>, pub env: CuaEnv }
pub struct CuaStepRequest { pub step: StepIndex, pub window: WindowGeometry, pub frame: FrameImage,
    pub cursor: Option<Point<WindowSpace>>, pub prev: Vec<PrevResult>, pub masked: MaskedRegions,
    pub tree: TreeText }                                              // + tree (cua.md: a11y text to the model)
pub enum TreeText { Absent, Present(String) }                         // untrusted screen text; cua-run renders it
pub struct WindowGeometry { pub logical: Size<WindowSpace>, pub scale: Scale120 }
pub struct FrameImage { pub source: ImageSource, pub layout: FrameLayout }
pub enum FrameLayout { Raw { format: PixelFormat, size: DeviceSize, stride: u32 }, Encoded(MediaKind) }
pub enum PrevResult { Done, Refused(String), NotRun, Failed(String), UserDeclined, UserActed }
pub struct MaskedRegions(pub u16);
pub struct CuaStepReply { pub thought: Option<String>, pub actions: Vec<CuaAction<WindowSpace>>,
                          pub dropped: Vec<DroppedAction>, pub safety: Vec<SafetyHint> } // safety only adds asks
pub enum CuaStepFailure { Unparseable, ModelFailed(ModelError) }

// event.rs — streaming on the Open fd
pub enum ClientFrame { Request(InferRequest), Cancel }
pub enum InferEvent { Routed(ServedBy), Waiting(Readiness), TextDelta(String), ThoughtDelta(String),
    ToolCall(ToolCallPart), ActionProposed(CuaAction<WindowSpace>), Usage(TokenUsage), Finished(InferReply) }
pub enum InferReply { Chat(ChatReply), Embed(EmbedReply), CuaStep(CuaStepReply), Refused(InferRefusal), Failed(ModelError), Cancelled }
pub enum InferRefusal { RequiresCloud(DataClass), Unavailable, NeedsGrant, Denied, OverBudget, Unsupported }
pub enum ModelError { Unreachable, RateLimited(u32), Unauthorized, Refused, Unreadable, NotReady, ContextOverflow, Unparseable }
pub struct ChatReply { pub text: String, pub tool_calls: Vec<ToolCallPart>, pub usage: TokenUsage, pub served: ServedBy }
pub struct ServedBy { pub account: AccountId, pub model: ModelId, pub locality: Locality }   // unchanged home

// readiness.rs / choice.rs — picker data (models.md §3.8)
pub enum Readiness { Ready, Loading, Loadable, Downloading(Permille), Downloadable, Unavailable }
pub enum AiKind { Llm, ComputerUse, Embeddings, Speech, ImageGen, Rerank }
pub struct TierMap { pub rows: Vec<TierRow> }   pub struct PickerRow { /* models.md */ }

// model.rs: Model::chat(&self, &ChatRequest, &mut impl ChatSink) -> impl Future<Output = Result<ChatReply, ModelError>> + Send
// porter-client: Transport::open(&self, need: &Need, class: DataClass, tier: Tier) -> Future<Result<Self::Session, TransportError>>;
//   trait InferSession { send(ClientFrame); next() -> InferEvent }; Transport::infer removed; Accounts::session added.
```

### 3.3 stoker `cua-action` (merged models.md §3.1 + cua.md §3.0)

```rust
pub trait CoordSpace: Copy + Eq + core::fmt::Debug + 'static {}
pub enum WindowSpace {}  pub enum ImageSpace {}  pub enum GridSpace {}       // uninhabited markers
pub struct Coord(pub u32);                                                    // WindowSpace: logical px, window-local
pub struct Point<S: CoordSpace> { pub x: Coord, pub y: Coord, space: PhantomData<S> }
pub struct Size<S>, Rect<S> { origin: Point<S>, size: Size<S> }, Length<S>(Coord, PhantomData<S>)
pub struct Scale120(pub u32);  pub struct GridMax(pub u16);  pub struct DeviceSize { pub w: u32, pub h: u32 }
pub enum PixelFormat { Xrgb8888, Argb8888, Xbgr8888, Abgr8888 }
pub struct NodeId(pub u32);                                                   // a11y node, stable within one run
pub enum Target<S: CoordSpace> { Point(Point<S>), Node(NodeId) }             // Node: frozen now, used when a dialect supports it (Q-M6)

pub enum CuaAction<S: CoordSpace> {
    Click  { at: Target<S>, button: Button, count: ClickCount, mods: BTreeSet<Modifier> },
    MoveTo { at: Target<S> },
    Drag   { from: Target<S>, to: Target<S>, button: Button },
    Type   { text: TypedText },
    Key    { chord: Chord, repeat: Repeat },
    Scroll { at: Target<S>, dir: ScrollDir, by: ScrollBy<S> },
    Wait   { for_ms: WaitMs },
    Zoom   { region: Rect<S> },
    Observe,
    Finish { outcome: FinishOutcome, summary: Summary, extracted: Vec<Extracted> },  // + extracted (cua)
    Ask    { question: Summary, choices: Vec<Choice> },
}
pub struct Extracted { pub name: FieldName, pub value: ExtractedText }        // cua-run labels Untrusted(Screen)
pub enum Button { Left, Right, Middle }  pub enum ClickCount { One, Two, Three }
pub enum Modifier { Ctrl, Alt, Shift, Super }
pub struct Chord { pub mods: BTreeSet<Modifier>, pub key: keyboard_types::Key }  // seat-input maps Key -> keysym
pub struct Repeat(u8);  pub enum ScrollDir { Up, Down, Left, Right }
pub enum ScrollBy<S> { Notches(Notches), Distance(Length<S>) }
pub struct WaitMs(u32); pub struct TypedText(String); pub struct Summary(String); pub struct Choice(String);
pub enum FinishOutcome { Done, Failed, Infeasible }
pub enum ActionClass { Observe, Pointer, Keyboard, Conclude }
impl<S> CuaAction<S> { pub fn class(&self) -> ActionClass; pub fn map_points<T, E>(self, f, g) -> Result<CuaAction<T>, E>; } // Node passes through
pub enum WireDialect { AnthropicToolset20260801, AnthropicComputer20251124, OpenAiComputer, GeminiComputerUse }
pub enum TextDialect { UiTars15 }  pub enum ToolDialect { QwenComputerUse /* + Holo31 if S0 says so */ }
pub enum CuaDialect { Wire(WireDialect), Text(TextDialect), Tool(ToolDialect) }
pub enum ModelSpace { Image, Grid(GridMax) }
```

### 3.4 almanac (memory.md §3, with these changes)

```rust
// almanac-core
pub type ThingRef = prov::EntityId;  pub type ThingKind = prov::EntityKind;  pub type ThingKey = prov::EntityKey;
pub struct ThingView { pub thing: ThingRef, pub title: String, pub subtitle: String }
pub struct Record { pub space: SpaceId, pub occurred: UnixSeconds, pub actor: prov::Actor,
                    pub effect: prov::Effect, pub label: prov::Label, pub body: EventBody, pub cause: Cause }
pub enum EventBody {
    Thing  { verb: Verb, thing: ThingView, sources: Vec<ThingView> },
    File   { change: FileChange, file: FileView, why: FileWhy },
    Search { app: AppName, text: String, scope: ThingKind, results: Count },
    Memory { op: MemoryOp },
    Area(AreaPayload),                         // replaces Action, Session, Cua, Policy, Consent variants
}
pub struct AreaPayload { pub area: AreaTag, pub kind: KindTag, pub json: JsonText,  // the owner's serde form
                         pub things: Vec<(ThingView, ThingRole)> }                  // for cascade-forget
pub enum AreaTag { Docket, Cua, Companion }    pub enum ThingRole { Subject, Source }
pub struct EventRef { pub space: SpaceId, pub replica: ReplicaId, pub seq: Seq }
pub enum MemoryItem { Fact(FactId), Event(EventRef) }               // sill's Subject key
pub struct Fact { /* memory.md §3.3 */ pub label: prov::Label, pub by: prov::Actor, .. }
pub enum Settlement { Keep(prov::ConfirmReceipt), Discard }          // was Verdict
pub enum Caller { App(AppId), Router, Cuad, ShellUi }                 // Companion removed (§4.2): it reads via the router
// MemoryRequest / MemoryReply / ForgetScope / ForgetPlanView / TimelineQuery / TimelinePage /
// TimelineEntry / FactView / DraftView / Hunk / SpaceStatus / Refusal: memory.md §3.9-§3.11 verbatim,
// with Settle(FactId, Settlement) and Propose only from Router or ShellUi.
```

Caller matrix change (memory.md §3.9): the `Companion` column moves to `Router` (acting for
`Actor::Companion`, Space from the Invocation); `Cuad` may `Record` only `Area(Cua)` bodies with an
`Actor::Companion{role: Cua}` actor. Untrusted-labelled `Propose` always lands in `pending/`.

### 3.5 docket-core (actions.md §3.3-§3.13 with the addendum applied; renames from §2)

```rust
// ids
pub struct TaskId(String); pub struct TurnId(u64); pub struct CallId(u64); pub struct UndoId(u64);
pub struct UndoToken(String); pub struct Handle(u64); pub struct LabelText(String); pub struct IconName(String);
pub struct ActionRef { pub app: AppName, pub name: ActionName }
pub struct IntentsVocab(pub u32); // CURRENT = 1

// manifest ($XDG_DATA_DIRS/quire/intents/<AppName>.toml); actions.md §3.3 + addendum
pub struct ActionDecl { pub name: ActionName, pub label: LabelText, pub on: TargetKind, pub params: Vec<ParamDecl>,
    pub effect: prov::Effect, pub classes: BTreeSet<DataClass>, pub undo: UndoSupport, pub reach: AgentReach,
    pub latency: Latency, pub result: ResultShape, pub keys: KeyHint, pub lasting: Lasting, pub dry_run: DryRun }
pub enum TargetKind { Nothing, One(EntityKind), Many(EntityKind), Text, Files }   // was `Target`
pub struct ParamDecl { pub name: ParamName, pub label: LabelText, pub ty: ParamType, pub need: ParamNeed, pub sink: ArgSink }
pub enum ParamNeed { Required, Optional, Defaulted(Value) }
pub enum ArgSink { Inert, Recipient, Destination, Body, Path, Query }
pub enum UndoSupport { Token, NotUndoable }
// validate: Read ⇒ NotUndoable; UndoableWrite ⇒ Token; Outbound MAY declare Token (an app-held send delay, e.g. mailo's undo-send);
//           Outbound must declare a Recipient or Destination sink; a write with no undo must be Destructive.
pub enum Lasting { No, AgentMemory, Staged }       // Staged: untrusted input lands in a pending queue (memory.propose)
pub enum DryRun { None, Preview }
pub enum ResultShape { Nothing, Value { ty: ParamType, trust: TitleTrust }, Entities(EntityKind) }  // + trust floor for returned text
pub enum TitleTrust { AppAuthored, ThirdParty(prov::Source) }

// values and context
pub enum Value { Text(String), Integer(i64), Decimal(Decimal), Date(CivilDate), DateTime(UnixSeconds),
    Duration(Seconds), Choice(ChoiceId), Entity(EntityId), Entities(Vec<EntityId>), File(FileRef), Url(String),
    Handle(Handle), List(Vec<Value>), Record(BTreeMap<ParamName, Value>) }
pub type Args = BTreeMap<ParamName, Labelled<Value>>;
pub enum TargetValue { Nothing, Entities(Vec<EntityId>), Text(TextTargetRef), Files(Vec<FileRef>) }
pub struct ContextSnapshot { pub app: AppName, pub window: Labelled<String>, pub here: Here, pub selection: Selection,
    pub visible: Visible, pub text_target: TextTarget, pub privacy: WindowPrivacy }          // actions.md §3.4
pub struct ContextKeep { pub query: Keep, pub results: Keep, pub selection: Keep, pub window: Keep }  // chips the person kept
pub enum Keep { Kept, Dropped }
pub enum Reveal<T> { Plain(T), Handle(Handle) }    // was Visibility<T>
pub enum Preview { None, Text {..}, Facts(Vec<FactLine>), Person {..}, Thread {..}, Event {..}, Image(FileRef),
    Pdf {..}, File(FileFacts), List(Vec<EntityRef>), TextChange {..}, Moves(Vec<FileMove>),
    Message { to: Vec<Labelled<String>>, subject: Labelled<String>, body: Labelled<String> } }

// turns and sessions (new: who may record a turn)
pub struct TurnIn { pub text: String, pub origin: Origin, pub keep: ContextKeep }
pub struct UserTurn { pub id: TurnId, pub text: String, pub at: UnixSeconds, pub from: TurnSource }
pub enum TurnSource { Launcher, Field(AppName) }   // Field turns: TaskPolicy capped to that app + Read; widening confirms
pub enum Origin { Launcher, InWindowField, Companion, Shortcut, Mcp, AppInternal }

// calls
pub struct CallRequest { pub action: ActionRef, pub target: TargetValue, pub args: Args, pub origin: Origin }
pub struct Invocation { pub call: CallId, pub action: ActionName, pub target: TargetValue, pub args: Args,
                        pub actor: prov::Actor, pub origin: Origin, pub space: SpaceId }
pub struct Outcome { pub value: Option<Labelled<Value>>, pub said: Option<LabelText>, pub show: Preview,
                     pub undo: Undoable, pub follow: Follow }
pub enum Undoable { No, Yes(UndoToken) }  pub enum Follow { Nothing, Open(EntityId), Next(CallRequest) }
pub enum AppRefusal { NeedsParam { param: ParamName, options: Vec<EntityRef> }, NotFound(EntityId), Stale(EntityId), Busy, Unsupported, Failed(FailText) }
pub enum CallRefusal { App(AppRefusal), Denied(DenyCode), Unconfirmed(ConfirmEnd), Halted(SpaceScope),
    Paused(BreakerTrip), OverBudget(BudgetKind), NoSuchAction(ActionRef), BadArgs { param: ParamName, why: ArgFault },
    AppUnavailable(AppName), Timeout }
pub enum DenyCode { OutsideTask, NotAllowed, NeedsUser, Repeated }
pub enum CallProgress { Reviewing, Previewing, Confirming(ConfirmId), Dispatched, Running(Permille) }  // Request.Progress body

// undo journal
pub struct UndoEntry { pub id: UndoId, pub at: UnixSeconds, pub actor: prov::Actor, pub app: AppName, pub action: ActionName,
    pub said: LabelText, pub token: UndoToken, pub run: Option<RunId>, pub session: Option<SessionId>, pub state: UndoState }
pub enum UndoState { Available, Undoing, Undone { by: prov::Actor }, Failed(UndoFault), Expired }
pub enum UndoScope { Entry(UndoId), Run(RunId), Task(TaskId) }       // Run.UndoAll

// confirmation (addendum A8)
pub struct ConfirmRequest { pub id: ConfirmId, pub space: SpaceId, pub actor: prov::Actor, pub app: AppName,
    pub action: LabelText, pub effect: prov::Effect, pub count: Count, pub detail: ConfirmDetail, pub lines: Vec<ArgLine>,
    pub why: Vec<AskReason>, pub taint: TaintNote, pub offer: ConfirmOffer, pub gesture: Gesture, pub anchor: Anchor, pub expires: Seconds }
pub enum ConfirmDetail { Recipients(Vec<Shown>), TextDiff { before: Shown, after: Shown }, Moves(Vec<FileMove>),
                         Entities(Vec<EntityRef>), Destination(Shown), Preview(Preview), Plain }
pub enum Shown { Plain(String), Quoted { text: String, from: prov::Source } }
pub enum TaintNote { Clean, ReadUntrusted(BTreeSet<prov::Source>) }
pub enum ConfirmOffer { OnceOnly, OnceOrAlways }  pub enum Gesture { Press, HoldToConfirm }
pub enum Anchor { Window(WindowKey), Launcher, Centre }
pub enum ConfirmAnswer { Allowed { scope: GrantScope, receipt: prov::ConfirmReceipt }, Ended(ConfirmEnd) }
pub enum ConfirmEnd { Refused, Dismissed, Expired, Cancelled }

// planner/reader contract (actions.md §3.8)
pub struct PlannerView { pub turns: Vec<UserTurn>, pub context: ContextView, pub actions: Vec<ActionCard>,
    pub handles: Vec<HandleCard>, pub history: Vec<StepLine>, pub taint: Integrity, pub task_policy: Option<TaskPolicy> }
pub struct ReaderAsk { pub inputs: Vec<Handle>, pub want: ValueSchema, pub task: ReaderTask }
pub enum ReaderTask { Classify, Extract, Summarise, Compare }
pub enum ValueSchema { Choice(Vec<ChoiceId>), Integer { min: i64, max: i64 }, Date, DateTime,
    EntitiesAmong(Vec<EntityId>), Text { max: CharCount }, Record(Vec<(ParamName, ValueSchema)>), List { of: Box<ValueSchema>, max: Count } }
pub trait Reader: Send + Sync {   // readerd implements; intentd calls it via Reader1
    fn extract(&self, ask: ReaderAsk, inputs: Vec<Quarantined<String>>) -> impl Future<Output = Result<Value, ReaderError>> + Send;
}

// per-task policy, breaker, strictness: addendum A1, A3 verbatim (TaskPolicy, ActionMatch, TrustedPattern,
// PolicyChange, Widening, Coverage, covers, compare, PolicyWriter; Breaker, BreakerTrip; Strictness).

// the CUA gate (new; cuad -> intentd)
pub enum RunMode { InPlace, AgentWorkspace, NestedSession }
pub enum WindowTrust { Quire, Flatpak, Native, Shell }
pub struct CuaAsk { pub run: RunId, pub step: u32, pub app: AppName, pub trust: WindowTrust, pub mode: RunMode,
    pub space: SpaceId, pub action: CuaAction<WindowSpace>, pub node: Option<NodeFacts>,
    pub effect: prov::Effect, pub basis: EffectBasis, pub screen: Label }   // screen label: Untrusted(Screen{app})
pub enum EffectBasis { NodeTypedAction(ActionName), WindowClass(WindowClass), DefaultTable }
pub enum WindowClass { Ordinary, MailCompose, Terminal, Payments, Admin, PasswordManager, Banking }
pub struct NodeFacts { pub role: String, pub name: Labelled<String> }
pub enum GateAnswer { Run, Refused(CallRefusal) }   // confirmations are resolved inside the router before answering

// audit (stored by intentd as almanac EventBody::Area{area: Docket})
pub enum AuditRecord { /* actions.md §3.11 + addendum: Review{stage}, Call{decided}, Confirm, Undo, Halt, TaskPolicy, Breaker */ }
pub enum HaltCause { KillChord, ControlCentre, StopKey, Budget(BudgetKind), Breaker(BreakerTrip) }   // Esc -> StopKey (⌘.)

// app side (docket-client)
pub trait IntentProvider: Send + Sync {
    fn manifest(&self) -> &ValidManifest;
    fn perform(&self, inv: Invocation) -> impl Future<Output = Result<Outcome, AppRefusal>> + Send;
    fn dry_run(&self, inv: Invocation) -> impl Future<Output = Result<Preview, AppRefusal>> + Send;
    fn undo(&self, token: UndoToken, actor: prov::Actor) -> impl Future<Output = Result<(), UndoFault>> + Send;
    fn search(&self, text: &str) -> impl Future<Output = Vec<Hit>> + Send;
    fn preview(&self, id: &EntityId) -> impl Future<Output = Preview> + Send;
    fn suggest(&self, ask: SuggestAsk) -> impl Future<Output = Vec<EntityRef>> + Send;
}
pub trait ContextSource: Send + Sync { fn snapshot(&self, scope: ContextScope) -> ContextSnapshot; }   // docket-ds implements over ds-intents
pub trait SummonTarget: Send + Sync { fn summon(&self, serial: SummonSerial, origin: SummonOrigin) -> SummonAnswer; } // docket-ds over ds CompanionPort
pub enum SummonAnswer { TookField, TookAnchored, Restored, Declined }
```

### 3.6 docket `companion-wire` (new; the `org.quire.Companion1` bodies)

```rust
pub struct AskWire { pub session: SessionId, pub turn: TurnId, pub parent_window: WindowKey }
pub struct AnswerWire { pub task: TaskId, pub phase: AnswerPhase, pub body: AnswerBody, pub footer: FooterWire }
pub enum AnswerPhase { Thinking, Streaming, NeedsYou(NeedsYou), Done, Failed, Cancelled }
pub enum NeedsYou { Confirm(ConfirmId), Form(FormWire), Question { text: String, choices: Vec<String> } }
pub enum AnswerBody {
    Text { lines: Vec<Reveal<String>> },            // handles resolved for display by Session.Display, never by a model
    DraftReply { to: Vec<EntityRef>, subject: Reveal<String>, body: Reveal<String>, actions: Vec<CardWire> },
    ProposedEvent { title: String, when: DateTimeRange, people: Vec<EntityRef>, actions: Vec<CardWire> },
    Plan(PlanWire), Replace { original: Reveal<String>, proposed: Reveal<String>, undo: Option<UndoId> },
    Form(FormWire), Refused(RefusalWire),
}
pub struct PlanWire { pub steps: Vec<PlanStepWire> }
pub struct PlanStepWire { pub id: StepId, pub action: ActionRef, pub label: LabelText, pub effect: prov::Effect,
                          pub state: StepWireState, pub call: Option<CallId> }
pub enum StepWireState { Pending, Running, Done { undo: Option<UndoId> }, Failed(CallRefusal), Skipped, Undone }
pub struct CardWire { pub id: CardActionId, pub label: LabelText, pub effect: prov::Effect, pub call: CallRequest }
pub struct FormWire { pub action: ActionRef, pub params: Vec<ParamDecl>, pub values: Args }     // from AppRefusal::NeedsParam
pub enum RefusalWire { NeedsCloud(DataClass), NotAllowed(DenyCode), NoWay { app: AppName }, OverBudget(BudgetKind), Failed(String) }
pub struct FooterWire { pub served: Vec<ServedBy>, pub sources: Vec<EntityRef>, pub keep: ContextKeep }
pub enum SessionRecord { Opened { space: SpaceId }, Asked { turn: TurnId, task: TaskId }, Finished { task: TaskId, phase: AnswerPhase }, Closed }
```

### 3.7 quire (ux.md §3.1-§3.6 frozen as written, with these changes)

```rust
// ds-core::vocab — one home for companion view vocabulary
pub enum CompanionPresence { Idle, Listening, Working, Acting, Waiting }     // ux §3.1
pub enum MemoryVerb { Forget, Keep, Discard, OpenSource, Export }            // was in ds and sill-launcher
pub enum EffectMark { Read, UndoableWrite, Outbound, Destructive }           // view of prov::Effect
pub enum ActorMark { You, Companion }                                         // was ds Actor
pub struct Tally(pub u32);                                                    // view count

// ds-intents (new crate): presentational context + the two app seams; no docket types
pub struct ThingMark { pub kind: String, pub key: String, pub title: String, pub subtitle: String }
pub trait ContextModel { /* thing props on Row/List, text targets on edit surfaces; ux/actions fill */ }
pub trait CompanionPort: 'static { fn on_summon(&self, h: Callback<SummonSerial, SummonAnswerMark>);
    fn ask(&self, serial: SummonSerial, prompt: String, chips: Vec<ContextChip>);
    fn answers(&self) -> ReadSignal<Option<(SummonSerial, AnswerView)>>; fn cancel(&self, serial: SummonSerial); }
pub struct SummonSerial(pub u64);
// ux types moved: CompanionPort, SummonSerial, SummonAnswer(Mark), use_prompt_target live here; ds re-exports.

// ds companion components: as ux §3.2-§3.5 except: UndoKey deleted (ReplacePhase::Applied{undo: ds::stack::UndoToken});
// AnswerId deleted; Need -> FieldNeed; Actor -> ActorMark; counts are Tally; run_controls takes `undoable: Tally`.
// ds-shell: ConfirmView { .., gesture: GestureMark { Press, Hold } }, ConfirmChoice { Allow { always: AlwaysChoice }, Deny },
//           TaintLine (was TaintNote), GlowLook (was GlowState), glow_spec(&GlowLook, Scheme) -> GlowSpec.
// ds-settings: Page::Intelligence (label per Q-U8).
```

### 3.8 sill (ux.md §3.7 with the merges)

```rust
// sill-launcher (closed enums gain variants; warn the sill session before landing)
ProviderKind:  ... , Things, Companion, Runs, Memory            // ALL: [_; 13], this order
Subject:  Entity(docket_core::EntityRef), Answer(TaskId), Run(RunId), Activity(UndoId), Memory(almanac_core::MemoryItem)
Preview:  Intent(docket_core::Preview), Answer(TaskId), Run(RunId), Memory(almanac_core::MemoryItem)
Activation: Intent(docket_core::CallRequest), Companion(CompanionAct)       // no Activation::Cua
pub enum CompanionAct { Ask { prompt: String }, Card { task: TaskId, action: CardActionId }, Promote { task: TaskId },
    Run { run: RunId, control: RunControl }, Undo { scope: UndoScope }, Memory { item: MemoryItem, verb: MemoryVerb }, Mode(LauncherMode) }
pub enum RunControl { Watch, TakeOver, HandBack, Pause, Resume, Stop, UndoAll }    // Replay: later (procedures)
// sill-ipc: ShowRequest::Companion(CompanionRequest { Tap, Summon, Dismiss, StopAll })
// sill-model::companion: Tap, Summon machines (ux §4.1-4.2); presence_of; run_state_of; run_controls(state, place, undoable)
```

### 3.9 cua (cua.md §3.1-§3.9 with these changes)

- `cua-action` types come from §3.3; `Lpx/WinPoint/WinSize/WinRect` are deleted (`Point/Size/Rect<WindowSpace>`); `GlobalPoint(i32,i32)` stays private to `seat-input::region`.
- `cua-run`: `TargetedAction { window: WindowRef, action: CuaAction<WindowSpace> }`, `WindowRef(u32)`;
  `turn(&Observation) -> (CuaStepRequest, FrameAttachment)`; seam
  `trait CuaModel { fn begin(&mut self, b: CuaBegin) -> Future<Result<(), CuaModelError>>; fn step(&mut self, r: CuaStepRequest, a: FrameAttachment, sink: &mut impl StepSink) -> Future<Result<CuaStepReply, CuaModelError>>; }`
  with `CuaModelError { Transient, Parse, Fatal }`, impls `InferdCuaModel` (cuad, over porter-client) and `ScriptedModel` (cua-fake, over model-replay).
- `classify_effect(..) -> (prov::Effect, docket_core::EffectBasis)`; `EffectClass`, `CuaVerdict`, `ConfirmText`, `DenyReason` deleted.
  Authorising sends `docket_core::CuaAsk` through `Intents1.Gate.Check`; `RunInput::Verdict(GateAnswer)`;
  `RunInput::ConfirmPending(ConfirmId)` (from Request `Progress(Confirming)`) drives `SuspendLease(Consent)` on stock, then `Request.Proceed`.
  `Granting` calls `Intents1.Gate.Grant(app, space)`.
- `RunBudget { steps, active, stall_frames, actions_per_min, model_failures, spend }` (no `denials`); `GateAnswer::Refused(Paused(_))` → `Finished(Blocked(PolicyDenials))`; `Refused(Halted(_))` → `Paused`.
- `cua-bus` (new, serde-only): `RunStateSlug`, `CuaTask` (the `Start` body), `CuaResult`, `CuaOutcome`, `CuaRecord` (actor `prov::Actor::Companion{role: Cua{run}}`, served_by `porter_infer::ServedBy`), `ControlRefusal`, plus the `Cua1` XML (cua.md §3.8, unchanged; ux's `Replay`/`UndoAll` additions rejected: undo is `Intents1.Run.UndoAll(Run(id))`).
- cuad also serves `IntentProvider1` as app **`org.quire.Cua`** with one action
  `cua.run.start { goal: Text(sink Body), target: Entity(desktop.window) | app: Text, mode: Choice }`,
  `effect: Read`, `latency: Long`, `reach: Hidden` while `cua.enabled = off`. Each pixel step is gated on its own.

---

## 4. D-Bus names and daemons

### 4.1 Daemons (merged)

| Daemon | Repo | Merges | Bus name / object | Serves | Calls |
|---|---|---|---|---|---|
| **inferd** (existing) | porter | models' engine host; 31's MCP host role removed (C16) | `org.quire.Inference1` `/org/quire/Inference1`; `org.quire.SettingsModule1` at `/org/quire/Inference1/settings` | `Availability`, `Open(need,class,tier)->h` (fd stream of ClientFrame/InferEvent, memfds), **`Prepare(need,class,tier)->s`**, `Usage`, `Rescan`; signal **`EnginesChanged()`**; property **`Gpu s`** | engines over Unix sockets (transient systemd units) |
| **intentd** | docket | actions' router, ux's "security" checks, cua's per-step PDP, the planner's memory path | `org.quire.Intents1` `/org/quire/Intents1` | §4.2 | `IntentProvider1` on apps, `Confirm1` (sill), `Reader1`, `Memory1` (Record; Recall for the built-in Memory provider), inferd (reviewer, policy writer) |
| **companiond** | docket | ux's "agent daemon", actions' "companion", cua's "planner/router that calls Start", memory's "agent-loop" | `org.quire.Companion1` `/org/quire/Companion1` | §4.2 | `Intents1` (role companion), inferd (planner) |
| **readerd** | docket | actions' quarantined reader | `org.quire.Reader1` `/org/quire/Reader1` | `Extract(session s, ask s) -> s` (caller must be intentd) | `Intents1.Session.Resolve`, inferd (reader: `Need::Llm{StructuredOutput}`, no tools) |
| **memoryd** | almanac | ux's "memory daemon" | `org.quire.Memory1` `/org/quire/Memory1` (memory.md §3.10; ux's names mapped C27) | Record / Recall / Control | inferd (embed, consolidation) |
| **cuad** | cua | cua-integration's cuad | `org.quire.Cua1` `/org/quire/Cua1` (+ `/run/<id>`); `IntentProvider1` as `org.quire.Cua` | cua.md §3.8 | `Intents1.Gate`, inferd (ComputerUse, class Screen), `Memory1.Record`, `com.system76.CosmicComp.Ei.GetSenderSocket` (stock) or `quire-agent-v1` (fork) |
| sill daemon (existing) | sill | ux's confirm phase A and glow phase A | serves **`org.quire.Confirm1`** `/org/quire/Confirm1` and `IntentProvider1` as `org.quire.Shell` | `Confirm(request s)->o`, `Cancel(id s)` | all of the above as a client |
| actions-mcp | docket | — | not on the bus (stdio/unix socket), off by default | MCP tools generated from the registry | `Intents1` as role mcp |

### 4.2 Interfaces (bodies are JSON of the named type in `s`, envelope `{vocab, body}`; caller identity is always derived from the connection, never sent)

**`org.quire.Intents1`** (intentd). actions.md §3.12 with the addendum, plus the rows marked new.

| Interface | Members | Callers |
|---|---|---|
| `.Registry` | `Manifests() -> a(ss)`; sig `ManifestChanged(s)` | any |
| `.Index` | `Push(batch s)`, `Reset(epoch t)` | owning app |
| `.Search` | `Query(text s, scope s, gen t) -> s`; `Cancel(gen t)`; sig `Hits(gen t, hits s)` | launcher, companion, apps |
| `.Run` | `Perform(call s, parent_window s) -> o`; `Preview(entity s) -> s`; `Suggest(ask s) -> s`; `Undo(entry t) -> o`; **`UndoAll(scope s) -> o`** (`UndoScope`) | per role |
| `.Context` | `Current(session s) -> s` (`ContextView`; the snapshot taken at the turn) | launcher, companion |
| `.Session` | `Open(space s) -> s`; `Turn(session s, turn s) -> t` (`TurnIn`; **launcher and field roles**; derives/narrows TaskPolicy; snapshots context); `Close(s)`; `Resolve(handle t) -> s` (**reader only**); **`Display(session s, handle t) -> s`** (launcher and the summoning app only: text for the screen, never a model); **`Read(session s, ask s) -> s`** (companion: `ReaderAsk` → readerd → labelled value or a new handle); `TaskPolicy(session s) -> s`; `Widen(session s, turn t, change s) -> o` | launcher, field, companion, reader |
| **`.Gate`** (new) | `Grant(app s, space s) -> o` (CUA consent, `ActionGrant` with caller Cua); `Check(ask s) -> o` (`CuaAsk` → `GateAnswer`) | cua |
| `.Control` | `Halt(scope s, cause s)`, `Resume(scope s)`, `State() -> s`, `Journal(filter s) -> s`; sigs `Halted(s)`, `Resumed(s)`, `JournalChanged(t)`, **`BreakerTripped(session s)`** | compositor, control (sill) |
| `.Request` `/org/quire/Intents1/request/<n>` | `Close()`, **`Proceed()`** (cua: lease suspended, show the confirm); sigs `Progress(s)` (`CallProgress`), `Response(u, s)` | requester |

Caller roles (`intentd.toml`): `launcher` = sill; `field` = any quire app recording a turn from its own
field (TurnSource::Field); `companion` = companiond; `reader` = readerd; `cua` = cuad; `mcp` =
actions-mcp; `confirm` = sill (the only Confirm1 server it trusts); `compositor` = the fork; `control` = sill.

**`org.quire.IntentProvider1`** (every provider: apps, `org.quire.Shell`, `org.quire.Cua`; `org.quire.Memory` is built into intentd)
`Perform(inv s)->s`, **`DryRun(inv s)->s`**, `Undo(token s, actor s)->s`, `Context(scope s)->s`,
**`Summon(serial t, origin s)->s`** (`SummonAnswer`; ds answers via docket-ds), `Search(text s, gen t)->s`,
`Preview(entity s)->s`, `Suggest(ask s)->s`, `Resolve(keys s)->s`; sigs `UndoChanged(token s, state s)`, `IndexStale(epoch t)`.

**`org.quire.Confirm1`** (sill): `Confirm(request s) -> o` (Request carries `Response(u, answer s)`),
`Cancel(id s)`. Phase A: a `sill-overlays` sheet, receipts `SheetFallback`. Phase B: the same sill
surface placed on the fork's `quire_trusted_surface_v1` (hardware input only, nothing can draw over
it, the agent seat cannot focus it), receipts `HardwareSeat`.

**`org.quire.Companion1`** (companiond; replaces ux's sketch: no Presence, Undo, Activity or PauseSpace members)

| Member | Notes |
|---|---|
| `Prepare()` | on summon; calls `Inference1.Prepare` for the planner role so the first turn skips a cold start |
| `Open(space s) -> s` | opens an `Intents1` session as role companion; returns `SessionId` |
| `Ask(ask s) -> o` | `AskWire` (session + a turn the UI already recorded); returns `/org/quire/Companion1/answer/<task>` |
| `Close(session s)` | |
| sigs `AnswerAdded(o)`, `AnswerRemoved(o)` | |
| `.Answer` on each answer object: property `View s` (`AnswerWire`); sig `Updated(s)`; `Act(action s) -> o` (a card action → `Run.Perform`); `Cancel()` | |

**`org.quire.Memory1`** (memoryd): memory.md §3.10 unchanged.
**`org.quire.Cua1`** (cuad): cua.md §3.8 unchanged.
**`org.quire.Inference1`**: models.md §3.8 table (additive).
**`org.quire.Reader1`**: `Extract(session s, ask s) -> s`.
**Wayland** (cua repo `quire-agent-protocol`): `quire-agent-v1` (manager, lease: `set_glow(state, rect)`
replaces ux's `quire_agent_glow_v1`), later `quire_trusted_surface_v1` (sill client) and
`quire_sensitive_v1` (shell-host client). Stock: `com.system76.CosmicComp.Ei.GetSenderSocket`.

Name rule: every interface we own is `org.quire.<Name>1`; objects at `/org/quire/<Name>1`; errors
`org.quire.<Name>1.Error.<Variant>` mapped 1:1 from the refusal enum.

---

## 5. Cross-area flows (who owns each step)

### (a) Double-tap ⌘ in mailo's search with 12 results → "forward the Lisbon receipts to accounting" → plan → confirm → done → undo

| # | Step | Owner |
|---|---|---|
| 1 | Toshy emits the chord → `sill companion tap` (route B) or `summon` (route A) → `ShowRequest::Companion` | ux (sill), Toshy spike |
| 2 | `Tap` machine → `Summon` machine: mailo focused, owns `org.quire.Mail` with a manifest → `AskApp` → `IntentProvider1.Summon(serial)`; sill calls `Companion1.Prepare()` in parallel | ux (sill), docket (method) |
| 3 | In mailo, docket-ds asks ds's `CompanionPort`: the search field answers `TookField`; **ds-intents freezes the context at this moment** (query "lisbon receipts", 12 visible results, selection) before the field turns into a prompt; chips Query, Results(12) | quire ds-intents, docket-ds |
| 4 | User types and sends. docket-ds: `Companion1.Open(space)` → `Intents1.Session.Turn(session, TurnIn{text, InWindowField, keep})` (role field, `TurnSource::Field(org.quire.Mail)`) → `Companion1.Ask` | docket |
| 5 | intentd: PolicyWriter derives a TaskPolicy from the turn only (Mail actions up to Outbound, kind mail.message, max 12; recipients unresolved); Field turns are capped to Mail + Read elsewhere | docket |
| 6 | companiond/agent-loop: `PlannerView` (result titles are third-party → handles); planner calls `contacts.search("accounting")` (Read, AllowFinal; Contacts is a trusted store → the contact is a Trusted sink); if it must pick which results are receipts, `Session.Read(ReaderAsk{Classify, EntitiesAmong})` → plain entity ids, planner taint becomes Untrusted | docket (+ models for the planner/reader models) |
| 7 | Plan card `Plan[forward 12 → Accounting]` streams to mailo's prompt (answer `Updated`); plans run at once, confirmations happen per call (C40) | docket, ux |
| 8 | `Run.Perform(mail.message.forward, Many(12), {to: Entity(contact)})`: no halt, budget ok; consent first use Mail×Mail×Space; Ruling **Ask** (Outbound, tainted planner; Rule of Two also fires); `Previewing`: mailo `DryRun` → `Preview::Message`; `Progress(Confirming)` → answer phase `NeedsYou(Confirm)` → presence Waiting | docket, mailo |
| 9 | `Confirm1.Confirm` → sill sheet (phase A) / trusted surface (phase B): "Forward 12 messages to Accounting?", recipients from the DryRun, offer OnceOnly (taint), arm delay → Allow → `ConfirmReceipt` | ux (sill), cua fork (phase B) |
| 10 | Dispatched → mailo performs (actor Companion, undo entry labelled on mailo's own stack), returns `Undoable::Yes(token)` valid for mailo's undo-send window → `UndoEntry` in the journal, `JournalChanged`; AuditRecords → `Memory1.Record` (Area{Docket}, things = 12 messages + contact) | docket, mailo, almanac |
| 11 | Answer `Done`, card shows Undo; presence Idle | docket, ux |
| 12 | Undo: card/strip → `Intents1.Run.Undo(entry)` (actor User) → `IntentProvider1.Undo` → mailo cancels the held send → `Undone{by: User}`. ⌘Z in mailo does the same through mailo's stack and `UndoChanged`. After the window: `Failed(Gone)` shown honestly | docket, mailo, ux |

### (b) Launcher prompt that needs a CUA run in a third-party app

| # | Step | Owner |
|---|---|---|
| 1 | Launcher companion mode → sill records `Session.Turn` (role launcher) → `Companion1.Ask` | ux, docket |
| 2 | agent-loop tier rule: typed action → hook (D-Bus/portal/MPRIS) → CUA. No typed action for Firefox → `Run.Perform(org.quire.Cua: cua.run.start{goal, app, mode})` (Read, AllowFinal; Hidden unless `cua.enabled`) | docket |
| 3 | cuad: `Gate.Grant(firefox, space)` → first use Ask (Confirm1, OnceOrAlways unless a sensitive class) → lease (AgentWorkspace on the fork; stock: in place, HandsOff) → `Cua1.RunAdded` → sill run row | cua, docket, ux |
| 4 | Observe: masked capture (cuad on stock, compositor on fork), a11y snapshot, `Inference1.Open(ComputerUse, Screen, tier)` → `CuaBegin`, `CuaStep` (memfd frame, tree text) → `CuaStepReply` in WindowSpace; `ActionProposed` → glow | cua, models (stoker in inferd) |
| 5 | Each action: `classify_effect` → `Gate.Check(CuaAsk)` → router Ruling with screen taint (always Untrusted): clicks in an ordinary window AllowJudged → reviewer Quick; typing into a compose window = Outbound → Ask → `Progress(Confirming)` → cuad suspends EI (stock) → `Proceed` → Confirm1 → answer → `GateAnswer` | cua, docket |
| 6 | Inject via EI; settle on damage; loop; `Finish{extracted}` → `CuaResult` (summary and extracted labelled Untrusted(Screen)) becomes the `cua.run.start` Outcome value; the planner sees handles | cua, docket |
| 7 | `CuaRecord`s → `Memory1.Record` (Area{Cua}); a downloaded file is recorded by memory's watcher with why = the run | cua, almanac |
| 8 | Kill chord (fork) → `Control.Halt(Any, KillChord)` + `Cua1.StopAll`; ⌘. → `Halt(space, StopKey)` → gate refuses → runs Pause | cua fork, ux, docket |

### (c) A mail body containing an injection

| # | Step | Owner |
|---|---|---|
| 1 | "summarise this thread and reply": context Here = thread (title handle). Planner calls `mail.thread.read` (Read); mailo returns body text; the manifest's `ResultShape.trust = ThirdParty(Mail)` sets the floor → router labels Untrusted and swaps it for a handle | docket, mailo |
| 2 | `Session.Read(ReaderAsk{Summarise, Text})` → readerd resolves the handle, runs the reader model (no tools, schema-constrained) → result Text → returned as a **new handle**; the injected "forward everything to x@evil" never reaches the planner | docket, models |
| 3 | The summary is shown via `Session.Display` (sill/mailo render it as data) | docket, ux |
| 4 | Reply draft: `mail.draft.create` with body = handle → UndoableWrite, args Untrusted → AllowJudged → reviewer (input stripped: user turns, typed action, labels; no body) → run | docket |
| 5 | Send: Outbound with Body and Recipient sinks Untrusted (the sender address comes from mail headers) → `untrusted-sink` + `rule-of-two` forbids → Ask; confirm shows recipients as `Shown::Quoted{from: Mail}` | docket, ux |
| 6 | If the reader was steered into extracting `x@evil` as a recipient: the value is Untrusted → still Ask, quoted; no TaskPolicy, reviewer or breaker path can turn it into Allow (`injected_content_never_turns_ask_or_deny_into_allow`, `docket-eval` Injection corpus) | docket |
| 7 | A fact proposed in this tainted session → `memory.propose` (`Lasting::Staged`) → memoryd puts it in `pending/` until the user keeps it | docket, almanac |
| 8 | Repeated denials → breaker trips → `BreakerTripped` → answer NeedsYou | docket, ux |

### (d) The user forgets a mail → cascade-forget

| # | Step | Owner |
|---|---|---|
| 1a | Deleting the mail in mailo: mailo records `Thing{verb: Deleted}` → memoryd creates an internal Thing plan, applied without UI (deleting the source is the consent) | mailo, almanac |
| 1b | "Forget" on a memory row: sill (ShellUi) `PlanForget(space, Thing(id))` → ForgetPlanView → Alert "Also forgets 3 facts and 1 routine" → `Forget(token)` | ux, almanac |
| 2 | Closure: events whose subjects/sources include the thing (including `Area` payloads by their `things` lists: router audit, CUA records), facts linked transitively, pending facts, procedures, kept CUA thumbnails, index docs | almanac |
| 3 | Apply: index → memfiles → event bodies (headers stay, Q-Me3) → `Memory.Forgot` → WAL truncate; signal `Forgotten` → sill refreshes | almanac, ux |
| 4 | Outside memory: mailo pushes `Index.Push(removals)` to intentd's shadow index; undo journal holds no entity ids; inferd audit holds no content; intentd sessions keep no content after Close | mailo, docket, porter |
| 5 | Forgetting a memory never deletes the mail itself (stated in the Alert) | ux |

Gaps found by the walk and fixed above: who may record a turn (TurnSource); context captured before
the field becomes a prompt; showing untrusted text to the person (Session.Display); how the planner
reaches the reader (Session.Read); how cuad's confirm waits for EI suspension (Request.Proceed);
memory reads bypassing router taint (memory via the router); event payloads needing `things` for
cascade; outbound undo windows; UndoAll for CUA runs (only typed entries).

---

## 6. Freeze plan

Rules for every agent: commits as Po Hsuan Lai <pohsuanlai0208@gmail.com> (author and committer);
quire/sill merges go through the `~/X-wt/gate` worktree and fast-forward master only on GATE GREEN;
the gate runs each command with its exit code checked (`set -euo pipefail`, never `cargo test | tail`);
tests never touch the real system (private buses, scratch XDG/HOME, no GPU, no real engine). Warn the
sill session before any sill-launcher closed-enum change lands. Behaviour is `todo!()` behind frozen
signatures, each listed in FINDINGS.md; shape tests (round trips, introspection, compile-fail, pinned
JSON, step tables of the pure parts) pass.

### 6.1 Spikes (run first; what each blocks)

| Spike | Where | Blocks | Does not block |
|---|---|---|---|
| **S0 Holo-3.1-4B chat template** (read `chat_template.jinja`, `preprocessor_config.json`, `config.json` via WebFetch; read-only) | stoker agent, step 0 | `ToolDialect` variants, `GridMax`/`PatchFactor` values, `catalog/holo-3.1-4b.toml`, cua-parse fixtures (W1b) | geometry and every other stoker type |
| **S-SQL** SQLCipher bundled build with FTS5 + `cargo deny` | scratch crate `~/rs-wt/spikes/sqlcipher` | wave 0's rusqlite line → almanac freeze (eventlog/recall hold rusqlite types) | — |
| **S-Cedar** `cargo deny` for cedar-policy 4.13 | wave 0 agent | wave 0 line → docket freeze | — |
| **Spike A** stock cosmic-comp 1.8 nested: capture + EI inject (cua.md §6) | `~/rs-wt/spikes/cua-a/`, then moved to `~/cua/spike/` | cua fill 2 (seat-input `ei`, maskcap `wayland`, cua-desktop `cosmic`), stock `mode_support` values, Q-P2 | the cua freeze |
| **Spike T** Toshy double-tap route (does a tap-vs-hold remap on ⌘ disturb ⌘-click/⌘-drag?) | manual check queued in `sill/docs/manual-checks.md` (needs the real keyboard) | sill W4 wiring of `Tap` vs `Summon`, `dist/toshy/companion_tap.py`; default route A until it runs | the sill freeze (both variants frozen) |
| models S1 (engines on Unix sockets under `PrivateNetwork`) | dev script, scratch user unit | stoker W2b, inferd W3 | freeze |
| memory S1 (embedding throughput), S3 (fscrypt), S4 (fanotify), S5 (sqlite-vec), S6 (xattr) | dev scripts | memoryd embedder default, Q-Me1, Q-P4, Q-P5 | freeze |
| cua Spike B (KWin), E (a11y), C/D (fork) | dev scripts | the KWin backend, the atspi backend, fork fill | freeze |

### 6.2 Freeze wave (one agent per repo; stages in dependency order)

| Stage | Agent (repo) | Owns | Verify |
|---|---|---|---|
| 0 | **deps** (quire) | `docs/workspace-deps.toml` only (§1.4 lines, after S-SQL and S-Cedar) | quire gate |
| 1 | **stoker** (new) | everything in `~/stoker`; S0 first; commits `cua-action` first so porter can path-patch it | porter-style gate |
| 1 | **quire** | `ds-core/src/vocab.rs`, `ds-intents/**` (new), `ds/src/components/companion/**` + its `mod.rs` line, `ds/src/assembly/sheets.rs`, `ds-shell/src/{confirm,tokens/glow.rs}`, `ds-style` orb tokens (proposed values, marked), `ds-settings/src/schema/key.rs`; docs: `design/22` rows (all `ai.*`, `agent.*`, `memory.*`, `cua.*`, `companion.*` keys), `design/32-COMPANION.md` (from COMPANION.md), `design/33-AGENT.md` (this spec); **not** design/31, TextField or CommandPalette | quire gate incl. `check-consumer.sh`, `check-doc-paths.sh` |
| 2 | **porter** | `crates/prov/**` (new), porter-core (`space.rs`, consent generic + Space, ComputerUse, VocabVersion 2), porter-provider (`Discovery::Supervised`, `providers/local.toml`), porter-infer (§3.2), porter-dbus + `dbus/org.quire.Inference1.xml`, porter-client, porter-fake, inferd skeleton modules, ARCHITECTURE/FINDINGS/boundary rows, **and `~/quire/design/31-ACCOUNTS.md`** (§2.3, §3.3, §4.4, §4.5, §5.5 MCP role removed, §11) | porter gate |
| 3 | **almanac** (new) | everything in `~/almanac` | porter-style gate, `--exclude recall-fastembed` |
| 3 | **docket** (new) | everything in `~/docket` (including companion crates, `docket-ds`, `manifests/org.quire.Memory.toml`, `default.cedar` stubs with ids `rule-of-two`, `untrusted-sink`, the XML for Intents1, IntentProvider1, Confirm1, Companion1, Reader1) | porter-style gate + `cargo test -p policy-point -p docket-eval` |
| 4 | **cua** (new) | everything in `~/cua` (incl. `cua-bus`, both XML/protocol files); joins nothing to quire (reis/atspi landed in stage 0) | porter-style gate |
| 4 | **fork-setup** | `~/cosmic-comp` at the 1.8 tag, branch `quire`, `src/quire/mod.rs` hook, stub globals | `cargo build --release && cargo test`; runs nested |
| 5 | **sill** | ux.md §6 F2 file list with §3.8 here; design/22 is the quire agent's | sill gate |
| 5 | **detent** | `detent-model/src/page.rs`, `detent-ui/src/icons.rs`, fixtures | detent gate + `scripts/gen-fixtures.sh` |

Porter-style gate (stoker, porter, almanac, docket, cua):

```bash
set -euo pipefail
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./scripts/check-boundary.sh
cargo deny check licenses
echo "GATE GREEN"
```

quire: the same plus `./scripts/check-consumer.sh`. sill: `cargo test --workspace --features sill/debug`
in place of the plain test line. almanac: add `--exclude recall-fastembed` to clippy and test.

Cross-repo deps during the freeze: `[patch]` path entries to sibling checkouts (quire CONSUMING.md §1);
pinned git revs replace them in fill wave 1, stage by stage, once each upstream freeze is merged.

### 6.3 Fill waves (file ownership = crate or listed directory; area specs hold the detail)

| Wave | Parallel agents (repo: owned paths) | Source |
|---|---|---|
| F1 pure | stoker: vision-prep; cua-parse (+fuzz/); model-catalog + supervisor `step/budget`; model-replay + ScriptedProvider. porter: matching arm, `picker_rows`, `tier_choice`, session step, FakeModel streaming, `prov` lattice. almanac: eventlog; memfiles; recall (+fastembed); seal + watch. docket: policy-point (Cedar, grid); action-review (cascade, breaker); docket-router (call step, covers/compare, SessionSaw, journal, Previewing; owns docket-fake); docket-dbus + docket-client; docket-eval; agent-loop. cua: cua-run; a11y-tree; seat-input; maskcap. quire W1: orb, chips, prompt, TextField/CommandPalette prop pass-through. | models §6, memory §6, actions §6 + addendum, cua §6, ux §6 |
| F2 io/backends | stoker: model-http + openai-compat (fixtures by dev script); supervisor io (after models S1); cua-session. almanac: almanac-service + almanac-fake. docket: intentd (private bus, seams wired to almanac-client); actions-mcp; readerd; companiond. cua (after Spike A/E): seat-input `ei`; maskcap `wayland`; a11y-tree `atspi`; cua-desktop `cosmic`. quire W2/W3: cards, plan, replace, form, rows, ds-shell views; ds-intents ContextModel (after Q-P7). | same |
| F3 daemons | porter: inferd bridge, engines, fd session server with memfds, Prepare, `cua_run`; client `DbusTransport::open`. almanac: almanac-dbus, almanac-client, memoryd + unit. cua: cuad, cua-dbus codec, cua-fake harness, `dist/`. docket: docket-ds against quire. sill W4/W5 (on fakes): companion service, providers, launcher mode, bar/control centre, overlays (confirm, glow phase A). | same |
| F4 integration | mailo session (manifest, provider, actor on UndoStack, undo-send window); sill W6 (replace fakes; nested compositor, private bus, scratch HOME); fork fill F1-F7 with acceptance scripts; acceptance `dev/cua-step-holo.sh` (GPU, by hand) | — |
| Later | cloud backends + cua-vendors bodies, mistral.rs spike, weight downloader, procedures, KWin backend, `quire_sensitive_v1`, recall-sqlite-vec, egress, secret-cell | — |

---

## 7. Conflicts and their resolutions

| # | Conflict | Resolution |
|---|---|---|
| C1 | Specs guessed crate/repo names (`docket-*`, `almanac-*`, `quire-intents`, two crates named `cua-wire`) | §1.3 names; models' `cua-wire` → `cua-vendors`; `quire-intents` → `docket-core` |
| C2 | `prov` home: docket (actions) vs prov/almanac (memory); docket↔almanac repo cycle | `prov` in porter; almanac stores other areas' payloads opaquely |
| C3 | `Effect` four homes | `prov::Effect`; ds `EffectMark` view |
| C4 | Two `Actor` enums + cua's `Companion(CuaRun)` + ux view | merged `prov::Actor` (§3.1) |
| C5 | `RunId(u64)` vs `RunId(String)` | String slugs in prov |
| C6 | `ThingRef` vs `EntityId` | aliases of `prov::EntityId` |
| C7 | Two `cua-action` definitions (typed spaces vs `Lpx`, Node targets, `Done{extracted}`) | merged §3.3; `TargetedAction` in cua-run |
| C8 | Float / i32 / physical-pixel coordinates (cua-integration, cua.md, research) | `Coord(u32)` window logical px + `Scale120` |
| C9 | cua wanted `CuaModel` in model-provider with `dyn FnMut`; models runs the session in inferd | inferd hosts `cua-session`; cuad's `CuaModel` seam is in cua-run over `Inference1` |
| C10 | cuad links `policy-point` vs COMPANION "one decision point in the router" | `Intents1.Gate`; one PDP, one breaker, one audit |
| C11 | CUA grant home | docket `ActionGrant` with caller Cua (both specs agreed) |
| C12 | cua per-run `denials` budget vs router breaker | breaker only |
| C13 | Confirm1 served by the fork (actions) vs sill sheet + trusted surface (ux) | sill serves Confirm1 in both phases; the fork supplies `quire_trusted_surface_v1` |
| C14 | "CUA must not ship on SheetFallback" (actions C3) vs "stock first" (cua Q2) | on stock a CUA confirm shows only after cuad stops emulating (`Request.Proceed`); shipping call is Q-P2 |
| C15 | `Need::Cua` vs `Need::ComputerUse` | ComputerUse, VocabVersion 2 |
| C16 | inferd as MCP host (31 §5.5) vs `actions-mcp` edge-only | actions-mcp; 31 edited |
| C17 | async-openai in the build map | not adopted |
| C18 | memory re-declares policy/consent/CUA/session payloads | `EventBody::Area`; owners keep types |
| C19 | companion reads memory directly, so the router cannot compute taint | memory reads/writes for the planner are router actions of `org.quire.Memory` (built into intentd); `Caller::Companion` removed |
| C20 | "Lasting from untrusted → Ask" (actions) vs pending facts (memory) | `Lasting::Staged`: pending queue is the confirmation |
| C21 | Undo/activity duplicated in Companion1 and Cua1 (ux) | router journal and `Run.Undo/UndoAll`; `Replay` later |
| C22 | UndoAll offered for every CUA run (ux) | only when the journal holds typed entries for the run |
| C23 | Companion1 `Presence` property vs sill derivation | sill-model `presence_of` only |
| C24 | Three `GlowState`s and two glow protocols | cua-desktop wire + `quire-agent-v1.set_glow`; ds-shell `GlowLook`; style push later |
| C25 | `Activation::Intent{action,args}` (ux) vs `Intent(CallRequest)`; `Activation::Cua` (cua) vs `Companion(Run)` (ux); `CuaRunProvider` vs `RunsProvider` | `Intent(CallRequest)`; `CompanionAct::Run`; `RunsProvider`; ProviderKind order Things, Companion, Runs, Memory |
| C26 | Esc halts/pauses (actions, cua-integration) vs Esc closes one level (ux) | ux: ⌘. and Stop stop; `HaltCause::StopKey` |
| C27 | ux's memory method names vs memory.md | memory.md: `PlanForget`/`Forget(token)`/`Settle`/`Consolidation`/`Export(options, h)` |
| C28 | "context area" `Summon` method | `IntentProvider1.Summon` |
| C29 | `ds-intents` using docket-client (actions) vs "quire never names daemon crates" (ux) | quire `ds-intents` is view-only; `docket-ds` adapter in docket |
| C30 | design/30 §3.4 defers companion work; new catalogue rows need the user | freeze types now; fills wait on Q-P7 |
| C31 | Settings key duplicates | §2 last row |
| C32 | Name collisions (Verdict, Decision, Need, Fact, Visibility, Usage, Role, Latency2, CallId, Repeat, Space, Budget, TaintNote, ConfirmAnswer, MemoryVerb, Actor) | §2 renames |
| C33 | COMPANION item 4 "no screen clicking" vs the CUA tier | `cua.enabled = off`, cua manifest Hidden; Q-P3 |
| C34 | `Authorization` (actions) vs `TaskPolicy` (addendum) | TaskPolicy |
| C35 | Reviewer may allow grey-zone and sees quoted content (first auto-mode message) | reviewer only tightens; stripped input (addendum C9) |
| C36 | "Turn: launcher only" vs prompts from in-app fields (COMPANION 7) | role `field` + `TurnSource::Field` cap |
| C37 | No path to show untrusted text to the person | `Session.Display` |
| C38 | No path from planner to reader | `Session.Read` → readerd |
| C39 | Outbound actions cannot carry undo tokens | allowed when the app holds the send (undo window) |
| C40 | Plan Draft + Run (ux P4) vs item 9 run-without-asking | plans run at once; gated calls confirm; Draft only when the planner asks |
| C41 | llama.cpp default (build map) vs vLLM for Holo-3.1-4B | per catalog entry; Q-M1 |
| C42 | Toshy route A (settled wording) vs B (ux rec.) | default A; Spike T may switch to B |
| C43 | Plain memory files vs per-Space keys | sealed per file by default; Q-Me1 |
| C44 | sqlite-vec needs `unsafe` | ExactScan default; Q-P5 |
| C45 | notify 9-rc (survey) vs 8.2 (pinned) | 8.2 |
| C46 | `HandBack` two meanings | `Run.HandBack` resumes; handover is `Start` with `origin = Handover` |
| C47 | cuad inside shell-host vs own daemon | own daemon |
| C48 | second `wl_seat` vs reis | EI via reis always |
| C49 | masking in compositor vs capturer | both by backend, `MaskWhere` recorded |
| C50 | P1 inline replace runs at once (item 9) vs waits for Apply (agent-ux P1, ux `ReplacePhase::Proposed`) | user decides (Q-U9); both phases frozen |
| C51 | `sill companion stop-all` "pause" vs cua `StopAll` "cancel" | stop-all = `Halt(space, StopKey)` → runs Pause; kill chord = `Halt(Any)` + `Cua1.StopAll` |
| C52 | Build map lists `egress`, `secret-cell` with no spec | deferred to the first cloud backend |
| C53 | Every planner role needs a model; 16 GB fits one | one resident model; SecondOpinion from another family on demand, absent → Ask; Q-M2 |

<!-- paths: end -->
