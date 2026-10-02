<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/cua.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# Area: cua — the computer-use runtime and its desktop plumbing

Status: plan for the freeze. Sources read: BRIEF.md, COMPANION.md, cua-integration.md, research-ecosystem-{agents,infra}.md, research-cua-models.md, research-agent-memory §3, intents-research/{models,agent-ux}.md, quire design/31-ACCOUNTS.md (§2.3-2.4, §4, §5.5, §11), quire CONVENTIONS.md and ARCHITECTURE.md (`Machine`), porter (ARCHITECTURE, CONVENTIONS, FINDINGS, check-boundary.sh, dbus XML), sill-launcher (`Provider`, `Activation`), and shell-host's nested-compositor harness (`scripts/dev/`).

I also checked the installed binaries on this machine (Fedora 44: cosmic-comp 1.8.0, kwin 6.7.5) and the nested cosmic-comp globals the harness recorded (`~/shell-host/scripts/dev/out/globals.txt`):

- **Window capture on stock cosmic-comp 1.8.** It advertises `ext_image_copy_capture_manager_v1`, `ext_foreign_toplevel_image_capture_source_manager_v1`, `ext_foreign_toplevel_list_v1`, `zcosmic_toplevel_info_v1` (its `geometry` event exists since v2), `zcosmic_toplevel_manager_v1` (`move_to_ext_workspace` since v4), `ext_workspace_manager_v1`, `zwp_virtual_keyboard_manager_v1`, `zwp_text_input_manager_v3`, `cosmic_a11y_manager_v1` and `wp_security_context_manager_v1`. It has **no** `zwlr_virtual_pointer` and **no** `ext_transient_seat`.
- **Input on stock cosmic-comp 1.8.** The EIS server is reached over D-Bus as `com.system76.CosmicComp.Ei` at `/com/system76/CosmicComp/Ei`, method `GetSenderSocket(device_types)`. reis is vendored in the binary. The exact argument types and who may call it (`COSMIC_ENFORCE_DBUS_OWNERS`) are [unverified]; Spike A settles them.
- **KWin 6.7.5.** The plugins `eis.so` (`org.kde.KWin.EIS.RemoteDesktop`), `screencast.so` (`zkde_screencast` stream_window over PipeWire, with off-screen rendering of windows) and `screenshot.so` are installed. `libkwin` has no ext-image-copy-capture.
- **Licences.** cosmic-comp is GPL-3.0-only, and our deny.toml keeps GPL out. So the fork is a separate GPL repo, and no crate of ours may depend on it. The private protocol crate it uses is ours, under MIT OR Apache-2.0.
- **Protocol bindings.** wayland-protocols 0.32.13 (already pinned) contains ext-image-copy-capture, ext-image-capture-source, ext-foreign-toplevel-list, ext-transient-seat and ext-workspace.

---

## 1. Scope and non-goals

**In scope**
- **`cuad`**: the daemon that owns the step loop (observe → model step → policy check → act → settle). It also owns budgets, the CUA consent gate, masking policy, audit records, the per-window lease, and the `org.quire.Cua1` control plane.
- **The run state machine**: pure, with every state, input, effect and transition (§4).
- **Observations**: masked window frame, an optional a11y tree, typed context, the agent cursor and the outcomes of the previous step. Also the `a11y-tree` model (snapshot, prune, render to text, diff, selectors).
- **`seat-input`**: the agent seat over reis/libei against the compositor's EIS server, plus pure action→input planning and keymap/text handling.
- **`maskcap`**: masked per-window capture over ext-image-copy-capture (stock) or the fork's masked source, plus damage-based settling and stall digests.
- **`cua-desktop`**: windows, geometry, leases, workspaces, glow, the kill chord, and the stock consent guard.
- **Run modes**: in place, agent workspace, nested session; which backend supports which.
- **The cosmic-comp fork**: what must change, against what works on stock protocols.
- **Event-log records and procedures**: the run → event-log record types (written through the memory area's `eventlog`), and the learned-procedure file format (later).
- **Spikes first**, each with acceptance criteria.

**Non-goals**
- Model choice, the model wire formats, action parsing, image resize/encode, and inferd routing. These belong to the **models** area. cua consumes `cua-action` and the `CuaModel` seam.
- Cedar policy text, the typed-action registry, `prov` labels, and the confirmation wording. These belong to the **actions** area. cua builds the request and executes the verdict.
- Event-log storage, hash chain, memfiles and recall. These belong to the **memory** area. cua defines its record payloads only.
- The launcher's run strip and orb drawing. These belong to whoever owns sill's launcher and companion UX; cua provides the `Cua1` bus they subscribe to.
- The planner and router tiering decision (typed action, then hook, then CUA). cua is called; it does not decide to be called.
- uinput, in any form, ever.

**Conflicts with the docs, and the resolution I propose**
1. **COMPANION settled item 4 ("No screen clicking") against the CUA tier.** cua-integration proposes replacing item 4. Until the user confirms, `cuad` ships with `cua.enabled = off` and nothing else depends on it. *User question Q1.*
2. **Floating-point coordinates.** cua-integration uses `f32` `LogicalPoint`; porter's rule allows floats only in `EmbedVector`. Resolution: integer `Lpx(i32)` window-local logical px. Model-frame coordinate types (normalised 0-999 grid, model-pixel space) stay inside the models area's adapters.
3. **"Second wl_seat" (cua-integration §4.2) against "seat-input over reis" (COMPANION build map).** Resolution: one client path. Injection is always EI through reis. On the fork, the EI sender context is bound to a compositor-side agent seat; on stock, it lands on the user's seat (`SeatIsolation::SharedSeat`).
4. **Where masking happens.** cua-integration masks in the compositor; the infra survey masks in the capturing process. Resolution: both, chosen by backend. `MaskReport.applied_by: MaskWhere {Compositor, Cuad}` records which one ran, and pixels never leave `cuad` unmasked in either case.
5. **Where `cuad` runs.** cua-integration offers "a daemon, or a module inside the shell host". Resolution: its own daemon. It holds a privileged Wayland connection and an EIS fd, needs its own systemd sandbox, and must not share a process with GPU UI. shell-host stays a UI host.
6. **`HandBack` has two meanings in cua-integration** (user→agent at the start, and resuming after a takeover). Resolution: `Run.HandBack` only resumes a taken-over run. Starting from the user's manual work is `Manager.Start` with `origin = Handover`.
7. **No CUA need or CUA grant in porter.** porter's `Need` has no `cua` kind and `GrantKey` has no Space. Resolution: the models area adds `Need::Cua` (a 31-ACCOUNTS edit). The CUA grant `(app, Space) → Once/Always` lives in the actions area's action-grant store, not in porter's account grants (§7).

---

## 2. Repos and crates

### New repo `~/cua` (working name; Q3), MIT OR Apache-2.0, porter-style

Workspace conventions are copied from porter: `rust-toolchain.toml` at 1.98.1, quire's `deny.toml` verbatim, `CONVENTIONS.md` pointing at quire's with the porter additions (closed sets are enums with serde slugs and no `Word`; async seams are `-> impl Future + Send`; closed sets of implementations are enums, not `dyn`; `todo!()` only behind a frozen interface and listed in FINDINGS; nothing ambient below the daemon). It also gets `ARCHITECTURE.md`, `FINDINGS.md`, `dbus/`, `protocols/` and `scripts/`.

| Crate | Purpose | I/O | Portable |
|---|---|---|---|
| `a11y-tree` | snapshot model (`A11ySnapshot`, `A11yNode`, `Role`), prune, render to model text, diff, `NodeSelector`; trait `A11ySource`; feature `atspi` = AT-SPI backend | none by default; zbus behind `atspi` | core yes |
| `seat-input` | `InputOp`, `plan(action) -> Vec<InputOp>`, chord/keymap mapping, `InputReceipt`; trait `AgentSeat`; feature `ei` = reis client backend | none by default; reis behind `ei` | core yes |
| `maskcap` | `Frame`, `PixelBuf`, `MaskRegion`, `MaskReport`, `apply_masks`, `FrameDigest`, settle logic; traits `WindowCapture` and `CaptureSession`; feature `wayland` = ext-image-copy-capture plus the quire masked source | none by default | core yes |
| `cua-desktop` | `ToplevelInfo`, `LeaseRequest`/`LeaseGrant`, `BackendCaps`, `GlowState`, `DesktopEvent`; traits `Desktop` and `LeaseControl`; feature `cosmic` = cosmic-client-toolkit, ext-workspace and the quire-agent protocol | none by default | core yes |
| `quire-agent-protocol` | `protocols/quire-agent-v1.xml` plus wayland-scanner bindings; features `client` and `server` (the fork uses `server`) | none | yes |
| `cua-run` | pure core: ids, `CuaTask`, `CuaResult`, `RunMode`, `Budget`, `Observation`, `ActionOutcome`, `classify_effect`, `LeaseBook`, the run machine (`Run::step`), stall detection, `CuaRecord` (event-log payloads), `Procedure` types, the `Cua1` wire bodies, `mode_support` | none | yes |
| `cua-dbus` | `org.quire.Cua1` zbus proxies and skeletons, `dbus/org.quire.Cua1.xml`, codec, introspection test | zbus | Linux |
| `cua-fake` | test-only: `FakeSeat`, `FakeEis` (an in-process reis EIS server on a socketpair), `FakeCapture`, `FakeDesktop`, `FakeA11y`, `ScriptedModel` (over models' replay provider), `FakePdp`, `MemoryEvents`, `FixedClock`, `run_harness` | none | yes |
| `cuad` | the daemon: env and settings, effect executor, zbus server, caller checks, backend selection; `dist/cuad.service` and the D-Bus activation file | everything | Linux |

**Allowed edges** (checked by `scripts/check-boundary.sh`, copied from porter's script and adapted):

| Crate | May depend on (ours and other areas') |
|---|---|
| `a11y-tree` | `cua-action` (models), `prov` (actions) |
| `seat-input` | `cua-action` |
| `maskcap` | `cua-action` |
| `cua-desktop` | `cua-action`; `quire-agent-protocol` (feature `cosmic`) |
| `cua-run` | `cua-action`, `a11y-tree`, `seat-input`, `maskcap`, `cua-desktop` (all default features), `prov`, `quire-intents` (types), `porter-core` (`DataClass`, `AppId`, `Locality`) |
| `cua-dbus` | `cua-run` |
| `cua-fake` | everything above with default features, plus `reis` (for `FakeEis`), plus models' `model-provider` (replay) |
| `cuad` | all of the above with their I/O features, plus `model-provider` (models), `policy-point` (actions), `eventlog` (memory), `porter-client` (`DbusTransport`) |

**External boundary.** Every crate except `cua-dbus`, `cuad` and the backend features never reaches `zbus`, `zvariant`, `tokio`, `wayland-client`, `wayland-backend`, `reis`, `atspi`, `reqwest` or `hyper` (default features). `cua-run` additionally never reaches `cedar-policy` or `rusqlite`: verdicts and records cross as data.

### Fork repo `~/cosmic-comp` (GPL-3.0-only, branch `quire`, rebased on upstream release tags)

All quire code goes in `src/quire/` (agent seat, masked source, lease and glow globals, input-origin gate, kill chord), with small hook calls into upstream files. The fork depends on `quire-agent-protocol` (feature `server`) by git rev. The double-tap ⌘ detection (COMPANION item 7) also lands here later; whoever owns it shares the branch. Ownership: Q11.

### Adopted crates (from quire's pinned block unless marked "joins the block")

- **Already pinned:** `wayland-client 0.31`, `wayland-backend 0.3`, `wayland-protocols 0.32.13` (staging), `cosmic-client-toolkit` (pinned rev), `xkbcommon 0.8` (text → keysym → keycode on the EIS keymap), `zbus 5.19` (tokio), `tokio 1`, `serde`/`serde_json`, `thiserror 2`, `image 0.25.6` (PNG for thumbnails only), `memfd 0.6` (shm buffers for capture), `rustix 1.1`.
- **Joins the block:** `reis 0.7` (MIT; pin the minor that the fork's cosmic-comp `Cargo.lock` resolves; the README says its API may change) and `atspi 0.30` (Apache-2.0 OR MIT).
- **Later, joins the block:** `ashpd 0.13` (RemoteDesktop/ScreenCast portal fallback if `GetSenderSocket` is owner-restricted) and `pipewire` (KWin window capture; [unverified] version and licence).
- **From other areas:** `cua-action`, `model-provider` (models); `prov`, `policy-point`, `quire-intents` (actions); `eventlog`, `memfiles` (memory); `porter-core` (`DataClass::Screen`, `AppId`, `Locality`).

Image resize and encoding are models' `vision-prep`. cua hands over raw masked pixels.

---

## 3. Frozen interfaces

### 3.0 What cua needs from `cua-action` (owned by **models**; listed so both specs agree) — freeze now

```rust
// cua-action: types only, serde (kind/v adjacently tagged), no floats.
pub struct WindowRef(pub u32);                 // per-run handle; cuad maps it to a ToplevelIdent
pub struct Lpx(pub i32);                       // logical px, window-local
pub struct WinPoint { pub x: Lpx, pub y: Lpx } // origin = top-left of the window geometry (the cropped frame)
pub struct WinSize { pub w: Lpx, pub h: Lpx }
pub struct WinRect { pub origin: WinPoint, pub size: WinSize }
pub struct NodeId(pub u32);                    // a11y node id, stable within one run
pub enum ActionTarget { Point(WinPoint), Node(NodeId) }   // Q4: freeze both now
pub enum Button { Left, Right, Middle }
pub enum Modifier { Ctrl, Alt, Shift, Super }
pub struct Chord { pub mods: BTreeSet<Modifier>, pub key: KeyName }   // KeyName = xkb keysym name, parsed once
pub struct ClickCount(pub u8);                 // 1..=3, checked at parse
pub struct Repeat(pub u8);
pub struct ScrollSteps(pub i16);               // wheel detents; sign = direction
pub struct Ms(pub u32);
pub enum CuaAction {
    Click  { at: ActionTarget, button: Button, count: ClickCount, mods: BTreeSet<Modifier> },
    Move   { to: ActionTarget },
    Drag   { from: ActionTarget, to: ActionTarget, button: Button },
    Type   { text: String },
    Key    { chord: Chord, repeat: Repeat },
    Scroll { at: ActionTarget, dx: ScrollSteps, dy: ScrollSteps },
    Wait   { for_: Ms },
    Zoom   { region: WinRect },
    Done   { summary: String, extracted: Vec<Extracted> },
    Ask    { question: String, choices: Vec<String> },
}
pub struct Extracted { pub name: String, pub value: String }   // cua-run labels these Untrusted(Screen)
pub struct TargetedAction { pub window: WindowRef, pub action: CuaAction }

// The model turn, built by cua-run from an Observation; adapters resize/encode it.
pub struct CuaTurn { pub step: u32, pub frame: RawFrame, pub geometry: TurnGeometry,
                     pub tree_text: TreeText, pub prev: Vec<PrevNote>, pub notes: Vec<String> }
pub enum TreeText { Absent, Present(String) }
pub struct RawFrame { pub format: PixelFormat, pub size: PxSize, pub stride: u32, pub data: Arc<[u8]> }
pub enum PixelFormat { Xrgb8888, Argb8888 }
pub struct TurnGeometry { pub logical: WinSize, pub scale: Scale120 }   // Scale120: wp_fractional_scale units
pub struct StepReply { pub thought: Thought, pub actions: Vec<TargetedAction>, pub usage: Usage }
pub enum Thought { None, Text(String) }
pub enum StepEvent { ThoughtDelta(String), ActionProposed(TargetedAction) }
```

`CuaModel` / `CuaSession` (trait in models' `model-provider`). cua relies on exactly this:

```rust
fn step(&mut self, turn: CuaTurn, sink: &mut dyn FnMut(StepEvent), cancel: &CancelToken)
    -> impl Future<Output = Result<StepReply, CuaModelError>> + Send;
fn usage(&self) -> Usage;
```

`CuaModelError` must let cua tell retryable from fatal: `Transient`, `Parse` (counted as a step) and `Fatal` kinds.

### 3.1 `a11y-tree` — freeze now (the `atspi` backend body later)

```rust
pub struct A11ySnapshot {
    pub source: TreeSource, pub taken: Stamp,
    pub nodes: Vec<A11yNode>,              // arena; index 0 = window root
    pub truncated: Truncation,
}
pub enum TreeSource { Atspi, QuireApp }    // QuireApp = accesskit_unix from a Blitz app (still AT-SPI on the wire)
pub enum Truncation { Complete, Pruned { dropped: u32 } }
pub struct A11yNode {
    pub id: NodeId, pub parent: Parent, pub role: Role,
    pub name: Labelled<String>,            // screen text: Untrusted(Screen{app,window})
    pub value: NodeValue, pub states: BTreeSet<NodeState>,
    pub bounds: WinRect,                   // window-local logical px
    pub actions: Vec<NodeAction>,          // AT-SPI action names, parsed
    pub typed: TypedHint,                  // quire widgets: the typed action this control maps to
    pub sensitivity: Sensitivity,
}
pub enum Parent { Root, Node(NodeId) }
pub enum Role { Window, Dialog, Button, ToggleButton, CheckBox, RadioButton, ComboBox, MenuBar, Menu,
    MenuItem, Tab, TabList, Link, Entry, PasswordText, Text, Label, Heading, List, ListItem, Tree,
    TreeItem, Table, Cell, Slider, SpinButton, ScrollBar, ProgressBar, Image, ToolBar, Document,
    Section, Other(u32) /* raw AT-SPI role */ }
pub enum NodeValue { None, Text(Labelled<String>), Range { now: i64, min: i64, max: i64 } }
pub enum NodeState { Enabled, Focusable, Focused, Selected, Checked, Expanded, Editable, Visible, Showing }
pub enum NodeAction { Click, Press, Activate, Toggle, Expand, Other(String) }
pub enum TypedHint { None, Action(ActionName) }     // ActionName from quire-intents
pub enum Sensitivity { Normal, Secret }             // PasswordText role or a quire Sensitivity flag
pub struct NodeBudget(pub u32);                     // setting cua.tree.max_nodes (proposed 400)

pub fn prune(s: &A11ySnapshot, budget: NodeBudget) -> A11ySnapshot;          // visible+actionable first
pub fn render(s: &A11ySnapshot) -> String;   // "[12] button \"Save\" enabled,focusable @340,20 80x24"; Secret values never rendered
pub fn diff(a: &A11ySnapshot, b: &A11ySnapshot) -> TreeDiff;               // added/removed/changed ids
pub fn secret_rects(s: &A11ySnapshot) -> Vec<WinRect>;                      // mask hints for maskcap
pub fn resolve(s: &A11ySnapshot, sel: &NodeSelector) -> Resolution;         // procedures (later)
pub enum Resolution { One(NodeId), Many(Vec<NodeId>), None }
pub struct NodeSelector { pub path: Vec<SelectorStep> }     // later
pub struct SelectorStep { pub role: Role, pub name: NameMatch }
pub enum NameMatch { Any, Exact(String), Slot(SlotName) }

pub trait A11ySource: Send + Sync {
    fn snapshot(&self, target: &A11yTarget, budget: NodeBudget)
        -> impl Future<Output = Result<A11ySnapshot, A11yError>> + Send;
}
pub struct A11yTarget { pub pid: Pid, pub app: AppName, pub title: String, pub geometry: WinSize }
#[derive(thiserror::Error)]
pub enum A11yError { NoBus, NotExposed, NoMatchingWindow, Timeout, Gone, Protocol(String) }
```

### 3.2 `seat-input` — freeze now (the `ei` backend body fill)

```rust
pub enum InputOp {
    MoveTo(WinPoint),
    Button { button: Button, dir: Press },
    Key { key: Keycode, dir: Press },        // evdev code in the EIS keymap
    Text(String),                            // only if caps.text has TextPath::AgentTextInput
    Scroll { dx: ScrollSteps, dy: ScrollSteps },
    Gap(Ms),                                 // between press/release, double-click spacing
}
pub enum Press { Down, Up }
pub struct Keycode(pub u32);
pub struct Keymap { /* xkb keymap received from EIS, parsed once */ }
pub enum TextPath { EiKeymap, VirtualKeyboard, AgentTextInput }
pub struct SeatCaps { pub isolation: SeatIsolation, pub text: Vec<TextPath>, pub scroll: ScrollKind }
pub enum SeatIsolation { AgentSeat, SharedSeat }
pub enum ScrollKind { Discrete, Smooth }

/// Pure. Node targets are resolved by the caller to a WinPoint (centre of bounds, clamped inside).
pub fn plan(action: &PlannedAction, keymap: &Keymap, caps: &SeatCaps) -> Result<Vec<InputOp>, PlanError>;
pub struct PlannedAction { pub action: CuaAction, pub at: ResolvedPoints, pub window: WinSize }
pub enum PlanError { OutsideWindow(WinPoint), Unmappable(char), NotAnInput /* Done/Ask/Zoom/Wait */, UnknownKey(String) }

pub struct InputReceipt { pub lease: LeaseId, pub ops: u32, pub text_path: Option<TextPath>,
                          pub origin: InputOrigin, pub at: Stamp }
pub enum InputOrigin { AgentSeat(LeaseId), SharedSeatViaEi }

pub trait AgentSeat: Send + Sync {
    fn caps(&self) -> SeatCaps;
    fn keymap(&self) -> &Keymap;
    fn attach(&self, lease: &LeaseGrant) -> impl Future<Output = Result<(), SeatError>> + Send;   // start_emulating
    fn perform(&self, lease: LeaseId, ops: &[InputOp]) -> impl Future<Output = Result<InputReceipt, SeatError>> + Send;
    fn detach(&self, lease: LeaseId) -> impl Future<Output = Result<(), SeatError>> + Send;      // stop_emulating
}
#[derive(thiserror::Error)]
pub enum SeatError { NotConnected, Refused, OutsideLease, TargetGone, Suspended, Keymap(String), Protocol(String) }
```

Backend enum in `cuad`: `SeatBackend { Ei(EiSeat) }`; tests use `FakeSeat` and `EiSeat` over `FakeEis`. Mapping from the window to the EI region: on the fork, the lease's EI region *is* the window (identity). On stock, `GlobalPoint = window_origin + WinPoint`, taken from `cua-desktop` geometry. That transform lives in `seat-input::region` (pure).

### 3.3 `maskcap` — freeze now

```rust
pub struct Frame {
    pub seq: FrameSeq, pub at: Stamp,
    pub pixels: PixelBuf,                 // already masked; device px, cropped to window geometry
    pub logical: WinSize, pub scale: Scale120,
    pub damage: Vec<WinRect>,             // since previous frame of this session
    pub masks: MaskReport,
}
pub struct PixelBuf { pub format: PixelFormat, pub size: PxSize, pub stride: u32, pub data: Arc<[u8]> }
pub struct MaskReport { pub applied_by: MaskWhere, pub whole: WholeMask, pub regions: Vec<MaskedRegion> }
pub enum MaskWhere { Compositor, Cuad }
pub enum WholeMask { None, Blacked(MaskReason) }
pub struct MaskedRegion { pub rect: WinRect, pub why: MaskReason }
pub enum MaskReason { PasswordField, SurfaceDeclared, A11ySecret, SensitiveWindow, OtherSpace, OverlayOutsideLease }
pub struct MaskRegion { pub rect: WinRect, pub why: MaskReason }

pub fn apply_masks(buf: PixelBuf, scale: Scale120, regions: &[MaskRegion]) -> (PixelBuf, Vec<MaskedRegion>); // pure, returns new buffer
pub fn digest(buf: &PixelBuf) -> FrameDigest;          // for stall detection and the event log (no pixels stored)
pub struct FrameDigest(pub [u8; 32]);
pub fn settle(events: &[DamageAt], quiet: Ms, timeout: Ms, since: Stamp, now: Stamp) -> Settle;   // pure
pub enum Settle { Quiet, Waiting { wake: Stamp }, TimedOut }

pub trait WindowCapture: Send + Sync {
    type Session: CaptureSession;
    fn open(&self, lease: &LeaseGrant) -> impl Future<Output = Result<Self::Session, CaptureError>> + Send;
}
pub trait CaptureSession: Send {
    fn frame(&mut self, want: FrameWant, hints: &[MaskRegion]) -> impl Future<Output = Result<Frame, CaptureError>> + Send;
    fn damage(&mut self) -> impl Future<Output = Result<DamageAt, CaptureError>> + Send;  // next damage event
}
pub enum FrameWant { Now, AfterDamage }
#[derive(thiserror::Error)]
pub enum CaptureError { SourceGone, Refused, Withheld /* hidden window not rendered */, Format, Protocol(String) }
```

Backends: `CaptureBackend { StockToplevel(..), QuireMasked(..) }`. On stock, `hints` (from `a11y-tree::secret_rects`, sensitive-window rules and the text-input heuristic) are applied by `apply_masks` in `cuad` (`applied_by = Cuad`). On the fork, hints are sent to the compositor and the buffer arrives masked (`applied_by = Compositor`).

### 3.4 `cua-desktop` — freeze now

```rust
pub struct ToplevelIdent(pub String);    // ext_foreign_toplevel_handle_v1.identifier
pub struct ToplevelInfo { pub ident: ToplevelIdent, pub app: AppName, pub title: String, pub pid: Pid,
    pub geometry: GlobalRect, pub workspace: WorkspaceRef, pub state: BTreeSet<ToplevelState>, pub trust: WindowTrust }
pub enum WindowTrust { Quire, Flatpak, Native, Shell /* never leasable */ }
pub enum RunMode { InPlace, AgentWorkspace, NestedSession }
pub struct LeaseId(pub u64);
pub struct LeaseRequest { pub window: ToplevelIdent, pub mode: RunMode, pub space: SpaceId, pub placement: ReleasePlacement }
pub enum ReleasePlacement { ReturnToUser, Stay }
pub struct LeaseGrant { pub id: LeaseId, pub window: ToplevelInfo, pub seat: SeatIsolation, pub ei: EiEndpoint }
pub enum EiEndpoint { LeaseSocket(OwnedFd) /* fork */, SenderSocket(OwnedFd) /* stock GetSenderSocket */ }
pub enum GlowState { Off, Working, Acting { at: WinRect }, WaitingForYou }
pub enum Suspend { TakeOver, Consent, Paused }
pub enum DesktopEvent {
    WindowClosed(ToplevelIdent), GeometryChanged(ToplevelIdent, GlobalRect),
    LeaseRevoked { lease: LeaseId, why: RevokeReason }, PhysicalInput { lease: LeaseId },
    ConsentShown, ConsentHidden, KillChord,
}
pub enum RevokeReason { KillChord, WindowClosed, SpaceChanged, Compositor }

pub struct BackendCaps { pub kind: BackendKind, pub seat: SeatIsolation, pub masking: MaskWhere,
    pub consent: ConsentGuard, pub hidden: HiddenCapture, pub glow: GlowDraw }
pub enum BackendKind { CosmicFork, CosmicStock, KwinStock }
pub enum ConsentGuard { CompositorOrigin, CuadSuspendsInput }
pub enum HiddenCapture { Renders, Withheld }
pub enum GlowDraw { Compositor, ShellOnly }

pub trait Desktop: Send + Sync {
    fn caps(&self) -> BackendCaps;
    fn windows(&self) -> impl Future<Output = Result<Vec<ToplevelInfo>, DesktopError>> + Send;
    fn next_event(&self) -> impl Future<Output = DesktopEvent> + Send;
}
pub trait LeaseControl: Send + Sync {
    fn lease(&self, req: LeaseRequest) -> impl Future<Output = Result<LeaseGrant, DesktopError>> + Send;
    fn release(&self, lease: LeaseId) -> impl Future<Output = Result<(), DesktopError>> + Send;
    fn suspend(&self, lease: LeaseId, why: Suspend) -> impl Future<Output = Result<(), DesktopError>> + Send;
    fn resume(&self, lease: LeaseId) -> impl Future<Output = Result<(), DesktopError>> + Send;
    fn glow(&self, lease: LeaseId, glow: GlowState) -> impl Future<Output = Result<(), DesktopError>> + Send;
}
#[derive(thiserror::Error)]
pub enum DesktopError { NoSuchWindow, NotLeasable(WindowTrust), ModeUnsupported(RunMode), Busy, Protocol(String) }
```

### 3.5 `cua-run` (pure core) — freeze now

```rust
pub struct RunId(pub u64);
pub struct StepIndex(pub u32);
pub struct QuestionId(pub u64);
pub struct Stamp(pub u64);                           // ms from cuad's clock origin

pub struct CuaTask {
    pub goal: Labelled<String>,                      // Trusted(User) when typed by the user
    pub target: TaskTarget, pub mode: RunMode, pub space: SpaceId,
    pub budget: Budget, pub data_in: Vec<Labelled<Value>>, // quire-intents Value
    pub success: SuccessCheck, pub plan_step: PlanStepRef, pub origin: RunOrigin,
    pub procedure: ProcedureUse,                     // later; frozen variant None
}
pub enum TaskTarget { Window(ToplevelIdent), App(AppName) /* later: launch into the mode */ }
pub enum RunOrigin { Planner, Handover { since: Stamp } }
pub enum SuccessCheck { ModelSays, TreeHas(NodeSelector), Both(NodeSelector) }
pub enum ProcedureUse { None, Hint(ProcedureId), Replay(ProcedureId) }
pub struct Budget { pub steps: u32, pub active: Ms, pub stall_frames: u8, pub actions_per_min: u16,
                    pub denials: u8, pub model_failures: u8, pub spend: MicroUsd }

pub struct Observation {
    pub step: StepIndex, pub window: WindowInfo, pub frame: Frame,      // maskcap::Frame (masked)
    pub tree: TreeObs, pub context: ContextObs, pub cursor: CursorObs, pub prev: Vec<ActionOutcome>,
}
pub struct WindowInfo { pub window: WindowRef, pub app: AppName, pub title: Labelled<String>,
    pub size: WinSize, pub scale: Scale120, pub focus: Focus, pub visibility: Visibility, pub trust: WindowTrust }
pub enum Focus { Focused, Unfocused }
pub enum Visibility { Shown, Occluded, HiddenWorkspace, Minimized }
pub enum TreeObs { Absent(TreeAbsence), Present(A11ySnapshot) }
pub enum TreeAbsence { NotExposed, NoMatch, Timeout, Off }
pub enum ContextObs { Absent, Present(Labelled<quire_intents::Context>) }
pub enum CursorObs { Unknown, At(WinPoint) }
pub struct ActionOutcome { pub action: TargetedAction, pub result: ActionResult }
pub enum ActionResult { Done, NotExecuted, Refused(DenyReason), UserDeclined, Failed(String),
                        Answered(Labelled<String>), UserActed { since: Stamp }, CheckFailed }

pub enum EffectClass { Read, UndoableWrite, Outbound, Destructive }   // mirrors actions' Effect; From impl there
pub enum EffectSource { NodeTypedAction(ActionName), WindowClass(WindowClass), DefaultTable }
pub enum WindowClass { Ordinary, MailCompose, Terminal, Payments, Admin, PasswordManager, Banking }
pub fn classify_effect(a: &CuaAction, node: Option<&A11yNode>, class: WindowClass) -> (EffectClass, EffectSource);

pub struct AuthAsk { pub run: RunId, pub step: StepIndex, pub action: TargetedAction, pub effect: EffectClass,
    pub source: EffectSource, pub app: AppName, pub trust: WindowTrust, pub space: SpaceId,
    pub mode: RunMode, pub taint: Label /* prov */ }
pub enum CuaVerdict { Allow, Ask(ConfirmText), Deny(DenyReason) }
pub struct ConfirmText(pub String);       // produced by the router from the action, never by the model
pub enum DenyReason { Policy(String), OutsideLease, SecretField, Budget }

pub struct CuaResult { pub run: RunId, pub outcome: CuaOutcome, pub summary: Labelled<String>,
    pub extracted: Vec<Labelled<Extracted>>, pub steps: u32, pub cost: MicroUsd }
pub enum CuaOutcome { Done, Failed(FailReason), Blocked(BlockedOn), Cancelled(CancelledBy), BudgetOut(BudgetKind) }
pub enum FailReason { WindowGone, Model, Capture, Input, CheckFailed }
pub enum BlockedOn { ConsentDenied, LeaseRefused, PolicyDenials, ModeUnsupported(RunMode), NeedsCapability }
pub enum CancelledBy { User, KillSwitch, Planner }
pub enum BudgetKind { Steps, Active, Stall, Rate, Spend }

pub fn mode_support(caps: &BackendCaps, mode: RunMode) -> ModeSupport;
pub enum ModeSupport { Full, HandsOff /* shared seat: physical input auto-pauses */, Unsupported(ModeGap) }
pub enum ModeGap { HiddenCaptureWithheld, NoNestedHost, ShellWindow }
pub fn turn(obs: &Observation) -> CuaTurn;            // Observation -> model turn (tree rendered, prev notes, mask notes)
pub struct LeaseBook { /* window -> run; pure */ }    // one run per window, one window per run (v1)
pub fn stalled(history: &[FrameDigest], limit: u8) -> Stall;
pub enum Stall { Moving, Stalled }
```

The run machine has the same shape as `ds_core::Machine`, but is not tied to it: `cua-run` is portable and sits below the design system, like porter.

```rust
pub struct Run { pub id: RunId, pub task: CuaTask, pub phase: RunPhase, pub used: BudgetUse, pub taint: Label,
                 pub digests: Vec<FrameDigest>, pub pending: ActionQueue }
impl Run {
    pub fn start(id: RunId, task: CuaTask, at: Stamp) -> (Run, Vec<RunEffect>);
    pub fn step(self, input: RunInput, at: Stamp) -> (Run, Vec<RunEffect>);
    pub fn wake(&self) -> Option<Stamp>;
}
pub enum RunPhase {                          // see §4 for transitions
    Granting, Leasing, Observing(StepIndex), Thinking(StepIndex), Authorising(StepIndex),
    Confirming(StepIndex, ConfirmText), Acting(StepIndex), Settling(StepIndex, Settle),
    Verifying(StepIndex), WaitingForUser(StepIndex, QuestionId), Paused(StepIndex),
    TakenOver(StepIndex, Stamp), Finished(CuaOutcome),
}
pub enum RunInput {
    Grant(GrantVerdict), Leased(LeaseGrant), LeaseRefused(DesktopError),
    Observed(Box<Observation>), CaptureFailed(CaptureError),
    ModelEvent(StepEvent), ModelReplied(StepReply), ModelFailed(ModelFailure),
    Verdict(CuaVerdict), ConsentAnswered(Answer), Injected(InputReceipt), InjectFailed(SeatError),
    Damage(DamageAt), Checked(CheckResult),
    UserPause, UserResume, UserCancel, UserAnswer(QuestionId, Labelled<String>), UserTakeOver, UserHandBack,
    Desktop(DesktopEvent), Tick,
}
pub enum GrantVerdict { Allowed, Ask, Denied }
pub enum Answer { Allow, Deny }
pub enum ModelFailure { Transient, Parse, Fatal }
pub enum CheckResult { Pass, Fail }
pub enum RunEffect {
    CheckGrant { app: AppName, space: SpaceId }, ShowGrantPrompt { app: AppName, space: SpaceId },
    AcquireLease(LeaseRequest), ReleaseLease(LeaseId), SuspendLease(LeaseId, Suspend), ResumeLease(LeaseId),
    Capture { want: FrameWant, hints: HintSource }, Snapshot(NodeBudget), FetchContext,
    CallModel(CuaTurn), CancelModel,
    Authorise(AuthAsk), ShowConsent(ConfirmText), DismissConsent,
    Inject(PlannedAction), WatchSettle { quiet: Ms, timeout: Ms }, CancelSettle,
    VerifyTree(NodeSelector), Glow(GlowState),
    Emit(CuaSignal), Record(CuaRecord),
}
pub enum CuaSignal { StateChanged(RunStateSlug), StepStarted(StepIndex), ThoughtDelta(StepIndex, String),
    ActionProposed(StepIndex, TargetedAction), ActionDone(StepIndex, ActionOutcome),
    AskUser(QuestionId, AskBody), Finished(CuaResult) }
```

### 3.6 Event-log records (payloads; storage is **memory**'s `eventlog`) — freeze now

Actor is `Actor::Companion(CuaRun(RunId))`. The kind slug is the serde tag (`cua.run.started`, …). No pixels and no thought text are stored. A thought becomes a digest; a frame becomes a digest and a size.

```rust
pub enum CuaRecord {
    RunStarted { run: RunId, goal: Labelled<String>, plan_step: PlanStepRef, app: AppName, space: SpaceId,
                 mode: RunMode, backend: BackendKind, served_by: ServedByNote, locality: Locality, budget: Budget },
    Step { run: RunId, n: StepIndex, thought: ThoughtDigest, action: TargetedAction, effect: EffectClass,
           source: EffectSource, authorised_by: AuthorisedBy, result: ActionResult,
           frame: FrameNote, input: Option<InputReceipt> },
    Asked { run: RunId, n: StepIndex, question: Labelled<String> },
    Confirmed { run: RunId, n: StepIndex, answer: Answer, by: ConfirmBy },
    TakenOver { run: RunId, n: StepIndex }, HandedBack { run: RunId, n: StepIndex, user_events: Vec<EventId> },
    Finished { run: RunId, outcome: CuaOutcome, steps: u32, cost: MicroUsd, extracted: Vec<EventLinkedValue> },
    ProcedureLearned { procedure: ProcedureId, from: RunId },                    // later
}
pub enum AuthorisedBy { Policy, User }
pub enum ConfirmBy { PhysicalInput, ShellPrompt /* stock guard */ }
pub enum FrameNote { NotKept { digest: FrameDigest, size: PxSize }, Kept { thumb: ThumbRef, expires: Stamp } }
```

Default: `NotKept`. The setting `cua.screens.keep` (proposed `off`) controls this; kept thumbnails go to the Space's encrypted store and are forget-cascaded by memory.

### 3.7 Procedures file format — later (Phase 3; types sketched now)

The file is `procedures/<app-id>/<slug>.md` in the Space's memory folder (memory's `memfiles` owns the folder). It is add-only, has TOML frontmatter between `+++` lines, and the body holds human notes:

```toml
procedure = "org.mozilla.firefox/download-invoice"
app = "org.mozilla.firefox"
created = "2026-10-02T10:00:00Z"
from_runs = ["ev:…"]                 # EventIds of cua.run.started
params = [{ slot = "month", ty = "text" }]     # slots, never stored values
effects = ["undoable_write"]
replay = "ask_each"                  # ask_each | pinned (user only)
[[step]]
pre  = { node = [{ role = "link", name = "Billing" }] }
act  = { click = { node = [{ role = "link", name = "Billing" }] } }
post = { node = [{ role = "heading", name = "Invoices" }] }
```

### 3.8 D-Bus `org.quire.Cua1` (`cua-dbus`, served by `cuad`) — freeze now

Bodies are JSON in `s` with the envelope `{ "vocab": 1, "body": … }`. This is one protocol with two carriers, as in 31 §4.3. Caller identity comes from the connection (porter `AppId`) and is never sent.

```xml
<node>
 <interface name="org.quire.Cua1.Manager">           <!-- /org/quire/Cua1 -->
  <method name="Start"><arg name="task" type="s" direction="in"/><arg name="run" type="o" direction="out"/></method>
  <method name="Runs"><arg type="ao" direction="out"/></method>
  <method name="StopAll"/>                              <!-- control-centre kill switch -->
  <property name="Backend" type="s" access="read"/>     <!-- cosmic_fork | cosmic_stock | kwin_stock | none -->
  <property name="Modes" type="a{ss}" access="read"/>   <!-- mode -> full | hands_off | unsupported:<gap> -->
  <signal name="RunAdded"><arg type="o"/></signal>
  <signal name="RunRemoved"><arg type="o"/></signal>
 </interface>
 <interface name="org.quire.Cua1.Run">               <!-- /org/quire/Cua1/run/<id> -->
  <method name="Cancel"/><method name="Pause"/><method name="Resume"/>
  <method name="Answer"><arg name="question" type="t" direction="in"/><arg name="answer" type="s" direction="in"/></method>
  <method name="TakeOver"/><method name="HandBack"/>
  <property name="State" type="s" access="read"/>       <!-- RunStateSlug -->
  <property name="Step" type="u" access="read"/>
  <property name="Goal" type="s" access="read"/>
  <property name="Window" type="s" access="read"/>      <!-- {app, title} -->
  <property name="Mode" type="s" access="read"/>
  <property name="Budget" type="s" access="read"/>      <!-- {limits, used} -->
  <signal name="StateChanged"><arg name="state" type="s"/></signal>
  <signal name="StepStarted"><arg name="step" type="u"/></signal>
  <signal name="ThoughtDelta"><arg name="step" type="u"/><arg name="text" type="s"/></signal>
  <signal name="ActionProposed"><arg name="step" type="u"/><arg name="action" type="s"/></signal>
  <signal name="ActionDone"><arg name="step" type="u"/><arg name="outcome" type="s"/></signal>
  <signal name="AskUser"><arg name="question" type="t"/><arg name="ask" type="s"/></signal>
  <signal name="Finished"><arg name="result" type="s"/></signal>
 </interface>
</node>
```

**Error names**, mapped from `ControlRefusal`:
- `org.quire.Cua1.Error.NotPermitted` — the caller is not on the allow-list.
- `.WrongState`
- `.UnknownQuestion`
- `.Disabled` — `cua.enabled` is off.
- `.WindowUnknown`
- `.LeaseHeld`
- `.ModeUnsupported`
- `.BadTask` — vocab or parse failure.

**Who may call what:**
- `Start`: only the companion planner/router's `AppId`.
- `Cancel`, `Pause`, `Resume`, `TakeOver`, `HandBack`, `StopAll`: the planner and the shell (sill launcher/orb).
- `Answer`: the shell only.

CUA consent and effect confirmations never come through `Answer`. They are drawn by the compositor (fork) or accounts-ui (stock), and their answer arrives as `RunInput::ConsentAnswered` from cuad's own prompter seam.

### 3.9 Private Wayland protocol `quire-agent-v1` (`quire-agent-protocol`) — freeze the XML in the freeze wave; the server side is fork fill

- **`quire_agent_manager_v1`** (global; offered only to clients cuad's systemd unit runs under, through the security-context and an app-id allow-list in the fork).
  - `create_lease(id: new_id quire_agent_lease_v1, toplevel: object ext_foreign_toplevel_handle_v1, mode: uint)`.
- **`quire_agent_lease_v1`**
  - Events: `granted`, `revoked(reason: uint)`, `ei_socket(fd)` (an EIS sender context bound to this lease's agent seat; absolute region = window geometry, window-local), `physical_input`, `geometry(w, h, scale120)`.
  - Requests: `get_capture_source(new_id ext_image_capture_source_v1)` (window content with compositor masks; used with stock `ext_image_copy_capture_manager_v1`), `set_mask_hints(array of rects+reason)`, `set_glow(state: uint, x, y, w, h)`, `suspend(why: uint)`, `resume`, `destroy`.
- **`quire_sensitive_v1`**: for clients (quire apps through shell-host/blitz) to declare sensitive surface regions or a whole surface. Later.

---

## 4. State machine (the run)

**Budget accounting.** Wall-time budget counts **active** time only (Observing through Settling and Verifying). Time spent in `Paused`, `WaitingForUser`, `Confirming`, `TakenOver` and `Granting` is the user's and is excluded. Every transition into `Finished` emits `ReleaseLease`, `Glow(Off)`, `Emit(Finished)` and `Record(Finished)`. `Finished` ignores every input (same state, no effects).

| From | Input | To | Effects |
|---|---|---|---|
| (start) | — | Granting | CheckGrant |
| Granting | Grant(Allowed) | Leasing | AcquireLease (mode from task; refused early if `mode_support` = Unsupported) |
| Granting | Grant(Ask) | Granting | ShowGrantPrompt |
| Granting | Grant(Denied) | Finished(Blocked(ConsentDenied)) | — |
| Leasing | Leased(g) | Observing(1) | Record(RunStarted), Glow(Working), Capture(Now), Snapshot, FetchContext, Emit(StepStarted) |
| Leasing | LeaseRefused | Finished(Blocked(LeaseRefused)) | — |
| Observing(n) | Observed(o) with stall | Finished(BudgetOut(Stall)) | — |
| Observing(n) | Observed(o), n > budget.steps | Finished(BudgetOut(Steps)) | — |
| Observing(n) | Observed(o) | Thinking(n) | CallModel(turn(o)) |
| Observing(n) | CaptureFailed(SourceGone) | Finished(Failed(WindowGone)) | — |
| Observing(n) | CaptureFailed(other) | Observing(n), once; then Finished(Failed(Capture)) | Capture(Now) |
| Thinking(n) | ModelEvent(e) | Thinking(n) | Emit(ThoughtDelta / ActionProposed) |
| Thinking(n) | ModelReplied(r), spend over | Finished(BudgetOut(Spend)) | — |
| Thinking(n) | ModelReplied(r), empty batch | Observing(n+1), counts as a model failure | Capture |
| Thinking(n) | ModelReplied(r) | head of the queue decides: Authorising / Settling (Wait) / Observing(n+1) (Zoom, crop hint) / Verifying (Done) / WaitingForUser (Ask) | Authorise(head) or Emit(AskUser), Record(Asked), Glow(WaitingForYou) |
| Thinking(n) | ModelFailed(Transient or Parse) under limit | Observing(n) | Capture |
| Thinking(n) | ModelFailed(Fatal), or over limit | Finished(Failed(Model)) | — |
| Authorising(n) | Verdict(Allow) | Acting(n) | Inject, Glow(Acting{at}) |
| Authorising(n) | Verdict(Ask(t)) | Confirming(n,t) | SuspendLease(Consent) on stock, ShowConsent(t), Glow(WaitingForYou) |
| Authorising(n) | Verdict(Deny(r)) | Observing(n+1); after `budget.denials` → Finished(Blocked(PolicyDenials)) | rest NotExecuted; Record(Step); Capture |
| Confirming(n) | ConsentAnswered(Allow) | Acting(n) | ResumeLease, Record(Confirmed), Inject |
| Confirming(n) | ConsentAnswered(Deny) | Observing(n+1) | ResumeLease, Record(Confirmed), prev = UserDeclined |
| Acting(n) | Injected(rc) | Settling(n) | Record(Step), WatchSettle |
| Acting(n) | InjectFailed(TargetGone) | Finished(Failed(WindowGone)) | — |
| Acting(n) | InjectFailed(other) | Observing(n+1) | rest NotExecuted, Record(Step) |
| Acting(n) | rate limit exceeded | Finished(BudgetOut(Rate)) | — |
| Settling(n) | Damage / Tick → Quiet or TimedOut, queue non-empty | Authorising(n) | Authorise(next) |
| Settling(n) | → Quiet or TimedOut, queue empty | Observing(n+1) | Capture(AfterDamage), Snapshot, Emit(StepStarted) |
| Verifying(n) | Checked(Pass) | Finished(Done) | — |
| Verifying(n) | Checked(Fail) | Observing(n+1) | prev = CheckFailed |
| WaitingForUser(n,q) | UserAnswer(q, a) | Observing(n+1) | prev = Answered(a); Record |
| any active, Confirming, WaitingForUser | UserPause | Paused(n) | CancelModel, CancelSettle, DismissConsent, SuspendLease(Paused), Glow(WaitingForYou); the queue is dropped |
| active, with ModeSupport = HandsOff | Desktop(PhysicalInput) with `cua.autopause=on` | Paused(n) | same as UserPause |
| Paused(n) | UserResume | Observing(n+1) | ResumeLease, Capture, Glow(Working) |
| any non-terminal except TakenOver | UserTakeOver | TakenOver(n, now) | CancelModel, DismissConsent, SuspendLease(TakeOver), Glow(Off), Record(TakenOver) |
| TakenOver(n, t) | UserHandBack | Observing(n+1) | ResumeLease, prev = UserActed{since: t}, Record(HandedBack), Capture |
| any non-terminal | UserCancel | Finished(Cancelled(User)) | CancelModel |
| any non-terminal | Desktop(KillChord), or StopAll | Finished(Cancelled(KillSwitch)) | CancelModel |
| any non-terminal | Desktop(LeaseRevoked / WindowClosed) | Finished(Failed(WindowGone)) or Finished(Cancelled(KillSwitch)) | — |
| any active | Tick past `budget.active` | Finished(BudgetOut(Active)) | CancelModel |

`RunStateSlug` (bus and record) is one of `granting`, `leasing`, `observing`, `thinking`, `authorising`, `confirming`, `acting`, `settling`, `verifying`, `waiting_for_user`, `paused`, `taken_over`, `finished`.

**Mapping to presence** (COMPANION item 8):
- Observing and Thinking → working.
- Acting and Settling → acting-here.
- Confirming, WaitingForUser and Paused → waiting-for-you.
- TakenOver and Finished → idle.

**Lease book** (one run per window): `Free → Held(run) → Suspended(run) → Free`; a second `Start` on a held window is refused with `LeaseHeld`.

---

## 5. Tests that pin the shapes

Pure tests use `const CASES` tables, named by behaviour; the I/O feature tests run only against fakes. Live checks are dev scripts.

**`cua-run`**
- `wire_bodies_round_trip`: every `Cua1` body and every `CuaRecord` variant survives serde, with `kind`/`v` tags and the vocab envelope.
- `record_kind_slugs_are_stable`: a table of variant → `cua.run.started` and the rest.
- `run_transitions_table`: one row per §4 line; asserts the next phase and the exact effects list.
- `finished_ignores_every_input`.
- `pause_drops_queue_and_resumes_with_fresh_observation`: the queue length goes from k to 0, and Resume emits Capture.
- `deny_marks_rest_not_executed_and_tells_model`: `prev` holds the exact outcomes.
- `consent_ask_suspends_lease_on_stock_only`: `ConsentGuard` drives `SuspendLease`.
- `active_budget_excludes_user_time`: used time before and after a long Paused stretch is unchanged.
- `stall_after_n_equal_digests`.
- `spend_budget_trips_after_reply_usage`.
- `classify_effect_table`: node typed action wins; then MailCompose/Terminal/Payments make Type and Enter `Outbound`; otherwise the default table.
- `mode_support_table`: every `BackendCaps` combination × `RunMode`.
- `turn_renders_tree_without_secret_values` and `turn_notes_masked_regions`.
- `lease_book_one_run_per_window`.
- `extracted_values_are_untrusted_screen`.

**`a11y-tree`**
- `prune_keeps_actionable_visible_first_within_budget`
- `render_is_stable_golden` (fixtures `fixtures/a11y/{gtk4_dialog,firefox_form,quire_mail}.json`)
- `secret_rects_cover_password_text`
- `diff_reports_added_removed_changed`
- `selector_resolution_table` (later)

**`seat-input`**
- `plan_click_double_right_mods_table`
- `plan_drag_emits_down_moves_up`
- `plan_rejects_point_outside_window`
- `chord_parse_table` (`ctrl+shift+t`, `super`, unknown key)
- `text_to_keycodes_on_us_keymap`
- `unmappable_char_errors_named` (`你` with EiKeymap only)
- `stock_region_transform_round_trips`
- `ei_seat_against_fake_eis`, under feature `ei`: handshake, bind, start_emulating, pointer_absolute and button frames received in order by `FakeEis`, detach sends stop_emulating.

**`maskcap`**
- `apply_masks_blacks_exact_device_pixels_at_scale_150`: a 2-lpx rect becomes exactly 3 device px, the same rounding as shell-host F-f.
- `whole_window_mask_blacks_all`
- `digest_differs_on_one_pixel`
- `settle_quiet_waiting_timeout_table`

**`cua-desktop`**
- `glow_state_and_revoke_reason_wire_values` (these map to protocol uints)
- `lease_request_refuses_shell_trust` (pure validation)

**`cua-dbus`**
- `introspection_matches_xml`: fails until `dbus/org.quire.Cua1.xml` equals the skeleton's introspection, as in porter.
- `error_names_cover_control_refusal`.

**`cuad`**
- `end_to_end_scripted_run_done`, over `cua-fake::run_harness` with a zbus p2p connection on a socketpair, `ScriptedModel` replaying a click-then-done cassette, `FakeSeat`, `FakeCapture` and `FakePdp`. It asserts the signals in order, two records, and that the lease is released.
- `start_refused_for_non_planner_caller`.
- `kill_chord_finishes_all_runs`.
- `stock_consent_guard_injects_nothing_while_prompt_shown`.

**Fakes needed (`cua-fake`):**
- `FakeEis`: a reis `eis` server in-process.
- `FakeCapture`: scripted frames and damage.
- `FakeDesktop`: windows, events and caps per `BackendKind`.
- `FakeA11y`: loads the fixtures.
- `ScriptedModel`: built on models' replay cassettes.
- `FakePdp`: a verdict table.
- `MemoryEvents`: an eventlog sink.
- `FixedClock`.

---

## 6. Spikes, then work breakdown

Spikes run on throwaway compositors only, with a private runtime dir and a private D-Bus bus (shell-host's `accept-lock.sh` pattern). The person's `wayland-0` is never touched. Spike code lives in `cua/spike/` and `scripts/dev/spike-*.sh` and writes `scripts/dev/out/spike-*.json`. The scripts reuse `~/shell-host/scripts/dev/{nested.sh,kwin-virtual.sh}` through `SHELL_HOST_DIR`.

**Spike A: stock cosmic-comp 1.8, nested (3 days)**
1. **Capture.** Capture one toplevel through `ext_foreign_toplevel_image_capture_source` plus ext-image-copy-capture, at scale 1.0 and 1.5. Pass if:
   - frame size equals `round(logical × scale)`;
   - p95 capture latency is under 50 ms at 1920×1080 shm;
   - damage-quiet settling detects the end of a GTK animation within quiet + 50 ms.
   Record whether frames arrive for an occluded window, a minimized window, and a window on another workspace (this decides `HiddenCapture` for stock).
2. **Inject.** Introspect `com.system76.CosmicComp.Ei.GetSenderSocket` (record its signature and whether a non-portal caller is allowed; if not, measure the `RemoteDesktop.ConnectToEIS` portal path). Then a reis client clicks at window-local points read by a probe client that prints `wl_pointer` coordinates. Pass if:
   - the click lands within ±1 lpx after the geometry transform;
   - `ctrl+s` arrives as the chord;
   - one scroll detent arrives as one detent.
   Text `héllo 你好`: record which characters fail on EiKeymap and on virtual-keyboard.
3. **Matrix.** GTK4 (gtk4-demo), Qt6, Firefox, Chromium, foot and a Blitz app: open a menu and select, combo box, slider drag, drag-and-drop, text entry, scroll. Record the pass rate.
4. **Interference.** Show whether EI moves the user's cursor and steals focus. Expected yes, which confirms fork F1.
5. **Coordinate frames.** Record the offsets between the capture frame, xdg window geometry (CSD shadows), AT-SPI window extents and the EI region.
**Go:** items 1 and 2 pass → `CosmicStock` is a backend (AgentWorkspace only if hidden windows render; InPlace = HandsOff).

**Spike B: KWin `--virtual` (2 days).** Capture through `org.kde.KWin.ScreenShot2.CaptureWindow` or `zkde_screencast` stream_window (PipeWire). Inject through `org.kde.KWin.EIS.RemoteDesktop`. Run the Spike A matrix subset. **Go:** KWin becomes the nested-session host and the portability reference (`KwinStock`).

**Spike C: fork agent seat (1 week, needs the fork).** A second smithay seat bound to a lease-scoped EIS context. Pass if:
- the agent clicks in window A while the physical pointer hovers B, the user's cursor does not move and B keeps keyboard focus;
- menus, popups and combo boxes work for at least 80% of the matrix in GTK4, Qt6, Firefox and Chromium;
- 0 agent events reach layer-shell, session-lock or consent surfaces.
**No-go** (under 80%): fork mode "focus borrow" (EI on the user's seat but gated to the lease, with physical input pausing the run) becomes the baseline.

**Spike D: masking (3 days, fork).** Check which toolkits declare `content_purpose = password` through text-input-v3, and which expose role `PasswordText` over AT-SPI: GTK4 entry, Qt6, Firefox, Chromium. Pass if the mask fully covers the field in 100% of declared cases. Record the undeclared ones (the honest limit).

**Spike E: a11y (2 days).** AT-SPI snapshots of GTK4, Qt6, Firefox, Chromium (`--force-renderer-accessibility`) and a Blitz app (accesskit_unix). Pass if:
- p95 snapshot time is under 150 ms for 400 or fewer nodes;
- bounds are correct within ±2 lpx in window coordinates;
- the window match (pid + title) is unambiguous.
Also check whether accesskit can carry `TypedHint` (a custom property or `author_id`).

### Freeze wave (one agent per repo)

Prerequisite: models' `cua-action`, actions' `prov` and `quire-intents` types, and memory's `eventlog` record seam are frozen first, or in the same wave with this spec's §3.0 as the contract. Spikes A, B and E run in parallel; they need no types.

- **Agent `cua-freeze`** owns all of `~/cua`:
  - the workspace, CONVENTIONS, ARCHITECTURE (crate map, one-home table, traits, recipes), FINDINGS (every `todo!()`);
  - `dbus/org.quire.Cua1.xml` and `protocols/quire-agent-v1.xml`;
  - every crate's types and traits, with `todo!()` bodies for behaviour;
  - all shape tests in §5 that need no behaviour: wire round-trips, slugs, introspection, fixtures load;
  - `scripts/check-boundary.sh`;
  - joining reis and atspi into quire's `docs/workspace-deps.toml` first (a quire edit, coordinated).
- **Agent `fork-setup`** owns `~/cosmic-comp`: fork at the upstream 1.8 tag, branch `quire`, an empty `src/quire/mod.rs` hooked in, `quire-agent-protocol` (server) advertising stub globals. It builds and runs nested.

### Fill waves (disjoint file ownership)

- **Fill 1 (pure, 4 agents in parallel)**
  1. `crates/cua-run/src/{run/**, budget.rs, stall.rs, effect_class.rs, lease_book.rs, mode.rs, turn.rs}`
  2. `crates/a11y-tree/src/{prune.rs, render.rs, diff.rs, secret.rs}` plus fixtures
  3. `crates/seat-input/src/{plan.rs, chord.rs, keymap.rs, region.rs}`
  4. `crates/maskcap/src/{apply.rs, digest.rs, settle.rs}`
- **Fill 2 (backends, 4 agents; each after its spike)**
  1. `crates/seat-input/src/ei/**` plus `crates/cua-fake/src/eis.rs`
  2. `crates/maskcap/src/wayland/**`
  3. `crates/a11y-tree/src/atspi/**`
  4. `crates/cua-desktop/src/cosmic/**`
- **Fill 3 (1 agent):** `crates/cuad/**`, `crates/cua-dbus/src/codec.rs`, `crates/cua-fake/src/{harness.rs, model.rs, policy.rs, events.rs, desktop.rs, capture.rs}`, and `dist/`.
- **Fill 4 (1 agent, `~/cosmic-comp` `src/quire/**` only):** fork changes F1-F7 below, then Spikes C and D become acceptance scripts in `~/cua/scripts/dev/accept-{agent-seat,masking}.sh` (with `COSMIC_COMP=~/cosmic-comp/target/release/cosmic-comp`).
- **Fill 5 (later):** procedures (`cua-run/src/procedure.rs`, plus replay in cuad), NestedSession mode (KWin host), the KWin backend, the `quire_sensitive_v1` client in shell-host.

### Fork changes (cosmic-comp) against stock

| # | Change | On stock, instead |
|---|---|---|
| F1 | Agent seat: a smithay `Seat` per lease. Its EIS context comes from `lease.ei_socket`; focus is limited to the leased toplevel's surface tree (its popups and subsurfaces); its own cursor sprite. Upstreamable form: implement `ext-transient-seat-v1` and bind EI contexts to it; the lease filter stays ours | EI through `GetSenderSocket` lands on the user's seat: `SharedSeat`, InPlace = HandsOff |
| F2 | `InputOrigin {Physical, Agent(lease), RemoteDesktop}` on every event. Consent surfaces, the session lock and shell layer surfaces accept Physical only; the agent seat cannot focus them | cuad stops emulating while its own consent prompt shows (`CuadSuspendsInput`); third-party prompts (polkit) overlapping the window remain a stated residual risk |
| F3 | Masked capture source: masks from text-input password purpose, `quire_sensitive_v1`, cuad's `set_mask_hints`, and sensitive-window rules, all applied before the buffer leaves | stock toplevel source plus `apply_masks` inside cuad |
| F4 | Lease and glow: the compositor draws the glow and the agent cursor from lease state | the shell's orb only (`GlowDraw::ShellOnly`) |
| F5 | Kill chord: a hardware chord handled in the compositor revokes every lease | control-centre `StopAll` only |
| F6 | Settling from capture-session damage | already stock (ext-image-copy-capture frames wait for damage) |
| F7 | Hidden agent-workspace windows keep rendering and receiving frame callbacks (cosmic-comp #2777 withholds them) | depends on Spike A1; if not, AgentWorkspace is Unsupported on stock |

### Verify commands

`~/cua`:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
./scripts/check-boundary.sh
cargo deny check licenses
```

Spikes and acceptance (dev only, never in CI): `scripts/dev/spike-{a,b,e}.sh`, `scripts/dev/accept-{agent-seat,masking}.sh`.

Fork: `cargo build --release && cargo test` in `~/cosmic-comp`, then the `~/cua` acceptance scripts against it.

---

## 7. Dependencies on other areas, and questions for the user

**models**
- `cua-action` exactly as §3.0: integer `Lpx`, `ActionTarget::Node`, `CuaTurn`, `StepReply`, `StepEvent`.
- `CuaModel`/`CuaSession` with cooperative cancel, and the `Transient`/`Parse`/`Fatal` error split.
- The replay provider for `ScriptedModel`.
- `Need::Cua` added to porter and 31-ACCOUNTS, with `DataClass::Screen` keeping the OnDevice floor.
- Frame transport to inferd as an fd/memfd, never base64 on the bus.

**actions**
- `prov`: `Labelled<T>`, `Label`, `Source::Screen{app, window}`, join.
- `policy-point`: `decide(AuthAsk-shaped request) -> Allow | Ask(ConfirmText) | Deny`, with Cedar context fields `effect`, `taint`, `window_trust`, `app_id`, `mode`, `space`, `data_class = screen`.
- The CUA grant store keyed by `(AppName, SpaceId)` with Once/Always, denial wins, and a "sensitive classes never Always" list.
- `quire-intents` `Context`, `Value`, `ActionName`, `Effect` (with a `From` mapping to `EffectClass`).
- The router's tier decision that calls `Start`.

**memory**
- `eventlog` append taking `CuaRecord` as a namespaced kind with actor `Companion(CuaRun)`.
- The Space's encrypted thumbnail store, with forget cascade, for `FrameNote::Kept`.
- `memfiles` folder `procedures/` (later).
- Supplying the user's recent events for `HandedBack.user_events` and Handover starts.

**launcher / companion UX (sill)**
- `CuaRunProvider` in sill-launcher; a new `Activation::Cua(RunControl)` variant (following the warn-before-lint-rules practice).
- The orb presence mapping from §4.
- The Ask form calls `Run.Answer`.

**accounts / porter**
- `AppId` derived from the connection, for cuad's caller checks.
- accounts-ui draws the CUA grant prompt on stock.

**design/22-SETTINGS** rows needed, each with its proposed default:

| Key | Proposed |
|---|---|
| `cua.enabled` | off |
| `cua.default_mode` | agent_workspace |
| `cua.budget.steps` | 40 |
| `cua.budget.active_s` | 600 |
| `cua.budget.stall_frames` | 3 |
| `cua.budget.actions_per_min` | 60 |
| `cua.budget.denials` | 3 |
| `cua.budget.model_failures` | 2 |
| `cua.budget.spend_microusd` | 0 = local only |
| `cua.settle.quiet_ms` | 250 |
| `cua.settle.timeout_ms` | 3000 |
| `cua.wait.max_ms` | 5000 |
| `cua.autopause` | on |
| `cua.tree.max_nodes` | 400 |
| `cua.screens.keep` | off |
| `cua.screens.keep_days` | 7 |

**Questions only the user can answer**
1. Confirm replacing COMPANION item 4 with: "typed actions and hooks first; a CUA tier, off by default per app, inside the lease and consent stack."
2. Ship v1 on stock cosmic-comp (shared seat, HandsOff in place, masking inside cuad, consent guard by suspending input), with the fork as phase 2? Or require the fork before any release? My recommendation is stock first; the fork is required before "keep working while it acts".
3. The repo name (`cua` is a placeholder), and confirmation that `cuad` is its own daemon rather than part of shell-host.
4. Should actions target a11y nodes (`ActionTarget::Node`) when a tree exists? My recommendation is yes, and to freeze both forms now.
5. Default run mode: agent workspace (out of sight, glow plus peek) or in place?
6. The budget defaults above, and who may raise them.
7. Which hardware kill chord the compositor reserves.
8. Browser grants: Once/per-session only, or is "Always" allowed?
9. CJK and Unicode typing: may the agent use the virtual-keyboard keymap trick on stock (it targets the user's focus), or only the fork's agent text-input?
10. Procedures: auto-propose promotion to a typed action, or only on an explicit "save as routine"?
11. Is KWin a product target (portable CUA) or only the nested-session host and spike reference? And who owns the cosmic-comp fork overall (cua, or a compositor area shared with the double-tap ⌘ work)?
12. Keep screenshots off by default with no thumbnails? Recommended: yes.

### Critical files for implementation
- /home/pohsuanlai/rs-wt/companion/cua-integration.md
- /home/pohsuanlai/porter/ARCHITECTURE.md (the repo shape, boundary script and frozen-trait conventions to copy)
- /home/pohsuanlai/quire/design/31-ACCOUNTS.md (§4.4 Inference1, §4.5 consent, §5.5 data-class floors)
- /home/pohsuanlai/shell-host/scripts/dev/README.md (the nested cosmic-comp and headless KWin harness the spikes reuse)
- /home/pohsuanlai/sill/crates/sill-launcher/src/activation.rs and provider.rs (where the run strip plugs in)

<!-- paths: end -->
