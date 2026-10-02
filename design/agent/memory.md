<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/memory.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# Area: memory — implementation spec (plan for the freeze)

Status: PROPOSAL to lock. Sources: COMPANION.md item 11, Security items 1, 2 and 6, Deferred; research-agent-memory.md §7; research-os-memory.md §8 P1–P10; research-ecosystem-infra.md §2 and §7 (crates 2–4); cua-integration.md §6; intents-research/models.md; design/31 and porter (the code shape is copied from porter); design/21-SPACES; quire CONVENTIONS.md and docs/workspace-deps.toml.
Marks: **[freeze now]** means the type, signature or format is fixed in the freeze wave, with `todo!()` bodies where the behaviour is not built yet. **[later]** means the place is named now and the shape is decided when the work happens. **[Q]** means only the user can answer.

---

## 1. Scope and non-goals

**In scope**
- **`eventlog`.** The per-Space event log. It is typed, append-only and hash-chained, and it is also the audit log. Each event records: actor, provenance label, Space, effect, the things it is about and the things it came from. Retention can erase an event's body without breaking the chain. Old prefixes can be pruned behind checkpoints.
- **`memfiles`.** The per-Space agent memory as plain markdown files. Facts are add-only, dated, and linked to their sources. Facts derived from untrusted text are held in a pending area until the user confirms them. Deleting a source cascades. A nightly consolidation produces a diff the user can review and revert.
- **`recall`.** A rebuildable index. FTS5 lexical search through rusqlite. An `Embedder` trait and a `VectorIndex` trait, with exact scan as the default backend and sqlite-vec as a swappable backend. Results from both are fused (RRF).
- Per-Space keys and encryption at rest; XDG storage layout.
- Retention and "do not remember" rules, keyed by thing type, app, path or Space; pausing memory.
- File-change capture through inotify now and fanotify later. The watcher's observations are joined with the reasons apps supply ("why").
- **`memoryd`**, the daemon that owns all of the above, serving the D-Bus name **`org.quire.Memory1`**. It covers record, recall, forget, timeline and export, plus pending facts, consolidation, rules, pause and verify.
- The data the user control UI needs: timeline pages, "what it knows about X", forget preview and execution, export, pending facts, the consolidation diff, rules and status. **Drawing that UI is the "ux" area.**

**Non-goals (owned elsewhere)**
- The planner's working memory and the handover context (query, results, selection) belong to the agent-loop area. Only what the user confirms reaches `memfiles`.
- The launcher's entity "shadow index" belongs to the intents/actions router. It may reuse the `recall` crate, but it is not memoryd's data.
- The policy decision point and Cedar belong to actions. memoryd only stores the decisions as audit events.
- AT-SPI capture of third-party apps belongs to the a11y/CUA areas. They record through the same `Record` API (`Channel::Atspi`).
- Screenshots: none are stored by memoryd. CUA step thumbnails are the CUA area's own store (cua-integration §6.2).
- **Deferred, room left but not decided:** sync between machines, bi-temporal validity, and whether other people's data counts as memory (§3.11).

---

## 2. Repos and crates

### 2.1 Placement

A **new repo `~/almanac`** (github.com/PoHsuanLai/almanac). It is shaped like porter: a workspace, frozen interfaces, `todo!()` stubs listed in FINDINGS.md, `scripts/check-boundary.sh`, and `dbus/*.xml` checked against the skeletons. The daemon is **`memoryd`**, named for the concept like porter's `accountd`, `inferd` and `syncd`. The bus name is `org.quire.Memory1`.

Why a separate repo: memory is not accounts. The three owned crates (`eventlog`, `memfiles`, `recall`) are meant to be reusable. memoryd also has its own confinement unit.

**Portability boundary.**
- Builds on macOS and Windows, with no zbus, tokio, oo7 or inotify: `almanac-core`, `almanac-seal`, `eventlog`, `memfiles`, `recall`, `almanac-service`, `almanac-client` (without its `dbus` feature).
- Linux only: `almanac-watch`, `almanac-dbus`, `memoryd`.

### 2.2 Crates

| Crate | Purpose | Direct deps (ours) | External |
|---|---|---|---|
| `almanac-core` | Vocabulary: `ThingRef`, `ThingKind`, `KindPattern`, `Verb`, `EventBody`, `EventHeader` fields, `EventRef`, `FactId`, `Fact`, `Link`, `RememberRule`, `Retention`, `ForgetScope`, the wire `MemoryRequest`/`MemoryReply`, timeline, export manifest, `Caller` | porter-core, prov | serde, serde_json, thiserror |
| `almanac-seal` | `SpaceKey`, subkey derivation, `DbKey`, `seal`/`unseal` for files, the `KeyStore` trait, `MemoryKeys` (feature `testing`), `Oo7Keys` (stub) | almanac-core | blake3, chacha20poly1305, zeroize, getrandom; oo7 behind feature `oo7` |
| `eventlog` | Canonical header bytes, the chain (`link`, `verify_chain`), the `LogRead`/`LogWrite` traits, `SqliteLog` (SQLCipher), `MemoryLog` (feature `testing`), checkpoints | almanac-core, almanac-seal | rusqlite, blake3 |
| `memfiles` | Topic file format (`parse_topic`/`render_topic`), `Store`, the `Vault` trait, `PlainDir`, `SealedDir`, `MemoryVault` (testing), pending, procedures folder, cascade lookup | almanac-core, almanac-seal | jiff |
| `recall` | Traits `Embedder` and `VectorIndex`; `Fts5` lexical; `ExactScan` vectors; `chunk`; `fuse_rrf`; `Index` (rebuild/upsert/search); `FakeEmbedder` (testing). Generic: knows nothing of almanac-core. | none | rusqlite |
| `recall-fastembed` | `FastembedEmbedder` (in-process CPU/CUDA ONNX). Kept out of `recall` so ort's binary download never enters the gate. | recall | fastembed 7.1, ort 2.0.0-rc.13 (via fastembed) |
| `recall-sqlite-vec` [later, Q6] | `SqliteVec: VectorIndex` | recall | sqlite-vec 0.1.10-alpha.4 |
| `almanac-service` | memoryd's core over seams: admission, forget planner, fact pipeline, consolidation machine, retention sweep, timeline assembly, export writer, Space lifecycle; traits `Consolidator` and `Clock` | core, seal, eventlog, memfiles, recall | tar |
| `almanac-watch` | `FileWatch` trait, `InotifyWatch` (notify 8.2), `FanotifyWatch` [later], the pure `join` of observed changes and app-supplied reasons | almanac-core | notify, nix [later] |
| `almanac-dbus` | `org.quire.Memory1` proxies and skeletons, introspection, the codec (stub) | almanac-core | zbus |
| `almanac-client` | App-facing `Memory` API with a `Transport` trait: `InProcess`, `Absent` (no-op on other desktops), `DbusTransport` (feature `dbus`) | core, service; dbus (feature) | — |
| `almanac-fake` | Test harness: `fake_service`, `FixedClock`, `ScriptedConsolidator`, fixtures, golden files | core, seal, eventlog, memfiles, recall, service | — |
| `memoryd` | The daemon. Wiring, the system clock, `InferdEmbedder` and `InferdConsolidator` (through porter-client), the systemd unit | all of the above + porter-client | tokio, zbus, oo7 |

**Allowed edges** (in `check-boundary.sh`, porter style):
- `almanac-core` → `porter-core`, `prov`.
- `almanac-seal` → `almanac-core`.
- `eventlog` and `memfiles` → `almanac-core`, `almanac-seal`.
- `recall` → nothing of ours.
- `almanac-service` → `core`, `seal`, `eventlog`, `memfiles`, `recall`.
- `almanac-watch` and `almanac-dbus` → `almanac-core`.
- `almanac-client` → `core`, `service`, and `dbus` only behind its feature.
- `memoryd` → everything, plus `porter-client`.

**Forbidden reach** (any path, default features): every crate except `almanac-dbus`, `almanac-client[dbus]` and `memoryd` must never reach `zbus zvariant tokio reqwest hyper oo7 ort fastembed`. `almanac-core` must also never reach `rusqlite` or `notify`.

### 2.3 Adopted externals (versions as surveyed 2026-10-02; re-check at freeze)

- **From quire's pinned block:** serde 1, serde_json 1, thiserror 2, zbus 5.19 (tokio), tokio 1, jiff 0.2, **notify 8.2**. The ecosystem survey named notify 9.0.0-rc.5; use the pinned 8.2. A bump is a toolchain-wave decision.
- **New lines for `quire/docs/workspace-deps.toml` (wave 0):**
  - `rusqlite = { version = "0.40", features = ["bundled-sqlcipher-vendored-openssl"] }`. mailo is on 0.40 `bundled`, in a separate tree, so there is no conflict. Spike S2 verifies that FTS5 is compiled in.
  - `blake3 = "1"`, `chacha20poly1305 = "0.10"` (XChaCha20-Poly1305), `zeroize = "1"`, `getrandom = "0.3"`, `tar = "0.4"`, `tempfile = "3"` (dev only).
  - `oo7 = "0.6"`. porter is also waiting for this line.
  - `fastembed = "7.1"`, used only by `recall-fastembed`.
  - `nix = "0.30"` for fanotify [later].
- **porter-core** (git dependency at the frozen rev): `AppName`, `AppId`, `DataClass`, `Usage`, `UnixSeconds`, `Count`, `Bytes`, `ModelId`, `Dims`. porter-core is pure and portable, so there is one home per concept.
- **prov** (actions area): `Label`, `Integrity`, `Confidentiality`, the `Confirmation` witness, and probably `SpaceId` and `Actor` (§7).

---

## 3. Frozen interfaces

### 3.1 Shared identity (home: `prov` if the actions area agrees, else `almanac-core`) [freeze now]

```rust
/// One Space: a hard wall for memory. Stable across sessions (unlike COSMIC workspace ids,
/// design/21 §11.3); minted once, slug grammar of porter-core::id.
pub struct SpaceId(String);                 // "work", "home", "s-01j9zk..."; reserved: "desktop" [Q2]

/// Who did it. Shared-undo labels (COMPANION item 10) use the same type.
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum Actor {
    User { via: AppName },                          // reported by the app the user acted in
    Companion { session: SessionId, tier: AgentTier },
    App { app: AppName },                           // an app acting on its own (sync, rules)
    ThirdParty { app: AppName, channel: Channel },  // non-quire app, observed with consent
    System { part: SystemPart },                    // memoryd itself, the router, the watcher
    Unknown,                                        // a file change nobody explained
}
pub enum AgentTier { Planner, Reader, Cua { run: RunId } }
pub enum Channel { Atspi, Dbus, Portal, Mpris, FileWatch }
pub enum SystemPart { Memory, Router, Watcher, Consolidation, Retention }
pub struct SessionId(String);  pub struct RunId(String);   // slug grammar
```

### 3.2 almanac-core vocabulary [freeze now]

```rust
/// A typed thing an app owns. Same identity as the intents EntityId; if actions freezes
/// `EntityId` in a light crate, ThingRef becomes a re-export of it (one home).
pub struct ThingRef { pub app: AppName, pub kind: ThingKind, pub key: ThingKey }
pub struct ThingKind(String);      // "mail.thread", "files.file", "calendar.event"
pub struct ThingKey(String);       // app-stable, opaque, <= 256 bytes, no control chars
pub struct KindPattern(String);    // "mail.*", "files.file", "*"
pub struct ActionName(String);     // "mail.thread.archive" (re-export from intents when frozen)

/// A thing as the user saw it when the event happened (forgettable, lives in the body).
pub struct ThingView { pub thing: ThingRef, pub title: String, pub subtitle: String }

pub enum Verb {
    Opened, Viewed, Created, Edited, Saved, Renamed, Moved, Copied, Deleted, Archived,
    Restored, Sent, Received, Replied, Forwarded, Shared, Downloaded, Imported, Exported,
    Searched, Selected, Pinned, Unpinned,
}
/// The four effects of COMPANION item 9 (home: actions; re-exported).
pub enum Effect { Read, UndoableWrite, Outbound, Destructive }

/// One event's typed body. `kind()` gives the header's KindTag; one match, here.
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum EventBody {
    Thing   { verb: Verb, thing: ThingView, sources: Vec<ThingView> },
    Action  { action: ActionName, targets: Vec<ThingView>, outcome: ActionOutcome, undo: UndoRef },
    File    { change: FileChange, file: FileView, why: FileWhy },
    Search  { app: AppName, text: String, scope: ThingKind, results: Count },
    Session { session: SessionId, step: SessionStep },          // companion session start/prompt/end
    Cua     { run: RunId, step: CuaStep },                      // cua.* (cua-integration §6.1), opaque to us
    Policy  { request: PolicyRequestView, decision: Decision }, // audit from policy-point
    Consent { grant: GrantView, answer: ConsentAnswerView },
    Memory  { op: MemoryOp },                                   // memoryd's own audit
}
pub enum ActionOutcome { Pending, Done, Refused, Failed, Undone }
pub enum UndoRef { None, Token(String) }
pub enum FileChange { Created, Modified, Renamed { from: SpacePath }, Deleted, Closed }
pub struct FileView { pub path: SpacePath, pub inode: u64, pub content: ContentDigest }
pub enum FileWhy { Explained { cause: ThingRef, verb: Verb, by: Actor }, Unexplained }
pub enum MemoryOp {
    FactAdded { fact: FactId, topic: TopicPath }, FactConfirmed { fact: FactId },
    FactRejected { fact: FactId }, Forgot { plan: PlanDigest, counts: ForgetCounts },
    Read { by: Actor, scope: ReadScope, facts: Vec<FactId>, events: Count },
    Consolidated { run: RunId, hunks: Count }, Reverted { run: RunId },
    Exported { counts: ExportCounts }, Paused { until: UnixSeconds }, Resumed,
    RuleChanged { rule: RuleId }, Checkpoint { cut: Seq, link: Link32 },
    Dropped { count: Count, reason: DropReason },
}
/// Policy/Consent/Cua/Session payload structs are owned by their areas and re-declared here
/// with exactly their serde form (round-trip tested against their bytes) [later per area].

/// What a writer sends. Stamped by memoryd with seq, recorded time, replica and caller.
pub struct Record {
    pub space: SpaceId,
    pub occurred: UnixSeconds,        // when it happened (event time)
    pub actor: Actor,                 // checked against the caller class (§3.8)
    pub effect: Effect,
    pub label: prov::Label,
    pub body: EventBody,
    pub cause: Cause,                 // None | Event(EventRef) | Undo(String) | Plan(SessionId)
}

pub struct Seq(pub u64);                         // per (space, replica), from 1, gapless
pub struct ReplicaId(pub [u8; 16]);              // one per Space per machine: room for sync
pub struct EventRef { pub space: SpaceId, pub replica: ReplicaId, pub seq: Seq }
pub struct KindTag(String);                      // "thing.archived", "action.performed", "file.created"
```

**Retention and "do not remember"** [freeze now]:
```rust
pub struct RuleId(String);
pub struct RememberRule { pub id: RuleId, pub scope: RuleScope, pub mode: RememberMode, pub retention: Retention }
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum RuleScope { Space(SpaceId), App(AppName), Kind(KindPattern), Path(PathGlob), Thing(ThingRef), Actor(ActorClass) }
pub enum RememberMode { Full, HeaderOnly, Never }
pub enum Retention { Days(DayCount), WhileSourceExists, UntilForgotten }
pub struct RuleSet { pub rules: Vec<RememberRule>, pub defaults: Vec<(KindPattern, Retention)> }

pub enum Admission { Keep { retention: Retention }, HeaderOnly { retention: Retention }, Drop(DropReason) }
pub enum DropReason { Paused, Never(RuleId), ThingMarked, SpaceLocked, CallerNotAllowed, SpaceUnknown }

/// Pure. Audit-class events (actor Companion or System(Router), body Policy/Consent/Action/Cua)
/// are never Dropped: at worst HeaderOnly. Most specific scope wins (Thing > Path > Kind > App >
/// Actor > Space); at equal specificity Never > HeaderOnly > Full.
pub fn admit(record: &Record, rules: &RuleSet, state: &SpaceState, marks: &Marks) -> Admission;
```

**Default retention** (proposed values; these become design/22 settings keys under `memory.retention.*`):

| Kind | Default |
|---|---|
| `search`, selections | 30 days |
| unexplained `file.*` | 7 days |
| things (mail, files, events) | `WhileSourceExists` |
| `session.*`, `cua.*` | 30 days |
| policy, consent, memory audit | bodies 90 days, headers 1 year |
| facts | `UntilForgotten` |

### 3.3 Facts [freeze now]

```rust
pub struct FactId(String);             // 26-char lowercase ULID-style: time + 80 random bits (sync-safe)
pub struct TopicPath(String);          // "people/sam-lee", "prefs/meetings"; [a-z0-9-/], <= 4 levels
pub struct Fact {
    pub id: FactId,
    pub text: FactText,                // one paragraph, <= 2 KiB, no metadata comment syntax inside
    pub recorded: UnixSeconds,         // when learned
    pub by: Actor,
    pub label: prov::Label,
    pub links: Vec<Link>,              // at least one, except user-said facts
    pub supersedes: Vec<FactId>,
    pub valid: Validity,               // room for bi-temporal; v1 writes and reads only Unstated
}
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum Link { Event(EventRef), Thing(ThingRef), Fact(FactId), Run(RunId) }
pub enum Validity { Unstated }                      // deferred: From/Until/Between later
pub enum FactState { Pending, Active, Superseded { by: FactId } }   // derived, never stored
pub struct FactDraft { pub topic: TopicPath, pub text: FactText, pub links: Vec<Link>, pub supersedes: Vec<FactId> }
```

### 3.4 File formats [freeze now]

**Layout (XDG; all paths come from injected `Dirs`, never from `dirs::*` deep in the code):**
```
$XDG_DATA_HOME/quire/memory/
  spaces.toml                         # SpaceMeta per Space: id, created, replica, vault kind, format
  system/events.db                    # desktop-level log: Space created/deleted, checkpoint anchors
  <space>/events.db (+ -wal)          # eventlog, SQLCipher, key = derive(space, "eventlog")
  <space>/facts/INDEX.md              # primer, regenerated by consolidation, <= 200 lines
  <space>/facts/<topic>.md            # topic files (sealed or plain per vault)
  <space>/pending/<fact-id>.md        # untrusted-derived drafts awaiting confirmation
  <space>/procedures/<app>/<name>.md  # CUA procedures (format: cua area; same metadata trailer)
  <space>/consolidation/<run>.toml    # the diff of one run + pre-images for revert (kept until next run)
$XDG_CACHE_HOME/quire/memory/<space>/index.db   # recall index, SQLCipher; deletable, rebuilt
$XDG_CONFIG_HOME/quire/memory.toml               # RuleSet + defaults (written only by memoryd)
$XDG_RUNTIME_DIR/quire/memory/edit/<space>/...   # decrypted edit copies (tmpfs), Q1
```

**Topic file** (the canonical plain form; sealed vaults store exactly these bytes, sealed):
```markdown
---
format: quire-memory 1
topic: people/sam-lee
title: Sam Lee
---

## 2026-10-01

- Prefers meetings after 10:00.
  <!-- fact: 01j9zk3m0q8h2v6x4c1b7n5t2a; at: 2026-10-01T09:12:44Z; by: user/org.quire.Mail; trust: user; from: thing org.quire.Mail mail.thread 7f3a -->
- Works on the Q4 roadmap with Ana.
  <!-- fact: 01j9zm...; at: 2026-10-01T11:02:10Z; by: companion; trust: inferred; from: ev work 3c1f..:430; supersedes: 01j9zk... -->
```

Format rules:
- Date headings are derived from `at` in the injected time zone.
- A bullet with no comment is a user edit, parsed as `Block::Unstamped`. Consolidation stamps it (`by: user`, `trust: user`) in its diff.
- Unknown lines are kept as `Block::Verbatim`.
- The keys `valid:` and `origin:` are reserved and must not be interpreted yet (bi-temporal and sync).

Contract: `render_topic(&parse_topic(s)?) == s` for any `s` that `render_topic` produced. Parsing is strict on the comment grammar and lenient everywhere else.

**Sealed file:** `b"QMEM\x01"` ‖ 24-byte nonce ‖ XChaCha20-Poly1305(ciphertext). The associated data is `space ‖ vault-relative path`, so a sealed file moved to another path or another Space fails to open. The key is `derive(space_key, "quire-memory 1 files")`.

**Event header canonical bytes (v1)**, the input to the chain:
```
"QEL1" | seq u64be | replica [16] | occurred i64be | recorded i64be
| lp(actor json) | lp(kind tag) | effect u8 | lp(label json) | lp(cause json)
| body_digest [32] | prev_link [32]
lp(x) = u32be length ‖ bytes; json = serde_json of the typed value (declaration order; golden-pinned)
body_digest = blake3::keyed_hash(derive(space_key,"digest"), body json)   // erased bodies can't be guessed
link        = blake3(header bytes)          // genesis prev_link = blake3("QEL1 genesis" ‖ space ‖ replica)
```
The header holds no thing ids or text. Subjects and sources are in the body and in the `things` table, both erasable.

**events.db schema** (SQLCipher, `PRAGMA secure_delete=ON`, WAL; `wal_checkpoint(TRUNCATE)` after an erase):
`events(seq PK, occurred, recorded, actor, kind, effect, label, cause, body_digest, prev, link)`,
`bodies(seq PK, json)`, `things(app, kind, key, seq, role)` with `role ∈ {subject, source}`,
`aliases(app, kind, old_key, new_key)` (file renames), `checkpoints(cut, link, at)`, `meta(format, replica)`.

**Export** (the fd receives a tar stream) [freeze now]:
```
quire-memory-export-1/manifest.json   # ExportManifest: format, spaces, counts, created, chain heads
<space>/events.jsonl                   # one line per entry: header fields + link + body-or-"erased"
<space>/facts/**.md  <space>/procedures/**.md  <space>/pending/**.md   (plain, never sealed)
rules.toml
```
The index and the keys are never exported. `events.jsonl` can be verified by third parties for the chain. Bodies can only be checked against their digests by someone holding the digest subkey, which is exported only if the user ticks "include verification key" [Q10].

### 3.5 almanac-seal [freeze now]

```rust
pub struct SpaceKey(/* [u8;32], zeroize, redacted Debug, no serde */);
pub struct DbKey(/* hex pragma key derived from SpaceKey, redacted Debug */);
pub enum Purpose { Eventlog, Index, Files, Digest }
pub fn derive(key: &SpaceKey, space: &SpaceId, purpose: Purpose) -> SubKey;    // blake3::derive_key
pub fn seal(key: &SubKey, aad: &Aad, plain: &[u8], nonce: Nonce) -> Vec<u8>;     // nonce injected (testable)
pub fn unseal(key: &SubKey, aad: &Aad, sealed: &[u8]) -> Result<Vec<u8>, SealError>;
pub enum SealError { BadMagic, Truncated, Unauthentic, UnknownVersion(u8) }

pub trait KeyStore: Send + Sync {
    fn get(&self, space: &SpaceId) -> impl Future<Output = Result<SpaceKey, KeyError>> + Send;
    fn create(&self, space: &SpaceId) -> impl Future<Output = Result<SpaceKey, KeyError>> + Send;
    fn destroy(&self, space: &SpaceId) -> impl Future<Output = Result<(), KeyError>> + Send;
}
pub enum KeyError { Locked, Missing, Store(String) }
// Impls: MemoryKeys (testing), Oo7Keys (stub until oo7 is pinned; Secret Service item
// attributes { "xdg:schema": "org.quire.Memory.SpaceKey", "space": <id> }).
```

### 3.6 eventlog [freeze now]

```rust
pub struct Head { pub seq: Seq, pub link: Link32 }
pub struct Entry { pub header: Header, pub link: Link32, pub body: BodyState }
pub enum BodyState { Present(EventBody), Erased }
pub struct Header { pub seq: Seq, pub replica: ReplicaId, pub occurred: UnixSeconds, pub recorded: UnixSeconds,
                    pub actor: Actor, pub kind: KindTag, pub effect: Effect, pub label: prov::Label,
                    pub cause: Cause, pub body_digest: Digest32, pub prev: Link32 }
pub fn header_bytes(h: &Header) -> Vec<u8>;                 // pure, golden-tested
pub fn link(h: &Header) -> Link32;
pub fn verify_chain(from: &Checkpoint, entries: &[Entry], digest_key: &SubKey) -> ChainReport;
pub enum ChainReport { Intact { head: Head, erased: Count }, Broken { at: Seq, why: Break } }
pub enum Break { LinkMismatch, Gap, BodyDigestMismatch, CheckpointMismatch }

pub trait LogRead {
    fn head(&self) -> Result<Head, LogError>;
    fn page(&self, q: &PageQuery) -> Result<Vec<Entry>, LogError>;   // newest-first, cursor, filter
    fn touching(&self, thing: &ThingRef, role: RoleFilter) -> Result<Vec<Seq>, LogError>;
    fn scan(&self, from: Seq) -> Result<Vec<Entry>, LogError>;        // for verify and rebuild
}
pub trait LogWrite: LogRead {
    fn append(&mut self, header: NewHeader, body: Option<EventBody>) -> Result<Entry, LogError>;
    fn erase_bodies(&mut self, seqs: &[Seq]) -> Result<Count, LogError>;
    fn prune_before(&mut self, cut: Seq) -> Result<Checkpoint, LogError>;   // prefix only
}
pub enum LogError { Locked, Corrupt { at: Seq }, Full, Sqlite(String), Schema { found: u32 } }
// Impls: SqliteLog (open(path, &DbKey)), MemoryLog (testing).
```

### 3.7 memfiles [freeze now]

```rust
pub struct VaultPath(String);                    // relative, no "..", validated at construction
pub trait Vault: Send + Sync {
    fn list(&self, dir: &VaultPath) -> Result<Vec<VaultPath>, VaultError>;
    fn read(&self, p: &VaultPath) -> Result<Vec<u8>, VaultError>;
    fn write_atomic(&self, p: &VaultPath, bytes: &[u8]) -> Result<(), VaultError>;  // tmp + rename
    fn remove(&self, p: &VaultPath) -> Result<(), VaultError>;
}
// Impls: PlainDir, SealedDir(PlainDir + SubKey), MemoryVault (testing).

pub struct TopicFile { pub topic: TopicPath, pub title: String, pub blocks: Vec<Block> }
pub enum Block { Fact(Fact), Unstamped(String), Verbatim(String) }
pub fn parse_topic(text: &str) -> Result<TopicFile, ParseError>;
pub fn render_topic(file: &TopicFile, tz: &jiff::tz::TimeZone) -> String;

pub struct Store<V: Vault> { /* vault, space */ }
impl<V: Vault> Store<V> {
    pub fn topics(&self) -> Result<Vec<TopicPath>, MemfilesError>;
    pub fn read(&self, t: &TopicPath) -> Result<TopicFile, MemfilesError>;
    pub fn append(&self, t: &TopicPath, fact: Fact) -> Result<(), MemfilesError>;       // add-only
    pub fn stage(&self, fact: Fact, topic: TopicPath) -> Result<(), MemfilesError>;     // pending/
    pub fn pending(&self) -> Result<Vec<(TopicPath, Fact)>, MemfilesError>;
    pub fn settle(&self, id: &FactId, verdict: Verdict) -> Result<(), MemfilesError>;   // Confirm(Confirmation)|Reject
    pub fn derived_from(&self, links: &[Link]) -> Result<Vec<FactId>, MemfilesError>;  // transitive over Link::Fact
    pub fn remove(&self, ids: &[FactId], plan: &PlanDigest) -> Result<Count, MemfilesError>; // only with a plan
    pub fn write_primer(&self, primer: &Primer) -> Result<(), MemfilesError>;
}
pub enum MemfilesError { Vault(VaultError), Parse { path: VaultPath, err: ParseError }, NotPending(FactId), NoSuchTopic(TopicPath) }
```
`Verdict::Confirm` takes a `prov::Confirmation`, which only the ShellUi caller path can mint (§3.8).

### 3.8 recall [freeze now] (generic, reusable)

```rust
pub struct DocId(pub String);
pub struct Doc { pub id: DocId, pub text: String, pub at: i64, pub facets: Facets }  // facets: kind, app, trust tier
/// Embeddings are floats end to end; so this is PartialEq without Eq.
pub struct Vector(pub Vec<f32>);
pub struct EmbedderCard { pub model: String, pub dims: u32, pub max_tokens: u32, pub metric: Metric }
pub enum Metric { Cosine, Dot }
pub enum Urgency { Interactive, Background }

pub trait Embedder: Send + Sync {
    fn card(&self) -> &EmbedderCard;
    fn embed(&self, texts: &[String], urgency: Urgency)
        -> impl Future<Output = Result<Vec<Vector>, EmbedError>> + Send;
}
// Impls: FakeEmbedder (hashed bag-of-words, deterministic), FastembedEmbedder (recall-fastembed),
// InferdEmbedder (memoryd: porter InferRequest::Embed, Usage::Background, class = doc's DataClass).

pub trait VectorIndex: Send {
    fn card(&self) -> &EmbedderCard;                               // which space the vectors live in
    fn upsert(&mut self, items: &[(DocId, Vector)]) -> Result<(), IndexError>;
    fn remove(&mut self, ids: &[DocId]) -> Result<Count, IndexError>;
    fn nearest(&self, q: &Vector, k: TopK, allow: &Allow) -> Result<Vec<Ranked>, IndexError>;
    fn clear(&mut self) -> Result<(), IndexError>;
}
// Impls: ExactScan (BLOB column in index.db; default), SqliteVec [later, Q6].

pub struct Fts5 { /* rusqlite table in index.db */ }       // one impl: not a trait
pub fn chunk(text: &str, max_tokens: u32) -> Vec<Chunk>;   // pure
pub fn fuse_rrf(lists: &[Vec<Ranked>], k: RrfK) -> Vec<Fused>;   // pure; Fused { id, score_millionths: u32, why }
pub enum HitWhy { Lexical { rank: u32 }, Semantic { rank: u32 }, Both { lexical: u32, semantic: u32 } }

pub struct Index<V: VectorIndex> { /* fts, vectors, meta */ }
pub enum IndexState { Absent, Building { done: Count, total: Count }, Ready, Stale(StaleWhy), LexicalOnly(DegradedWhy) }
pub enum StaleWhy { EmbedderChanged, FormatChanged, TruthNewer }
impl<V: VectorIndex> Index<V> {
    pub fn state(&self) -> IndexState;
    pub async fn rebuild(&mut self, docs: impl Iterator<Item = Doc>, e: &impl Embedder) -> Result<(), IndexError>;
    pub async fn upsert(&mut self, docs: &[Doc], e: &impl Embedder) -> Result<(), IndexError>;
    pub fn remove(&mut self, ids: &[DocId]) -> Result<Count, IndexError>;
    pub async fn search(&self, q: &SearchQuery, e: &impl Embedder) -> Result<Vec<Fused>, IndexError>;
}
pub enum IndexError { Embed(EmbedError), CardMismatch, Sqlite(String) }
pub enum EmbedError { Unavailable, Refused(String), TooLong, Failed(String) }
```
`search` falls back to lexical-only and reports `LexicalOnly` when the embedder is unavailable. Search never fails just because the GPU is busy.

memoryd's documents:
- `DocId = "f:<fact>"` for facts. Text is the fact's text.
- `DocId = "e:<replica>:<seq>"` for event bodies that carry searchable text (titles, search text). Text comes from `ThingView`.
- `DocId = "t:..."` [later], for app-donated thing content.

### 3.9 almanac-service: seams, requests, caller classes [freeze now]

```rust
pub trait Clock: Send + Sync { fn now(&self) -> UnixSeconds; }
pub trait Consolidator: Send + Sync {
    fn draft(&self, input: ConsolidationInput) -> impl Future<Output = Result<Draft, ConsolidateError>> + Send;
}
// Impls: ScriptedConsolidator (fake), InferdConsolidator (memoryd; inferd chat, Task::Extract, Usage::Background).

pub enum Caller { App(AppId), Router, ShellUi, Companion { session: SessionId, space: SpaceId, taint: Taint } }

#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum MemoryRequest {
    // writers
    Record(Record), RecordBatch(Vec<Record>), ExplainFile(FileWhyClaim), Mark(MarkRequest),
    // recall (Companion within its Space, ShellUi anywhere)
    Search(RecallQuery), Facts(FactQuery), Related(ThingRef), Provenance(SpacePath), Primer(SpaceId),
    Propose(SpaceId, FactDraft),
    // control (ShellUi only; Companion may PlanForget but never Forget/Confirm)
    Spaces, Status(SpaceId), Timeline(SpaceId, TimelineQuery),
    PlanForget(SpaceId, ForgetScope), Forget(PlanToken),
    Pending(SpaceId), Settle(FactId, Verdict),
    Consolidation(SpaceId), RunConsolidation(SpaceId), Revert(RunId),
    Rules, SetRule(RememberRule), RemoveRule(RuleId),
    Pause(SpaceId, UnixSeconds), Resume(SpaceId), Verify(SpaceId), Rebuild(SpaceId),
    Export(ExportOptions),          // the fd travels beside the request (D-Bus `h`)
}
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum MemoryReply { Recorded(EventRef), RecordedBatch(EventRef, Count), Ok, Hits(Vec<RecallHit>),
    Facts(Vec<FactView>), Related(Vec<EventSummary>), Provenance(FileProvenance), Primer(String),
    Proposed(FactId, FactState), Spaces(Vec<SpaceSummary>), Status(SpaceStatus), Timeline(TimelinePage),
    Plan(ForgetPlanView), Forgot(ForgetReport), Pending(Vec<FactView>), Consolidation(DraftView),
    Rules(RuleSet), Verified(ChainReport), Exported(ExportManifest), Refused(Refusal) }
pub enum Refusal { NotAllowed, SpaceLocked, SpaceUnknown, OutsideSpace, PlanStale, PlanExpired,
    NoSuchFact, NotPending, Busy, Invalid(String) }
pub enum ForgetScope { Event(EventRef), Thing(ThingRef), Fact(FactId), Range(UnixSeconds, UnixSeconds),
    App(AppName), Kind(KindPattern), Space }
```

**Authorisation matrix.** The pure function is `allowed(&Caller, &MemoryRequest) -> Allowed`. The caller is derived by the transport: D-Bus sender → pid → `AppId` (porter R12: unsandboxed callers are advisory). `Router` and `ShellUi` are fixed `AppName`s configured in memoryd.

| Request | App | Router | Companion | ShellUi |
|---|---|---|---|---|
| Record/Batch | own app's things; actor ∈ {User via self, App self} | any actor incl. Companion, System(Router) | no (goes through router) | no |
| ExplainFile, Mark | own | yes | no | yes |
| Search/Facts/Related/Provenance/Primer | no (v1) | no | own Space only; every call logs `Memory.Read` | yes |
| Propose | no | no | own Space; tainted session or untrusted label → Pending | yes (user-said → Active) |
| PlanForget | no | no | own Space | yes |
| Forget, Settle, Rules/SetRule, Pause, Export, Revert, Verify, Rebuild, Timeline, Status | no | no | no | yes |

**Recall hits carry labels:** `RecallHit { doc, text, at, label: prov::Label, links, why }`. The planner applies taint from `label`. The quarantined reader gets no recall at all.

### 3.10 D-Bus `org.quire.Memory1` [freeze now]

memoryd runs on the session bus and is activatable. Object: `/org/quire/Memory1`. Complex bodies are the serde JSON of the §3.9 types in an `s` argument (one protocol, two carriers: the same as the intents v1 proposal and porter's wire enums).

```
interface org.quire.Memory1.Record
  Record(s space, s record)                 -> (s event_ref)
  RecordBatch(s space, s records)           -> (s first_ref, u count)
  ExplainFile(s why)                        -> ()
  Mark(s mark)                              -> ()

interface org.quire.Memory1.Recall
  Search(s space, s query)                  -> (s hits)
  Facts(s space, s query)                   -> (s facts)
  Related(s space, s thing)                 -> (s events)
  Provenance(s space, s path)               -> (s provenance)
  Primer(s space)                           -> (s markdown)
  Propose(s space, s draft)                 -> (s fact_id, s state)

interface org.quire.Memory1.Control
  Spaces()                                  -> (s spaces)
  Status(s space)                           -> (s status)
  Timeline(s space, s query)                -> (s page)
  PlanForget(s space, s scope)              -> (s plan)          # plan carries token + preview
  Forget(s token)                           -> (s report)
  Pending(s space)                          -> (s facts)
  Settle(s fact, s verdict)                 -> ()
  Consolidation(s space)                    -> (s draft)
  RunConsolidation(s space)                 -> ()
  Revert(s run)                             -> ()
  Rules()                                   -> (s rules)
  SetRule(s rule)                           -> ()
  RemoveRule(s id)                          -> ()
  Pause(s space, t until)                   -> ()
  Resume(s space)                           -> ()
  Verify(s space)                           -> (s report)
  Rebuild(s space)                          -> ()
  Export(s options, h out)                  -> (s manifest)
  signal Recorded(s space, s event_ref, s kind)
  signal Forgotten(s space, s report)
  signal PendingChanged(s space, u count)
  signal ConsolidationReady(s space, s run)
  signal StatusChanged(s space, s status)
  property Version u  (= 1)
```
Errors: `org.quire.Memory1.Error.{NotAllowed,SpaceLocked,SpaceUnknown,PlanStale,PlanExpired,Invalid,Busy}` map 1:1 to `Refusal`.

### 3.11 Control UI data shapes (for the ux area) [freeze now]

```rust
pub struct TimelineQuery { pub before: Option<Cursor>, pub limit: Count, pub filter: TimelineFilter }
pub struct TimelineFilter { pub actors: ActorFilter, pub apps: Vec<AppName>, pub kinds: Vec<KindPattern>,
                            pub trust: TrustFilter, pub range: Option<(UnixSeconds, UnixSeconds)> }
pub enum ActorFilter { Everyone, You, Companion, Apps, Unknown }
pub struct TimelinePage { pub entries: Vec<TimelineEntry>, pub next: Option<Cursor> }
pub struct TimelineEntry { pub event: EventRef, pub occurred: UnixSeconds, pub actor: Actor, pub kind: KindTag,
    pub effect: Effect, pub label: prov::Label, pub things: Vec<ThingView>,   // empty when erased
    pub body: EntryBody, pub derived_facts: Count, pub undo: UndoRef }
pub enum EntryBody { Present, Erased { by: EraseCause } }   // EraseCause: Forgotten | Expired | HeaderOnly
pub struct FactView { pub fact: Fact, pub topic: TopicPath, pub state: FactState,
    pub sources: Vec<SourceView>, pub used: UseCount, pub last_used: Option<UnixSeconds> }
pub enum SourceView { Present(ThingView), Event(TimelineEntry), Purged }
pub struct ForgetPlanView { pub token: PlanToken, pub expires: UnixSeconds,
    pub events: Count, pub facts: Vec<FactView>, pub procedures: Count, pub index_docs: Count, pub pending: Count }
pub struct DraftView { pub run: RunId, pub hunks: Vec<Hunk>, pub state: RunState }
pub enum Hunk { Tidy(TidyHunk), Promote { fact: Fact, to: Lands }, Supersede { old: FactId, new: Fact },
                Flag { facts: Vec<FactId>, note: String }, ExternalEdit { topic: TopicPath, before: String, after: String },
                Stamp { topic: TopicPath, text: String } }
pub enum Lands { Active, Pending }
pub struct SpaceStatus { pub state: SpaceState, pub index: IndexState, pub chain: ChainHealth, pub usage: Bytes,
    pub events: Count, pub facts: Count, pub pending: Count, pub last_run: Option<RunId> }
```
The UI renders the sentence ("You archived *Q4 budget*") from `actor`, `kind` and `things`. Labels belong to the UI (porter convention), so memoryd never sends prose.

---

## 4. State machines (pure `step` functions in almanac-service; effects returned as values)

**4.1 Space lifecycle** (`SpaceState`)

| State | Event | Next | Effects |
|---|---|---|---|
| Locked | KeyAvailable | Open | open DBs, verify head vs system anchor |
| Locked | Record | Locked | buffer ≤ 512 in RAM, else count `Dropped(SpaceLocked)` |
| Open | Pause(until) | Paused(until) | log `Memory.Paused`; emit StatusChanged |
| Paused(u) | Record | Paused | admit: user/app → Drop(Paused); audit-class → HeaderOnly |
| Paused(u) | Tick ≥ u / Resume | Open | log `Resumed` |
| Open/Paused | ForgetConfirmed(Space) | Deleting | — |
| Deleting | Done | Gone | destroy key, remove dirs, anchor final head in system log |
| any | KeyLost | Locked | close DBs |

**4.2 Fact lifecycle**

| From | Event | To |
|---|---|---|
| — | Propose, label trusted, session untainted | Active (append to topic) |
| — | Propose, label untrusted or session tainted | Pending (pending/) |
| Pending | Settle(Confirm(witness)) | Active; label declassified with the witness |
| Pending | Settle(Reject) / age > 14 d (proposed) | removed; `FactRejected` logged |
| Active | a newer fact supersedes it | Superseded (derived at read time; the file is unchanged) |
| any | plan applied that contains it | removed from file, index and pending; `Forgot` logged |

**4.3 Forget plan**

| From | Event | To |
|---|---|---|
| — | PlanForget(scope) | Planned{digest over the closure, expires = now + 10 min} |
| Planned | Forget(token), digest still equal | Applying → Applied (report) |
| Planned | Forget(token), closure changed | Stale (refuse `PlanStale`) |
| Planned | now > expires | Expired |

The closure is computed by `plan_forget(scope, &LogRead, &facts_graph) -> Plan` (pure over snapshots). It covers:
- matching events, and events whose subjects or sources match;
- facts linked to any of those or to the thing (transitively through `Link::Fact`), pending facts, procedures, and index documents.

Apply order: index → memfiles → event bodies, then `Memory.Forgot`, then a WAL truncate. The order is idempotent, so a crash is re-run from the plan.

A deleted source automatically creates an internal `Thing` plan, applied without the UI. Example: an app records `Thing{verb: Deleted}`, or the watcher sees `file.deleted`. The user deleting the source is the confirmation.

**4.4 Consolidation run** (`RunState`)

| From | Event | To |
|---|---|---|
| Idle | Nightly tick ∧ desktop idle ∧ on AC | Due |
| Due | — | Gathering (active facts + events since the last run's cut) |
| Gathering | input ready | Drafting (Consolidator) |
| Drafting | Draft ok | Checking (`check_draft` pure) |
| Checking | invalid hunks dropped | Proposed |
| Proposed | — | Applied (Tidy, Stamp, Supersede and trusted Promote auto-apply with pre-images kept; untrusted Promote → Pending) [Q7] → emit ConsolidationReady |
| Applied | Revert(run) | Reverted |
| Drafting | error / embedder busy | Failed(reason) → Idle (retry next night) |

`check_draft` invariants:
- Every Promote cites links that are inside the input.
- The new label is the **join** of the cited sources' labels, so untrusted content cannot be laundered.
- A draft never removes a fact.
- It never touches a topic outside the Space.

**4.5 Index** is `IndexState` (§3.8):
- Absent → Building → Ready.
- Ready → Stale(EmbedderChanged) when the configured card differs from the stored one.
- Ready → LexicalOnly when the embedder becomes unavailable.
- Stale or LexicalOnly → Building when it returns.

**4.6 File-why join** (`almanac-watch::join`, pure)

| Input | Within ±2 s (proposed) on the same path, same inode or content digest | Outcome |
|---|---|---|
| Observed + Why | yes | `File{why: Explained}` with the why's actor |
| Observed alone | window elapsed | `File{why: Unexplained}`, actor `Unknown` (or `App` if pid → app is resolved) |
| Why alone | window elapsed | recorded as the app said; `FileView.content` is the app's digest |
| Rename pair (inotify cookie) | — | `Renamed{from}` and an `aliases` row (memory follows the file) |

---

## 5. Tests that pin the shapes

Harness: `almanac-fake` provides:
- `fake_service(ScriptedConsolidator)` → a `MemoryService` over `MemoryKeys`, `MemoryLog`, `MemoryVault`, `Index<ExactScan>` (in-memory SQLite) and `FakeEmbedder`;
- `FixedClock(NOW)`, three Spaces (`work`, `home`, `desktop`), and a scratch `Dirs` from `tempfile`;
- fixtures: `mail_thread_archived`, `file_saved_from_attachment`, `companion_forwarded`, `cua_run_step`, `policy_ask`;
- golden files under `crates/*/tests/golden/`.

No test touches a bus, a keyring, the network or real XDG directories. The D-Bus test only introspects skeletons in memory.

| Crate | Test | Asserts |
|---|---|---|
| core | `wire_round_trip` | Every `MemoryRequest`, `MemoryReply`, `EventBody`, `Fact` and `RememberRule` variant survives serde (one table). |
| core | `kind_tags_are_one_match` | `EventBody::kind()` gives the expected tag per variant. |
| core | `admit_table` | Rows: paused user event → Drop(Paused); paused companion action → HeaderOnly; a Never rule on an app drops that app's events; Thing beats Kind beats App; audit-class events are never dropped. |
| core | `ids_parse_only_their_grammar` | FactId, TopicPath, SpaceId, ThingKey, KindPattern accept and reject. |
| seal | `seal_round_trip`, `unseal_rejects_moved_file` | AAD binds path and Space. |
| seal | `derive_is_stable_golden` | Fixed key → fixed subkeys (pins the derivation contexts). |
| seal | `space_key_debug_is_redacted` | |
| eventlog | `header_bytes_golden` | Byte-exact v1 encoding. |
| eventlog | `chain_verifies_after_append` | |
| eventlog | `chain_detects_edit_gap_reorder` | One row per `Break`. |
| eventlog | `erasing_bodies_keeps_chain_intact` | Before: Intact with 0 erased; after: Intact with n erased, and the bodies read `Erased`. |
| eventlog | `prune_prefix_leaves_checkpoint` | Verification from the checkpoint passes; a pruned middle is impossible. |
| eventlog | `sqlite_and_memory_logs_agree` | The same script on both gives identical entries (contract test). |
| eventlog | `wrong_key_is_locked` | SqliteLog opened with another DbKey → `LogError::Locked`. |
| memfiles | `topic_round_trip_golden` | render ∘ parse is the identity on the goldens. |
| memfiles | `unstamped_and_verbatim_survive` | |
| memfiles | `append_is_add_only` | File bytes before are a prefix-preserving subset after, existing blocks unchanged. |
| memfiles | `untrusted_fact_lands_in_pending` | |
| memfiles | `confirm_requires_witness` | Does not compile or construct without `Confirmation`, plus a runtime test of the path. |
| memfiles | `derived_from_is_transitive` | |
| memfiles | `vault_contract` | The same suite over PlainDir, SealedDir and MemoryVault. |
| recall | `fuse_rrf_table` | |
| recall | `chunk_table` | |
| recall | `exact_scan_nearest_orders_by_cosine` | |
| recall | `rebuild_equals_incremental` | The same docs, rebuilt versus upserted one by one → the same search results. |
| recall | `embedder_card_change_marks_stale` | |
| recall | `search_degrades_to_lexical` | Embedder returns Unavailable → hits with `HitWhy::Lexical`, state LexicalOnly. |
| recall | `remove_drops_from_both_indexes` | |
| service | `record_then_timeline_shows_it` | |
| service | `forget_thing_cascades` | After deleting a mail thread, its events' bodies are erased, derived facts and their dependants are gone, index docs are gone, and `Forgot` is logged. Counts equal the plan preview exactly. |
| service | `plan_goes_stale_when_new_derivation_appears` | |
| service | `deleted_file_auto_forgets` | |
| service | `consolidation_cannot_launder_untrusted_sources` | A Promote citing a mail-derived event lands in Pending with an untrusted label. |
| service | `consolidation_revert_restores_pre_images` | |
| service | `companion_cannot_read_other_space` | Refusal OutsideSpace. |
| service | `companion_reads_are_audited` | |
| service | `companion_cannot_forget_or_settle` | |
| service | `app_cannot_claim_companion_actor` | |
| service | `retention_sweep_erases_expired_bodies` | |
| service | `export_tar_layout_golden` | Manifest and paths; no index and no keys inside. |
| service | `space_delete_destroys_key` | |
| service | `space_state_machine_table` | One table per machine in §4. |
| watch | `join_table` | The §4.6 rows. |
| watch | `inotify_sees_create_rename_delete_in_scratch_dir` | Scratch dir only. |
| dbus | `introspection_matches_xml` | Porter style; prints the new XML on mismatch. |
| client | `absent_transport_is_noop` | |
| client | `in_process_end_to_end` | An app records, the shell UI reads the timeline. |

---

## 6. Work breakdown

**Wave 0 (prerequisites, sequential, user/toolchain owner)**
1. Add the dependency lines of §2.3 to `quire/docs/workspace-deps.toml`, including the rusqlite sqlcipher line.
2. The actions area freezes `prov` (Label, Integrity, Confirmation, and SpaceId and Actor if they agree, §7).
3. Create the empty repo `~/almanac`, copying porter's `rust-toolchain.toml` (1.98.1), `.cargo/config.toml`, `deny.toml`, `rustfmt.toml` and `CONVENTIONS.md` pointer.

**Freeze wave (one agent; owns all of `~/almanac`)**
- Every crate in §2.2 except `recall-sqlite-vec`.
- All of §3 as compiling types, traits and XML.
- Pure functions that pin formats are implemented now: `header_bytes`, `link`, `verify_chain`, `parse_topic`/`render_topic`, `seal`/`unseal`, `derive`, `admit`, `fuse_rrf`, `chunk`, `join`, `allowed`, and the §4 step tables.
- `todo!()` bodies (each listed in FINDINGS.md): SqliteLog I/O, the Store disk paths, `Index` SQLite, `Oo7Keys`, `InotifyWatch`, the dbus codec, memoryd serving, `InferdEmbedder`, `InferdConsolidator`, `FastembedEmbedder`.
- Write ARCHITECTURE.md (crate map, one-home table, traits, recipes: "add an event body variant", "add a rule scope", "add a vector backend"), FINDINGS.md and `scripts/check-boundary.sh`.
- The §5 shape tests: the pure ones pass, and the contract tests are written against the fakes.

**Fill wave 1 (parallel; file ownership = crate directory)**
- A: `eventlog` (SqliteLog, MemoryLog, prune, verify).
- B: `memfiles` (Store, PlainDir, SealedDir, pending, derived_from).
- C: `recall` (Fts5, ExactScan, Index, FakeEmbedder) and `recall-fastembed`.
- D: `almanac-seal` and `almanac-watch` (InotifyWatch). `Oo7Keys` stays a stub until oo7 is pinned.

**Fill wave 2 (one agent; owns `almanac-service`, `almanac-fake`)**
- Admission, the forget planner and apply, the fact pipeline, the consolidation machine and `check_draft`, the retention sweep, timeline assembly, the export writer, Space lifecycle.

**Fill wave 3 (one agent; owns `almanac-dbus`, `almanac-client`, `memoryd`, `dist/`)**
- The codec, the serving skeletons, Dbus/InProcess transports, memoryd wiring, `InferdEmbedder` and `InferdConsolidator` (blocked on porter's `Broker::infer` and `DbusTransport` fills).
- A systemd user unit with sandboxing: `PrivateNetwork=yes`, `ProtectSystem=strict`, `ReadWritePaths=` for the memory dirs only, `ProtectHome=read-only` for watching, and a Landlock ruleset in main.

**Spikes (parallel with wave 1, dev scripts under the test rules)**
- S1: embedding throughput on the 5070 Ti, fastembed CUDA versus inferd/llama-server embeddings, comparing EmbeddingGemma-300M and Qwen3-Embedding-0.6B (COMPANION "before building").
- S2: SQLCipher bundled build with FTS5 enabled; `cargo deny` result for OpenSSL.
- S3: whether fscrypt exists on this machine's btrfs (`/home` is btrfs, kernel 7.2); decides whether Q1 has a kernel option.
- S4: fanotify `FAN_MARK_FILESYSTEM` with the privilege model, and what unprivileged fanotify reports.
- S5: sqlite-vec activity re-check.
- S6: xattr survival on btrfs. The design does not depend on xattrs, so this is only for the optional hint.

**Verify commands** (porter's gate, every exit code checked; `recall-fastembed` is excluded because ort downloads binaries):
```bash
cd ~/almanac
cargo fmt --all --check
cargo clippy --workspace --exclude recall-fastembed --all-targets --all-features -- -D warnings
cargo test --workspace --exclude recall-fastembed --all-features
./scripts/check-boundary.sh
cargo deny check licenses
# network-allowed dev check, run by hand: cargo check -p recall-fastembed
```

---

## 7. Dependencies on other areas, conflicts, and questions

**Other areas**
- **actions:**
  - `prov`: `Label`, `Integrity`, `Confidentiality::Private(SpaceId)`, `join`, and a `Confirmation` witness that only ShellUi consent paths can mint.
  - Proposed homes in `prov`: `SpaceId`, `Actor` (shared-undo labels use the same type), `Effect`.
  - `EntityId`/`EntityKind`/`ActionName` in a light crate, which ThingRef re-exports.
  - The router is the only `Router` caller. It records `Action` events write-ahead (`outcome: Pending`, then `Done`) and policy decisions. Proposal: the router refuses outbound and destructive actions when memoryd cannot give a receipt (fail-closed audit). That is the router's call.
- **policy-point:** the `Decision` and request-view serde forms, re-declared in `EventBody::Policy`.
- **porter/inferd:**
  - `InferRequest::Embed` with `Usage::Background` and the doc's `DataClass`, so mail text keeps its on-device floor.
  - Chat for consolidation.
  - The embedding model's identity comes from `ServedBy.model` into the stored `EmbedderCard`.
  - inferd owns the GPU queue, so the default embedder is `InferdEmbedder`. `FastembedEmbedder` exists for standalone or portable use.
- **cua:** `cua.*` payloads (opaque `CuaStep` re-declared), procedures written through memfiles with the same metadata trailer, and the run-link purge on forget.
- **ux:** draws the timeline, "what it knows", forget preview, pending, the consolidation diff, rules and pause. Uses §3.11 only. Pause appears in the presence glow.
- **quire Spaces/settings:** the stable `SpaceId` per workspace (design/21 §11.3) and the design/22 rows for `memory.retention.*`, `memory.pending_ttl_days`, `memory.join_window_ms`, `memory.consolidation.*`.
- **agent-loop:** reads `Primer` at session start and `Search` as needed. The handover working memory is its own.

**Conflicts found, with the proposed resolution**
1. **"Plain files" against "each Space has its own key".** Proposed: the format is plain markdown, sealed at rest per file by default (`SealedDir`). "Open in editor" decrypts a copy into tmpfs and re-seals on save, with edits made outside memoryd shown as `ExternalEdit` hunks. A `PlainDir` vault can be chosen per Space. [Q1]
2. **Append-only hash chain against cascade-forget.** The chain commits to a keyed digest of the body, not the body itself. Forgetting erases bodies and thing rows. Headers (kind, actor, time) remain as audit. Physical erasure on btrfs copy-on-write is only guaranteed by destroying a Space's key; per-item forget is logical deletion plus SQLite `secure_delete`. This limit must be stated to the user. [Q4]
3. **sqlite-vec is named as an implementation, but the no-unsafe rule forbids it.** Registering the extension in Rust needs `unsafe` (`sqlite3_auto_extension` or `load_extension`). Default to `ExactScan`. sqlite-vec would be an isolated `recall-sqlite-vec` crate only if the user allows that one exemption. It is also a stale single-maintainer alpha. [Q6]
4. **notify 9.0.0-rc.5 in the survey against 8.2 in the pinned block:** use 8.2.
5. **research P3 (one SQLite with bi-temporal edges as the truth) against the settled files-as-truth:** follow the settled decision. Edges exist only as derived index data. `Validity::Unstated` and the reserved `valid:` key leave room for bi-temporal later.
6. **Sync is deferred, room left:** `ReplicaId` in every header, chains per (Space, replica), globally unique `FactId`s, add-only bullet files that merge by union, and a reserved `origin:` key. Nothing is decided.

**Questions only the user can answer**
1. **Q1, files at rest:** sealed per file by default with "open in editor" through tmpfs, or plain files with only disk encryption, or a per-Space choice?
2. **Q2, Space identity:** where is the stable `SpaceId` minted (quire `SpaceStore` beside `SpaceLook`?), and is there a `desktop` scope for memory outside any Space (for example personal preferences)?
3. **Q3, strict cascade:** when a fact has two sources and one is deleted, is the fact deleted (proposed: yes, strict) or kept on the remaining source?
4. **Q4, forget and audit:** may audit headers (kind, actor, time) outlive a forget? If not, forget must cut the chain with a redaction checkpoint, which weakens tamper evidence.
5. **Q5, file capture:** fanotify filesystem marks need CAP_SYS_ADMIN. Allow a tiny privileged helper, or use inotify on Space roots only for v1?
6. **Q6, sqlite-vec:** allow one `unsafe` exemption crate for it, or ship exact scan now and usearch only if scale demands it?
7. **Q7, consolidation:** auto-apply tidy, stamp and trusted hunks (revertible, the diff is visible), or require review of every hunk?
8. **Q8, retention:** confirm the default table in §3.2 and the 14-day TTL for pending facts.
9. **Q9, recall callers:** may apps (not just the companion and shell) query memory, for example mailo asking "related to this thread"? v1 says no.
10. **Q10, export:** plain tar only, or also an encrypted export (age to a user recipient), and may the export include the verification subkey?
11. **Deferred, unchanged** (COMPANION Deferred): sync, bi-temporal validity, and other people's data in memory. Room is left for each, and none is decided.

### Critical Files for Implementation
- /home/pohsuanlai/rs-wt/companion/COMPANION.md (item 11, Security 1/2/6, Deferred)
- /home/pohsuanlai/porter/ARCHITECTURE.md and /home/pohsuanlai/porter/scripts/check-boundary.sh (the repo shape to copy)
- /home/pohsuanlai/porter/crates/porter-core/src/{id.rs,app_id.rs,units.rs,data_class.rs} and /home/pohsuanlai/porter/crates/porter-infer/src/{request.rs,reply.rs} (reused types; Embed path)
- /home/pohsuanlai/quire/docs/workspace-deps.toml (wave-0 dependency additions)
- /home/pohsuanlai/quire/CONVENTIONS.md and /home/pohsuanlai/quire/design/21-SPACES.md (rules; Space identity question)

<!-- paths: end -->
