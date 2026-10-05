# 31 Accounts: the account and capability layer

Status: this whole document is **proposed** (2026-09-30) unless a line says settled. Its
interfaces are **frozen** in the porter repo (section 11): where a frozen type differs from the
first proposal, the text below already says the frozen form. It
replaces the "28-ACCOUNTS" design doc promised by `20-SURFACES.md#117-accounts-system-service`
(28 went to Customization). Status column: **P** = proposed, **S** = settled, **O** = option
(a phased ambition the user has not decided; section 8). Confidence: **H** official source read
this pass, **M** secondary source or research file not re-checked, **U** unverified (the
design must not depend on it).

Research inputs, cited by tag and section:

| Tag | File | What it holds |
| --- | --- | --- |
| [P] | `~/rs-wt/accounts/providers.md` | provider capability matrix, auth, credentials a desktop may ship |
| [A] | `~/rs-wt/accounts/ai.md` | AI providers, auth legality, discovery, OS precedents, broker controls |
| [R] | `~/rs-wt/accounts/prior-art.md` | mailo survey, GOA, KAccounts, Android, Apple, Windows, secrets, sync engines, portals |

Other docs: `20-SURFACES.md` §1.17 (Accounts), §2.1 Mail, §2.3 Files, §2.8 Notes, §2.9 Photos;
`22-SETTINGS.md#94-live-modules-the-one-dynamic-path`; `30-CATALOGUE.md` §2 (component names);
quire `CONVENTIONS.md` §3-§6 (types, traits, effects).

## 1. Goals, non-goals, constraints

### 1.1 The idea in one paragraph

Each provider **declares** what it can do as typed capabilities. An app never asks "is there a
Google account"; it asks "is there an account that can do `Storage { write, delta >= Poll }`"
and gets candidates, a grant and a ready client. The user signs in once, in one place, and
every app and shell surface that holds a grant uses that account. Local AI runtimes are
accounts too, discovered without sign-in, and win by default.

### 1.2 Goals

| # | Goal | St |
| --- | --- | --- |
| G1 | One sign-in per account for the whole desktop; several accounts side by side (20 §1.17) | P |
| G2 | Capabilities, not brands: apps query a closed, versioned capability vocabulary (§2) | P |
| G3 | Providers are data files; code exists per protocol family, not per provider (§3) | P |
| G4 | Refresh tokens, passwords and API keys never leave the daemon (§4.6) | P |
| G5 | Consent per (app, account, capability, data class), revocable, shown in Settings (§4.5) | P |
| G6 | A Photos library that syncs through any Storage account like the reference's cloud photo library (§5.3) | P |
| G7 | One AI account layer for every app and shell surface, local-first, with a global local-only switch and spend caps (§5.5) | P |
| G8 | mailo keeps working on macOS and Windows: the core is portable, the daemon is one front end (§4.2) | P |
| G9 | Honest UI: a limited capability says why (verification, append-only, app folder only) | P |

### 1.3 Non-goals

| Non-goal | Why |
| --- | --- |
| Reverse-engineered access (iCloud Drive/Photos via Apple ID password, Gemini CLI or Claude.ai tokens) | breaks without notice, violates terms, gets user accounts suspended ([P] Apple; [A] §3; §1.4) |
| A universal sync engine that also does mail and calendars | protocol-native sync (IMAP CONDSTORE/QRESYNC, JMAP state, CalDAV sync-token) is better and already in mailo ([R] §3 "Sync engine placement", pitfall 7) |
| Files-on-demand placeholders (FUSE / VFS) in v1 | no good Linux story yet ([R] Windows, Nextcloud client); selective sync first |
| Server-side push for Google/Microsoft | needs a public HTTPS endpoint or a GCP Pub/Sub project; a desktop polls ([P] §2 Google, Microsoft) |
| Claiming per-app isolation for unsandboxed native apps | same-UID processes can read each other; consent there is advisory ([R] pitfall 4) |

### 1.4 Constraints found, and the rule each becomes

| # | Constraint | Conf. | Source | Rule |
| --- | --- | --- | --- | --- |
| C1 | Google Photos Library API lost `photoslibrary`, `.readonly`, `.sharing` on 2025-03-31; only `appendonly`, `readonly.appcreateddata`, `edit.appcreateddata` remain; reading the user's existing library is only possible through the Picker API, one user-driven session at a time | H | developers.google.com/photos/support/updates (fetched 2026-09-30); [P] §2 Google | **R1** Google Photos declares `Photos { library_read: PickerOnly, upload: Yes, albums: AppCreated }`. It is an upload target and an import source, never a library mirror. The UI says "Backs up to Google Photos; cannot show your existing Google Photos library". |
| C2 | Gmail `mail.google.com/`, `gmail.readonly`, `gmail.modify` are restricted; `gmail.send` is sensitive. Drive `drive`, `drive.readonly`, `drive.metadata*` are restricted; `drive.file` and `drive.appdata` are non-sensitive. Restricted scopes need verification, and a security assessment (CASA) at least every 12 months when the app can reach the data "from or through a third-party server" | H | developers.google.com/workspace/gmail/api/auth/scopes; .../drive/api/guides/api-specific-auth; .../production-readiness/restricted-scope-verification (all fetched 2026-09-30) | **R2** Google Drive is `Storage { scope: AppFolder }` via `drive.file` by default; `Full` only when the build's client is verified for `drive`. **R3** Google restricted data never touches a server of ours (no relay, no self-hosted server path, no cloud AI routing by default), so the build stays in the local-client case; whether that removes the CASA requirement is our reading of the policy, to be confirmed with Google during verification, and the plan does not depend on it. **R4** Each account carries `Restriction` metadata (§2.5) so the UI can show "limited: unverified build". |
| C3 | Unverified Google apps: 100-user cap and a warning screen; Testing-mode refresh tokens expire after 7 days | M | [P] §2 Google, §3 | **R5** Client IDs are a registry per issuer per build channel (mailo's sign-in module already does this) with a "bring your own client ID" override; `TokenLifetime::SevenDays` shows a re-sign-in reminder instead of a silent failure. |
| C4 | iCloud: no public Drive or Photos API; mail, contacts and calendars through IMAP/SMTP, CardDAV, CalDAV with app-specific passwords (2FA required, up to 25, all revoked when the Apple Account password changes); new-format Reminders not exposed | H (passwords), M (absence of API) | support.apple.com/en-us/102654 (fetched 2026-09-30); [P] §2 Apple | **R6** iCloud declares Mail, Calendar, Contacts only. Storage and Photos are `Absent` with the reason "Apple offers no access". Import from an Apple privacy export or a local folder is the Photos path. A revoked app password surfaces as `NeedsReauth`, not a sync error. |
| C5 | Anthropic: OAuth is only for Claude plans in Claude Code and Anthropic's own apps; third parties may not offer Claude.ai login, route requests through plan credentials, or collect or store Claude.ai tokens; developers use API keys or a cloud provider | H | code.claude.com/docs/en/legal-and-compliance (fetched 2026-09-30); [A] §3 | **R7** Claude is reachable only by Console API key or Bedrock/Vertex/Foundry credentials. The add-account sheet offers no "Sign in with Claude" and says so. |
| C6 | Google suspended Gemini/Antigravity subscribers whose Gemini CLI or Antigravity OAuth tokens were used by third-party tools (from Feb 2026) | M | github.com/google-gemini/gemini-cli/discussions/20632; winbuzzer.com/2026/02/23/google-bans-ai-subscribers-openclaw-no-refunds-xcxwbn; [A] §3 | **R8** Gemini only by AI Studio key or Vertex credentials. Never read another program's token store. |
| C7 | OpenAI "Sign in with ChatGPT" lets registered third-party apps spend the user's ChatGPT plan allowance through the Responses API; open-source and locally hosted apps are covered by a documented path with a `client_id` bound to the user and workspace at registration and a persistent per-install host id; paid or hosted apps fill an interest form | H (docs page), M (details) | developers.openai.com/siwc/token-sharing-open-source (fetched 2026-09-30); thenewstack.io/sign-in-with-chatgpt | **R9** ChatGPT sign-in is a v2 option (`Auth::OAuthPlan`), behind a spike that confirms our registration path; v1 uses OpenAI API keys. Plan-billed accounts count requests against a rate budget, not money. |
| C8 | OpenRouter mints a user-owned API key by OAuth PKCE with a localhost callback on any port; codes expire in 10 minutes; no spend limit is set at mint time | H | openrouter.ai/docs/guides/overview/auth/oauth (fetched 2026-09-30) | **R10** OpenRouter is the one-click cloud AI account (`Auth::OAuthMintsKey`); our spend cap (§5.5) is the limit, and the page links to OpenRouter's own per-key limit. |
| C9 | Push is unavailable to a serverless desktop for Google and Microsoft; real push only for JMAP, IMAP IDLE, Dropbox and Box long-poll, Nextcloud `notify_push` | M | [P] §4 | **R11** `Delta::{None, Poll, Push}` is part of every sync-able capability; apps adapt refresh cadence; syncd owns one polling scheduler with backoff. |
| C10 | Nothing on Linux gives per-app secret isolation; Flatpak's app id is the only reliable identity | H | freedesktop Secret Service spec; [R] §2 secrets, portals | **R12** Grants bind to a Flatpak app id when there is one; native apps are identified by their systemd app scope and the UI marks such grants "unsandboxed". |
| C11 | Microsoft Graph delegated scopes are user-consentable, but tenants may force admin consent; OneNote is delegated-only since 2025-03-31 with no delta | M | [P] §2 Microsoft | **R13** `Probed` provenance: a 403 at add time downgrades the capability to `Absent { reason: TenantConsent }`. |
| C12 | KDE and GNOME communities are hostile to AI features in 2026 | M | theregister.com/software/2026/09/24/talk-of-ai-in-kde-sets-the-community-ablaze/5298854; [A] §5 | **R14** AI is off until the user turns it on in first run or Settings; local-only is the default policy; one switch removes every cloud AI account from every answer. |

Checked, not depended on: iOS 27 "Extensions" (a user-picked AI provider behind system
features) is press-only (M: macrumors.com/2026/05/05/ios-27-third-party-chatbots-apple-intelligence);
"Aion Instruct" replacing Phi Silica (Oct-Nov 2026) is on Microsoft's Phi Silica page (H,
fetched 2026-09-30). Both show the pattern §5.5 follows: a user-picked provider behind system
surfaces, with readiness states.

## 2. The capability vocabulary

### 2.1 Shape

A **closed set** (CONVENTIONS §4: closed sets are enums), **versioned** as a whole:
`VocabVersion(2)`. Adding a kind or a field is a vocabulary bump, released with the daemon and
the client crate together; an older client ignores kinds it does not know on the wire. This
keeps GOA's lesson (a compiled list of *providers* is the trap, [R] pitfall 1) apart from the
vocabulary: providers are data (§3); only a new *kind of thing an account can do* needs code.

```rust
pub enum Capability {
    Identity(IdentityCap), Mail(MailCap), Calendar(PimCap), Contacts(PimCap),
    Tasks(PimCap), Notes(NotesCap), Storage(StorageCap), Photos(PhotosCap),
    Llm(LlmCap), Embeddings(EmbedCap), Speech(SpeechCap), ImageGen(ImageGenCap),
    Rerank(RerankCap), KeyValue(KeyValueCap), Push(PushCap),
}
pub enum CapabilityKind { Identity, Mail, Calendar, Contacts, Tasks, Notes, Storage, Photos,
    Llm, Embeddings, Speech, ImageGen, Rerank, KeyValue, Push }   // serde snake_case slug
```

Every closed set's stable slug is its serde `snake_case` form (files, bus, consent store). porter
does not depend on quire's `ds-core`, so these enums do not implement `Word`: the account core
builds below the design system for mailo on every platform; UI crates map slugs to labels.

Every field is an enum or a newtype; no `bool` (CONVENTIONS §4). Shared small enums:

| Enum | Variants |
| --- | --- |
| `Access` | `None`, `Read`, `ReadWrite` |
| `Delta` | `None`, `Poll`, `Push` (ordered: a request asks for a minimum) |
| `Offered` | `Absent`, `Present` |
| `QuotaReport` | `Unreported`, `Reported` |

### 2.2 Data and PIM kinds

| Kind | Fields | Notes | St |
| --- | --- | --- | --- |
| `Identity` | `profile: Offered` (name, avatar), `verified_address: Offered`, `sign_in: Offered` (can act as an OpenID identity for other sites later) | every account has at least the label | P |
| `Mail` | `access: Access`, `send: Offered`, `delta: Delta`, `transport: MailTransport {Imap, Jmap, Graph, GmailApi, Pop3}`, `labels: LabelModel {Folders, Labels}` | mailo's `AccountCaps` becomes the discovered detail (§4.2) | P |
| `Calendar`, `Contacts`, `Tasks` (`PimCap`) | `access`, `delta`, `transport: PimTransport {CalDav, CardDav, Jmap, Graph, GoogleApi}`, `collections: Offered` (several lists) | Tasks on iCloud: legacy lists only (C4) | P |
| `Notes` | `access`, `delta`, `transport: NotesTransport {NextcloudNotes, OneNote, ImapNotes}` | OneNote has no delta (C11) | P |
| `Storage` | `access: Access`, `delta: Delta`, `quota: QuotaReport`, `scope: StorageScope {AppFolder, Full}`, `hashes: HashKind {None, Md5, Sha1, Sha256, QuickXor, Dropbox}`, `ranges: Offered` (partial download), `chunked_upload: Offered` | `AppFolder` is what `drive.file` and Dropbox app-folder give (R2) | P |
| `Photos` | `library_read: LibraryRead {None, PickerOnly, Full}`, `upload: Offered`, `albums: Albums {None, AppCreated, Full}`, `video: Offered`, `delta: Delta` | only providers with photo semantics; a Storage folder is a Photos library through §5.3, not through this kind | P |
| `KeyValue` | `delta: Delta`, `max_item: Bytes` | small encrypted items (settings log, §6.4); served by any Storage account via syncd, or natively by a self-hosted server | O |
| `Push` | `channel: PushChannel {ImapIdle, JmapEventSource, LongPoll, WebSocket}` | derived from the others, exposed so the scheduler can plan | P |

### 2.3 AI kinds

Declared per **model**, aggregated per account ([A] §7). Feature sets are `BTreeSet<Feature>`
rather than flags.

| Kind | Fields | St |
| --- | --- | --- |
| `Llm` | `features: BTreeSet<LlmFeature {Chat, Tools, Vision, AudioIn, Pdf, StructuredOutput, Reasoning, PromptCache}>`, `context: Tokens`, `max_output: Tokens`, `wire: LlmWire {ChatCompletions, Responses, Messages, GenerateContent, OllamaNative}` | P |
| `Embeddings` | `dims: Dims`, `modalities: BTreeSet<Modality {Text, Image}>`, `max_input: Tokens`, `max_batch: Count` (the most texts one call takes), `prompts: EmbedPrompts { query, document: PrefixText }` (the text put before a query and before a passage; both empty for a symmetric model, so an index and its queries cannot disagree) | P |
| `Speech` | `modes: BTreeSet<SpeechMode {Stt, Tts, Realtime}>`, `languages: LanguageSet` | P |
| `ImageGen` | `modes: BTreeSet<ImageMode {TextToImage, Edit, Inpaint}>`, `max_side: Px` | P |
| `Rerank` | `max_docs: Count` | P |
| `ComputerUse` | `environments: BTreeSet<CuaEnv {Desktop, Browser, Mobile}>`, `batching: CuaBatching {One, Many}`, `zoom: Offered`, `max_image: Px`, `wire: LlmWire` | P |

`ComputerUse` is a model that operates a window from screenshots (a vision model with a
computer-use dialect; the dialect detail stays in stoker's catalog and is not asked for). Its
need is `CuaNeed { environments }`, a subset test. It joined with `VocabVersion(2)`, which also
added `DataClass::Voice` and the Space dimension of a grant (§4.5). Speech is one capability
kind with `modes`; the model picker splits it by direction (`AiKind::SpeechIn` for `Stt`,
`SpeechOut` for `Tts`) because the two are chosen separately.

Two more properties sit beside every AI capability, not inside it:

| Property | Values | Why |
| --- | --- | --- |
| `Locality` | `OnDevice`, `LocalNetwork`, `Cloud { region: Option<Region> }` | the routing policy and the data-class rules read it (§5.5) |
| `Tier` | `Fast`, `Balanced`, `Best` | the user maps tiers to models per account; apps ask for a tier, never a model id ([A] §7) |
| `Billing` | `Free`, `Metered(PriceTable { input_per_mtok, output_per_mtok: MicroUsd })`, `PlanBudget` | spend caps (§5.5) |

### 2.4 Provenance

Each effective capability records how it is known ([R] §3; mailo's `AccountPlan` vs
`AccountCaps` split):

| Provenance | Source | Examples | Wins over |
| --- | --- | --- | --- |
| `Declared` | the provider file (§3.1) | "Fastmail: Mail over JMAP" | nothing |
| `Curated` | our shipped model table, for providers whose model list has no capability data | OpenAI, vLLM ([A] §3) | Declared |
| `Discovered` | the protocol's own session answer at add time and on reconnect | IMAP CAPABILITY, JMAP session, Nextcloud OCS capabilities, Ollama `/api/show`, Anthropic and Gemini model lists, OpenRouter models | Declared, Curated |
| `Probed` | a small real call | Graph 403 on tenant consent (R13), a one-image vision probe, a WebDAV `sync-collection` REPORT | all |

Effective capability = (Declared, refined by Curated and Discovered, corrected by Probed)
minus the user's per-account toggles. A change emits `CapabilityChanged` ([R] pitfall 10).

Frozen shape: a `Claim { subject: Subject {Account, Model(ModelId)}, offer: Offer, provenance }`
where `Offer` is `Present(Capability)` or `Absent { kind, reason: AbsentReason {ProviderOffersNone,
TenantConsent, UnverifiedBuild, TurnedOff, NotOnServer} }`; `effective(claims, toggles)` keeps one
claim per (subject, kind), the highest provenance winning, and turns a kind toggled off into
`Absent { TurnedOff }`. `matches(need, offer) -> Match {Fits, Short(Shortfall), Absent(reason),
OtherKind}` is the one place a need meets an offer; `Shortfall` names the first field that falls
short so the UI can say why (G9).

### 2.5 Restriction metadata

```rust
pub struct Restriction {
    pub verification: Verification,   // NotNeeded | Verified | Unverified { user_cap: Count }
    pub token_lifetime: TokenLifetime, // Standard | SevenDays | UntilPasswordChange
    pub consent: TenantConsent,        // User | AdminRequired | AdminGranted
    pub limits: Vec<Limit>,            // Limit { kind: CapabilityKind, reason: LimitReason }
}                                      // LimitReason: AppendOnly | PickerOnly | AppFolderOnly | ProviderOffersNone
```

The detent page (§5.6) turns each variant into one secondary line. Limits are per kind: one
Google account is `PickerOnly` for Photos and `AppFolderOnly` for Storage at once.

## 3. Providers

### 3.1 Declaration: data file, family code

A provider is a TOML file (the KAccounts lesson, [R] §2 KAccounts), installed at
`/usr/share/<repo>/providers/*.toml` and `$XDG_DATA_HOME/<repo>/providers/*.toml` (user files
win). It names **families**, which are code:

```toml
id = "fastmail"
label = "Fastmail"
mark = "fastmail"                        # ProviderMark glyph (30 §2.11)

[auth]
kind = "oauth_pkce"                      # none | local_runtime | password | app_password | login_flow_v2 | local_bridge
issuer = "fastmail"                      #   | key_pair | api_key | oauth_pkce | oauth_mints_key | oauth_plan | cloud_identity

[discovery]
kind = "jmap_session"                    # autoconfig | well_known | jmap_session | nextcloud_ocs | fixed | model_list
                                         #   | probe_ports, with v = { ports = [11434] }

[[capability]]
family = "jmap"
kind = "mail"
v = { access = "read_write", send = "present", delta = "push", transport = "jmap", labels = "folders" }

[[capability]]
family = "webdav"
endpoint = "https://myfiles.fastmail.com/"
kind = "storage"
v = { access = "read_write", delta = "poll", quota = "reported", scope = "full", hashes = "none", ranges = "present", chunked_upload = "absent" }
```

The file is the serde form of porter's `ProviderSpec`: each row names its `family`, an optional
fixed `endpoint`, and the capability as `kind` plus `v` with **every** field written (a missing
field refuses the file; nothing defaults). An `issuer` is required exactly for the OAuth kinds
(`Issuer`: google, microsoft, dropbox, box, fastmail, openrouter, openai). A provider with AI
rows adds `[ai]` with `locality` and `billing`, and only such a provider may. A user file with a
system file's `id` replaces it. porter ships `providers/{nextcloud,google,ollama,local}.toml`.
**[accounts plan 2026-10-05]** A provider file may add `[matching]` with `domains` and
`mx_suffixes`, so an address or its MX picks the provider; mailo's brand presets become files
this way (`microsoft`, `fastmail`, `icloud`, `yahoo`, `gmx`, `generic-imap`, `generic-dav`,
`generic-jmap`). A capability row may carry discovered endpoints at sign-in instead of a fixed
`endpoint`; the account stores them per claim (`ServiceEndpoint`: family, URL, TLS mode, login
name; never a secret) and a granted app reads them on its candidate.

| Layer | Form | Closed? | Adding one needs |
| --- | --- | --- | --- |
| Capability kind | Rust enum (§2) | yes, versioned | a vocab bump |
| Family (protocol engine) | Rust enum `Family` + one crate or module each | yes | code |
| Auth kind | Rust enum `AuthKind` (in `porter-core`) | yes | code |
| Provider | TOML file | no | a file |
| Account | row in accountd's store + secrets | no | the user |

Families: `Imap`, `Smtp`, `Pop3`, `Jmap`, `CalDav`, `CardDav`, `WebDav`, `Graph`, `GmailApi`,
`GoogleCalendar`, `GooglePeople`, `GoogleTasks`, `GoogleDrive`, `GooglePhotosUpload`,
`GooglePhotosPicker`, `Dropbox`, `S3`, `NextcloudNotes`, `ChatCompletions`, `Responses`,
`Messages`, `GenerateContent`, `OllamaNative`, `ComfyWorkflow`.

### 3.2 Initial provider table (declared capabilities)

Legend: **Y** yes, **L** limited (reason in Notes), **-** absent. Mail/PIM/Storage from [P] §1;
AI from [A] §2.

| Provider | Auth | Mail | Cal | Contacts | Tasks | Notes | Storage | Photos | Delta | Notes | Wave |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Nextcloud | LoginFlowV2 | - (separate IMAP) | Y CalDAV | Y CardDAV | Y VTODO | Y Notes API | Y WebDAV Full, quota | via Storage | Poll; Push with `notify_push` | the most complete self-hostable set | v1 |
| Microsoft (personal + work) | OAuth PKCE, shipped client | Y IMAP/Graph | Y Graph | Y Graph | Y To Do | L OneNote, no delta | Y OneDrive Full | via Storage (Camera Roll is files) | Poll (`/delta`) | tenant consent probed (R13) | v1 |
| Google | OAuth PKCE, shipped or BYO client | L restricted (C2) | Y | Y People | Y | - (Keep is Workspace-only) | L AppFolder (`drive.file`) | L PickerOnly + upload | Poll | R1-R5 | **TODO** (owner, 2026-10-05: Google is left as a TODO; `google.toml` stays shipped, no family code) |
| Generic IMAP/SMTP + DAV | password / app password, autoconfig | Y | Y | Y | Y | L IMAP notes | Y WebDAV | via Storage | Idle / Poll | mailo's discovery | v1 |
| iCloud | app-specific password | Y | Y | Y | L legacy lists | - | - | - | Idle / Poll | R6 | v2 |
| Fastmail | OAuth (registered) or app password | Y JMAP | Y | Y | L | - | Y WebDAV | via Storage | Push | best-behaved provider | v2 |
| Dropbox | OAuth PKCE, shipped key | - | - | - | - | - | Y Full or AppFolder | via Storage | Push (long-poll) | cursor resets (409) | v2 |
| Generic JMAP | password / token | Y | Y | Y | - | - | - | - | Push | | v2 |
| S3 / Backblaze B2 | key pair | - | - | - | - | - | Y, no delta, no quota | via Storage (our manifest) | None | backup target | v2 |
| Proton | local Bridge | L Bridge only, paid | - | - | - | - | L SDK pre-1.0 | - | Idle | Drive when the SDK settles (U) | v3 |
| Box | OAuth | - | - | - | - | - | Y | via Storage | Push (long-poll) | low priority | v3 |

AI providers:

| Provider | Auth | Llm | Embed | Speech | ImageGen | Rerank | Locality | Discovery | Wave |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Ollama | none | Y | Y | - | - | - | OnDevice | probe `:11434`, `/api/show` capabilities (H) | v1 |
| llama.cpp server | none / key | Y | Y | - | - | Y | OnDevice | probe `:8080`, `/props` (M) | v1 |
| LM Studio | none | Y | Y | - | - | - | OnDevice | probe `:1234` (M) | v2 |
| vLLM | none / key | Y | Y | Y STT | - | - | OnDevice / LocalNetwork | user URL, curated | v2 |
| ComfyUI | none | - | - | - | Y (workflow registry) | - | OnDevice | probe `:8188` + our workflow files | when an ImageGen request kind and a consumer exist |
| Anthropic | API key; Bedrock/Vertex/Foundry | Y | - | - | - | - | Cloud | `/v1/models` capabilities (H per [A]) |v1, after stoker's cloud backends |
| OpenAI | API key; ChatGPT sign-in later (R9) | Y | Y | Y | Y | - | Cloud | curated table |v1, after stoker's cloud backends |
| OpenRouter | OAuth mints key (R10) | Y | Y | L | Y | Y | Cloud | `/api/v1/models` modalities (H) |v1, after stoker's cloud backends |
| Gemini | AI Studio key; Vertex | Y | Y | Y TTS | Y | - | Cloud | `models.list` (H per [A]) | v2 |
| Mistral, xAI | API key | Y | Y (Mistral) | Y | Y (xAI) | - | Cloud | model lists (M) | v2 |
| Azure OpenAI / Foundry | Entra or key; endpoint + deployments | Y | Y | Y | Y | - | Cloud { region } | per deployment, user form | v3 |
| GitHub Models | PAT `models:read` | Y | Y | - | - | - | Cloud | catalog (M) | v3 |
| Claude.ai, Gemini CLI, Antigravity plan tokens | - | refused (R7, R8) | | | | | | | never |

### 3.3 Local AI auto-discovery

| Step | Rule | St |
| --- | --- | --- |
| Probe | inferd (§4.1) probes the default ports above on `127.0.0.1` at start, on a D-Bus `Rescan`, and when a user unit named in settings starts; plus URLs the user listed | P |
| Identify | the runtime's own endpoint names the runtime and its models (`/api/tags` + `/api/show`, `/props`, `/v1/models`) | P |
| Account | each running runtime is an account with `Auth::None`, `Locality::OnDevice`, provenance `Discovered`; it appears in detent without a sign-in and disappears (state `Offline`, not deleted) when the process stops | P |
| Supervised | engines inferd runs itself (llama.cpp, vLLM, a speech host; confined, no network) are one account, `local` (`Discovery::Supervised`, `providers/local.toml`); its models come from the catalog at provenance `Curated`, and the account is present while an engine is stopped (readiness `Loadable`), unlike a probed runtime, which goes `Offline` | P |
| LAN | a URL on another host is `LocalNetwork` (e.g. a Tailscale peer); never auto-probed | P |
| GPU arbitration | one 16 GB GPU (RTX 5070 Ti): inferd serialises local LLM and ComfyUI jobs, asks Ollama to unload idle models (`keep_alive`), exposes the `Gpu` property (`idle`, `busy`, `loading`) so apps show a queue state; CPU engines (speech) cost no VRAM | P |
| Readiness | `Readiness {Ready, Loading, Loadable, Downloading(Permille), Downloadable, Unavailable}` per model, the pattern of the browser and Windows on-device APIs ([A] §5); a session whose engine is loading gets `Waiting(Loading)` events and the app shows "working", never its own spinner | P |

## 4. Service architecture

### 4.1 Processes

| Process | Owns | Why separate | St |
| --- | --- | --- | --- |
| `accountd` | account registry, provider registry, token broker, secrets, consent store, discovery and probes, the Settings live module | the control plane; small, must never stall | P |
| `syncd` | the sync journal, anchors, the polling scheduler, dataset plug-ins (Photos first) | a failing backend or a long upload cannot block sign-in ([R] §3) | P |
| `inferd` | the AI broker: routing, policy, spend, audit, wire adapters, engine supervision, GPU queue, streaming sessions | streams and big payloads; restartable alone | P |
| sheet host | the sheets accountd must draw itself: add account, consent prompt, chooser, re-auth, device code. It serves `org.quire.AccountsSheet1` (§4.4) and draws quire's `ds-shell::accounts` views from porter's `SheetView`. **[departs §8 item 14, 2026-10-05]** On our desktop the host is **sill** (an Overlay with `Sheet` material and exclusive keyboard, the pattern of its `Confirm1` sheet and polkit agent; phase B moves it to casement's trusted surface); where accountd is absent (macOS, Windows, another Linux desktop) the app hosting the core in process (mailo) draws the same view; a standalone `porter-sheet` for other Linux desktops with accountd is later | an app cannot draw its own consent; same split as portal backends ([R] §2 portals) | P |

All are user-session services, D-Bus activated on Linux, single-instance through latchkey
([R] §1). syncd gets tokens and authenticated streams from accountd like any client; inferd also
uses accountd's `Peer` interface (§4.4) for verdicts on behalf of the calling app, API-key
resolution and reporting probed local runtimes.

**Caller roles [refines §4.5, 2026-10-05].** One caller table (`porter-dbus` `ProcCallers`, shared
by accountd and inferd) maps a connection's pid, through its cgroup scope or Flatpak info and
`identity_of`, to an `AppId` and a role from a shipped table (`/etc/porter/callers.toml`, the
user file wins): `App`, `Settings` (detent), `SheetHost` (sill), `PorterDaemon` (inferd, syncd),
`Agent` (intentd, companiond, readerd, cuad, quire-do, actions-mcp), `Cua` (cuad, for inferd). A
sender with no app scope and no Flatpak info is refused. accountd refuses every account call from
the `Agent` role (`Choose`, `AddAccount`, `Reauthenticate`, `IssueToken`, `OpenAuthenticated`);
agents keep inferd.

### 4.2 Crates and the extraction from mailo

The core comes out of mailo the way latchkey did, coordinated with the mailo session: mailo
moves to depending on it in the same change (CONVENTIONS §3: no alias left behind).

Frozen as the porter repo (`porter-*` crates; section 11). **[2026-10-05]** The proposal's
`accounts-auth` and `accounts-discover` become `porter-oauth` and `porter-discover`, beside
`porter-families` (the `Provider` implementations, one feature per family, built by accountd
and by mailo in process), `porter-dav` (mailo's WebDAV reply parsing, moved) and `porter-proxy`
(the authenticated relays of `OpenAuthenticated`). Each keeps a pure core; its HTTP half
(hyper + hyper-rustls from the pinned block, not reqwest) sits behind a named feature.

<!-- paths: skip -->
| Crate | Content | From mailo | I/O | Portable |
| --- | --- | --- | --- | --- |
| `porter-core` | ids, `Account`, `Capability` (§2), `Need`, `matches`, `effective`, `Restriction`, `AuthKind`, `Credential`, `SecretKey`, `SecretPurpose`, `Grant`, `decide`, `DataClass`, `AppId`, the wire protocol | `mail-domain/src/account.rs` (`OAuthIssuer`, `Credential` with its redacting `Debug`, `SecretKey`, `SecretPurpose`) | none | yes |
| `porter-provider` | `ProviderSpec` and the file parser, `ProviderSet`, `Family`, `Issuer`, the `Provider`/`ProviderSession` traits | `presets/` become provider files | none | yes |
| `porter-secrets` | `Secrets` trait (get, put, delete, delete_account) + backends: `Oo7Secrets` (Linux; oo7 0.6 is in the pinned block), `KeyringSecrets` over keyring-core's native stores with mailo's chunking (macOS, Windows), `MemorySecrets` for tests | `mail-runtime/src/secrets.rs` + `secrets/chunks.rs` | keyring | yes |
| `porter-service` | accountd's core over its seams (providers, secrets, `Sheets`, `RegistryStore`, audit, `Clock`): answers every request; hosted by accountd or in process | new | none | yes |
| `porter-client` | the app-facing API (§5.1); trait `Transport` with `DbusTransport`, `SocketTransport` (latchkey), `InProcess` | new | per transport | yes |
| `porter-dbus` | the three buses as zbus proxies and skeletons, `dbus/*.xml` | new | D-Bus | Linux |
| `accountd` | zbus front end, consent store, probes; also serves the latchkey front end where D-Bus is absent | new | yes | Linux first |
| `porter-sync`, `syncd` | `Replica` contract (§6.1), dataset kinds; journal and scheduler in syncd | new | SQLite | core yes |
| `storage-*` (later) | one per family: `webdav`, `graph`, `gdrive`, `dropbox`, `s3` | new (mailo's CardDAV WebDAV code reused) | HTTP | yes |
| `porter-infer`, `inferd` | typed request model, routing, policy, spend, audit, `Model` trait; adapters later (`ChatCompletions`, `Responses`, `Messages`, `GenerateContent`, `OllamaNative`, `ComfyWorkflow`) | new | HTTP (adapters) | core yes |
| `porter-oauth`, `porter-discover` | OAuth PKCE, loopback, device code, client registry (shipped `/usr/share/porter/clients.toml` per channel, a user-supplied override written only by detent), renewal, OpenRouter key mint; autoconfig, SRV, MX, `.well-known`, JMAP session, Nextcloud OCS, port probes | `mail-runtime/src/{oauth,signin,renewal,loopback,discover}.rs`, `mail-proto/src/discover` | HTTP | yes |
| `porter-families` | `Provider` implementations (sign-in, discover, open, revoke) per family: Nextcloud (Login Flow v2), generic IMAP/SMTP + DAV, Microsoft; moved out of accountd so mailo builds the same families in process | new | HTTP | yes |
| `porter-dav`, `porter-proxy` | WebDAV PROPFIND/REPORT/sync-collection/quota parsing; the IMAP, SMTP and HTTP relays behind `OpenAuthenticated` | `mail-pim/src/dav` (vCard and iCal values stay in mail-pim) | HTTP, TLS | yes |
| `mail-proto`, `mail-pim` | stay in mailo, now "family" crates for Mail, Calendar, Contacts | unchanged | none | yes |

`mail-domain` keeps mail-shaped types (`Incoming`, `Outgoing`, `AccountCaps` for IMAP detail);
`AccountPlan` references a `porter_core::AccountId` and a `Grant`. The OAuth client ID stays
deployment config, never in the account (as mailo's `AuthPlan` comment already says). porter's
`ARCHITECTURE.md` section 8 maps every mailo type to its porter home; mail signing keys
(OpenPGP, S/MIME, filed by fingerprint) stay mailo's.
<!-- paths: end -->

### 4.3 Transports

| Platform | Link | Who hosts the core | St |
| --- | --- | --- | --- |
| Linux, our desktop | `DbusTransport` to `org.quire.Accounts1` | accountd | P |
| Linux, another desktop; macOS; Windows | `SocketTransport` (framed messages over latchkey's socket / named pipe) | accountd built without zbus, or mailo's own agent hosting the core | P |
| mailo standalone, tests | `InProcess` (in process) | the app | P |

The request and reply types are one serde enum each in `porter-core` (`AccountsRequest`,
`AccountsReply` with `Refusal`); D-Bus and latchkey both carry them, so there is one protocol
with two carriers. A socket frame is a 4-byte big-endian length and a JSON envelope
`{ vocab, body }` (16 MiB at most). The caller's identity is never in a request: each transport
derives it from the connection.

### 4.4 D-Bus API sketch

Bus name `org.quire.Accounts1`; root `/org/quire/Accounts1`. Shaped like a portal (Request
objects with a `Response` signal, `handle_token`), so a later `org.freedesktop.portal`
proposal can reuse it ([R] §2 portals).

| Interface | Member | Signature (frozen) | Notes |
| --- | --- | --- | --- |
| `.Manager` | `Query(need: (sa{sv}), class: s, usage: s) -> a(osa{sv})` | candidates the caller **already holds a grant for**: account path, label, then provider, capability, restriction, grant by name | no bulk enumeration ([R] Android `GET_ACCOUNTS`) |
| | `Availability(need, class, usage) -> s` | `granted`, `available_needs_consent`, `denied`, `needs_account`, `unsupported` | reveals no identities |
| | `Choose(need, class, usage, parent_window: s, options: a{sv}) -> o` | Request; the sheet host shows a chooser with only matching accounts; the `Response` carries the account path and a new grant | the one way an app learns of a new account |
| | `AddAccount(provider_hint: s, parent_window: s, options: a{sv}) -> o` | Request; opens the add sheet | apps offer "Add Account…" |
| `.Account` (per object, `/org/quire/Accounts1/account/<id>`) | properties `Id s`, `Provider s`, `Label s`, `State s` (`ok`, `needs_reauth`, `offline`, `limited`), `Capabilities a(sa{sv})` | readable only with a grant | |
| | `Reauthenticate(parent_window: s, options: a{sv}) -> o` | Request | |
| `.Grants` | `Revoke(grant: s)`; `List() -> a(sa{sv})` (caller's own) | | |
| `.Tokens` | `IssueToken(grant: s, audience: s) -> (ssx)` | kind (`bearer`, `xoauth2`, `api_key_handle`), value, expiry; short-lived access tokens only | refresh tokens never leave (§4.6) |
| | `OpenAuthenticated(grant: s, endpoint: s) -> h` | a socket fd to a daemon-side authenticated relay. **v1 [departs §7.1/§7.2, owner 2026-10-05]**: IMAP (the relay logs in; the app sees a `* PREAUTH` greeting), SMTP (the relay does EHLO, STARTTLS and AUTH; the app sees a greeting and an EHLO reply without them), HTTP/1.1 for WebDAV, CalDAV, CardDAV and OCS (the relay adds `Authorization`, speaks TLS, reaches only the endpoint's origin and drops an app's own `Authorization`) | password protocols without releasing the password: a password never leaves accountd, to any app, sandboxed or not |
| `.Request` (per sheet) | `Close()`; signal `Response(response: u, results: a{sv})` | 0 done, 1 cancelled, 2 other | the portal Request shape |
| Signals on `.Manager` | `AccountAdded(o)`, `AccountRemoved(o)`, `CapabilityChanged(o)`, `NeedsReauth(o)`, `GrantChanged(s)` | sent only to holders of a relevant grant (unicast) | |
| `org.quire.SettingsModule1` | at `/org/quire/Accounts1/settings`: `Describe`, `Get`, `Set`, `Changed` | 22 §9.4 (declared there, built in quire's `ds-settings` feature `live`, not in porter-dbus); keys `accounts.<id>.service.<kind>` (toggle), `accounts.<id>.grant.<grant>` (Revoke), `accounts.<id>.remove` (destructive, `Alert{Critical}`), `accounts.<id>.reauth`, `accounts.clients.<issuer>` (a user-supplied client id, Advanced) | detent reads it; `Set` only from the `Settings` role |
| `.Peer` | `Verdicts(app, need, class, usage)`, `ResolveKey(grant) -> h` (the key on a sealed memfd, never a string), `ReportLocal(provider, claims, state)` | only the `PorterDaemon` role [2026-10-05] | inferd asks with the app it derived from its own connection |
| `org.quire.AccountsSheet1` (served by the sheet host) | `Open(handle, parent_window, view: s)`, `Update(handle, view: s)`, `Close(handle)`, signal `Input(handle, input: s)` | only accountd's connection may `Open`; only the sheet owner's `Input` counts [refines §4.6: a typed password travels inward here, nothing outward] | the add, re-auth, device-code and consent conversations [2026-10-05] |
| `.Manager` (mailo migration) | `Adopt(legacy)` | the daemon reads an app's own legacy keyring entries itself (gated by an `[adopt]` table, `org.quire.Mail = "mailo"`), so no credential crosses a transport | [2026-10-05] |

A need on the bus is `(sa{sv})`: the kind's slug and its fields by name, each field's value its
slug; `class` and `usage` are `DataClass` and `Usage` slugs. The full introspection is porter's
`dbus/org.quire.Accounts1.xml`, checked against the skeletons by a test.

syncd serves `org.quire.Sync1` at `/org/quire/Sync1` (`Datasets() -> as`, `Status(dataset: s) ->
a{sv}` with a `quota` key (used, total) from `Replica::quota`, `Pause(s)`, `Resume(s)`, signals `Progress(s, a{sv})`, `Conflict(s, a{sv})`); inferd
serves `org.quire.Inference1` at `/org/quire/Inference1`:

| Member | Signature | Notes |
| --- | --- | --- |
| `Availability` | `(need, class, options) -> s` | `need` may be `computer_use` or `speech`; `options` is an `a{sv}` whose reserved key is `traceparent` |
| `Open` | `(need, class, tier, options) -> h` | `options` is an `a{sv}`: the reserved key `traceparent` carries the caller's W3C trace context (a version 00 string, ids only, never content) so one task is one trace across daemons; absent, inferd starts its own root; an unknown key is ignored. A Unix stream socket pinned to the one model the route chose. The client writes `ClientFrame` frames (`Request(InferRequest)`, `Cancel`, `Audio`, `EndOfAudio`); inferd writes `InferEvent` frames (`Routed`, `Waiting`, `TextDelta`, `ThoughtDelta`, `ToolCall`, `ActionProposed`, `Usage`, `Heard`, `Spoken`, exactly one `Finished` per turn). Framing is the `{vocab, body}` envelope of porter-core's wire (4-byte length, 16 MiB cap). `ImageSource::Attached(i)` names the i-th memfd received with SCM_RIGHTS on that frame, so screenshots are never base64 on the bus |
| `Prepare` | `(need, class, tier, options) -> s` | warms the engine the route would pick and answers a `Readiness` slug (a refusal answers with its slug); no microphone, no request |
| `Usage` | `() -> a{sv}` | |
| `Rescan` | `()` | |
| signal `EnginesChanged` | `()` | broadcast: engine state is not personal; listeners re-read readiness with `Prepare` or the settings module |
| property `Gpu` | `s` | `idle`, `busy`, `loading` |
| `org.quire.SettingsModule1` | at `/org/quire/Inference1/settings`: the picker rows per kind and the `ai.model.<kind>.<tier>` map | 22 §9.4 (declared there, not in porter-dbus) |

Speech and computer-use steps travel on the `Open` fd, so the interface has no member for them.
The full introspection is porter's `dbus/org.quire.Inference1.xml`, checked by the same test.

### 4.5 Consent

| Rule | Detail | St |
| --- | --- | --- |
| Unit | a grant is `GrantKey { app: AppId, account, kind: CapabilityKind, class: DataClass, usage: Usage {Interactive, Background}, space: SpaceScope {Any, Only(SpaceId)} } -> Decision {Allow, Deny}` plus `GrantScope {Once, Always}` and its time; the newest grant for exactly the key decides, a denial winning a tie (`decide`). `Grant<K = GrantKey>` and `decide<K: Eq>` are generic over the key, so the action router keeps its own grants (an action, a caller, a Space) in the same shape; account grants made from the sheet cover `SpaceScope::Any` | P |
| Answers | "Allow" stores one grant for the account picked; "Don't Allow" stores an `Always` denial for every account offered, so the app is not prompted again until Settings changes it; closing the sheet stores nothing; a `Once` grant is spent by the first token issued under it | P |
| Data classes | `AppOwn`, `Mail`, `Calendar`, `Contacts`, `Notes`, `Files`, `Photos`, `Clipboard`, `Screen`, `Voice`, `Public` (closed enum; `Voice` is the person's own voice audio) | P |
| Prompt | by capability, not by provider: "Photos wants to keep its library in your Nextcloud files" (`Alert` on a `Sheet{Centre}` from the sheet host) | P |
| First-party apps | Mail, Photos, Calendar, Notes, Files and the shell still get one prompt at first use; no silent pre-grant, so the per-app list in detent is complete | P |
| Identity of caller | `AppId { name: AppName (reverse DNS), isolation: Isolation {Flatpak, Unsandboxed, InProcess} }`, established by the transport, never sent by the caller. Flatpak: app id from the sandbox info of the caller's pid (`GetConnectionCredentials`, pidfd); native: the systemd `app-<id>-*.scope` cgroup, marked "unsandboxed" (R12) | P |
| Flatpak reach | `--talk-name=org.quire.Accounts1` finish arg; consent is enforced inside the daemon | P |
| Storage | the consent store is accountd's own table, PermissionStore-shaped, so a portal can adopt it; persisted with the registry as `$XDG_STATE_HOME/porter/registry.json` (`Persisted { vocab, accounts, grants, toggles }`, atomic write; JSON, not SQLite: tens of rows, no SQLCipher in mailo); a vocabulary bump now needs a migration test [2026-10-05] | P |
| Voice | the caller `org.quire.Voice` with class `Voice` and an `OnDevice` route holds a shipped default grant, because the person's first-use voice consent (a sill sheet) is the grant; any other route for `Voice` needs `ai.local_only` off, a floor change and an explicit grant; an app transcribing its own files uses its own class and the normal grant | P |
| Audit | grants, denials, revocations, token issues (with audience), proxy opens, sign-ins, re-auths, removals, adoptions: time, app, account, capability; never content; `$XDG_STATE_HOME/quire/accountd/audit.jsonl`, the shape of inferd's [2026-10-05] | P |
| Add and allow | when an app's `Choose` finds no fitting account and the person adds one from that sheet, the last step reads "Add, and allow Mail to use it": one grant, not a second prompt [refines, 2026-10-05] | P |
| Audience | `IssueToken` refuses an audience outside the grant's kind (`AudienceNotGranted`) | P |

### 4.6 Secrets

| Rule | St |
| --- | --- |
| Refresh tokens, passwords, API keys and E2E device keys live in the Secret Service through oo7 (Linux), keyring-core stores elsewhere, attributes `{service, account, purpose}`; never SQLite, never config files, never logs (mailo's rule, [R] §1) | P |
| Apps receive short-lived access tokens, an `ApiKeyHandle` (inferd resolves it through `Peer.ResolveKey`; the key itself never leaves), or an authenticated relay fd (`OpenAuthenticated`, v1); there is no password token kind [2026-10-05] | P |
| Backends: `Oo7Secrets` on Linux; `KeyringSecrets` (keyring-core's Apple and Windows stores, mailo's chunking) on macOS and Windows, its blocking calls on a dedicated thread; not the `keyring` crate's Secret Service store (blocking zbus, the known zbus/tokio hazard) [2026-10-05] | P |
| For AI, apps never see keys at all: inferd makes the call | P |
| Removing an account wipes secrets, grants, sync journal rows and caches in one step, after an `Alert{Critical}` naming what is removed | P |
| No Secret Service (headless): oo7's file backend keyed by the Secret portal, or refuse with `Unavailable` | P |

## 5. App integration

### 5.1 The client library

```rust
use porter_client::{Accounts, Found};
use porter_core::{need::StorageNeed, DataClass, Need, consent::Usage};

let accounts = Accounts::connect(&env).await?;       // the first reachable link in env: D-Bus, then the socket
                                                     // (an app hosting the core: Accounts::over(InProcess::new(service, app)))
let need = Need::Storage(StorageNeed {
    access: Access::ReadWrite,
    delta: Delta::Poll,                              // minimum
    scope: StorageScope::AppFolder,                  // minimum: AppFolder accepts either scope
    quota: QuotaReport::Unreported,
});
let candidate = match accounts.find(&need, DataClass::Photos, Usage::Interactive).await? {
    Found::One(candidate) => candidate,
    Found::Several(list) => pick(list),              // AccountPicker over granted accounts, no daemon call
    Found::NeedsConsent(offer) => accounts.request_grant(&offer, &window).await?,  // Manager.Choose
    Found::None(why) => return Ok(View::NoAccount(why)),  // NeedsAccount | Denied | Unsupported
};
let token = accounts.token(&candidate, &Audience("webdav".into())).await?;  // short-lived; ask again on 401
```

| Piece | Rule | St |
| --- | --- | --- |
| `Need` | mirrors `Capability` with minimums; one variant per kind; protocol fields (transport, wire, hashes) are not asked for | P |
| `Found` | closed enum; the app renders every arm | P |
| `Accounts` | `connect`, `over`, `find`, `request_grant`, `add_account`, `token`, `grants`, `revoke`, `infer`; a refusal is `ClientError::Refused(Refusal)` or `InferRefused(InferRefusal)` | P |
| Family clients | `StorageClient`, `MailClient` (mailo), `PimClient`, `LlmClient` (talks to inferd) | P |
| UX pieces (quire) | `AccountPicker` (a `PopUpButton` listing granted accounts + "Add Account…"), `NoAccount` (`EmptyState` with an action), `AccountBadge` (`ProviderMark` + `Label`), `LimitedNote` (`Label{Secondary}` from `Restriction`) | P |

### 5.2 Mail (mailo)

| Step | Change | St |
| --- | --- | --- |
| 1 | mailo's account add flow moves to accountd's add sheet; mailo calls `Choose`/`AddAccount` | P |
| 2 | mailo's `Secrets`, OAuth, renewal come from porter; on Linux via `DbusTransport`, elsewhere `InProcess` | P |
| 3 | IMAP/JMAP/SMTP/Graph engines and protocol-native sync stay in mailo; `AccountCaps` stays mailo's discovered detail under `Capability::Mail` | P |
| 3a | for a password account the IMAP, SMTP and CardDAV engines take a pre-authenticated stream from `OpenAuthenticated` (IMAP `PREAUTH`) instead of a password; OAuth accounts keep XOAUTH2 or bearer tokens from `IssueToken` [2026-10-05] | P |
| 4 | Contacts and calendars move to the Calendar app and a Contacts reader through the same grants | P |

### 5.3 Photos on Storage

The honest route to the reference's cloud photo library ([P] §4): **the library lives in a
folder of any Storage account**; provider photo APIs are extras.

| Part | Design | St |
| --- | --- | --- |
| Library home | `Photos Library/` in the chosen account's Storage (AppFolder is enough) | P |
| Originals | human-readable placement `Originals/2026/09/IMG_1234.HEIC`, identity by content hash (BLAKE3) in the manifest; same file twice is stored once | P |
| Edits | non-destructive recipe beside the original: `Edits/<asset-id>.json` (20 §2.9) | P |
| Manifest | `Library/log/` append-only segments of asset records (hash, path, capture date, place, favourite, hidden, albums, deleted-at), LWW per field with a hybrid logical clock | P |
| Derived | thumbnails and previews local only (regenerated); optional upload later | P |
| Local | a SQLite catalogue plus a cache; "Optimize storage" keeps previews and fetches originals on open | P |
| Delete | tombstone into Recently Deleted for 30 days (setting `photos.recently_deleted_days`), then the original is removed | P |
| Import | local folders, camera/phone (MTP later), Apple privacy export, Google Photos Picker session (C1) | P |
| Upload targets | Google Photos `appendonly` as an optional one-way backup, album "From <desktop name>" (R1) | P |
| On-device search | faces, objects, places by local models through inferd with `DataClass::Photos` pinned to OnDevice (20 §2.9) | P |

Provider verdicts for the library home:

| Provider | Home? | Why |
| --- | --- | --- |
| Nextcloud, OneDrive, Dropbox, Fastmail, WebDAV | yes, bidirectional | files with delta or etags |
| Google Drive | yes, AppFolder | `drive.file` sees the folder we create (R2) |
| S3 / B2 | yes, backup-grade | no delta: syncd keeps the manifest as the change feed |
| Google Photos | upload target and picker import only | C1 |
| iCloud | no | C4 |

### 5.4 Files and Notes

| App | Use | St |
| --- | --- | --- |
| Files (20 §2.3) | every Storage grant appears under a "Locations" `SectionHeader` in the sidebar; browse online, selective sync to a local folder (v2); placeholders deferred (§1.3) | P |
| Open/save | the file-chooser backend lists Storage accounts beside local places (v2, O) | O |
| Notes (20 §2.8) | local notes by default; `Notes` capability (Nextcloud Notes, IMAP notes) as a sync target in v2 | P |

### 5.5 The AI broker (inferd)

| Aspect | Rule | St |
| --- | --- | --- |
| One API | apps send a typed `InferRequest` (`Chat { messages, shape: Text | Json(schema) | Choice(strings), tier, class, usage, tools, control }`, `Embed { inputs, role: Query | Document, dims, class, usage }`, `Task`, `CuaBegin`, `CuaStep`, `Transcribe`, `Speak`) on a streaming session (`Accounts::session`, or `Accounts::infer` to read to the end); events stream and one `Finished(InferReply)` ends each turn, with `ServedBy { account, model, locality }`; wire formats stay inside inferd's adapters ([A] §4). Messages carry text, images (inline base64 or an attached memfd), tool calls and tool results, and thoughts with their seal (apps never construct a thought; inferd keeps it across tool turns); `Cancel` ends a turn and drops the engine stream | P |
| Chat controls | `ChatRequest.control: ChatControl { tool_choice: Auto | Never | Required | Named(name), tool_calls: One | Many, max_output: Knob<Tokens>, reasoning: EngineDefault | Off | On(Effort), sampling: Knob<Sampling { temperature, top_p, top_k, min_p, seed }>, stop }`; `Knob` is `Off` (the model's catalog default) or `Set(v)`, so an app that does not care writes no number; numbers are thousandths (`Permille`), never floats. `ChatReply` carries `stop: StopReason {EndTurn, ToolUse, MaxTokens, StopSequence, ContentFilter}` (a planner tells a turn cut by `max_output` from a finished one) and `thought: Option<String>`; `TokenUsage` carries `cached` (prompt tokens the engine served from its cache). A `Named` tool choice an engine cannot enforce is refused, not downgraded | P |
| Embeddings | `Embed` takes a `role` (a search query or an indexed passage); inferd puts the model's `prompts` prefix before each text, batches to `max_batch` and checks every reply for the right count and width before a vector reaches a store (a wrong-width vector is an error, never stored). Models that ignore a requested `dimensions` are not sent one; the width comes from the capability | P |
| Retry and structured output | inferd owns both, so every client gets the same behaviour. Retry: a 5xx, a rate limit (honouring `Retry-After`) or a loading engine is retried with a bounded backoff, never after the first event reached the client. Structured output: for `shape: Json` and `Choice` inferd constrains decoding where the engine supports the shape, otherwise asks through a tool call or the prompt, validates the final text against the shape and repairs at most a configured number of times (a repair prompt names the field and the expected shape, never the model's output); a client gets a reply that already passes the check or `Unparseable`, and a reply cut by `max_output` is never repaired | P |
| Task layer | above raw chat: `Task {Summarise, Rewrite, Extract, Classify}` (embeddings, images, computer-use steps and speech are their own requests; speech to text is not a task because its input is audio); the shell uses these, not model ids | P |
| Computer use | `CuaBegin { goal, hints, env }` then `CuaStep` requests on the same session: a window frame (raw pixels on a memfd preferred), geometry, cursor, what became of the last actions, a mask count and the accessibility tree as text; the reply is window-space `CuaAction`s plus dropped and add-only safety hints. The session needs a `ComputerUse` capability and class `Screen`; history lives in the session, so the session is pinned to one model | P |
| Speech | `Transcribe` (streaming or batch; 16 kHz mono S16LE `Audio` frames of at most one second, in order, then `EndOfAudio`) answers `Heard` events (partial, final, language) and a `Transcribed` reply; `Speak` answers `Spoken` audio events and a `Spoke` reply. The class of a `Speak` is the class of its text. inferd keeps no audio after a turn and audits only `audio_ms` | P |
| Model picker | settings `ai.model.<kind>.<tier>` = `<account>/<model>` for the kinds `Llm`, `ComputerUse`, `Embeddings`, `SpeechIn`, `SpeechOut`, `ImageGen`, `Rerank`; the picker is a plain list per kind (this computer first, ready before loadable before downloadable, then catalog order) with readiness, memory fit and licence beside each row and no ranking or recommended row; a non-commercial model is listed and never chosen for the person | P |
| Routing | `route(ask, candidates, policy)`: drop cloud under local-only (nothing left: `Unavailable`), drop what the class floor forbids (nothing left: `RequiresCloud(class)`), keep granted ones (else `NeedsGrant` or `Denied`), drop stopped spend caps (else `OverBudget`); rank `OnDevice > LocalNetwork > Cloud`, then the user's per-tier choice, then price | P |
| Data-class policy | per class, a `Floor {OnDevice, LocalNetwork, Anywhere}`: `Mail`, `Calendar`, `Photos`, `Notes`, `Files`, `Contacts`, `Screen`, `Clipboard`, `Voice` default to `OnDevice`; `Public` and `AppOwn` default to `Anywhere` (settings `ai.floor.<class>`, rows for 22 to add); a blocked request returns `RequiresCloud(class)`, never a silent downgrade to a cloud model | P |
| Local-only switch | one setting (`ai.local_only`, default on) removes every Cloud account from every answer (R14) | P |
| Per-app permission | grant per (app, AI kind, data class); background use (indexing) is a separate grant with a cheap-tier preference | P |
| Spend caps | `SpendCap { scope: Account | App, period: Daily | Monthly, limit: MicroUsd, warn_at: Permille }` from response usage and a price table (OpenRouter publishes prices); warn at `ai.spend.warn_permille` (proposed 800), stop when a request would reach the limit; `PlanBudget` accounts count requests (not modelled yet) | P |
| Rate and queue | token bucket per app and account; interactive first; `Retry-After`; GPU queue (§3.3) | P |
| Indicator | a bar item shows when a request leaves the machine, naming the provider | P |
| Audit | app, account, model, tokens, bytes, time; content never stored by default | P |
| Tools | a chat request carries the functions a model may call (`ToolDecl`, from the companion's typed actions); the model's calls come back as `ToolCall` events and parts. inferd only passes them: the router (`docket`) gates every call, and MCP is an edge binary there (`actions-mcp`), not an inferd role | P |

### 5.6 detent's Accounts page and first run

detent renders the list through `org.quire.SettingsModule1` (22 §9.4); flows that need a
browser or a secret are sheet-host sheets that detent opens with `AddAccount` /
`Reauthenticate`.

| Screen | Components (design/30) | St |
| --- | --- | --- |
| Sidebar | `Row` "Accounts" and `Row` "Intelligence" in the `List{SourceList}` | P |
| Accounts list | `List{Inset}` of `Row`: leading `ProviderMark`, title the address, detail the enabled kinds ("Mail, Calendar, Files"), trailing `Accessory::Chevron`; state `NeedsReauth` shows `Badge{Alert}` and a `Button{Inline}` "Sign In…"; footer `Button{Push}` "Add Account…" | P |
| Account detail | `SectionHeader` "Services" then one `Row` per capability with `Accessory::Toggle`; a limited one shows a `Label{Secondary}` from `Restriction`; `SectionHeader` "Apps using this account" with `Row`s per grant and a Revoke `Button{Inline}`; `Button{Push, role Destructive}` "Remove Account…" → `Alert{Critical}` | P |
| Add sheet (the sheet host) | `Sheet{Attach::Window, Regular}`: `List{Inset}` of provider `Row`s, "Other…" → `FieldRow`s of `TextField` (address, server, `Secure` password); OAuth step shows `ProgressIndicator{Spinner}` "Continue in your browser" and a `Button` "Copy link"; discovery result shown as `Row`s with `Toggle`s before Done | P |
| Consent | `Alert{Informational}` with app icon, one sentence, "Don't Allow" / "Allow" | P |
| Intelligence page | `Toggle` "Use cloud models" (inverse of local-only), `List` of AI accounts (local ones marked "On this computer"), per-tier `PopUpButton`, spend `TextField` + `Stepper`, "Usage" `Table` (P2), per-app `Row`s with `Toggle` | P |
| First run | two optional steps in the setup flow: "Sign in to your accounts" (the provider list, "Skip") and "Intelligence" (`RadioGroup`: Off / On this computer only / Also cloud accounts; default On this computer only when a local runtime is found, else Off) | P |

## 6. Sync engine

### 6.1 Contract

One small trait per backend family (File Provider and rclone shape, [R] §2 sync engines),
implemented by each `storage-*` crate and by the fake that tests drive (CONVENTIONS §5).

```rust
pub trait Replica: Send + Sync {   // porter-sync; async fns written as `-> impl Future + Send`
    async fn changes(&self, from: Cursor) -> Result<ChangePage, ReplicaError>; // Cursor::{Start, At(Anchor)}; changes + next + More|Done
    async fn fetch(&self, item: &RemoteId, range: ByteRange) -> Result<Blob, ReplicaError>;
    async fn put(&self, item: PutItem, base: BaseVersion) -> Result<(RemoteId, RemoteVersion), PutRefused>;
    async fn remove(&self, item: &RemoteId, base: BaseVersion) -> Result<RemoteVersion, PutRefused>;
    fn features(&self) -> StorageCap;
}
pub enum BaseVersion { Absent, At(RemoteVersion) }          // PutItem.target: New(ItemPath) | Existing(RemoteId)
pub struct Conflict { item: RemoteId, base: BaseVersion, remote: RemoteSide } // Changed(v) | Deleted(v) | Exists(id, v)
pub enum PutRefused { Conflict(Conflict), Quota, Forbidden, Transient(RetryAfter) }
pub enum ReplicaError { AnchorExpired, Unauthorized, Transient(RetryAfter), Gone }
```

`Cursor::Start` is a full listing of what exists, without tombstones: what a replica reads after
`AnchorExpired`. A removal returns the tombstone's version and leaves `Change::Tombstone { id,
version, deleted_at }` in the feed. porter-sync's `MemoryReplica` is the reference semantics the
contract tests drive.

| Rule | Detail | St |
| --- | --- | --- |
| Anchors | opaque per replica: Drive page token, Graph delta link, Dropbox cursor, WebDAV sync-token or etag tree, S3 our manifest's head | P |
| Anchor expiry | `AnchorExpired` → full listing reconciled by content hash, no re-upload of known content | P |
| Base version | every write carries the version it was based on; a mismatch is a `Conflict`, never an overwrite | P |
| Conflicts | stored as objects `{item, base, local, remote}` in the journal, shown by the owning app, resolved per dataset rule (§6.2) | P |
| Tombstones | kept with a deletion time until every replica acknowledges, then compacted | P |
| Journal | SQLite per dataset: items (local id, remote id, hash, remote version, base version, state), anchors, tombstones, conflicts | P |
| Scheduler | one per syncd: push where declared, else poll with backoff; batched wake-ups; paused on metered networks (setting) | P |
| Idempotence | every operation resumable; uploads chunked where `chunked_upload` is `Present` | P |

### 6.2 Per-dataset models

| Dataset | Model | Conflict rule | Engine | Wave |
| --- | --- | --- | --- | --- |
| Photos originals | content-addressed, immutable | none possible (same hash = same thing) | syncd | v1 |
| Photos metadata and edits | manifest log, LWW per field, HLC | later clock wins per field; edit recipes keep both as versions | syncd | v1 |
| Files (selective sync) | whole-file versions by etag/hash | keep both: "name (conflict from <device>)" | syncd | v2 |
| Settings, Spaces, dock, widgets, style CSS | encrypted item log with cursor (§6.4), LWW per key, HLC; lists as small CRDTs only where order matters (dock) | per key; dock order merges | syncd `KeyValue` | v2 (O) |
| Keychain | encrypted items, LWW per item | conflict object shown in the Keychain UI | syncd | v3 (O) |
| Mail, calendars, contacts, tasks | protocol-native (CONDSTORE/QRESYNC, JMAP state, CalDAV/CardDAV sync-token, Graph delta) | the protocol's | mailo, Calendar app | v1 (mail) |
| Calendars and contacts until their apps exist | `PimMirror`: server to a local vdir, `$XDG_DATA_HOME/porter/vdir/<account>/<collection>`, read-only first; sill's calendar widget reads it (`calendar.sources = auto`) under its own grant [departs: placement, 2026-10-05] | the server wins | syncd | v1 |
| Notes | protocol-native where the provider has notes; else files | keep both | app | v2 |

### 6.3 Continuity (option)

Device-to-device hand-off (open this document on the other machine, shared clipboard, send a
file) over Tailscale between the user's machines, keyed by the desktop account's device keys
(§6.4). Discovery by the tailnet's peer list; transport a mutually authenticated channel with
those keys. v3, O.

### 6.4 End-to-end encryption (for the desktop-own account options)

| Part | Design | St |
| --- | --- | --- |
| Keys | a random root key per user; per-dataset keys derived from it; the root wrapped to each device's X25519 key | O |
| New device | approved from an existing device by QR or short code (the simplified trust circle, [R] Apple Keychain) | O |
| Recovery | a printed recovery key; without it and without a device, data is lost; said plainly at setup ([R] pitfall 9) | O |
| Metadata | item names and paths encrypted; hashes keyed (no convergent encryption) | O |
| Transport | any Storage grant (the user's Nextcloud, OneDrive, Google AppFolder, S3) as a dumb store; a server of ours is optional (§7 v3) | O |
| Libraries | RustCrypto (`chacha20poly1305`, `x25519-dalek`), BLAKE3; no hand-rolled primitives | O |

## 7. Phasing

### 7.1 v1: the smallest useful layer

| Item | Scope | St |
| --- | --- | --- |
| Crates | porter's `porter-core`, `-provider`, `-secrets`, `-service`, `-client` (frozen), plus `porter-families`, `-oauth`, `-discover`, `-dav`, `-proxy`, extracted from mailo where mailo has the code; mailo builds on them on all three platforms | P |
| accountd | the whole D-Bus API of §4.4, `OpenAuthenticated` included (owner, 2026-10-05), `Peer` and the sheet conversation; consent; Secret Service via oo7; settings module | P |
| Kinds | Identity, Mail, Calendar, Contacts, Tasks, Storage (with quota), Llm, Embeddings, Speech, ComputerUse; ImageGen and Rerank when a request kind and a consumer exist | P |
| Providers | Nextcloud, Microsoft (our own verified multi-tenant client), generic IMAP/SMTP + DAV; AI: Ollama, llama.cpp, the supervised `local`; Anthropic key, OpenAI key and OpenRouter once stoker's cloud backends exist. Google: **TODO** (owner, 2026-10-05) | P |
| syncd | journal, scheduler, `Replica` for WebDAV and Graph (Google Drive with the Google TODO); the `PimMirror` dataset (§6.2); the Photos dataset built and contract-tested | P |
| inferd | routing, local-only default, data-class floors, per-app grants, spend caps, audit, GPU queue, streaming sessions; no MCP (an edge in docket) | P |
| UI | detent Accounts and Intelligence pages, sheets drawn by sill (quire `ds-shell::accounts` views), first-run steps | P |
| Photos | the app is its own repo and starts after mail and calendar work end to end (owner, 2026-10-05); library on Storage, import from folders | P |

Acceptance (each an automated test on private buses, scratch `XDG_*`/`HOME`, a fake Secret
Service and fake servers; the real system is never touched):

1. Add a Nextcloud account by Login Flow v2 against a fake server; `Query(Storage)` from a
   second process returns nothing until the consent prompt is accepted, then one candidate.
2. A Microsoft account whose tenant refuses `Notes.ReadWrite` shows Notes as
   `Absent { TenantConsent }` (probe path) and emits `CapabilityChanged`.
3. No process other than accountd ever holds a refresh token (checked by scanning every IPC
   reply in the test harness).
4. mailo passes its suite with `InProcess` (macOS/Windows CI) and with `DbusTransport` (Linux).
5. Photos: 1 000 fixtures imported on machine A appear on machine B through a fake WebDAV;
   a favourite set on both while offline resolves by clock; a duplicate import stores one
   original; an expired anchor reconciles with zero re-uploads.
6. With `ai.local_only = on`, a `summarise` request with `DataClass::Mail` from Mail runs on
   Ollama; with Ollama stopped it returns `Unavailable`, never a cloud call.
7. A spend cap of 1 USD stops further OpenRouter calls for that app and shows the reason.
8. Removing an account leaves no secret, grant, journal row or cache file behind.
9. A Flatpak-identified test app reads an app-password IMAP account through `OpenAuthenticated`
   without receiving the password (moved from v2), and the scan of item 3 also finds no
   password in any IPC reply or frame.

### 7.2 v2

| Item | Acceptance | St |
| --- | --- | --- |
| iCloud, Fastmail, Dropbox, generic JMAP, S3/B2, LM Studio, vLLM, Gemini, Mistral, xAI | each provider file passes the provider conformance test against a recorded fake | P |
| Google (when the owner lifts the TODO): Calendar, People, Tasks, Drive AppFolder, Photos upload and picker on our client; Gmail and full Drive on a user-supplied client | conformance against a recorded fake; the 7-day `Testing` reminder | O |
| ChatGPT sign-in (R9), after a registration spike | a plan-billed account completes a Responses call and shows `PlanBudget` usage | O |
| Files: Locations in the sidebar, selective sync; open/save chooser lists Storage accounts | a file saved from a sandboxed app lands in OneDrive and appears on the other machine | O |
| Desktop-own account: settings, Spaces, dock, widgets, style CSS synced E2E over a Storage grant | changing the dock on A shows on B within one poll; the Storage account holds only ciphertext | O |
| MCP edge (`actions-mcp` in docket, not inferd) | Notes exposes "search notes" as a tool; a chat in the launcher calls it after a confirmation | P |

### 7.3 v3 (the ambitious options)

| Item | Acceptance | St |
| --- | --- | --- |
| Keychain: passwords and SSH keys synced E2E; passkeys when a browser integration exists | an SSH key added on A signs on B via the agent; recovery key restores on a fresh machine | O |
| Self-hostable server: relay, push, blob store, `KeyValue` natively; one Rust binary | two machines on different networks sync settings with push, server sees ciphertext only | O |
| Continuity over Tailscale | copy on A, paste on B; "continue this document" opens on B | O |
| GNOME Online Accounts compatibility: export Mail/Calendar/Contacts accounts on `org.gnome.OnlineAccounts` | Evolution sees our account without a second sign-in (GOA interface details from memory, U, to verify before building) | O |
| Portal proposal `org.freedesktop.portal.OnlineAccounts` upstream | spec draft submitted | O |
| Proton Drive, Box, Azure/Foundry, GitHub Models | conformance tests | O |

## 8. Open decisions (for the user; each has a recommendation)

**Answered 2026-10-05** (with the accounts plan's D1-D18, `~/rs-wt/accounts/PLAN.md` §7): 1 is
left as a TODO (no Google work for now); 2 (a); 3 as recommended without Google, with the cloud
AI keys after stoker's cloud backends and ComfyUI when an ImageGen consumer exists; 14 departs:
sill hosts the sheets on our desktop (§4.1). Also decided: `OpenAuthenticated` in v1 (§4.4);
syncd in v1 with `PimMirror`; JSON registry and jsonl audit; oo7 on Linux and keyring-core
stores elsewhere. Items 4-13 stand as recommended.

1. **Google OAuth client.** (a) ship our own client and pay for restricted-scope
   verification now; (b) ship our own client for sensitive and non-sensitive scopes only
   (Calendar, Contacts, Tasks, `drive.file`, Photos upload/picker) and let Gmail and full Drive
   use a user-supplied client until there are users; (c) user-supplied client only.
   **Recommend (b)**: brand and sensitive-scope review are free; restricted verification
   (and possibly a yearly CASA) waits until the product has users. (C2, C3)
2. **Microsoft client.** (a) ship one multi-tenant public client with publisher verification;
   (b) user-supplied. **Recommend (a)**: free, no assessment. (C11)
3. **v1 providers.** **Recommend** Nextcloud, Microsoft, Google (limited), generic
   IMAP/SMTP + DAV; AI: Ollama, llama.cpp, ComfyUI, Anthropic key, OpenAI key, OpenRouter.
   iCloud, Fastmail, Dropbox in v2. (§7.1)
4. **Photos library default home.** (a) ask at first launch among Storage grants; (b) local
   only until the user picks. **Recommend (a)**, with "This computer only" offered as the
   last choice.
5. **AI default.** (a) off until chosen; (b) local-only on when a local runtime is found;
   (c) cloud allowed. **Recommend (b)** with the first-run step making it explicit. (R14)
6. **ChatGPT sign-in.** (a) spike the open-source registration path in v2; (b) API keys only.
   **Recommend (a)**. (C7)
7. **Desktop-own account and E2E sync of settings/Spaces/dock/widgets/style CSS.** (a) yes in v2
   over the user's own Storage, no server of ours; (b) no. **Recommend (a)**.
8. **Keychain scope.** (a) none; (b) passwords + SSH keys in v3; (c) also passkeys.
   **Recommend (b)**; passkeys need browser integration we do not have.
9. **Self-hosted server.** (a) never; (b) v3, optional, one binary; (c) a hosted service of ours.
   **Recommend (b)**; (c) would put Google restricted data at risk of passing a server (R3).
10. **Continuity over Tailscale.** **Recommend** v3, after the desktop-own account exists.
11. **GNOME Online Accounts compatibility.** **Recommend** defer to v3 and verify GOA's
    interfaces first.
12. **Repos and boundaries.** (a) one new repo holding `accounts-*`, accountd, accounts-ui,
    syncd, inferd and the provider files, mailo depending on it by path like latchkey; (b) split
    AI (inferd) into its own repo. **Recommend (a)** now, split later if inferd grows. Name
    candidates: `porter` (holds the keys, forwards the deliveries), `lanyard`. **Recommend
    `porter`.**
13. **Consent for first-party apps.** (a) one prompt each; (b) silent pre-grant. **Recommend
    (a)**, so the per-app list is always the whole truth. (§4.5)
14. **Where the add/consent sheets live.** (a) accounts-ui owned by accountd; (b) inside
    detent. **Recommend (a)**: apps and detent both call it, and consent must not be drawn by
    the asking app.

## 9. Sources

- Research: [P] §1-§4, [A] §1-§7, [R] §1-§5 (files listed at the top).
- Fetched 2026-09-30 (H): developers.google.com/photos/support/updates;
  developers.google.com/workspace/gmail/api/auth/scopes;
  developers.google.com/workspace/drive/api/guides/api-specific-auth;
  developers.google.com/identity/protocols/oauth2/production-readiness/restricted-scope-verification;
  support.apple.com/en-us/102654; code.claude.com/docs/en/legal-and-compliance;
  developers.openai.com/siwc/token-sharing-open-source; openrouter.ai/docs/guides/overview/auth/oauth;
  learn.microsoft.com/en-us/windows/ai/apis/phi-silica; github.com/linux-credentials/oo7.
- Press or community (M): thenewstack.io/sign-in-with-chatgpt;
  github.com/google-gemini/gemini-cli/discussions/20632;
  winbuzzer.com/2026/02/23/google-bans-ai-subscribers-openclaw-no-refunds-xcxwbn;
  macrumors.com/2026/05/05/ios-27-third-party-chatbots-apple-intelligence;
  theregister.com/software/2026/09/24/talk-of-ai-in-kde-sets-the-community-ablaze/5298854.
<!-- paths: skip -->
- Code (read-only): `~/mailo/crates/mail-domain/src/account.rs` (`AuthPlan`, `OAuthIssuer`,
  `AccountCaps`, `SecretKey`, `SecretPurpose`, `Credential`), `mail-runtime/src/{oauth,signin,renewal,secrets}.rs`,
  `latchkey/src/lib.rs`.
<!-- paths: end -->
- Our docs: 20 §1.17, §2.1, §2.3, §2.8, §2.9; 22 §9.4; 30 §2; quire `CONVENTIONS.md` §2-§7.

## 10. Decision list (short, for the user)

1. Google: a TODO for now (2026-10-05).
2. Microsoft: ship one verified multi-tenant client.
3. v1 providers: Nextcloud, Microsoft, generic IMAP/DAV; AI: Ollama, llama.cpp, the supervised `local`; cloud keys after stoker's cloud backends.
4. Photos library home: ask at first launch among Storage accounts.
5. AI default: local-only on when a local runtime is found.
6. ChatGPT sign-in: spike the open-source registration in v2.
7. Desktop-own account with E2E sync of settings, Spaces, dock, widgets, style CSS: yes in v2, over your own Storage.
8. Keychain: passwords and SSH keys in v3; passkeys later.
9. Self-hosted server: optional, v3; no hosted service of ours.
10. Continuity over Tailscale: v3.
11. GNOME Online Accounts compatibility: defer to v3.
12. Repo: one new repo (`porter` suggested) for the account core, daemons and provider files.
13. First-party apps also ask for consent once.
14. Sign-in and consent sheets are drawn by a trusted sheet host that only accountd opens: sill on our desktop, the app hosting the core elsewhere.
15. Passwords never leave accountd: `OpenAuthenticated` relays in v1.

## 11. Frozen interfaces (porter)

The interfaces of sections 2 to 6 are frozen as code in the porter repo
(github.com/PoHsuanLai/porter, private): types, traits, the provider file format, the wire
protocol and the D-Bus signatures compile, pure logic is implemented with table tests, and
behaviour that needs a provider, a vendor, a bus or a keyring is stubbed. porter's
`ARCHITECTURE.md` is the map (section 5 there says what is built and what is stubbed; section 8
maps mailo's types). This document stays the source of truth: a change to a frozen interface is
an edit here and a change there in step.

| Section here | porter crate | Frozen as |
| --- | --- | --- |
| §2 vocabulary, provenance, restrictions | `porter-core` (and `prov` for the companion's shared ids, labels and `Actor`) | `Capability`, `CapabilityKind`, `VocabVersion(2)`, `ComputerUse`, `Need`, `matches -> Match`, `Claim`/`Offer`/`Subject`/`Provenance`, `effective`, `Restriction`/`Limit`, `Locality`/`Tier`/`Billing` |
| §3 providers | `porter-provider` | `ProviderSpec` (the TOML file), `parse_provider`, `ProviderSet`, `Family`, `Issuer`, `Discovery`, traits `Provider` and `ProviderSession` |
| §4.1-4.3 core, transports | `porter-service`, `porter-client` | `AccountService` over `Provider`, `Secrets`, `Prompter`, `Clock`; `AccountsRequest`/`AccountsReply`; `Transport` (`call` for accountd, `open_with` for an inference session, with `open` the same without trace options) with `DbusTransport`, `SocketTransport`, `InProcess`; `InferSession` |
| §4.4 D-Bus | `porter-dbus` | proxies and skeletons for `org.quire.Accounts1` (`Manager`, `Account`, `Grants`, `Tokens`, `Request`), `org.quire.Sync1`, `org.quire.Inference1` (`Open`, `Prepare`, `EnginesChanged`, `Gpu`); `dbus/*.xml` |
| §4.5 consent | `porter-core` `consent` | `Grant<K = GrantKey>`, `GrantKey` (with `space`), `decide<K: Eq> -> Verdict`, `SpaceId`/`SpaceScope`, `availability -> Availability`, `ConsentAsk`/`ConsentAnswer`, `AppId` |
| §4.6 secrets | `porter-secrets` | `Secrets` (get, put, delete, delete_account), `attributes`, `MemorySecrets`, `Oo7Secrets` (stub) |
| §5.1 client | `porter-client` | `Accounts`, `Found`, `ConsentOffer`, `NoAccount` |
| §5.5 AI broker | `porter-infer` | `InferRequest` (chat with tools and `ChatControl`, embeddings with a role, tasks, computer use, speech), `StopReason`, `OpenOptions`/`Traceparent`, `ClientFrame`/`InferEvent`/`InferReply`, `Readiness`, `AiKind`/`TierMap`/`PickerRow`, `Policy`/`Floor`, `route`, `SpendCap`/`spend_verdict`/`cost`, `AuditEntry`, trait `Model`, `Broker` (stub) |
| §6 sync | `porter-sync` | `Replica`, `Cursor`, `ChangePage`, `Conflict`, `DatasetKind`/`ConflictRule`, `MemoryReplica` |
| §4.1 processes | `accountd`, `syncd`, `inferd` | skeletons that build their service and exit "not implemented" |

Frozen decisions that differ from the first proposal of this document, each already applied in
its section: closed sets use serde slugs rather than `Word` (§2.1); `Restriction` holds per-kind
`limits` (§2.5); every capability row in a provider file writes every field (§3.1); `Usage`
(interactive or background) is part of every grant and query (§4.4, §4.5); `Availability` adds
`denied`; `Found::Several` is picked app-side from granted accounts; `Calendar` joins the
on-device floor; `Replica::changes` takes a `Cursor` and writes return versions (§6.1). The companion
freeze (2026-10) added: `VocabVersion(2)` with `ComputerUse`, `DataClass::Voice` and the Space
dimension of a grant (§2.3, §4.5); streaming inference sessions with tools, computer-use steps
and speech on the `Open` fd, `Prepare`, `EnginesChanged` and `Gpu` (§4.4, §5.5); and the removal
of inferd's MCP host role (§5.5).

The rig amendment (2026-10-03) added, before any fill: chat controls, a stop reason, a thought seal and `Choice` replies on the chat wire; an embedding role, batch limit and prompts; the `options` vardict of `Inference1` with the reserved `traceparent`; and the decision that inferd owns retry and structured-output repair (§5.5). The matching code is porter's `porter-infer`, `porter-core` and `porter-dbus`; stoker's `model-provider`, `model-wire` and `model-extract` hold the engine side.
