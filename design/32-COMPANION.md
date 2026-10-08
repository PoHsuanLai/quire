# 32 The companion

Status: **proposed**, except the items marked "settled", which the user decided on 2026-10-01 and
2026-10-02. The types and placeholder components named here are frozen in quire
(`30-CATALOGUE.md` section 2.13); every look is a placeholder for the user's design review. The
machinery behind the companion (models, actions, memory, computer use, the daemons) is
`33-AGENT.md`, which summarises the locked implementation spec in `agent/SPEC.md`.

The companion is the desktop's AI, used the way a person uses the computer: alongside you, always
there, one key away. This document says what it is and how it looks and behaves. It does not say
how the agent works inside.

## 1. Stance (user, 2026-10-01)

- The AI is a companion using the computer with you at the same time. It is always there, not per
  session, and one key away from your normal actions.
- The handover case: you are doing something by hand (searching mail, found too many), it gets
  tedious, you press one key and prompt the agent instead. What you had done (query, results,
  selection, view) is where it starts.
- Existing AI integrations "scratch the edges": giving context alone is not useful.
- No classic right-side chat panel. Something only a coherent desktop can do.

## 2. Settled

1. **One desktop-wide agent.** Its homes are the launcher and the control centre. Its presence is
   shown with subtle glow, dim and animation, integrated into the other surfaces. The `VoiceOrb`
   (`30-CATALOGUE.md` section 2.9) is the seed: the animation matters more than voice.
2. **No locking when both touch the same thing.** No freezing, no locks, no per-app concurrency
   interface. It just happens; the person learns when to step in. Apps must not become harder to
   write because of the agent.
3. **When it acts without asking:** follow current computer-use agents' norms; nothing beyond them.
4. **Reach:** typed actions and hooks first (the apps' typed actions, then what other apps already
   expose: D-Bus, portals, MPRIS, accessibility with consent), then a computer-use tier that is off
   by default and on per app, inside the lease and consent stack (user, 2026-10-02: this replaces
   the first draft's "no screen clicking").
5. **Coupling:** the launcher and apps couple fully: the launcher runs app actions headless, shows
   app previews, expands into compact forms, promotes into the app window.
6. **Context** comes through the same app interface (here, the selection as typed things, what is
   visible, the editable text target), mostly reported by quire components automatically.
7. **The key is a double tap of Command.** It works inside text fields and collides with nothing.
   In a field, that field becomes the prompt, keeping query, results and selection. Elsewhere, the
   launcher opens in companion mode with the focused window's context. Holding is voice (later).
   Tapping again, or Escape, returns. Detection is Toshy's for now and the compositor's later.
8. **Presence states**, each a look of the one orb and glow, never a panel: idle (still, zero
   frames), listening, working, acting here (a glow on the window it is acting in), waiting for
   you. Idle costs zero frames like every other surface.
9. **The acting rule falls out of typed actions:** every action declares an effect (read,
   undoable write, outbound, destructive). Read and undoable writes run; outbound and destructive
   ask. Outbound always asks under the default strictness (user, 2026-10-02); two reviewers of
   different families may let it run only under the `TrustMore` strictness.
10. **No locks means one shared undo:** agent actions land on the same undo history as the
    person's, labelled with who did them. Apps do nothing extra: the agent calls the same action
    path the person does.
11. **Memory, three layers.** An event log (the truth, with who did what and from where), agent
    memory as plain files (one folder per Space, add-only dated facts that link to what they came
    from, tidied nightly with a diff the person can read), and a rebuildable index. Spaces are
    hard walls with their own keys. Deleting a source deletes everything derived from it. Text
    read from mail or the web never writes lasting memory without the person's confirmation.

## 3. Presence, the orb and the glow

`CompanionPresence` (`ds-core::vocab`) is the one vocabulary: Idle, Listening, Working, Acting,
Waiting. It is derived in one place, sill's `presence_of`, with the priority Waiting, Acting,
Working, Listening, Idle. An app derives its in-field presence from its own prompt state.

- **The orb.** `CompanionOrb` wraps `VoiceOrb`. `orb_look(presence, motion)` is pure: Idle and
  Waiting are always still, Reduced motion makes every presence still, and a moving presence takes
  a period token. The ladder is Inline 14, Bar 16, Field 20, Module 40 and Hero 192 pixels
  (proposed). The tone (Calm, Bright, Held) is the stylesheet's and a design-review item.
- **The look table.** `orb_look(presence, motion)` (tone proposed):

  | Presence | Standard motion | Reduced motion |
  | --- | --- | --- |
  | Idle | inactive, no period, Calm | the same |
  | Listening | active, period Listen, Bright | inactive, no period, Bright |
  | Working | active, period Work, Bright | inactive, no period, Bright |
  | Acting | active, period Act, Bright | inactive, no period, Bright |
  | Waiting | inactive, no period, Held | the same |

- **Deriving the presence** (sill's `presence_of`, pure): Waiting if any confirmation, question,
  needs-you answer or taken-over run is open; otherwise Acting if a run is running in place or a
  typed action targets a visible window; otherwise Working if a prompt is sent or something runs
  aside; otherwise Listening if a prompt is being composed; otherwise Idle.
- **The glow per window.** None unless the window is leased. A leased window glows Acting while a
  step executes (the spot is the proposed point or the node's bounds), Working between steps and
  Waiting while paused on it.
- **The period tokens** `OrbListen`, `OrbWork` and `OrbAct` are duration tokens
  (`30-CATALOGUE.md` section 1.2): 12, 8 and 5 seconds, proposed. They are `Hold` tokens, since
  Reduced motion stills the orb instead of speeding it.
- **The glow.** The compositor draws the glow around the window the companion acts in; quire
  owns its values. `GlowLook` (None, Working, Acting with an optional spot, Waiting) goes through
  `glow_spec(look, scheme)` to a `GlowSpec`: three colours, a width, a blur, a period and a
  strength. None and Waiting have no period, so they cost no frames. Until the compositor draws it,
  the shell draws a phase A overlay around the window; the glow pulse while Working or Acting is
  allowed only while a run is live, never in Idle or Waiting (it is the one loop outside quire).
- **An unseen outcome.** A run that finished while the person was away leaves a still dot on the
  orb's corner and on its run row: the `ok` tone for `Outcome::Done`, the `danger` tone for
  `Outcome::Failed`. It does not move, so it costs no frames in any presence. Whether there is an
  unseen outcome is the consumer's: it passes `Some(outcome)` while there is one and `None` once
  the person has looked, and nothing here remembers. `CompanionPresence` is unchanged.

## 4. The prompt and the answers

- **A field becomes the prompt.** In a plain or search field the field itself is the prompt, the
  leading magnifier becomes the orb, and the query collapses into a `Query` chip and restores
  verbatim when prompt mode ends. `PromptHost` says where a summon goes by field kind: `Field`
  for plain and search fields, `Anchored` (a prompt at the selection, whose answer is an inline
  replace) for multiline fields and the editor, `Refused` for secure fields, where the summon falls
  through to the launcher with no field context. `prompt_step` is the pure machine (off, composing,
  sent, answered). `Input` mode is byte-for-byte today's field.
  The prompt machine, `prompt_step`:

  | State | Input | Next | Asks |
  | --- | --- | --- | --- |
  | Off | Summon with the kept query and chips | On, composing, empty text | Enter |
  | On, composing | Typed | the text replaced | none |
  | On, composing, text not empty | Send | On, sent | Submit |
  | On, sent | Answered | On, answered | none |
  | On, answered | Typed | On, composing (a follow-up) | none |
  | On, answered or sent | Escape | On, composing | CancelAsk if it was sent |
  | On, composing | Escape or a second summon | Off | Restore the kept query |
  | On, any phase | a second summon | Off | Restore the kept query, CancelAsk if it was sent |
  | On | a chip removed | the chip gone; a fixed chip ignores it | none |

- **Chips are the consent surface.** `ContextChip` (`ds-intents`) names what the prompt carries:
  the query, the results (with a count), the selection, the window, the app, the Space, a mention,
  a piece of text. A removable chip can be dropped; what is not shown is not sent. The context is
  frozen at the moment of the summon, before the field turns into a prompt.
- **Answers are cards.** `AnswerView` is Text, DraftReply, ProposedEvent, Plan, Replace, Form or
  Refused. Every card has a footer (which model served it and where it ran, the sources, the
  scope) and actions. An action carries its `EffectMark`; an outbound or destructive action is
  never drawn as the unmarked primary. Text the person reads is displayed as data: it never passes
  through a model on its way to the screen.
- **A plan** is groups of steps by effect with a state each and a phase (draft, running, stopped,
  finished, undoing, undone). Plans run at once; the calls that need confirming confirm one by one;
  a draft appears only when the planner asks for one. Excluded steps never run, Stop keeps what is
  done, and Undo all is offered only after a run.
- **Inline replace** shows the change in the person's field and waits for Apply (user,
  2026-10-02), then offers Undo through the field's own history.
- **A compact form** asks for the parameters an action still needs.
- **A refusal** says why in words the person can act on: it needs a cloud model they have not
  allowed, policy denies it, no typed action reaches the app and computer use is off for it, a
  budget ran out, or it failed.

## 5. The summon seam and runs

- **Summon.** sill's double-tap machine decides; if the focused app serves the companion, sill
  asks it. `CompanionPort` (installed by the app's companion client, `docket-ds`) answers
  synchronously: `TookField`, `TookAnchored`, `Restored` or `Declined`. A declined summon opens the
  launcher in companion mode. `NoPort` stands in where the feature is absent.
- **Runs.** A computer-use run is a row: goal, app, step against budget, the last thought, where it
  acts (in place, an agent workspace or a nested session), its state and the model that drives it.
  Controls are Watch, Take over, Hand back, Pause, Resume, Stop and Undo all; which are offered
  follows a table in sill. Escape closes one level, as everywhere; stopping is Command-period, the
  Stop button, or the compositor's hardware kill chord.
- **Activity.** The strip lists what was done, by whom, with Undo while the journal can undo it.

## 6. Memory

The control UI lives with the companion: a Memory filter in the launcher for the timeline and
forget, and the Intelligence settings page for export and the consolidation diff. `MemoryVerb`
(`ds-core::vocab`) is the one verb set: Forget, Keep, Discard, Open source, Export. A fact that
came from untrusted text is pending until the person keeps it. Forgetting names its consequences
("Also forgets 3 facts and 1 routine") and never deletes the source itself. The retention table
and the consolidation switches are `memory.*` settings (`22-SETTINGS.md` section 3.28).

## 7. Components and where they live

| Piece | Home |
| --- | --- |
| `CompanionPresence`, `MemoryVerb`, `EffectMark`, `ActorMark`, `Tally` | `ds-core::vocab` |
| `ThingMark`, `ContextChip`, `ChipKind`, `Removal`, `FieldMode`, `SummonSerial`, `SummonAnswerMark`, `ContextModel` | `ds-intents`, re-exported from `ds::components::companion` |
| `CompanionOrb`, `ContextChips`, prompt mode, `AnswerCard` and the answer shapes, `PlanList`, `ReplaceBar`, `RunRow`, `ActivityStrip`, `MemoryTimeline`, `ConsolidationView`, `ServedByChip`, `CompanionPort` | `ds::components::companion` (layer `companion`, above `chrome`, below `app`) |
| `ConfirmCard`, `glow_spec` | `ds-shell::confirm`, `ds-style::tokens::glow` |
| `OrbListen`, `OrbWork`, `OrbAct` | `ds-style::tokens::timing` |
| `Page::Intelligence` | `ds-settings::schema::key` |

`ds-intents` names no agent type. The adapter that converts the agent's wire to these marks
(`docket-ds`) lives in the docket repo, so quire never links an agent crate.

## 8. Confirmation

Confirmations are drawn on a surface apps and the agent cannot draw over or imitate, and accept
only real hardware input; synthetic input from the agent can never press Allow. Phase A is a
sill sheet (forgeable, for development); phase B is the same surface placed on the compositor's
trusted surface. `ConfirmCard` is its content:

- the asker and the target app as marks, a title the router generated from the action (never model
  text), the arguments as facts, and the verb of the default button;
- the effect, and a taint line when the companion read text it cannot vouch for ("It read a message
  from a sender you don't know");
- the offer (once only, or once or always), the gesture (press, or hold for what cannot be undone)
  and the arm state: for the arm delay (500 ms, proposed, `companion.confirm_arm_ms`) the buttons
  take no input.

## 9. Security

The agent never holds power directly. It asks brokers we own, the brokers check policy, and the
kernel boxes in everything that runs. The planner never reads untrusted text: a quarantined reader
returns typed values. One policy decision point decides every call from its effect, data class,
Space, caller and taint; tainted context downgrades outbound and destructive calls to "ask".
Consent is unforgeable (section 8). Daemons are confined by their units; local model runtimes have
no network. Everything is audited, undoable where possible, and budgeted per session and action
type, with a per-Space stop. The detail is `33-AGENT.md` and `agent/actions.md`.

## 10. Models and tooling

Open-source models only for now; the goal is the design and the interface, not the best model. A
provider interface with a picker, and one small resident model that drives the whole UX; a second
small model of another family loads on demand for a second opinion. Our own Rust where it is stable
and reusable; Python projects behind our interface where the field moves fast.

## 11. Deferred

Memory sync between machines, bi-temporal dating, and other people's data in your mail (decide
before building memory sync); voice; threads as windows; ambient suggestions; asking the screen.
