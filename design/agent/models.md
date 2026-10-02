<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/models.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# Area "models": the model and provider layer for both tiers

Status: PROPOSAL for the freeze. Sources read: BRIEF.md; COMPANION.md; cua-integration.md;
research-cua-models.md; research-ecosystem-agents.md and -infra.md; intents-research/models.md;
design/31-ACCOUNTS.md (including §11, the frozen interfaces); the porter repo (porter-core
`capability/ai.rs`, `need/`, `ai_props.rs`, `wire/frame.rs`, all of porter-infer, porter-dbus
`inference.rs` + `dbus/org.quire.Inference1.xml`, inferd, porter-fake `FakeModel`,
ARCHITECTURE.md, CONVENTIONS.md, FINDINGS.md, `scripts/check-boundary.sh`); quire CONVENTIONS.md
and `docs/workspace-deps.toml`; design/22 §9.4; design/30 §3.4 and VoiceOrb; sill-launcher
`Provider`. I also read the H Company local-inference page (2026-10-02). It serves Holo with
`vllm serve ... --enable-auto-tool-choice --tool-call-parser qwen3_coder --reasoning-parser qwen3
--limit-mm-per-prompt '{"image": 5}'`, or `llama-server -hf Hcompany/Holo-3.1-35B-A3B-GGUF`.
Neither the Holo-3.1-4B card nor that page gives the action schema or the coordinate convention:
**[unverified] until the chat template is read** (freeze step F1.0).

---

## 1. Scope and non-goals

**In scope (models):**
- `model-provider`: one trait for chat, tool-calling and vision LLMs and for computer-use models. It covers capabilities (`Caps`), streaming through a push sink, cancellation by drop, and the errors.
- Backends:
  - `model-http`: HTTP over TCP or Unix sockets, plus a pure SSE decoder.
  - `model-openai-compat`: llama-server, vLLM, LiteLLM and OpenRouter. Freeze now, fill in W2.
  - Cloud backends (`model-anthropic`, `model-openai-responses`, `model-gemini`): later. The user said open-source models only for now.
- `model-replay`: record and replay cassettes for tests.
- `model-catalog`: the curated model table and the engine profiles, as data files.
- `engine-supervisor`: start, readiness, idle unload, crash backoff and the VRAM budget for llama-server and vLLM.
- `vision-prep`: resize rules per model, a token estimate, and the `FrameMap` between window, image and grid coordinates.
- `cua-action`: the provider-neutral action enum, with coordinate spaces typed so that mixing them is a compile error.
- `cua-wire`: the Anthropic, OpenAI and Gemini computer-use encodings. Trait and enum frozen now, bodies later.
- `cua-parse`: the text and tool-call dialects of local models (UI-TARS-1.5, Qwen computer_use, Holo). Strict, total, fuzzed.
- `cua-session`: the model side of one CUA step: history, prompt assembly, parse with one repair, and mapping into window space.
- The model picker's data model (tier map, picker rows, readiness, fit).
- How all of this reaches apps and cuad through porter's inferd and `org.quire.Inference1`. These are additive extensions plus three marked changes to frozen porter types.

**Non-goals (owned elsewhere or later):**
- The CUA step loop, the leases, capture, masking, injection and `org.quire.Cua1`. These belong to cuad and the compositor (area "cua"). Models hands cuad window-space actions and nothing else.
- The planner's plan and session, the router, taint, Cedar policy and consent drawing (areas "agent" and "security").
- The intents registry, and turning `ActionDecl` into a JSON schema (area "intents"). Models only carries the `ToolDecl`s it is given.
- Picker UI pixels (detent's Intelligence page) and the orb (area "launcher"/"presence"). Models gives them data and events.
- Weight downloads (later: `hf` as a subprocess). For the first wave the user downloads weights by hand.
- Benchmarks or a bake-off (user: one small open model is enough).
- An in-process engine (mistral.rs is a spike, later).
- MCP (inferd's MCP host is 31 v2).

---

## 2. Repos, crates, dependency direction

### 2.1 Placement decision

All model-layer crates go in **a new portable repo** (working name **`stoker`**: it feeds the
engines; it sits beside porter, latchkey, sill and detent; the user names it, Q1).
`MIT OR Apache-2.0`, same CONVENTIONS text, its own ARCHITECTURE.md and FINDINGS.md.

Why a new repo rather than inside porter:
- These crates are useful outside our desktop and outside porter's account model.
- They must not depend on `porter-core`.
- porter's 31 §8.12 already foresees splitting AI out of porter.

inferd stays in porter (31 §4.1). It is the one place where porter types and stoker types meet:
the bridge is `inferd/src/bridge.rs`, the only mapping. One exception: `porter-infer` takes a
dependency on `cua-action` (pure, serde only), so that `CuaAction` has one home and is not
copied into the wire.

**Conflict resolved: two request models?** porter-infer's `ChatRequest` is the app-facing broker
wire (tier, class, usage; no model id). model-provider's `TurnRequest` is the engine-facing
request (a concrete model, sampling, grammar, native tools, prepared images). They are two
concepts at two layers, like `Capability::Mail` vs mailo's `AccountCaps`. There is one mapping
home (`inferd::bridge`), and that keeps the frozen porter wire independent of stoker churn.
Alternative: porter-infer depends on model-provider's message types. Rejected: it couples the
frozen app wire to the backend crate.

### 2.2 Crates in `stoker` (layers bottom-up; a lower crate never names a higher one)

| Crate | Content | I/O | Depends on (ours) |
|---|---|---|---|
| `cua-action` | geometry (`Point<S>`, `Size<S>`, `Rect<S>`, spaces), `CuaAction<S>`, keys, dialect names (`CuaDialect`) | none | none |
| `vision-prep` | `ResizeRule`, `fit`, `image_tokens`, `FrameMap`, `RawFrame`; feature `pixels`: `prepare` (resize and encode) | none (`pixels` is CPU only) | cua-action |
| `model-provider` | `TurnRequest`, `Message`, `Part`, `ToolSpec`, `TurnEvent`, `TurnSink`, `Caps`, `Provider` trait, `ProviderError`; feature `testing`: `ScriptedProvider` | none | cua-action, vision-prep |
| `cua-parse` | `parse_text`, `parse_tool_calls` for `TextDialect` and `ToolDialect` | none | cua-action, model-provider |
| `cua-wire` | the `WireCodec` trait and one module per `WireDialect` (tool specs, decode, result encode) | none | cua-action, model-provider, vision-prep |
| `cua-session` | `CuaSession` (pure: history window, prompt templates, parse and repair, map to window space) | none | all of the above |
| `model-replay` | cassette format, `ReplayProvider`, `RecordingProvider<P>` over a `CassetteSink` | file write through an injected sink | model-provider |
| `model-catalog` | `ModelEntry`, `EngineProfile`, parser of `catalog/*.toml`, the shipped files | none | model-provider, vision-prep, cua-action |
| `engine-supervisor` | pure `step` machine and `budget`; seams `EngineHost`, `ReadyProbe`, `GpuProbe`; features `systemd` (zbus transient units), `process` (tokio::process), `nvidia` (nvidia-smi runner) | behind features | model-catalog |
| `model-http` | `HttpTarget` (Tcp, Unix, Tls), a hyper client, the pure `SseDecoder`, `AuthHeader` (redacted Debug) | yes | none |
| `model-openai-compat` | pure `codec` (request JSON, SSE chunk to `TurnEvent`) plus `OpenAiCompat: Provider` | yes, through model-http | model-provider, model-http |
| `model-anthropic`, `model-openai-responses`, `model-gemini` | cloud backends (**later**) | yes | same |

Boundary script (stoker `scripts/check-boundary.sh`, porter's form):
- The pure set is cua-action, vision-prep (default features), model-provider, cua-parse, cua-wire, cua-session, model-catalog, model-replay, and engine-supervisor (default features).
- It never reaches `tokio hyper hyper-util rustls zbus reqwest`.
- vision-prep reaches `fast_image_resize` and `image` only with the `pixels` feature.

### 2.3 Changes in `porter` (extend, don't contradict)

| Crate | Change |
|---|---|
| porter-core | `Capability::ComputerUse(CuaCap)`, `CapabilityKind::ComputerUse`, `Need::ComputerUse(CuaNeed)`, its `matching::fit` arm; `VocabVersion(2)` |
| porter-provider | `Discovery::Supervised` (inferd-run engines; models from the catalog at `Provenance::Curated`); shipped file `providers/local.toml` |
| porter-infer | new modules `event` (stream), `cua` (step wire), `choice` (picker), `readiness`; marked changes to `request`, `reply`, `error`, `model`, `audit`; new edge to `cua-action` |
| porter-dbus | additive `Inference1` members (§3.8), XML regenerated |
| porter-client | `Transport::open` and `InferSession` replace `Transport::infer`; `Accounts::infer` becomes a helper over a session |
| porter-fake | `FakeModel` streams; `FakeInferSession` |
| inferd | modules `bridge`, `session`, `engines`, `cua_run`, `catalog`; `AdapterModel::Local(...)` |

Allowed new edges: `porter-infer -> cua-action`; `inferd -> cua-action, vision-prep, model-provider, cua-session, model-catalog, engine-supervisor, model-http, model-openai-compat`.

### 2.4 Dependencies: what joins quire's pinned block first (CONVENTIONS §10)

| Dep | Version | For | Note |
|---|---|---|---|
| hyper | 1 [check latest] | model-http | one HTTP stack for local Unix sockets and cloud TLS; area "security"'s egress proxy uses hyper-util too |
| hyper-util | 0.1.21 (research-infra, 2026-09-24) | model-http | client legacy pool, tokio rt |
| http-body-util | 0.1 [unverified] | model-http | streaming bodies |
| hyper-rustls + rustls | 0.27 / 0.23 [unverified] | model-http (cloud, later) | no OpenSSL |
| base64 | 0.22 [unverified] | porter-infer inline images, backends | quire ds-core has a codec, but porter and stoker may not depend on ds-core |
| fast_image_resize | 6.1.0 (2026-07-21) | vision-prep `pixels` | SIMD resize |
| blake3 | 1 [unverified] | model-replay image digests (31 §5.3 Photos already plans BLAKE3) | |
| proptest | 1 [unverified], dev only | cua-parse, vision-prep properties | cargo-fuzz targets live in a non-workspace `fuzz/` crate on nightly (dev script, Q9) |
| keyboard-types | 0.7 (pinned) **+ `serde` feature** | cua-action `Chord` | pinned line changes |
| tokio | 1 (pinned) **+ `net`, `io-util`, `process`** | model-http, engine-supervisor `process`, inferd | pinned line changes; daemons and io crates only |
| image | 0.25.6 (pinned) | vision-prep `pixels` encode | unchanged |
| zbus, toml, serde, serde_json, thiserror | pinned | | unchanged |

**Not adopted, and conflicts with the Build map:**
- **async-openai** (listed under "Adopt" in COMPANION). It is built on reqwest, and reqwest cannot speak over a Unix socket. Confined engines need a Unix socket (§3.9). The chat-completions wire is small, so we own a serde codec.
- genai and rig: our trait is smaller and ours.
- llguidance and tokenizers are not needed while constraints are server flags. Later, with an in-process engine.

---

## 3. Frozen interfaces

Every type follows the rules: no `bool` anywhere; data is `Eq`, so no floats are stored; newtypes for every unit; enums with data use `#[serde(tag="kind", content="v", rename_all="snake_case")]`; the derive order is the standard one; each wire or stored type gets a round-trip test. Doc comments are left out here for space.

### 3.1 `cua-action` (freeze now)

```rust
// geometry.rs: coordinate spaces as uninhabited markers; mixing spaces does not compile.
pub trait Space: Copy + Eq + core::fmt::Debug + 'static {}
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum WindowSpace {} // logical px of the leased window's content, origin top-left
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum ImageSpace {}  // pixels of the image the model was shown
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum GridSpace {}   // a normalised grid 0..=max over that image
impl Space for WindowSpace {} impl Space for ImageSpace {} impl Space for GridSpace {}

pub struct Coord(pub u32);                                 // serde transparent
pub struct Point<S: Space> { pub x: Coord, pub y: Coord, #[serde(skip)] space: PhantomData<S> }
pub struct Size<S: Space>  { pub w: Coord, pub h: Coord, #[serde(skip)] space: PhantomData<S> }
pub struct Rect<S: Space>  { pub origin: Point<S>, pub size: Size<S> }
pub struct Length<S: Space>(pub Coord, PhantomData<S>);
pub struct Scale120(pub u32);                               // device px per logical px, in 120ths (wp-fractional-scale)
pub struct GridMax(pub u16);                               // 999 (Gemini, cua-integration) or 1000 (Qwen3-VL family)

// action.rs
pub enum CuaAction<S: Space> {
    Click  { at: Point<S>, button: Button, count: ClickCount, mods: BTreeSet<Modifier> },
    MoveTo { at: Point<S> },
    Drag   { from: Point<S>, to: Point<S>, button: Button },
    Type   { text: TypedText },
    Key    { chord: Chord, repeat: Repeat },
    Scroll { at: Point<S>, dir: ScrollDir, by: ScrollBy<S> },
    Wait   { for_ms: WaitMs },
    Zoom   { region: Rect<S> },                            // observation request, no side effect
    Observe,                                               // "take a screenshot": next observation, no side effect
    Finish { outcome: FinishOutcome, summary: Summary },
    Ask    { question: Summary, choices: Vec<Choice> },
}
pub enum Button { Left, Right, Middle }
pub enum ClickCount { One, Two, Three }
pub enum Modifier { Ctrl, Alt, Shift, Super }
pub struct Chord { pub mods: BTreeSet<Modifier>, pub key: keyboard_types::Key }  // parse refuses Unidentified, Dead
pub struct Repeat(u8);                                     // 1..=20, checked by Repeat::new
pub enum ScrollDir { Up, Down, Left, Right }
pub enum ScrollBy<S: Space> { Notches(Notches), Distance(Length<S>) }
pub struct Notches(pub u16);
pub struct WaitMs(u32);                                    // <= 60_000 at parse; policy clamps further
pub struct TypedText(String);                              // <= 2048 chars; no control chars except \n \t
pub struct Summary(String);                                // <= 1024 chars
pub struct Choice(String);                                 // <= 128 chars, <= 8 choices
pub enum FinishOutcome { Done, Failed, Infeasible }

pub enum ActionClass { Observe, Pointer, Keyboard, Conclude } // the router's input; one match, here
impl<S: Space> CuaAction<S> {
    pub fn class(&self) -> ActionClass;
    pub fn map_points<T: Space, E>(self, f: impl Fn(Point<S>) -> Result<Point<T>, E>,
                                   g: impl Fn(Length<S>) -> Result<Length<T>, E>) -> Result<CuaAction<T>, E>;
}

// dialect.rs: names only, so that Caps and the catalog can refer to them without a cycle.
pub enum WireDialect { AnthropicToolset20260801, AnthropicComputer20251124, OpenAiComputer, GeminiComputerUse }
pub enum TextDialect { UiTars15 }                          // "Thought: .. Action: click(start_box='(x,y)')"
pub enum ToolDialect { QwenComputerUse }                   // server-parsed tool calls, "computer_use" with an action arg
pub enum CuaDialect { Wire(WireDialect), Text(TextDialect), Tool(ToolDialect) }
pub enum ModelSpace { Image, Grid(GridMax) }               // where a dialect's points live
```

Conflicts resolved:
- **(a) Floats.** cua-integration has `LogicalPoint { x: f32 }` and `Scroll.amount: f32`. CONVENTIONS §12 says data is `Eq`. So coordinates are `Coord(u32)` logical px; sub-pixel clicks are meaningless. Scroll is notches or a mapped length.
- **(b) Which pixels.** research-cua-models §5 says "physical screen pixels". cua-integration says window logical px. Window logical px wins: the compositor captures per window, and HiDPI is handled by `Scale120`.
- **(c) The grid size.** cua-integration says 0-999; the research says 0-1000. So `GridMax` is a parameter of the dialect profile.
- **(d) `TargetedAction { window }`** is cuad's (area "cua"): models never sees a `WindowRef`.
- **(e) Extras.** `Open{app}`, `Navigate{url}`, `Memo`, mouse down/up and hold_key are not variants. Parsers drop them as `DropReason::UnsupportedVerb`, so the model can never act through a verb the schema lacks.
- **Later (needs a user decision, Q4):** `Target::Node(NodeId)` as a second form of `at`.

### 3.2 `vision-prep` (freeze now; `prepare` body in W1)

```rust
pub enum ResizeRule {
    SmartResize { factor: PatchFactor, min_pixels: PixelCount, max_pixels: PixelCount }, // Qwen-VL family: round to factor, clamp area, keep aspect
    LongEdge { max_edge: Coord, max_pixels: PixelCount },  // Anthropic-style caps
    Identity,
}
pub struct PatchFactor(pub u32);       // 28 for Qwen2.5-VL / UI-TARS-1.5; 32 for Qwen3-VL/3.5 [unverified: read preprocessor_config.json]
pub struct PixelCount(pub u64);
pub struct DeviceSize { pub w: u32, pub h: u32 }           // buffer pixels = logical * scale / 120

pub fn fit(src: DeviceSize, rule: &ResizeRule) -> Result<Size<ImageSpace>, FitError>;   // mirrors qwen smart_resize exactly
pub fn image_tokens(rule: &ResizeRule, image: Size<ImageSpace>) -> ImageTokens;       // (h/f)*(w/f)/merge for SmartResize
pub struct ImageTokens(pub u32);

pub struct FrameMap { pub window: Size<WindowSpace>, pub image: Size<ImageSpace>, pub space: ModelSpace }
impl FrameMap {
    pub fn new(window: Size<WindowSpace>, scale: Scale120, rule: &ResizeRule, space: ModelSpace) -> Result<FrameMap, FitError>;
    pub fn image_to_window(&self, p: Point<ImageSpace>) -> Result<Point<WindowSpace>, MapError>;
    pub fn grid_to_window(&self, p: Point<GridSpace>) -> Result<Point<WindowSpace>, MapError>;
    pub fn window_to_image(&self, p: Point<WindowSpace>) -> Point<ImageSpace>;   // for "cursor at" hints and replay
    pub fn length_to_window<S: Space>(&self, l: Length<S>) -> Result<Length<WindowSpace>, MapError>;
}
pub enum MapError { OutOfFrame { x: Coord, y: Coord }, GridMismatch }   // outside the image: refuse, never clamp
pub enum FitError { ZeroArea, AspectTooExtreme, BelowMinimum }

pub struct RawFrame<'a> { pub pixels: &'a [u8], pub size: DeviceSize, pub stride: u32, pub format: PixelFormat }
pub enum PixelFormat { Xrgb8888, Argb8888, Xbgr8888, Abgr8888 }   // wl_shm formats cuad receives
pub enum Encoding { Png, Jpeg(JpegQuality) }
pub struct JpegQuality(u8);
#[cfg(feature = "pixels")]
pub fn prepare(raw: RawFrame<'_>, map: &FrameMap, enc: Encoding) -> Result<PreparedImage, PrepError>; // resize once, encode once
pub struct PreparedImage { pub media: MediaType, pub bytes: Vec<u8>, pub size: Size<ImageSpace> }
pub enum MediaType { Png, Jpeg }
```

Rounding rule (frozen): `x_window = round_half_up(x_image * window.w / image.w)`, done in u64. The result must be `< window.w`; if not, `OutOfFrame`. The grid rule is the same, with `max` as the divisor.

### 3.3 `model-provider` (freeze now)

```rust
pub struct ModelName(String);          // the engine's served name or the vendor id
pub struct CallId(String);  pub struct ToolName(String);  pub struct JsonText(String);  // JsonText: checked JSON at the boundary
pub struct SchemaText(JsonText);       // a JSON Schema

pub struct TurnRequest {
    pub model: ModelName,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolSpec>,
    pub tool_choice: ToolChoice,
    pub output: OutputShape,
    pub limits: Limits,
    pub reasoning: Reasoning,
}
pub struct Message { pub role: Role, pub parts: Vec<Part> }
pub enum Role { System, User, Assistant, Tool }
pub enum Part {
    Text(String),
    Image(ImageInput),
    Thought(String),                                       // reasoning content handed back where the wire wants it
    ToolCall(ToolCall),
    ToolResult(ToolResult),
}
pub struct ImageInput { pub media: MediaType, pub bytes: ImageBytes, pub detail: ImageDetail }
pub struct ImageBytes(Vec<u8>);                            // serde as base64 text
pub enum ImageDetail { Auto, Original }
pub struct ToolCall { pub id: CallId, pub name: ToolName, pub input: JsonText }
pub struct ToolResult { pub id: CallId, pub status: ToolStatus, pub parts: Vec<Part> }
pub enum ToolStatus { Ok, Error }
pub enum ToolSpec {
    Function { name: ToolName, description: String, parameters: SchemaText },
    Native(NativeTool),                                    // provider-built tools such as computer_toolset_20260801
}
pub struct NativeTool { pub dialect: WireDialect, pub config: JsonText }
pub enum ToolChoice { Auto, Never, Required, Named(ToolName) }
pub enum OutputShape { Free, JsonSchema(SchemaText), Regex(String), Lark(String) }  // regex/lark: local engines only
pub struct Limits { pub max_output: Tokens, pub temperature: Milli, pub stop: Vec<String> }
pub struct Milli(pub u16);   pub struct Tokens(pub u32);
pub enum Reasoning { Off, On(Effort) }   pub enum Effort { Low, Medium, High }

// Streaming: push events into a sink, as sill's ResultSink and cua-integration's StepSink do.
pub enum TurnEvent {
    TextDelta(String),
    ThoughtDelta(String),
    ToolCallStarted { index: CallIndex, id: CallId, name: ToolName },
    ToolCallDelta   { index: CallIndex, fragment: String },
    ToolCallDone(ToolCall),                                 // input checked as JSON
    Safety(SafetySignal),                                   // vendor confirmation hints; may only ADD asks
    Usage(Usage),
}
pub struct CallIndex(pub u16);
pub enum SafetySignal { RequireConfirmation(String), Blocked(String) }
pub struct Usage { pub input: Tokens, pub output: Tokens, pub images: ImageCount }
pub struct ImageCount(pub u16);
pub struct TurnEnd { pub stop: StopReason, pub usage: Usage, pub served: ModelName }
pub enum StopReason { EndTurn, ToolUse, MaxTokens, StopSequence, ContentFilter }
pub enum Flow { Continue, Stop }

pub trait TurnSink: Send { fn event(&mut self, event: TurnEvent) -> Flow; }

/// One endpoint (a running engine or a cloud account). Cancellation is drop: the future is
/// cancel-safe and dropping it closes the connection, which aborts generation in vLLM and
/// llama-server. Flow::Stop ends the turn early with StopReason::EndTurn.
pub trait Provider: Send + Sync {
    fn describe(&self) -> impl Future<Output = Result<Vec<ModelInfo>, ProviderError>> + Send;   // /v1/models, /props
    fn turn<K: TurnSink>(&self, request: &TurnRequest, sink: &mut K)
        -> impl Future<Output = Result<TurnEnd, ProviderError>> + Send;
}
pub struct ModelInfo { pub name: ModelName, pub context: Tokens }

pub enum ProviderError {   // thiserror; each arm is something a caller acts on
    Unreachable, NotReady, Timeout, RateLimited(RetrySeconds), Unauthorized,
    ContextOverflow { limit: Tokens }, BadRequest(String), Refused(String), Unreadable(String),
}

pub struct Caps {                                          // one concrete model behind one backend
    pub inputs: BTreeSet<InputKind>,                       // Text, Image, Audio
    pub tools: ToolSupport,                                // Absent | Native | ServerParsed
    pub output: BTreeSet<Constraint>,                      // JsonSchema, Regex, Lark
    pub reasoning: Support,                                // Absent | Present
    pub streaming: Support,
    pub images: ImageLimits,                               // { per_prompt: ImageCount, rule: ResizeRule, space: ModelSpace }
    pub context: Tokens, pub max_output: Tokens,
    pub computer_use: CuaSupport,                          // Absent | Dialect(CuaDialect, Batching, Zoom)
}
pub enum Batching { One, Many }   pub enum Zoom { Absent, Native }   pub enum Support { Absent, Present }
```

`ScriptedProvider` (feature `testing`) plays a `Vec<Script>` of events plus an end, and records every `TurnRequest` it was given, so tests can assert on prompt assembly.

**Conflict resolved.** cua-integration writes `Box<dyn CuaSession>`, and research-agents writes `async fn chat -> Stream`. porter CONVENTIONS rule 2 says async seams are `-> impl Future + Send`, and closed sets of implementations are enums, never `dyn`. That rule wins. inferd has `enum Backend { OpenAiCompat(..), Replay(..), ... }` implementing `Provider`. The push sink also avoids a `futures` dependency.

### 3.4 `cua-parse`, `cua-wire`, `cua-session`

```rust
// cua-parse (freeze now; UiTars15 and QwenComputerUse filled in W1)
pub enum InSpace { Image(Vec<CuaAction<ImageSpace>>), Grid(GridMax, Vec<CuaAction<GridSpace>>) }
pub struct Parsed { pub thought: Option<String>, pub actions: InSpace, pub dropped: Vec<Dropped> }
pub struct Dropped { pub verb: VerbText, pub reason: DropReason }   // VerbText: <= 64 chars
pub enum DropReason { UnsupportedVerb, MissingArgument, BadNumber, TooLong, OverBatchLimit }
pub enum ParseError { NoAction, Unterminated, Malformed { at: ByteOffset }, TooLarge }
pub fn parse_text(dialect: TextDialect, space: ModelSpace, text: &str, limits: ParseLimits) -> Result<Parsed, ParseError>;
pub fn parse_tool_calls(dialect: ToolDialect, space: ModelSpace, calls: &[ToolCall], limits: ParseLimits) -> Result<Parsed, ParseError>;
pub struct ParseLimits { pub max_input: ByteLen /* 32 KiB */, pub max_actions: ActionCount /* 8 */ }
// Rules: no eval, no panics on any input, verbs on an allow-list, numbers parsed as u32 within
// bounds. Unknown verbs are dropped and reported, never guessed.

// cua-wire (freeze trait + enum now; bodies later with the cloud backends)
pub trait WireCodec {
    fn tools(&self, image: Size<ImageSpace>) -> Vec<ToolSpec>;                     // NativeTool or Function per dialect
    fn decode(&self, calls: &[ToolCall], safety: &[SafetySignal]) -> Result<Parsed, WireError>;
    fn results(&self, done: &[StepResult], next: &ImageInput) -> Vec<Part>;          // tool_result / computer_call_output / function_response
}
pub enum StepResult { Done(CallId), Refused { id: CallId, why: String }, NotRun(CallId) }  // stop-at-first-failure text per vendor
pub fn codec(dialect: WireDialect) -> WireCodecs;   // closed enum implementing WireCodec

// cua-session (freeze now, fill W2)
pub struct CuaProfile { pub dialect: CuaDialect, pub rule: ResizeRule, pub space: ModelSpace,
                        pub history: FrameBudget /* last N images */, pub repair: RepairBudget /* 1 */, pub encoding: Encoding }
pub struct CuaTaskText { pub goal: String, pub hints: Vec<String> }   // typed data from the planner, not raw untrusted blobs
pub struct CuaSession { /* profile, goal, history: VecDeque<Turn>, repairs_left */ }
impl CuaSession {
    pub fn begin(profile: CuaProfile, task: CuaTaskText, model: ModelName) -> CuaSession;
    pub fn request(&self, obs: &ObservationIn, map: &FrameMap, image: ImageInput) -> TurnRequest; // pure prompt assembly
    pub fn absorb(self, reply: TurnTranscript, map: &FrameMap) -> (CuaSession, StepOutcome);       // parse, map, history
}
pub struct ObservationIn { pub step: StepIndex, pub cursor: Option<Point<WindowSpace>>, pub prev: Vec<StepResult>, pub masked: MaskedRegions }
pub struct TurnTranscript { pub text: String, pub thought: String, pub calls: Vec<ToolCall>, pub end: TurnEnd }
pub enum StepOutcome {
    Actions { thought: Option<String>, actions: Vec<CuaAction<WindowSpace>>, dropped: Vec<Dropped> },
    Repair(TurnRequest),               // parse failed and a repair is left: send this repair prompt
    Unparseable(ParseError),           // counts as a step; cuad decides
}
```

Prompt templates are data: `cua-session/prompts/{ui_tars_15,qwen_computer_use}.txt`, taken from Apache-2.0 model cards with attribution in the file header. Mapping happens inside the session: an out-of-frame point becomes a `Dropped` entry (reason `OutOfFrame` via `MapError`), never a clamped click.

### 3.5 `model-replay` (freeze now, fill W1)

Cassette format (stored, round-trip tested): JSON Lines, file `<name>.cassette.jsonl`.

```rust
pub struct CassetteHeader { pub vocab: CassetteVersion, pub backend: BackendLabel, pub model: ModelName, pub recorded: UnixSeconds }
pub struct Interaction { pub request: RequestPrint, pub events: Vec<TurnEvent>, pub end: Result<TurnEnd, ProviderError> }
pub struct RequestPrint(TurnRequest);   // image bytes replaced by ImageDigest (blake3) + size; no auth, no headers
pub enum ReplayMode { InOrder, ByRequest }
pub enum ReplayError { Exhausted, Mismatch { index: u32, want: Box<RequestPrint>, got: Box<RequestPrint> } }
pub struct ReplayProvider { /* cassette, mode, cursor */ }        // impl Provider
pub struct RecordingProvider<P: Provider, C: CassetteSink> { /* inner, sink */ }  // impl Provider
pub trait CassetteSink: Send + Sync { fn write(&self, line: &Interaction) -> Result<(), SinkError>; }
```

There are two levels of recording:
- **Turn cassettes**, the typed level above, for agent, planner and cuad tests.
- **Wire fixtures** (raw request JSON plus raw SSE bytes) for adapter codec tests.

By default nothing personal is recorded: images are stored as digests only, and recording is a dev-script action, never automatic.

### 3.6 `model-catalog` file format (freeze now)

`catalog/<id>.toml`, installed at `/usr/share/stoker/catalog/` and `$XDG_DATA_HOME/stoker/catalog/` (user files win). It follows porter's rule: every field is written, nothing defaults, and a missing field refuses the file.

```toml
id = "holo-3.1-4b"
label = "Holo 3.1 4B"
licence = { kind = "open", v = "Apache-2.0" }          # open(SPDX) | non_commercial(SPDX) | proprietary
source = { kind = "hugging_face", repo = "Hcompany/Holo-3.1-4B", revision = "<commit sha>" }
vram = { weights_mib = 10400, kv_per_1k_ctx_mib = 0, overhead_mib = 1500 }   # [est]; kv measured in W3
context = 32768
max_output = 4096
inputs = ["text", "image"]
tools = "server_parsed"
output = ["json_schema"]
reasoning = "present"
streaming = "present"
images = { per_prompt = 3, rule = { kind = "smart_resize", v = { factor = 32, min_pixels = 3136, max_pixels = 1003520 } }, space = { kind = "grid", v = 1000 } }  # [unverified: chat template + preprocessor_config]
computer_use = { kind = "dialect", v = { dialect = { kind = "tool", v = "qwen_computer_use" }, batching = "one", zoom = "absent" } }
roles = ["llm", "computer_use"]                        # which porter kinds it is offered under

[[engine]]
kind = "vllm"
args = ["--max-model-len", "32768", "--limit-mm-per-prompt", "{\"image\":3,\"video\":0}",
        "--enable-auto-tool-choice", "--tool-call-parser", "qwen3_coder", "--reasoning-parser", "qwen3",
        "--gpu-memory-utilization", "0.80"]
weights = { kind = "hf_snapshot" }

[[engine]]
kind = "llama_server"
args = ["--jinja", "--ctx-size", "32768", "--special"]  # --special keeps grounding control tokens
weights = { kind = "gguf", model = "<file>", mmproj = "<file>" }   # only if a GGUF + mmproj exists [unverified for 4B]
```

```rust
pub struct ModelEntry { pub id: CatalogId, pub label: String, pub licence: Licence, pub source: WeightSource,
    pub vram: VramEstimate, pub caps: Caps, pub roles: BTreeSet<Role>, pub engines: Vec<EngineProfile> }
pub struct EngineProfile { pub kind: EngineKind, pub args: Vec<EngineArg>, pub weights: WeightFiles }
pub enum EngineKind { LlamaServer, Vllm }               // MistralRs after the spike (later)
pub fn parse_entry(text: &str) -> Result<ModelEntry, CatalogError>;
pub fn command(entry: &ModelEntry, profile: &EngineProfile, paths: &EnginePaths, socket: &SocketPath) -> UnitSpec;  // pure
```

`EnginePaths` (program locations, HF cache dir) come from settings, never from ambient lookups. Two settings matter: `ai.engine.vllm.python` (the user's `~/vllm` uv environment) and `ai.engine.llama_server.path`.

### 3.7 `engine-supervisor` (freeze now; the pure part in W1, io in W2)

```rust
pub struct EngineId(String);                           // "<engine kind>:<catalog id>"
pub struct UnitSpec { pub program: ProgramPath, pub args: Vec<EngineArg>, pub env: Vec<EnvPair>, pub sandbox: Sandbox }
pub struct Sandbox { pub network: Network, pub read: Vec<PathBuf>, pub write: Vec<PathBuf>, pub gpu: GpuAccess, pub memory_max: MiB }
pub enum Network { None }                              // engines listen on a Unix socket only; no other variant until needed
pub enum GpuAccess { Nvidia, Absent }
pub struct MiB(pub u32);   pub struct MonoMs(pub u64);   pub struct Attempt(pub u8);

pub enum EngineState {
    Stopped,
    Starting { since: MonoMs, attempt: Attempt },
    Ready    { since: MonoMs, last_used: MonoMs },
    Stopping { since: MonoMs },
    Backoff  { until: MonoMs, attempt: Attempt },
    Failed(EngineFailure),
}
pub enum EngineFailure { NoRoom { need: MiB, free: MiB }, Exited { code: ExitCode }, NeverReady, BadProfile }
pub enum SupervisorIn { Want(EngineId), Used(EngineId), Exited { id: EngineId, code: ExitCode },
                        Probed { id: EngineId, probe: Probe }, Gpu(GpuMemory), Tick(MonoMs) }
pub enum Probe { Ready, Loading, Down }
pub enum SupervisorOut { Spawn(EngineId, UnitSpec), Stop(EngineId), Probe(EngineId), Changed(EngineId, EngineState), WakeAt(MonoMs) }
pub struct Supervisor { /* engines: Vec<(EngineSpec, EngineState)>, gpu: GpuMemory */ }
pub fn step(s: Supervisor, input: SupervisorIn, cfg: &SupervisorConfig) -> (Supervisor, Vec<SupervisorOut>);
pub fn budget(want: &EngineSpec, running: &[(EngineId, MiB, MonoMs)], gpu: GpuMemory, headroom: MiB) -> BudgetVerdict;
pub enum BudgetVerdict { Fits, EvictFirst(Vec<EngineId>), NoRoom { need: MiB, free: MiB } }
pub struct GpuMemory { pub total: MiB, pub used_by_others: MiB }   // others = ComfyUI, games: observed, never killed
pub struct SupervisorConfig { pub idle_unload: Duration, pub start_timeout: Duration, pub probe_every: Duration,
                              pub max_attempts: Attempt, pub backoff: (Duration, Duration), pub headroom: MiB }

pub trait EngineHost: Send + Sync {        // SystemdUnits (zbus StartTransientUnit), ChildProcesses (tokio), FakeEngineHost
    fn spawn(&self, id: &EngineId, unit: &UnitSpec) -> impl Future<Output = Result<(), HostError>> + Send;
    fn stop(&self, id: &EngineId) -> impl Future<Output = Result<(), HostError>> + Send;
    fn exited(&self, id: &EngineId) -> impl Future<Output = ExitCode> + Send;
}
pub trait ReadyProbe: Send + Sync { fn probe(&self, id: &EngineId) -> impl Future<Output = Probe> + Send; }   // GET /health over the socket
pub trait GpuProbe: Send + Sync { fn memory(&self) -> impl Future<Output = Result<GpuMemory, GpuError>> + Send; }  // nvidia-smi csv via an injected runner
```

The proposed values are settings, with design/22 rows needed (area "settings"):

| Key | Proposed |
|---|---|
| `ai.engine.idle_unload_s` | 600 |
| `ai.engine.start_timeout_s` | 180 (vLLM cold start) |
| `ai.engine.probe_ms` | 500 |
| `ai.engine.max_attempts` | 3 |
| `ai.engine.backoff_ms` | 2000 to 30000 |
| `ai.engine.vram_headroom_mib` | 1024 |

### 3.8 porter extensions (the frozen form; each is an edit to design/31 in step)

**porter-core: vocabulary bump to `VocabVersion(2)`** (31 §2.3 gains a row).

```rust
pub struct CuaCap { pub environments: BTreeSet<CuaEnv>, pub batching: CuaBatching, pub zoom: Offered,
                    pub max_image: Px, pub wire: LlmWire }
pub enum CuaEnv { Desktop, Browser, Mobile }
pub enum CuaBatching { One, Many }
pub struct CuaNeed { pub environments: BTreeSet<CuaEnv> }   // matches: subset; protocol fields not asked (porter rule)
// Capability::ComputerUse(CuaCap), CapabilityKind::ComputerUse, Need::ComputerUse(CuaNeed)
```

This is how cua-integration's "`Need` gains the `cua` kind" lands in porter's versioned vocabulary. The dialect detail stays in stoker's `Caps`; its summary becomes a `Claim` at `Provenance::Curated` (from the catalog) or `Discovered`.

**porter-provider:** `Discovery::Supervised`. inferd runs these engines. The account is present while the engine is stopped (readiness `Loadable`), unlike probed runtimes, which go `Offline`. File `providers/local.toml`: `auth = local_runtime`, `[ai] locality = on_device, billing = free`, with capability rows written from the catalog at run time.

**porter-infer: `request.rs` changes (changes a frozen type).**

```rust
pub struct ChatRequest { pub messages, pub shape, pub tier, pub class, pub usage,
                         pub tools: Vec<ToolDecl> }                    // + tools (31 §5.5 "tool calls join the request")
pub struct ToolDecl { pub name: ToolName, pub description: String, pub params: JsonSchemaText }   // from intents' ActionDecl
pub enum MessagePart { Text(String), Image(ImagePart), ToolCall(ToolCallPart), ToolResult(ToolResultPart) }   // + two
pub struct ToolCallPart { pub id: CallId, pub name: ToolName, pub args: JsonText }
pub struct ToolResultPart { pub id: CallId, pub status: ToolStatus, pub parts: Vec<MessagePart> }
pub struct ImagePart { pub media_type: String, pub source: ImageSource }   // bytes -> source (changes a frozen field)
pub enum ImageSource { Inline(Base64Bytes), Attached(AttachIndex) }        // Attached: a memfd passed with SCM_RIGHTS on the Open fd
pub enum InferRequest { Chat(ChatRequest), Embed(EmbedRequest), Task(TaskRequest),
                        CuaBegin(CuaBegin), CuaStep(CuaStepRequest) }      // + two
```

Why `bytes` becomes `source`: today `Vec<u8>` is serialized in JSON as an array of numbers, about 3.5 times the size. cua-integration requires frames to travel "never base64 over the bus", and memfd attachments meet that.

**porter-infer: `cua.rs` (new, freeze now).** This is the contract between cuad and inferd. Every point in it is in `WindowSpace`.

```rust
pub struct CuaBegin { pub goal: String, pub hints: Vec<String>, pub env: CuaEnv }
pub struct CuaStepRequest { pub step: StepIndex, pub window: WindowGeometry, pub frame: FrameImage,
                            pub cursor: Option<Point<WindowSpace>>, pub prev: Vec<PrevResult>, pub masked: MaskedRegions }
pub struct WindowGeometry { pub logical: Size<WindowSpace>, pub scale: Scale120 }
pub struct FrameImage { pub source: ImageSource, pub layout: FrameLayout }
pub enum FrameLayout { Raw { format: PixelFormat, width: u32, height: u32, stride: u32 }, Encoded(MediaKind) }  // Raw preferred: no double encode
pub enum PrevResult { Done, Refused(String), NotRun, Failed(String) }
pub struct MaskedRegions(pub u16);
pub struct CuaStepReply { pub thought: Option<String>, pub actions: Vec<CuaAction<WindowSpace>>,
                          pub dropped: Vec<DroppedAction>, pub safety: Vec<SafetyHint> }   // safety only ever adds asks
pub enum CuaStepFailure { Unparseable, ModelFailed(ModelError) }
```

**porter-infer: `event.rs` (new; closes FINDINGS "Inference is one-shot").**

```rust
pub enum ClientFrame { Request(InferRequest), Cancel }               // client -> inferd on the Open fd
pub enum InferEvent {                                                 // inferd -> client
    Routed(ServedBy),                // the "sent to <provider>" indicator and the orb read it
    Waiting(Readiness),              // the engine is loading: presence "working", never a spinner in the app
    TextDelta(String), ThoughtDelta(String),
    ToolCall(ToolCallPart),
    ActionProposed(CuaAction<WindowSpace>),   // as parsed, for the acting-here glow preview
    Usage(TokenUsage),
    Finished(InferReply),            // exactly one per request; ends the turn
}
pub enum InferReply { Chat(ChatReply), Embed(EmbedReply), CuaStep(CuaStepReply), Refused(InferRefusal),
                      Failed(ModelError), Cancelled }                 // + CuaStep, Failed, Cancelled
pub enum InferRefusal { RequiresCloud(DataClass), Unavailable, NeedsGrant, Denied, OverBudget,
                        Unsupported }                                 // + Unsupported: request kind does not fit the session's need, or a CUA need with class != Screen
pub enum ModelError { Unreachable, RateLimited(u32), Unauthorized, Refused, Unreadable,
                      NotReady, ContextOverflow, Unparseable }        // + serde, + three variants
pub struct ChatReply { pub text: String, pub tool_calls: Vec<ToolCallPart>, pub usage: TokenUsage, pub served: ServedBy }  // + tool_calls
pub struct AuditEntry { /* frozen fields */ pub images: Count }      // + images: "a frame was sent", never the frame
```

**porter-infer: `model.rs` (change).** `Model::chat` is replaced, not added beside: `chat(&self, request: &ChatRequest, sink: &mut impl ChatSink) -> impl Future<Output = Result<ChatReply, ModelError>> + Send`. `ChatSink::event(&mut self, InferEvent) -> Flow`. `FakeModel` follows.

**porter-infer: `choice.rs` and `readiness.rs` (new, freeze now): the model picker's data model.**

```rust
pub struct ModelRef { pub account: AccountId, pub model: ModelId }
pub enum AiKind { Llm, ComputerUse, Embeddings, Speech, ImageGen, Rerank }   // CapabilityKind's AI subset
pub struct TierRow { pub kind: AiKind, pub tier: Tier, pub model: ModelRef }
pub struct TierMap { pub rows: Vec<TierRow> }       // settings ai.model.<kind>.<tier> = "<account>/<model>"
pub fn tier_choice(map: &TierMap, kind: AiKind, tier: Tier, model: &ModelRef) -> TierChoice;   // feeds route() unchanged
pub enum Readiness { Ready, Loading, Loadable, Downloading(Permille), Downloadable, Unavailable }   // 31 §3.3 + Loading, Loadable
pub enum Fit { Fits, NeedsEviction, NeedsOffload, TooLarge }
pub enum LicenceClass { Open, NonCommercial, Proprietary }
pub struct PickerRow { pub model: ModelRef, pub label: String, pub kind: AiKind, pub locality: Locality,
                       pub billing: Billing, pub readiness: Readiness, pub fit: Fit, pub licence: LicenceClass,
                       pub chosen_for: BTreeSet<Tier> }
pub fn picker_rows(kind: AiKind, cards: &[PickerInput], map: &TierMap) -> Vec<PickerRow>;
// Order: OnDevice first, then Ready before Loadable before Downloadable, then label.
// A NonCommercial row is listed but never auto-chosen.
```

**`org.quire.Inference1` (additive, freeze now).** Existing members and signatures are unchanged.

| Member | Signature | Notes |
|---|---|---|
| `Availability` | unchanged | `need` may now be `computer_use` |
| `Open` | unchanged | the fd is a Unix stream socket. Client writes `ClientFrame` frames (porter-core framing: 4-byte length plus a `{vocab, body}` JSON envelope, 16 MiB cap). inferd writes `InferEvent` frames. `ImageSource::Attached(i)` names the i-th memfd received with SCM_RIGHTS on that frame. The route is chosen once per Open, so a session is pinned to one model (CUA history needs that). |
| `Prepare(need: (sa{sv}), class: s, tier: s) -> s` | **new** | warms the engine the route would pick and returns a `Readiness` slug. Called on double-tap ⌘ so the first turn does not pay a cold start. Refusals answer with the refusal slug. |
| `Usage`, `Rescan` | unchanged | |
| signal `EnginesChanged()` | **new**, broadcast | engine state is not personal. Listeners re-read readiness through `Prepare` (callers) or the settings module (detent). |
| property `Gpu s` | **new** | `idle`, `busy`, `loading` (31 §3.3 "`GpuBusy`") |
| `org.quire.SettingsModule1` at `/org/quire/Inference1/settings` | 22 §9.4 shape | detent's Intelligence page: `Describe` gives `PickerRow`s per kind as the choices of `ai.model.<kind>.<tier>`; `Set` writes the `TierMap` |

**porter-client (change).** `Transport::open(&self, need: &Need, class: DataClass, tier: Tier) -> impl Future<Output = Result<Self::Session, TransportError>> + Send`, with `type Session: InferSession`. `InferSession` has `send(ClientFrame)` and `next() -> InferEvent`. `Transport::infer` is removed. `Accounts::infer` and a new `Accounts::session` sit on top of it.

### 3.9 Engine confinement (freeze the shape; verify in W2)

Engines run as transient systemd user units. inferd's `SystemdUnits` calls `StartTransientUnit` with:
- `PrivateNetwork=yes`, `IPAddressDeny=any`;
- `ProtectHome=read-only` plus `BindReadOnlyPaths` for the weights directory;
- `DeviceAllow=/dev/nvidia*`, `MemoryMax`;
- env `HF_HUB_OFFLINE=1`.

Each engine listens on `$XDG_RUNTIME_DIR/inferd/<engine>.sock`. [unverified: llama-server listening on a Unix socket when `--host` ends in `.sock`; vLLM `--uds`. Spike S1.] Fallback if one engine cannot: a network namespace that has loopback only, with inferd reaching the engine through a socket the supervisor forwards. Raised as FINDINGS if needed.

`ChildProcesses` (tokio) is the host for macOS and Windows, and for desktops without systemd (unconfined, and marked so).

---

## 4. State machines

### 4.1 Engine lifecycle (`engine-supervisor::step`; time is an input)

| State | Event | Next | Effects |
|---|---|---|---|
| Stopped | Want(id), budget Fits | Starting{now, 1} | Spawn, Probe, WakeAt(now+probe), Changed |
| Stopped | Want(id), EvictFirst(v) | Stopped (pending) | Stop each in v; on their Exited, re-evaluate Want |
| Stopped | Want(id), NoRoom | Failed(NoRoom) | Changed |
| Starting | Probed Ready | Ready{now, now} | Changed |
| Starting | Probed Loading or Down, before timeout | Starting | WakeAt(now+probe), Probe |
| Starting | Tick past start_timeout | Backoff or Failed(NeverReady) | Stop, Changed |
| Starting or Ready | Exited(code) | Backoff{now+b(a), a+1}, or Failed(Exited) when a = max | Changed |
| Ready | Used | Ready{last_used = now} | none |
| Ready | Tick, now − last_used ≥ idle_unload | Stopping{now} | Stop, Changed |
| Ready | Want(other), EvictFirst includes this | Stopping | Stop |
| Stopping | Exited | Stopped | Changed |
| Backoff | Tick ≥ until, still wanted | Starting{now, attempt} | Spawn, Probe |
| Failed | Want(id) | as Stopped, with attempt reset | |
| any | Gpu(mem) | same | stored; the budget uses it |

Budget rule (pure):
- `need = weights + kv(ctx) + overhead`, and `free = total − used_by_others − Σ running`.
- If `need + headroom ≤ free`, the verdict is Fits.
- Otherwise, evict idle engines, least recently used first, until it fits. If that is still not enough, NoRoom.
- Engines in an active turn are never evicted. Those are the ones inferd marks with `Used` within `probe_ms`.

### 4.2 Inference1 fd session (`inferd::session`, a pure `step`)

| State | Event | Next | Effects / frames |
|---|---|---|---|
| Opened{need, class, tier} | (on open) route → Err(r) | Closed | Finished(Refused(r)) |
| Opened | route → Ok(chosen), engine not Ready | Waiting{chosen} | Want(engine), Routed, Waiting(Loading) |
| Opened or Waiting | engine Ready | Idle{chosen} | Routed (if not yet sent) |
| Waiting | engine Failed | Closed | Finished(Failed(NotReady)) |
| Idle | Request(r), kind fits need | InTurn{r} | start the model future |
| Idle | Request(r), kind does not fit (or CUA class ≠ Screen, or CuaStep before CuaBegin) | Idle | Finished(Refused(Unsupported)) |
| InTurn | model event | InTurn | TextDelta, ThoughtDelta, ToolCall, ActionProposed, Usage |
| InTurn | model done | Idle | meter spend, audit, Finished(reply) |
| InTurn | Cancel | Idle | drop the future (closes the HTTP stream), Finished(Cancelled), partial usage still audited |
| any | EOF or Close | Closed | drop the future; Release(engine) |
| Waiting | Request | Waiting (queued, depth 1) | |

### 4.3 CUA step (`cua-session::absorb`, inside inferd's `cua_run`)

| State | Event | Next |
|---|---|---|
| Begun | CuaStep(obs) | Prepared (`FrameMap::new`, `prepare` raw→image, `request`) |
| Prepared | turn done, parse Ok | Replied: actions mapped to WindowSpace, history pushed (last N frames kept) |
| Prepared | parse Err, repairs_left > 0 | Repairing: one repair TurnRequest, without a new frame |
| Repairing | parse Ok | Replied |
| Repairing or Prepared | parse Err, no repairs left | Replied(Unparseable): counts as a step; cuad decides |
| any | Cancel | Begun (history unchanged; the step never happened) |

### 4.4 SSE turn decoder (`model-http::SseDecoder` + `model-openai-compat::codec`, pure)

`Streaming → (ToolCallAccumulating per index) → Ended(TurnEnd)`, or `Errored`.
- Bytes are fed in arbitrary chunks.
- `[DONE]` ends the stream, as does `finish_reason` followed by the usage chunk.
- Tool-call argument fragments are joined per index and checked as JSON at `ToolCallDone`.
- A malformed line is `Unreadable`, never a panic.

---

## 5. Tests that pin the shapes (and fakes)

Shape tests land in the freeze wave and must compile against the `todo!()` bodies: round trips, compile-fail checks, introspection. Behaviour tests (marked W1+) land with their bodies.

| Crate | Test | Asserts |
|---|---|---|
| cua-action | `actions_round_trip` | every variant of `CuaAction<WindowSpace>` survives JSON, adjacently tagged |
| | `mixing_spaces_does_not_compile` (compile_fail doctest) | `let p: Point<WindowSpace> = image_point;` fails to compile |
| | `text_limits_refuse` (table) | `TypedText` over 2048 chars or with a control char, and `Repeat(0)`/`Repeat(21)`, are refused |
| | `class_of_every_variant` (table) | `ActionClass` per variant |
| vision-prep | `fit_matches_reference_smart_resize` (W1) | `fixtures/smart_resize.csv` rows (generated by `dev/smart-resize-vectors.py` from qwen-vl-utils, Apache-2.0) for factors 28 and 32 and 20 window sizes including 2560x1600 and 1366x768 |
| | `map_round_trips_within_one_px` (proptest, W1) | window→image→window moves a point by at most 1 px |
| | `outside_image_is_refused` (table) | x == image.w gives OutOfFrame; no clamp |
| | `grid_999_and_1000_differ` | the same grid point maps differently under `GridMax(999)` and `GridMax(1000)` |
| | `image_tokens_table` | tokens for known sizes |
| model-provider | `requests_and_events_round_trip` | every `Part`, `ToolSpec`, `TurnEvent` |
| | `scripted_provider_replays_and_records` | sink order and the recorded request |
| | `stop_flow_ends_turn` | `Flow::Stop` gives `StopReason::EndTurn` with no further events |
| cua-parse | `ui_tars_examples_parse` (W1) | model-card examples in `fixtures/ui_tars_15/*.txt` give the expected actions |
| | `qwen_tool_calls_parse` (W1) | `fixtures/qwen_cu/*.json` (recorded from Holo-3.1-4B by `dev/record-engine.sh`) |
| | `unknown_verbs_are_dropped_not_guessed` | `open_app(...)` gives `Dropped{UnsupportedVerb}` |
| | `never_panics` (proptest over arbitrary strings, W1) and the `fuzz/` targets `parse_text`, `parse_tool_calls` (dev script) | total parsing |
| | `limits_hold` | 9 actions give OverBatchLimit; 33 KiB gives TooLarge |
| cua-wire | `dialects_round_trip` (freeze); `anthropic_batch_stops_at_first_failure`, `gemini_safety_only_adds` (later, doc fixtures) | |
| cua-session | `history_keeps_last_n_frames`, `one_repair_then_unparseable`, `out_of_frame_becomes_dropped` (W2, ScriptedProvider) | |
| model-replay | `cassette_round_trips`, `images_are_digests`, `mismatch_names_index` | |
| model-catalog | `shipped_files_parse`, `missing_field_refuses`, `user_file_replaces_system` | |
| | `command_is_pure` (table) | the expected `UnitSpec` args for Holo under vLLM |
| engine-supervisor | `transitions` (one table, every row of §4.1) | state and effects |
| | `budget_table` | Fits, EvictFirst order (LRU, never active), NoRoom with numbers |
| | `crash_backoff_caps_attempts` | |
| model-http | `sse_decoder_any_chunking` (proptest over split points) | the same events whatever the chunking |
| | `unix_socket_round_trip` (W2) | a loopback fake server on a tempdir socket |
| model-openai-compat | `codec_request_golden` | request JSON for a `TurnRequest` with an image and a tool |
| | `codec_stream_fixture` | `fixtures/openai_compat/*.sse` (recorded from vLLM and llama-server) |
| | `tool_call_fragments_join` | |
| porter-core | `computer_use_round_trips`; `matching` rows for `CuaNeed` | `VocabVersion::CURRENT == 2` |
| porter-infer | `infer_frames_round_trip` | every `ClientFrame`, `InferEvent`, `InferReply`, `CuaStepRequest` |
| | `picker_rows_order` (table) | |
| | `tier_choice_from_map` | |
| | `session_transitions` (table, §4.2) | |
| | `cua_needs_screen_class` | |
| porter-dbus | `introspection` (existing test) | catches `Prepare`, `EnginesChanged` and `Gpu` against the XML |
| porter-client | `session_end_to_end` (InProcess + FakeModel) | Routed → deltas → Finished; Cancel gives Cancelled |
| inferd | `bridge_maps_every_part` (table) | ChatRequest ↔ TurnRequest; ToolCallPart ↔ ToolCall |

Fakes and fixtures:
- stoker: `ScriptedProvider`, `ReplayProvider`, `FakeEngineHost` (scripted exits), `FakeReadyProbe`, `FakeGpu`, a loopback OpenAI-compat server (test-only, in `model-http/tests/support/`).
- porter: `FakeModel` (streaming), `FakeInferSession`.
- Recorded fixtures come only from dev scripts against the user's own engine. They are never generated in CI.
- Tests never start a real engine, never touch the GPU, never use the real systemd or bus (`FakeEngineHost`), and never read `~/.cache/huggingface`.

---

## 6. Work breakdown

**Wave 0: deps (one agent, shared with the other areas).** Edit `~/quire/docs/workspace-deps.toml`:
- add the crates in §2.4 and the keyboard-types `serde` and tokio `net, io-util, process` features;
- add `stoker` crates as git deps pinned by rev, the way latchkey is;
- add design/22 rows for every `ai.*` key named here.

Verify with quire's gate.

**Freeze wave (parallel; one agent per repo):**

**F1 stoker** (new repo `~/stoker`, owns every file in it).
- F1.0: read Holo-3.1-4B's `chat_template.jinja`, `preprocessor_config.json` and `config.json` read-only through WebFetch of the hub raw files. This fixes the `ToolDialect`, `GridMax` or ImagePx, and the factor. If Holo's schema is not Qwen's `computer_use`, add `ToolDialect::Holo31` before the freeze.
- Then add, in order: workspace, CONVENTIONS.md (verbatim), ARCHITECTURE.md (crate map, one-home table, traits, recipes "add a backend", "add a dialect", "add a catalog entry"), FINDINGS.md (every `todo!()`), deny.toml, rust-toolchain.toml (porter's pin), `scripts/check-boundary.sh`.
- Then every crate's types and traits per §3, bodies as `todo!()`, the shape tests of §5, and `catalog/holo-3.1-4b.toml`.
- Commit cua-action first, so that F2 can path-patch it.

**F2 porter** (owns porter files and **design/31-ACCOUNTS.md** in quire, edited in step).
- §2.3 and §3.8 changes; the vocabulary bump; `providers/local.toml`; regenerated `dbus/org.quire.Inference1.xml`.
- ARCHITECTURE edges and `check-boundary.sh` rows (porter-infer→cua-action; inferd→stoker crates); FINDINGS stubs (`Broker::infer` streaming, session server, Prepare).
- inferd modules as skeletons.
- Before cua-action lands in the pinned block, porter depends on it through a `[patch]` path to `../stoker` (quire CONSUMING.md §1 pattern).
- design/31 edits: §2.3 (ComputerUse row), §3.3 (Readiness, Supervised), §4.4 (Inference1 members), §5.5 (streaming, tools, CUA steps, picker TierMap), §11 table.

Verify each repo with:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./scripts/check-boundary.sh
cargo deny check licenses
```

In quire, after the design/31 edit: `./scripts/check-doc-paths.sh` (a tool, not a gate).

**Fill wave W1 (pure; stoker agents own one crate each, plus one porter agent):**
- W1a vision-prep plus `dev/smart-resize-vectors.py` (run in the `~/vllm` uv environment).
- W1b cua-parse (UiTars15, QwenComputerUse), proptest, `fuzz/`.
- W1c model-catalog plus engine-supervisor `step` and `budget`.
- W1d model-replay plus `ScriptedProvider`.
- W1e porter: `matching` arm, `picker_rows`, `tier_choice`, `session` step, `FakeModel` streaming.

**W2 (io):**
- W2a model-http plus model-openai-compat; `dev/record-engine.sh` records fixtures from the user's engine (a manual run).
- W2b engine-supervisor `systemd`, `process` and `nvidia` features; Spike S1 (Unix sockets and `PrivateNetwork` for both engines; a dev script on a scratch user unit).
- W2c cua-session.

**W3 (integration, porter):**
- inferd `bridge`, `engines` (supervisor host), fd session server with memfd attachments, `Prepare`, `Broker::infer` streaming, `cua_run`; porter-client `DbusTransport::open`.
- Acceptance dev script `dev/cua-step-holo.sh`: real Holo-3.1-4B on the 5070 Ti, a recorded window frame (fixture PNG of a GTK window), one `CuaStep`, and the window-space actions printed. Not a test: it touches the GPU.

**W4 (later):** cloud backends plus cua-wire bodies (when the user turns on cloud); the mistral.rs spike; the weight downloader (`Downloading`); `Target::Node`.

---

## 7. Dependencies on other areas, and questions for the user

**Other areas** (map these names to the brief's actual area names):

| Area | Needs from models | Models needs from it |
|---|---|---|
| "cua" (cuad, compositor) | `cua-action` types, `CuaStepRequest`/`CuaStepReply` over the Inference1 fd, `ActionClass`, `Dropped` | raw frames as memfd with `WindowGeometry`; the lease's window; `MaskedRegions`; the loop's budgets; acting on `ActionProposed` for the glow |
| "agent" (planner and reader) | `Accounts::session`, `ChatRequest.tools`, `InferEvent` streaming, `Prepare` | which tier and need per role. The reader is `Need::Llm{StructuredOutput}` with `ReplyShape::Json` and no tools. |
| "intents" | carries `ToolDecl { name, description, params }` | `ActionDecl` → JSON Schema text (its home) |
| "security" (policy, prov, egress) | `ActionClass`, `SafetyHint` (add-only), `ServedBy`/locality, audit `images` | the egress proxy for cloud backends (`model-http` takes a proxy `HttpTarget`); `Screen` floor stays OnDevice |
| "memory" (eventlog, recall) | `Embed` requests unchanged; audit entries to eventlog | none |
| "launcher" and "presence" | `InferEvent::{Routed, Waiting, ThoughtDelta, ActionProposed}` for orb states; `Prepare` on double-tap ⌘; `PickerRow` for detent | the Intelligence page draws `PickerRow`s |
| "settings" (design/22) | rows for `ai.model.<kind>.<tier>`, `ai.engine.*`, `ai.cua.history_frames` (3), `ai.cua.repair_attempts` (1) | |

**Questions only the user can answer:**
1. **Repo.** Is a new portable repo right, and is `stoker` the name, or should these crates live inside porter?
2. **Engine for Holo-3.1-4B.** The build map says llama.cpp is the default. H Company documents vLLM for the 4B, and a 4B GGUF with mmproj is unverified. Recommendation: vLLM BF16 (10.4 GB) for this model, with the default chosen per catalog entry.
3. **One model for both tiers.** A planner and Holo-3.1-4B do not both fit in 16 GB. Should Holo-3.1-4B also serve as the planner (it does native function calling), or should the planner swap with the CUA under the eviction rule (cold starts of 30 s or more on vLLM)? Recommendation: one resident model for both.
4. **Addressing elements.** Should the model address elements by tree node (`Target::Node`) when a tree exists, or always by coordinates? This is cua-integration Q4; it decides whether `Target` is frozen now.
5. **Engine confinement.** Is it acceptable that engines run only as confined systemd transient units with no network (and as unconfined child processes elsewhere)?
6. **Picker granularity.** Is the picker per kind × tier (`ai.model.computer_use.balanced`), or should it use named roles (Planner, Reader, Operator) that map onto tiers?
7. **Idle unload.** Is 600 s right? Should a model be kept warm while the companion is open?
8. **Non-commercial models.** Should non-commercial models (Holo4) appear in the picker at all, or be hidden?
9. **Fuzzing.** Is a nightly-only `cargo-fuzz` dev script acceptable beside the stable proptest suite?

**Conflicts found, with the resolution proposed:**
1. Float coordinates in cua-integration vs "data is Eq": integer `Coord`.
2. `dyn` sessions and providers vs porter's RPITIT and closed enums: porter's rule wins.
3. Physical vs logical pixels: window logical px.
4. Grid 0-999 vs 0-1000: `GridMax` is a parameter.
5. "Need gains cua" becomes `Capability::ComputerUse` with `VocabVersion(2)`.
6. async-openai in the build map: not adopted, because confined engines need Unix sockets.
7. Three changes to frozen porter types (`ImagePart.source`, `Model::chat` streaming, `Transport::open` replacing `infer`), each edited in design/31 in the same change.

### Critical Files for Implementation
- /home/pohsuanlai/porter/crates/porter-infer/src/request.rs
- /home/pohsuanlai/porter/crates/porter-infer/src/model.rs
- /home/pohsuanlai/porter/crates/porter-dbus/src/inference.rs (with /home/pohsuanlai/porter/dbus/org.quire.Inference1.xml)
- /home/pohsuanlai/porter/crates/porter-core/src/capability/ai.rs (with need/ai.rs, matching.rs)
- /home/pohsuanlai/quire/design/31-ACCOUNTS.md and /home/pohsuanlai/quire/docs/workspace-deps.toml

<!-- paths: end -->
