# 33 The agent machinery

Status: **proposed**. A summary of the locked implementation spec for the companion
(`agent/SPEC.md`, written 2026-10-02, consolidating five area specs). `32-COMPANION.md` says what
the companion is; this file says which repos and daemons make it, where each type lives, and which
document holds the detail. Where this summary and `agent/SPEC.md` disagree, SPEC.md wins; where
SPEC.md and an area spec disagree, SPEC.md wins.

## 1. Reading order

1. `32-COMPANION.md`: the companion as the person meets it.
2. This file: the machinery in one page each.
3. `agent/SPEC.md`: every signature that crosses an area or repo boundary.
4. The area specs below, for state tables, test lists and fill-wave splits.
5. `agent/QUESTIONS.md`: the decisions that are still the user's, with the default each freeze uses.

## 2. The area specs

| Area | File | Holds |
| --- | --- | --- |
| models | `agent/models.md` | provider trait, backends, engine supervisor, catalogue, vision prep, inferd streaming |
| actions | `agent/actions.md`, `agent/actions-addendum.md` (the addendum wins) | typed actions, the router, policy, review, the planner and reader contract, auto mode, task policy |
| gating research | `agent/research-action-gating.md` | the research behind the policy grid and the breaker |
| memory | `agent/memory.md` | event log, memory files, recall, sealing, the memory daemon |
| computer use | `agent/cua.md` | accessibility tree, agent seat, masked capture, leases, the run machine |
| ux | `agent/ux.md` | quire components, sill surfaces, the summon and double-tap machines |
| consolidation | `agent/SPEC.md` | names, one home per type, frozen interfaces, flows, the freeze plan |

## 3. Repos

| Repo | Holds | Portable |
| --- | --- | --- |
| stoker (new) | the model layer: provider trait, backends, computer-use action and parse, vision prep, catalogue, engine supervisor | yes |
| porter | the account and capability layer, plus `prov` (labels and the shared who and what vocabulary), `SpaceId`, streaming inference, inferd | yes |
| almanac (new) | memory: event log, memory files, recall, the memory daemon | yes (daemon Linux) |
| docket (new) | typed actions, router, policy, review, the MCP edge, and the companion agent (loop, daemon, reader), plus the adapter quire apps use | core yes; daemons Linux |
| cua (new) | computer use: accessibility tree, agent seat, masked capture, leases, run machine, the daemon, the compositor protocol | core yes; daemon Linux |
| cosmic-comp fork (new, GPL) | the compositor's agent seat, masked source, trusted surface, glow, kill chord | no |
| quire | `ds-intents`, the companion components and vocabulary, design docs | yes |
| sill | launcher companion mode, the companion service, providers, overlays, settings domain | no |
| detent | the Intelligence page row and icon | no |

Direction, acyclic and enforced per repo by each `check-boundary.sh`: stoker, then porter, then
almanac, then docket, then cua, then sill. quire depends on none of them; docket's adapter depends
on quire's `ds-intents`.

## 4. Where the shared types live

The spec's section 2 holds the whole table; the ones quire's part touches:

- `Effect`, `Actor`, labels (`Integrity`, `Confidentiality`, `Label`), ids and receipts: porter's
  `prov`. quire's `EffectMark` and `ActorMark` are views of them, converted in sill.
- Presence: `ds-core::vocab::CompanionPresence`, derived only by sill's `presence_of`.
- Memory verb: `ds-core::vocab::MemoryVerb`, used by sill's launcher too.
- Confirmation: the wire request is docket's; quire's `ConfirmView`, `ConfirmChoice` (was
  `ConfirmAnswer`), `TaintLine` (was `TaintNote`) and `GestureMark` are the card's own view.
- Glow: the protocol values are the compositor's; quire's `GlowLook` (was `GlowState`) feeds
  `glow_spec`.
- Settings: `agent.undo.keep_h`, `cua.autopause` and `cua.default_mode` replace the three
  duplicate `companion.*` keys (`22-SETTINGS.md` sections 3.26 to 3.30).

## 5. Daemons and buses

| Daemon | Repo | Serves | Calls |
| --- | --- | --- | --- |
| inferd | porter | `org.quire.Inference1`: availability, open and prepare a model session, usage; engines over Unix sockets in transient units | engines |
| intentd | docket | `org.quire.Intents1`: registry, search, run, context, session, the computer-use gate, control, request | apps' providers, the confirm surface, the reader, memory, inferd |
| companiond | docket | `org.quire.Companion1`: prepare, open, ask, close, and an answer object per task | intentd, inferd |
| readerd | docket | `org.quire.Reader1`: the quarantined reader, a separate process | intentd, inferd |
| memoryd | almanac | `org.quire.Memory1`: record, recall, control | inferd |
| cuad | cua | `org.quire.Cua1`, and a provider as `org.quire.Cua` | intentd's gate, inferd, memory, the compositor |
| sill's daemon | sill | `org.quire.Confirm1` and a provider as `org.quire.Shell` | all of the above as a client |

Caller identity is always derived from the connection, never sent. Every interface we own is
`org.quire.<Name>1`, at `/org/quire/<Name>1`.

## 6. The rules that shape every flow

- The planner sees the person's words and typed metadata; untrusted text reaches it only as an
  opaque handle, and a quarantined reader turns text into typed values.
- One policy decision point, one breaker and one audit: every call, including each computer-use
  step, goes through the router's gate. A model can tighten a decision, never loosen it.
- The effect table: read runs; an undoable write runs, judged by a reviewer unless the whole task
  is the person's own words; outbound asks; destructive asks. `TrustMore` is the only loosening.
- Memory for the planner is a router action, so taint is computed in one place; a fact proposed in
  a tainted session lands pending until the person keeps it.
- Confirmations are the sill sheet now (phase A) and the compositor's trusted surface later.
- Typed actions come first, hooks second, computer use last, and computer use is off by default and
  on per app.

## 7. The four flows the spec walks

1. Double tap in mail's search with twelve results, "forward the Lisbon receipts to accounting":
   summon, the field becomes the prompt, a turn, a plan, a confirm, done, undo.
2. A launcher prompt that needs a computer-use run in a third-party app.
3. A mail body containing an injection: the reader, the handle, the confirm that quotes recipients.
4. The person forgets a mail and the forget cascades through memory.

`agent/SPEC.md` section 5 gives the owner of every step.

## 8. What quire owns in the freeze

`ds-core::vocab` (the companion vocabulary), `ds-intents`, `ds::components::companion`, the
`ds-shell` confirmation card and glow values, the orb duration tokens, `Page::Intelligence`, the
`22-SETTINGS.md` rows for every `ai`, `agent`, `memory`, `cua` and `companion` key, and this
documentation. Every behaviour behind a frozen signature is `todo!()`, listed in `FINDINGS.md`.
quire does not own `31-ACCOUNTS.md` (porter's agent edits it), the `TextField` and `CommandPalette`
prompt props (the quire fill), or anything in the other repos.

## 9. The freeze plan

The spec's section 6 orders the agents (dependency list, stoker, quire, porter, almanac and docket,
cua and the compositor fork, sill and detent), the spikes that block fills, and the fill waves.
Every repo's gate is format, clippy with warnings denied, tests, the boundary script and the
licence check, and quire's adds the consumer check; each exit code is checked and nothing is piped.
