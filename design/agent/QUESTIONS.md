<!-- paths: skip -->
<!-- Reference copy of ~/rs-wt/agent-spec/QUESTIONS.md for the companion freeze (2026-10-02). It names files of other repos and of crates that do not exist yet, so the path check skips it. -->
# Companion: decisions for the user

Merged from models.md, actions.md + addendum, memory.md, cua.md, ux.md and the consolidation
(SPEC.md). Duplicates are folded together. **Rec** is the default the freeze uses until you answer.
Ids match the references in SPEC.md.

## Platform and placement

| Id | Decision | Options | Rec |
|---|---|---|---|
| P1 | Repo names and placement | (a) `stoker` (models), `docket` (actions + the companion agent), `almanac` (memory), `cua` (computer use), `prov` inside porter; (b) other names; (c) fold models into porter | (a); name the `cua` repo (placeholder). Also: must action names carry their app's prefix (`mail.…` only from Mail)? Rec: yes |
| P2 | CUA on stock cosmic-comp | (a) ship on stock (shared seat, in place = hands-off, masking in cuad, confirm only after cuad stops its input), fork in phase 2; (b) require the fork before any CUA release | (a) for opt-in use; the fork is required before "keep working while it acts" |
| P3 | COMPANION item 4 ("no screen clicking") | (a) replace with "typed actions and hooks first; a CUA tier, off by default per app, inside the lease and consent stack"; (b) keep item 4, no CUA | (a); `cua.enabled = off` either way |
| P4 | File capture | (a) inotify on Space roots for v1; (b) a small privileged fanotify helper | (a) |
| P5 | sqlite-vec needs one `unsafe` crate | (a) exact scan now, usearch only if scale demands; (b) allow one exemption crate | (a) |
| P6 | Space identity | where `SpaceId` is minted (rec: sill's Space store beside SpaceLook) and whether a `desktop` scope exists for memory outside any Space | sill mints; `desktop` exists |
| P7 | design/30 §3.4 defers companion work | (a) lift it for types now (freeze wave adds ds vocab, ds-intents, companion components as "proposed"); (b) types only outside `ds` | (a) |
| P8 | Engine confinement | (a) engines only as confined systemd transient units without network (unconfined child processes on other OSes); (b) also allow unconfined on Linux | (a) |
| P9 | The fork | who owns it (rec: the cua repo's agent, shared branch with double-tap ⌘ later); which hardware kill chord it reserves; is KWin a product target or only the nested host | cua owns; chord: user picks; KWin = nested host + reference only |
| P10 | Toshy route for double-tap ⌘ | (A) Toshy detects the double tap and emits one chord; (B) Toshy maps a lone ⌘ tap and sill's tested `Tap` machine decides | B if Spike T shows ⌘-click/⌘-drag unharmed, else A |
| P11 | Fuzzing | nightly-only `cargo-fuzz` dev script beside the stable proptest suite | yes |

## Safety defaults

| Id | Decision | Options | Rec |
|---|---|---|---|
| S1 | Outbound under Default strictness | (a) always ask (COMPANION 9 exactly); (b) run when inside a TaskPolicy from your own words, all sinks trusted, and two reviewers of different families agree | (a) for v1; (b) behind `TrustMore` only |
| S2 | Destructive | (a) always ask; (b) under `TrustMore`, inside a TaskPolicy, trusted args, two reviewers agree | (b) as specified, `HoldToConfirm` otherwise |
| S3 | TaskPolicy | lifetime: 1 h or task end; narrowing: silent or shown | task end, capped at 1 h; silent, visible in Activity |
| S4 | Prompts typed into an app's own field | the TaskPolicy derived from them reaches only that app (+ reads elsewhere); widening confirms | yes |
| S5 | Budgets and eval gates | the session budgets (actions §3.7), CUA run budgets (cua §7), reviewer timeouts (300/3000/3000 ms), per-corpus FNR release targets; settings, or fixed by you | settings with the proposed defaults; FNR targets set by you, only you may raise them |
| S6 | Actions you pick yourself in the launcher | skip policy (the app's own alerts apply) or go through it | skip |
| S7 | External MCP clients | expose actions in v1 (off by default) or not at all | off by default, present |
| S8 | Shadow index of third-party titles (mail subjects in intentd) | allow, per-app opt-out, encrypted at rest when | allow, per-app opt-out, encrypted with the Space key in fill 2 |
| S9 | Browser CUA grants | Once/per-session only, or "Always" allowed | Once/per-session only |
| S10 | CJK/Unicode typing by CUA on stock | allow the virtual-keyboard keymap trick (it targets your focus) or only the fork's agent text input | fork only; stock types ASCII via EI |
| S11 | Screenshots of CUA runs | off, no thumbnails | off |

## Models

| Id | Decision | Options | Rec |
|---|---|---|---|
| M1 | Engine for Holo-3.1-4B | vLLM BF16 (10.4 GB, documented by H Company) or llama.cpp (4B GGUF + mmproj unverified) | vLLM for this entry; default engine per catalog entry |
| M2 | One GPU, many roles (planner, reader, policy writer, reviewer quick/deliberate, CUA, embeddings, consolidation) | (a) one resident model (Holo-3.1-4B) for all language roles, a second small model of another family loaded on demand for SecondOpinion; (b) swap per role under eviction (30 s+ cold starts) | (a); without a second model, high-impact AllowJudged cells fall back to Ask. Name the second family |
| M3 | Picker granularity | per kind × tier (`ai.model.computer_use.balanced`) or named roles (Planner, Reader, Reviewer, Operator) mapped onto tiers | roles mapped onto tiers |
| M4 | Idle unload | 600 s; keep warm while the companion is open | 600 s, warm while open |
| M5 | Non-commercial models (Holo4) | list in the picker (never auto-chosen) or hide | list, marked |
| M6 | Element targets | let models address a11y nodes when a tree exists, or coordinates only | frozen both; prompt with node ids where the dialect supports them |

## Memory

| Id | Decision | Options | Rec |
|---|---|---|---|
| Me1 | Memory files at rest | (a) sealed per file, "open in editor" through tmpfs; (b) plain with disk encryption; (c) per-Space choice | (a) default, (c) allowed |
| Me2 | Strict cascade | a fact with two sources, one deleted: delete it, or keep it on the remaining source | delete (strict) |
| Me3 | Forget vs audit | may event headers (kind, actor, time) outlive a forget | yes; full erasure only by deleting the Space key |
| Me4 | Nightly consolidation | auto-apply tidy/stamp/trusted hunks (revertible, diff visible) or review every hunk | auto-apply |
| Me5 | Retention | the default table (memory §3.2) and a 14-day TTL for pending facts | as proposed |
| Me6 | Apps querying memory ("related to this thread") | v1 no, or yes per grant | no in v1 |
| Me7 | Export | plain tar only, or also encrypted (age); include the verification subkey | plain tar; subkey only when ticked |
| Me8 | CUA procedures | auto-propose promotion to a typed action, or only on "save as routine" | only on "save as routine" |
| — | Deferred, unchanged | sync between machines, bi-temporal dating, other people's data in your mail | room left in the formats; decide before building memory sync |

## UX and design review (looks are placeholders until reviewed)

| Id | Decision | Options | Rec |
|---|---|---|---|
| U1 | Prompt mode in a search field | query collapses into a chip and the field empties, or stays as editable text | chip |
| U2 | Multiline and editor fields | anchored prompt at the selection (Writing Tools style) or fall through to the launcher | anchored |
| U3 | Orb | home with the launcher closed (bar item on by default, or hidden while idle); Waiting look; size ladder; tint (orb colours or accent) | bar item on; sizes 14/16/20/40/192 |
| U4 | Answers | card in the launcher's preview pane or replacing the list; headless "done" feedback as a toast with Undo or a line in the strip | preview pane; toast with Undo |
| U5 | Glow | width, colours, acting spot; is phase A (sill overlay, forgeable) fine before the fork | phase A for development |
| U6 | Confirmation | sheet on the target window or centred alert; 500 ms arm delay; phase A (sill sheet) shippable or dev only | sheet; 500 ms; dev only for CUA, typed actions may ship on it |
| U7 | Memory UI home | launcher "Memory" filter + detent page for export and the diff, or one dedicated window | launcher filter + detent |
| U8 | Settings page name | "Intelligence" (design/31) or "Companion" | Intelligence |
| U9 | Inline replace in your field | run at once with Undo (COMPANION 9) or show the change and wait for Apply (agent-ux P1) | wait for Apply |
| U10 | Undo wording for agent actions in app menus | "Undo Archive (Companion)" or plain, with the actor only in the strip | plain |
| U11 | Default CUA run mode | agent workspace (out of sight, glow + peek) or in place | agent workspace where supported, else in place |
| U12 | New catalogue rows and orb duration tokens (design/30) | approve CompanionOrb, ContextChips, AnswerCard family, PlanList, RunRow, ActivityStrip, MemoryTimeline, ConfirmCard and OrbListen/OrbWork/OrbAct | approve as "proposed" for the freeze |

## Answered (user, 2026-10-02)
- P1: repos stoker / docket / almanac / cua (name kept); prov in porter.
- P2: CUA ships on stock cosmic-comp, opt-in per app (off by default); the fork adds the agent seat later.
- S1: Outbound always asks under Default; two-reviewer auto only under TrustMore.
- U9: inline replace shows the change and waits for Apply.
- Every other question: the **Rec** column stands until the user says otherwise (P3 replaces
  COMPANION item 4 with the CUA tier, off by default, as the user directed on 2026-10-02).

<!-- paths: end -->
