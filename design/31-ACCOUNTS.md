# 31 Accounts: the account and capability layer

Status: this whole document is **proposed** (2026-09-30) unless a line says settled. It
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
| C3 | Unverified Google apps: 100-user cap and a warning screen; Testing-mode refresh tokens expire after 7 days | M | [P] §2 Google, §3 | **R5** Client IDs are a registry per issuer per build channel (mailo `signin.rs` already does this) with a "bring your own client ID" override; `TokenLifetime::SevenDays` shows a re-sign-in reminder instead of a silent failure. |
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
`VocabVersion(1)`. Adding a kind or a field is a vocabulary bump, released with the daemon and
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
    Llm, Embeddings, Speech, ImageGen, Rerank, KeyValue, Push }   // implements `Word`
```

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
| `Llm` | `features: BTreeSet<LlmFeature {Chat, Tools, Vision, AudioIn, Pdf, StructuredOutput, Reasoning, PromptCache}>`, `context: Tokens`, `max_output: Tokens`, `wire: LlmWire {ChatCompletions, Responses, Messages, GenerateContent}` | P |
| `Embeddings` | `dims: Dims`, `modalities: BTreeSet<Modality {Text, Image}>`, `max_input: Tokens` | P |
| `Speech` | `modes: BTreeSet<SpeechMode {Stt, Tts, Realtime}>`, `languages: LanguageSet` | P |
| `ImageGen` | `modes: BTreeSet<ImageMode {TextToImage, Edit, Inpaint}>`, `max_side: Px` | P |
| `Rerank` | `max_docs: Count` | P |

Two more properties sit beside every AI capability, not inside it:

| Property | Values | Why |
| --- | --- | --- |
| `Locality` | `OnDevice`, `LocalNetwork`, `Cloud { region: Option<Region> }` | the routing policy and the data-class rules read it (§5.5) |
| `Tier` | `Fast`, `Balanced`, `Best` | the user maps tiers to models per account; apps ask for a tier, never a model id ([A] §7) |
| `Billing` | `Free`, `Metered { price: PriceTable }`, `PlanBudget` | spend caps (§5.5) |

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

### 2.5 Restriction metadata

```rust
pub struct Restriction {
    pub verification: Verification,   // NotNeeded | Verified | Unverified { user_cap: Count }
    pub token_lifetime: TokenLifetime, // Standard | SevenDays | UntilPasswordChange
    pub consent: TenantConsent,        // User | AdminRequired | AdminGranted
    pub limit_reason: Option<LimitReason>, // AppendOnly | PickerOnly | AppFolderOnly | ProviderOffersNone
}
```

The detent page (§5.6) turns each variant into one secondary line.

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
kind = "oauth_pkce"                      # or app_password, login_flow_v2, local_bridge, key_pair, api_key, oauth_mints_key, cloud_identity, none
issuer = "fastmail"
[discovery]
kind = "jmap_session"                    # autoconfig | well_known | jmap_session | nextcloud_ocs | fixed | probe_ports
[[capability]]
kind = "mail"; family = "jmap"; delta = "push"
[[capability]]
kind = "contacts"; family = "jmap"; delta = "push"
[[capability]]
kind = "storage"; family = "webdav"; endpoint = "https://myfiles.fastmail.com/"; delta = "poll"; scope = "full"
```

| Layer | Form | Closed? | Adding one needs |
| --- | --- | --- | --- |
| Capability kind | Rust enum (§2) | yes, versioned | a vocab bump |
| Family (protocol engine) | Rust enum `Family` + one crate or module each | yes | code |
| Auth kind | Rust enum `AuthKind` | yes | code |
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
| Google | OAuth PKCE, shipped or BYO client | L restricted (C2) | Y | Y People | Y | - (Keep is Workspace-only) | L AppFolder (`drive.file`) | L PickerOnly + upload | Poll | R1-R5 | v1 (Cal, Contacts, Tasks, Drive AppFolder, Photos upload); Mail after verification |
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
| ComfyUI | none | - | - | - | Y (workflow registry) | - | OnDevice | probe `:8188` + our workflow files | v1 (the user already runs it, `~/comfy`) |
| Anthropic | API key; Bedrock/Vertex/Foundry | Y | - | - | - | - | Cloud | `/v1/models` capabilities (H per [A]) | v1 |
| OpenAI | API key; ChatGPT sign-in later (R9) | Y | Y | Y | Y | - | Cloud | curated table | v1 |
| OpenRouter | OAuth mints key (R10) | Y | Y | L | Y | Y | Cloud | `/api/v1/models` modalities (H) | v1 |
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
| LAN | a URL on another host is `LocalNetwork` (e.g. a Tailscale peer); never auto-probed | P |
| GPU arbitration | one 16 GB GPU (RTX 5070 Ti): inferd serialises local LLM and ComfyUI jobs, asks Ollama to unload idle models (`keep_alive`), exposes `GpuBusy` so apps show a queue state | P |
| Readiness | `Availability {Ready, Downloadable, Downloading, Unavailable}` per capability, the pattern of the browser and Windows on-device APIs ([A] §5) | P |

## 4. Service architecture

### 4.1 Processes

| Process | Owns | Why separate | St |
| --- | --- | --- | --- |
| `accountd` | account registry, provider registry, token broker, secrets, consent store, discovery and probes, the Settings live module | the control plane; small, must never stall | P |
| `syncd` | the sync journal, anchors, the polling scheduler, dataset plug-ins (Photos first) | a failing backend or a long upload cannot block sign-in ([R] §3) | P |
| `inferd` | the AI broker: routing, policy, spend, audit, wire adapters, GPU queue, MCP host | streams and big payloads; restartable alone | P |
| `accounts-ui` | the sheets accountd must draw itself: add account, consent prompt, chooser, re-auth | an app cannot draw its own consent; same split as portal backends ([R] §2 portals) | P |

All are user-session services, D-Bus activated on Linux, single-instance through latchkey
([R] §1). syncd and inferd get tokens from accountd like any client.

### 4.2 Crates and the extraction from mailo

The core comes out of mailo the way latchkey did, coordinated with the mailo session: mailo
moves to depending on it in the same change (CONVENTIONS §3: no alias left behind).

| Crate | Content | From mailo | I/O | Portable |
| --- | --- | --- | --- | --- |
| `accounts-core` | `Account`, `AccountId`, `ProviderSpec`, `Capability` (§2), `Restriction`, `AuthKind`, `Credential`, `SecretKey`, `SecretPurpose`, `Grant`, `DataClass`, pure `step` functions (effective capabilities, grant decisions, routing policy) | `mail-domain/src/account.rs` (`AuthPlan`, `OAuthIssuer`, `Credential` with its redacting `Debug`, `SecretKey`, `SecretPurpose`), `presets/` | none | yes |
| `accounts-auth` | OAuth PKCE, loopback listener, client registry, renewal, app passwords, Login Flow v2, OpenRouter key mint | `mail-runtime/src/{oauth,signin,renewal,loopback}.rs` | HTTP | yes |
| `accounts-secrets` | `Secrets` trait (get, put, forget) + backends: oo7 (Linux Secret Service, MIT), `keyring` (macOS, Windows), map fake for tests | `mail-runtime/src/secrets.rs` | keyring | yes |
| `accounts-discover` | autoconfig, `.well-known`, SRV, JMAP session, Nextcloud OCS, port probes | `mail-runtime/src/discover.rs`, `mail-proto/src/discover` | network | yes |
| `accounts-client` | the app-facing API (§5.1); trait `AccountsLink` with three implementations: `DbusLink`, `LatchkeyLink`, `EmbeddedLink` | new | per link | yes |
| `accountd` | zbus front end, consent store, probes; also serves the latchkey front end where D-Bus is absent | new | yes | Linux first |
| `sync-core`, `syncd` | journal, `Replica` trait (§6.1), dataset models | new | SQLite | core yes |
| `storage-*` | one per family: `webdav`, `graph`, `gdrive`, `dropbox`, `s3` | new (mailo's CardDAV WebDAV code reused) | HTTP | yes |
| `infer-core`, `inferd` | typed request model, adapters (`ChatCompletions`, `Responses`, `Messages`, `GenerateContent`, `OllamaNative`, `ComfyWorkflow`), policy | new | HTTP | core yes |
| `mail-proto`, `mail-pim` | stay in mailo, now "family" crates for Mail, Calendar, Contacts | unchanged | none | yes |

`mail-domain` keeps mail-shaped types (`Incoming`, `Outgoing`, `AccountCaps` for IMAP detail);
`AccountPlan` references an `accounts_core::AccountId` and a `Grant`. The OAuth client ID stays
deployment config, never in the account (as mailo's `AuthPlan` comment already says).

### 4.3 Transports

| Platform | Link | Who hosts the core | St |
| --- | --- | --- | --- |
| Linux, our desktop | `DbusLink` to `org.quire.Accounts1` | accountd | P |
| Linux, another desktop; macOS; Windows | `LatchkeyLink` (framed messages over latchkey's socket / named pipe) | accountd built without zbus, or mailo's own agent hosting the core | P |
| mailo standalone, tests | `EmbeddedLink` (in process) | the app | P |

The request and reply types are one serde enum in `accounts-core`; D-Bus and latchkey both
carry them, so there is one protocol with two carriers.

### 4.4 D-Bus API sketch

Bus name `org.quire.Accounts1`; root `/org/quire/Accounts1`. Shaped like a portal (Request
objects with a `Response` signal, `handle_token`), so a later `org.freedesktop.portal`
proposal can reuse it ([R] §2 portals).

| Interface | Member | Signature (sketch) | Notes |
| --- | --- | --- | --- |
| `.Manager` | `Query(need: (sa{sv}), data_class: s) -> a(o s a{sv})` | candidates the caller **already holds a grant for**: account path, label, effective capability, `Restriction`, verdict | no bulk enumeration ([R] Android `GET_ACCOUNTS`) |
| | `Availability(need) -> (s)` | `Granted`, `AvailableNeedsConsent`, `NeedsAccount`, `Unsupported` | reveals no identities |
| | `Choose(need, data_class, parent_window: s, options) -> o` | Request; accounts-ui shows a chooser with only matching accounts; reply carries the account path and a new grant | the one way an app learns of a new account |
| | `AddAccount(provider_hint: s, parent_window) -> o` | Request; opens the add sheet | apps offer "Add account…" |
| `.Account` (per object) | properties `Id`, `Provider`, `Label`, `State` (`Ok`, `NeedsReauth`, `Offline`, `Limited`), `Capabilities` | readable only with a grant | |
| | `Reauthenticate(parent_window) -> o` | Request | |
| `.Grants` | `Revoke(grant: s)`; `List() -> a(...)` (caller's own) | | |
| `.Tokens` | `IssueToken(grant: s, audience: s) -> (kind s, value s, expires x)` | `kind`: `Bearer`, `Xoauth2`, `ApiKeyHandle`; short-lived access tokens only | refresh tokens never leave (§4.6) |
| | `OpenAuthenticated(grant: s, endpoint: s) -> h` | a socket fd to a daemon-side authenticated proxy (IMAP LOGIN, WebDAV basic, app passwords) | password protocols without releasing the password |
| Signals | `AccountAdded(o)`, `AccountRemoved(o)`, `CapabilityChanged(o)`, `NeedsReauth(o)`, `GrantChanged(s)` | sent only to holders of a relevant grant (unicast) | |
| `org.quire.SettingsModule1` | at `/org/quire/Accounts1/settings`: `Describe`, `Get`, `Set`, `Changed` | 22 §9.4 | detent reads it |

syncd serves `org.quire.Sync1` (`Datasets`, `Status(dataset)`, `Pause`, `Resume`, signals
`Progress`, `Conflict`); inferd serves `org.quire.Inference1` (`Availability(need)`,
`Open(need, data_class, tier) -> h` returning an fd that carries a framed request/stream
session, `Usage`, `Rescan`).

### 4.5 Consent

| Rule | Detail | St |
| --- | --- | --- |
| Unit | a grant is `(AppId, AccountId, CapabilityKind, DataClass) -> Allow | Deny` plus `Scope::{Once, Always}` | P |
| Data classes | `AppOwn`, `Mail`, `Calendar`, `Contacts`, `Notes`, `Files`, `Photos`, `Clipboard`, `Screen`, `Public` (closed enum) | P |
| Prompt | by capability, not by provider: "Photos wants to keep its library in your Nextcloud files" (`Alert` on a `Sheet{Centre}` from accounts-ui) | P |
| First-party apps | Mail, Photos, Calendar, Notes, Files and the shell still get one prompt at first use; no silent pre-grant, so the per-app list in detent is complete | P |
| Identity of caller | Flatpak: app id from the sandbox info of the caller's pid (`GetConnectionCredentials`, pidfd); native: the systemd `app-<id>-*.scope` cgroup, marked "unsandboxed" (R12) | P |
| Flatpak reach | `--talk-name=org.quire.Accounts1` finish arg; consent is enforced inside the daemon | P |
| Storage | the consent store is accountd's own table, PermissionStore-shaped, so a portal can adopt it | P |
| Audit | grants, token issues, proxy opens: time, app, account, capability; never content | P |

### 4.6 Secrets

| Rule | St |
| --- | --- |
| Refresh tokens, passwords, API keys and E2E device keys live in the Secret Service through oo7 (Linux), `keyring` elsewhere, attributes `{service, account, purpose}`; never SQLite, never config files, never logs (mailo's rule, [R] §1) | P |
| Apps receive short-lived access tokens, an `ApiKeyHandle` (inferd resolves it; the key itself never leaves), or an authenticated proxy fd | P |
| For AI, apps never see keys at all: inferd makes the call | P |
| Removing an account wipes secrets, grants, sync journal rows and caches in one step, after an `Alert{Critical}` naming what is removed | P |
| No Secret Service (headless): oo7's file backend keyed by the Secret portal, or refuse with `Unavailable` | P |

## 5. App integration

### 5.1 The client library

```rust
use accounts_client::{Accounts, Need, Found, DataClass};

let accounts = Accounts::connect(&env).await?;              // picks Dbus, Latchkey or Embedded
let need = Need::Storage(StorageNeed {
    access: Access::ReadWrite,
    delta: Delta::Poll,               // minimum
    scope: ScopeNeed::AppFolderOrFull,
});
let handle = match accounts.find(&need, DataClass::Photos).await? {
    Found::One(h) => h,
    Found::Several(list) => accounts.choose(&need, DataClass::Photos, &window).await?,
    Found::NeedsConsent(offer) => accounts.request(offer, &window).await?,
    Found::None(why) => return Ok(View::NoAccount(why)),   // EmptyState + "Add Account…"
};
let drive: StorageClient = accounts.storage(&handle).await?; // typed family client, tokens renewed inside
```

| Piece | Rule | St |
| --- | --- | --- |
| `Need` | mirrors `Capability` with minimums; one variant per kind | P |
| `Found` | closed enum; the app renders every arm | P |
| Family clients | `StorageClient`, `MailClient` (mailo), `PimClient`, `LlmClient` (talks to inferd) | P |
| UX pieces (quire) | `AccountPicker` (a `PopUpButton` listing granted accounts + "Add Account…"), `NoAccount` (`EmptyState` with an action), `AccountBadge` (`ProviderMark` + `Label`), `LimitedNote` (`Label{Secondary}` from `Restriction`) | P |

### 5.2 Mail (mailo)

| Step | Change | St |
| --- | --- | --- |
| 1 | mailo's account add flow moves to accountd's add sheet; mailo calls `Choose`/`AddAccount` | P |
| 2 | mailo's `Secrets`, OAuth, renewal come from `accounts-*`; on Linux via `DbusLink`, elsewhere `EmbeddedLink` | P |
| 3 | IMAP/JMAP/SMTP/Graph engines and protocol-native sync stay in mailo; `AccountCaps` stays mailo's discovered detail under `Capability::Mail` | P |
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
| One API | apps send a typed request (messages, content parts, tools, schema, tier, `DataClass`) through `LlmClient`; wire formats stay inside inferd's adapters ([A] §4) | P |
| Task layer | above raw chat: `summarise`, `rewrite`, `extract`, `classify`, `embed`, `transcribe`, `image`; the shell uses these, not model ids | P |
| Routing | candidates = accounts with a matching capability and a grant; filter by policy; rank `OnDevice > LocalNetwork > Cloud`, then the user's per-tier choice, then cost | P |
| Data-class policy | per class, a floor: `Mail`, `Photos`, `Notes`, `Files`, `Contacts`, `Screen`, `Clipboard` default to `OnDevice`; `Public` and `AppOwn` default to `Any`; a blocked request returns `RequiresCloud { class }`, never a silent downgrade to a cloud model | P |
| Local-only switch | one setting (`ai.local_only`, default on) removes every Cloud account from every answer (R14) | P |
| Per-app permission | grant per (app, AI kind, data class); background use (indexing) is a separate grant with a cheap-tier preference | P |
| Spend caps | per account and per app, daily and monthly, from response usage and a price table (OpenRouter publishes prices); warn at 80 %, stop at 100 %; `PlanBudget` accounts count requests | P |
| Rate and queue | token bucket per app and account; interactive first; `Retry-After`; GPU queue (§3.3) | P |
| Indicator | a bar item shows when a request leaves the machine, naming the provider | P |
| Audit | app, account, model, tokens, bytes, time; content never stored by default | P |
| Tools (MCP) | apps register MCP servers (session socket); inferd is the MCP host, mediates each tool call under the app's grants, uses elicitation for confirmations, and offers a sampling-compatible entry so an MCP server can use the user's model (spec 2025-11-25, [A] §4) | P |

### 5.6 detent's Accounts page and first run

detent renders the list through `org.quire.SettingsModule1` (22 §9.4); flows that need a
browser or a secret are accounts-ui sheets that detent opens with `AddAccount` /
`Reauthenticate`.

| Screen | Components (design/30) | St |
| --- | --- | --- |
| Sidebar | `Row` "Accounts" and `Row` "Intelligence" in the `List{SourceList}` | P |
| Accounts list | `List{Inset}` of `Row`: leading `ProviderMark`, title the address, detail the enabled kinds ("Mail, Calendar, Files"), trailing `Accessory::Chevron`; state `NeedsReauth` shows `Badge{Alert}` and a `Button{Inline}` "Sign In…"; footer `Button{Push}` "Add Account…" | P |
| Account detail | `SectionHeader` "Services" then one `Row` per capability with `Accessory::Toggle`; a limited one shows a `Label{Secondary}` from `Restriction`; `SectionHeader` "Apps using this account" with `Row`s per grant and a Revoke `Button{Inline}`; `Button{Push, role Destructive}` "Remove Account…" → `Alert{Critical}` | P |
| Add sheet (accounts-ui) | `Sheet{Attach::Window, Regular}`: `List{Inset}` of provider `Row`s, "Other…" → `FieldRow`s of `TextField` (address, server, `Secure` password); OAuth step shows `ProgressIndicator{Spinner}` "Continue in your browser" and a `Button` "Copy link"; discovery result shown as `Row`s with `Toggle`s before Done | P |
| Consent | `Alert{Informational}` with app icon, one sentence, "Don't Allow" / "Allow" | P |
| Intelligence page | `Toggle` "Use cloud models" (inverse of local-only), `List` of AI accounts (local ones marked "On this computer"), per-tier `PopUpButton`, spend `TextField` + `Stepper`, "Usage" `Table` (P2), per-app `Row`s with `Toggle` | P |
| First run | two optional steps in the setup flow: "Sign in to your accounts" (the provider list, "Skip") and "Intelligence" (`RadioGroup`: Off / On this computer only / Also cloud accounts; default On this computer only when a local runtime is found, else Off) | P |

## 6. Sync engine

### 6.1 Contract

One small trait per backend family (File Provider and rclone shape, [R] §2 sync engines),
implemented by each `storage-*` crate and by the fake that tests drive (CONVENTIONS §5).

```rust
pub trait Replica {
    async fn changes(&self, from: Anchor) -> Result<ChangePage, ReplicaError>; // changes + next anchor + More|Done
    async fn fetch(&self, item: &RemoteId, range: ByteRange) -> Result<Blob, ReplicaError>;
    async fn put(&self, item: PutItem, base: BaseVersion) -> Result<RemoteVersion, PutRefused>;
    async fn remove(&self, item: &RemoteId, base: BaseVersion) -> Result<(), PutRefused>;
    fn features(&self) -> StorageCap;
}
pub enum PutRefused { Conflict(Conflict), Quota, Forbidden, Transient(RetryAfter) }
pub enum ReplicaError { AnchorExpired, Unauthorized, Transient(RetryAfter), Gone }
```

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
| Crates | `accounts-core`, `-auth`, `-secrets`, `-discover`, `-client` extracted from mailo; mailo builds on them on all three platforms | P |
| accountd | D-Bus API of §4.4 minus `OpenAuthenticated`; consent; Secret Service via oo7; settings module | P |
| Kinds | Identity, Mail, Calendar, Contacts, Storage, Photos (upload/picker), Llm, Embeddings, ImageGen | P |
| Providers | Nextcloud, Microsoft, Google (non-restricted scopes; Gmail only with a user-supplied client), generic IMAP/SMTP + DAV; AI: Ollama, llama.cpp, ComfyUI, Anthropic key, OpenAI key, OpenRouter | P |
| syncd | Replica for WebDAV, Graph, Google Drive (AppFolder); Photos dataset only | P |
| inferd | routing, local-only default, data-class floors, per-app grants, spend caps, audit, GPU queue; no MCP yet | P |
| UI | detent Accounts and Intelligence pages, accounts-ui sheets, first-run steps | P |
| Photos | library on Storage, import from folders and the Google Photos Picker, optional Google Photos upload | P |

Acceptance (each an automated test on private buses, scratch `XDG_*`/`HOME`, a fake Secret
Service and fake servers; the real system is never touched):

1. Add a Nextcloud account by Login Flow v2 against a fake server; `Query(Storage)` from a
   second process returns nothing until the consent prompt is accepted, then one candidate.
2. A Microsoft account whose tenant refuses `Notes.ReadWrite` shows Notes as
   `Absent { TenantConsent }` (probe path) and emits `CapabilityChanged`.
3. No process other than accountd ever holds a refresh token (checked by scanning every IPC
   reply in the test harness).
4. mailo passes its suite with `EmbeddedLink` (macOS/Windows CI) and with `DbusLink` (Linux).
5. Photos: 1 000 fixtures imported on machine A appear on machine B through a fake WebDAV;
   a favourite set on both while offline resolves by clock; a duplicate import stores one
   original; an expired anchor reconciles with zero re-uploads.
6. With `ai.local_only = on`, a `summarise` request with `DataClass::Mail` from Mail runs on
   Ollama; with Ollama stopped it returns `Unavailable`, never a cloud call.
7. A spend cap of 1 USD stops further OpenRouter calls for that app and shows the reason.
8. Removing an account leaves no secret, grant, journal row or cache file behind.

### 7.2 v2

| Item | Acceptance | St |
| --- | --- | --- |
| iCloud, Fastmail, Dropbox, generic JMAP, S3/B2, LM Studio, vLLM, Gemini, Mistral, xAI | each provider file passes the provider conformance test against a recorded fake | P |
| ChatGPT sign-in (R9), after a registration spike | a plan-billed account completes a Responses call and shows `PlanBudget` usage | O |
| `OpenAuthenticated` proxy for password protocols | a Flatpak test app reads an app-password IMAP account without receiving the password | P |
| Files: Locations in the sidebar, selective sync; open/save chooser lists Storage accounts | a file saved from a sandboxed app lands in OneDrive and appears on the other machine | O |
| Desktop-own account: settings, Spaces, dock, widgets, style CSS synced E2E over a Storage grant | changing the dock on A shows on B within one poll; the Storage account holds only ciphertext | O |
| MCP host in inferd | Notes exposes "search notes" as a tool; a chat in the launcher calls it after an elicitation prompt | P |

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
- Code (read-only): `~/mailo/crates/mail-domain/src/account.rs` (`AuthPlan`, `OAuthIssuer`,
  `AccountCaps`, `SecretKey`, `SecretPurpose`, `Credential`), `mail-runtime/src/{oauth,signin,renewal,secrets}.rs`,
  `latchkey/src/lib.rs`.
- Our docs: 20 §1.17, §2.1, §2.3, §2.8, §2.9; 22 §9.4; 30 §2; quire `CONVENTIONS.md` §2-§7.

## 10. Decision list (short, for the user)

1. Google: ship our own client for non-restricted scopes; Gmail and full Drive on a user-supplied client until verification is worth paying for.
2. Microsoft: ship one verified multi-tenant client.
3. v1 providers: Nextcloud, Microsoft, Google (limited), generic IMAP/DAV; AI: Ollama, llama.cpp, ComfyUI, Anthropic, OpenAI, OpenRouter keys.
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
14. Sign-in and consent sheets belong to the account service, not to detent or the app.
