# 05 Motion

> **Superseded in part by `30-CATALOGUE.md`.** 30 section 1.1-1.3 replace: section 2 principles on springs and overshoot, section 3 (all timing tokens, motion levels Calm and Extra, per-Look tables), section 4 (the keyframe catalogue: only keyframes 30 names survive), section 5 assignments, section 8 exits, section 10 shell motion, section 12 (open decisions on levels and looks), and section 14.2 and 14.5 (spring tokens, Reduced). One motion model, the macOS one; levels are Standard and Reduced only. Where this file and 30 disagree, 30 wins.

> **Second stamp (2026-10-02).** Also history, not to be built from: sections 4 and 11 (moved to `archive/05-MOTION-keyframes.md`), and in sections 6, 7, 9 and 13 every keyframe, per-Look value, `X--b` alias trick, stagger, sweep or spring pop that 30 section 1.3 does not name. What survives here is the timing and Blitz-port knowledge (section 9's renderer rules, section 7's timers, section 14's driven motion). One quiet Look: Mac motion only.


Status: draft for review, 2026-09-23. Extracted from the two prototypes; nothing here is new
design except where a row says **proposed**. `S` = `~/mailo-design/mailo-spaces.html`
(newest; wins every conflict), `C` = `~/mailo-design/mailo-charm.html`, `B` =
`~/mailo-design/mailo-charm.bak.html`. `S:123` means line 123 of S. "The plan" means
`~/.claude/plans/vast-toasting-peach.md`; "Appendix A/C" are its appendices.

## 1. What this governs

Every moving thing in every surface that renders through quire: the timing tokens (durations,
easings, scalars) per look and per motion level, the complete set of keyframes with their exact
text, which element plays which keyframe with which token, every CSS transition, every timer the
prototypes ran in JavaScript (restated as Rust-owned timers, because Blitz fires no
`animationend`), the exit each operation gets, the rules the Blitz port must follow, and the
motion of the desktop shell surfaces (dock, launcher, bar menus, notifications, workspace tint).
It does not govern what a behaviour *does* (that is `06-INTERACTIONS.md`), nor colour, radius
and shadow per look (that is `07-LOOKS.md`). If a surface needs motion that is not in this file,
the answer is to add a row here first, not to write a duration in a stylesheet: the ds lint rule
`RawDuration`/`RawEasing`/`Keyframes` rejects it (plan, "Design: `<ds>`", `lint::Rule`).

## 2. Principles

Quoted from the prototypes. Each one is a rule the port is reviewed against.

1. **Every animation is a pure function of state.** "Every animation here is a pure function of
   state. The charm has to come from the shapes, the icon language and the timing curves instead"
   (C:1116-1119). "Same inbox, same behaviour, every time" (C:1115). No randomness: the
   hash-seeded tilt B had was removed, because "stable per thread" still means two rows behave
   differently for a reason you can't see (C:1377-1388, "Why the randomness went").
2. **Springs only on contact.** "Nothing loops and nothing bounces on its own. The only overshoot
   in the app is in the 200ms after you touched something" (C:1123-1124). mailo's
   mailo's tokens.css restates it: "`--e-spring` overshoots and is spent only on contact".
3. **Exits accelerate; entrances decelerate or spring.** Every exit uses the exit curve
   `cubic-bezier(.55,0,.75,.2)` (`--e-exit`, C:40; S inlines it at S:327, S:328, S:351, S:376,
   S:570, S:572). Every entrance uses `--e-out` or `--e-spring`.
4. **Acting vs opening.** "**Acting** gets an *exit*. The row folds, the sidebar gulps, the undo
   toast appears. Something left." "**Opening** gets a *spring in place*. Nothing moves in the
   list, nothing is undoable yet, and the row stays exactly where it was until you choose. An
   exit animation here would be a lie — it would promise a commit that hasn't happened"
   (C:1339-1343). Snooze and Label open a picker; Archive, Trash, Read act.
5. **The destination gulps; a changed count bumps.** "the place that received it gulps; a count
   that changed bumps" (S:338). The seal is "a dot that pops beside the place you are in"
   (S:344).
6. **Variation with a reason.** "an unread thread folds 15% slower than a read one, because it
   weighed more" (C:1125-1126; rule at S:329, C:1089). It is the only variation between rows.
7. **Nothing loops.** "SMALL MOTION — every one keyed to a state change, none loops" (S:290).
   Known exceptions in the prototypes, all listed so the port can decide each one: the
   destination preview `dest` (`infinite alternate`, S:447), the sync halo `breathe` and `spin`
   (C:288, C:291), and the Pip mascot (excluded, section 11). An animated emoji's loops
   (section 4.11, design/25-EMOJI.md sections 5 and 6) are quire's one idle exception: finite
   plays, only for 20 s after a wake or a mood change, then 0 frames. Errors shake once and hold still:
   "A looping error animation is something you learn to ignore inside a day" (C:2326). A loop
   while an operation runs is bounded by design/26-DETAILS.md §3.3 R4 (`PendingGrace`,
   `PendingCap`); the grammar of small state details is that file.
8. **Rows rise only when a list is first shown.** "rows rise in only when a list is first shown,
   not on every re-render" (S:292-294).
9. **The body is opaque.** The message body is animated as one block; staggering lives in the
   chrome around it (C:1352-1358, `.frame` C:472).
10. **Don't drop the row until its exit ends.** "don't drop the row from the signal until the exit
    animation ends, or the node vanishes mid-flight" (C:1402-1404). In quire this is the
    `Roster` `Leaving` state plus a `settle()` timer (section 7), not `onanimationend`.
11. **Never celebrate a retraction.** Unstar plays the same pop with no sparks (C:2288; S:1526
    fires sparks only when `t.starred` becomes true).

## 3. Timing tokens

### 3.1 Base tokens per look (motion level Standard)

Values as declared. "inh." means the look does not declare the token and inherits Post's value
from `:root` (C:13-47). S declares only the Post subset (S:17-19) and inlines the exit curve.

| Token | Post (S = C) | Riso | Tide | Candy | Source |
| --- | --- | --- | --- | --- | --- |
| `--t-tap` | 90ms | 80ms | 140ms | 110ms | S:17, C:37, C:61, C:79, C:798 |
| `--t-quick` | 170ms | 150ms | 260ms | 190ms | same |
| `--t-move` | 250ms | 230ms | 420ms | 300ms | same |
| `--t-big` | 420ms | 460ms | 720ms | 520ms | same |
| `--t-ambient` | 5s (C only; S has none) | inh. 5s | 7s | 7s | C:37, C:79, C:798 |
| `--e-out` | `cubic-bezier(.22,.9,.30,1)` | inh. | `cubic-bezier(.25,.7,.25,1)` | `cubic-bezier(.3,.8,.3,1)` | S:18, C:38, C:80, C:799 |
| `--e-spring` | `cubic-bezier(.34,1.42,.52,1)` | `cubic-bezier(.34,1.8,.5,1)` | `cubic-bezier(.25,.7,.25,1)` (= its `--e-out`: no spring) | `cubic-bezier(.34,1.5,.5,1)` | S:18, C:39, C:62, C:81, C:800 |
| `--e-exit` | `cubic-bezier(.55,0,.75,.2)` (S inlines it) | inh. | `cubic-bezier(.4,0,.6,1)` | inh. | C:40, C:82 |
| `--overshoot` | 1.04 | 1.09 | 1.0 | 1.07 | S:19, C:41, C:63, C:83, C:801 |
| `--squish` | .955 | .90 | .985 | .925 | same |
| `--lift` | -2px | -4px | -1px | -3px | same |
| `--tilt` | 2.2deg (C only) | 3.6deg | 0deg | 2.6deg | C:41, C:63, C:83, C:801 |
| `--stagger` | 26ms | inh. 26ms | 44ms | 30ms | S:19, C:42, C:84, C:802 |

Where the scalars are read: `--overshoot` inside `pop-in`, `row-in`, `compose-rise`, `chip-in`,
`cmdk-in`; `--squish` on `:active` of `.cmd .pin .item .mini .btn .seg button .view .tool`;
`--lift` on `.row:hover`, `.att:hover`; `--tilt` only on the C drag ghost (C:583);
`--stagger` in row, strip and reader stagger delays. `--t-ambient` only in `breathe` (C:288).

### 3.2 Motion levels (C:172-176)

Levels are one attribute that rescales the whole system: "The setting rescales the one system
rather than switching animations off one by one" (mailo mailo's tokens.css, same values as C). Only
six tokens change; every other token keeps the look's value.

| Token | Calm | Standard | Extra | Reduced (new, plan) |
| --- | --- | --- | --- | --- |
| `--t-tap` | = look | = look | = look | 60ms |
| `--t-quick` | = look | = look | = look | 60ms |
| `--t-move` | = look | = look | = look | 60ms |
| `--t-big` | 300ms | = look | 560ms | 60ms |
| `--t-ambient` | = look | = look | = look | 60ms (the halo stops being ambient; see 12) |
| named durations (3.4) | = table | = table | = table | 60ms each |
| `--e-out` | = look | = look | = look | not specified (proposed: = look) |
| `--e-spring` | `var(--e-out)` | = look | `cubic-bezier(.34,2.0,.5,1)` | not specified (proposed: `var(--e-out)`, as Calm) |
| `--e-exit` | = look | = look | = look | not specified (proposed: = look) |
| `--overshoot` | 1.0 | = look | 1.14 | 1 (neutral) |
| `--squish` | 1 | = look | .86 | 1 (neutral) |
| `--lift` | = look | = look | = look | not specified (proposed: 0px, "neutral") |
| `--tilt` | 0deg | = look | 5deg | 0deg (neutral) |
| `--stagger` | 0ms | = look | 34ms | 0ms (neutral) |

Reduced is new in quire (plan, "Token model": "durations per MotionLevel{Calm,Standard,Extra,
Reduced} … Reduced = 60 ms everywhere"). It replaces the prototypes' two mechanisms:

- the media rule `@media (prefers-reduced-motion: reduce){ *, *::before, *::after{
  animation-duration:1ms !important; animation-iteration-count:1 !important;
  transition-duration:60ms !important; } }` (S:807-809, C:768-774), which Blitz cannot rely on
  (the plan resolves `prefers-reduced-motion` in Rust from the portal and always writes an
  explicit `data-motion`);
- C booting `calm` when the viewer asks for reduced motion (C:1467-1469), and C's port note
  `@media (prefers-reduced-motion: reduce) { :root { --overshoot:1; --squish:1; --stagger:0ms; } }`
  (C:1417).

Under Reduced every animation still runs once, at 60 ms, so that every `settle()` timer and
every state that waits for an exit keeps working; iteration counts are forced to 1 (loops stop).

### 3.3 Effective values per look and level (as the prototype cascade computes them)

The level rules `body[data-motion=…]` (C:173-176) have the same specificity as the look rules
`body[data-look=…]` and win by source order over Riso and Tide (declared earlier, C:50-85) but
**lose to Candy** (declared later, C:777-803). So in C, Calm and Extra change Post, Riso and
Tide, and change nothing on Candy, which is the look C boots (C:1465). The table shows what the
prototype actually renders for the six level tokens; section 12 asks which the port adopts.

| Look | Level | `--t-big` | `--e-spring` | `--overshoot` | `--squish` | `--tilt` | `--stagger` |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Post | Calm | 300ms | = e-out | 1.0 | 1 | 0deg | 0ms |
| Post | Standard | 420ms | (.34,1.42,.52,1) | 1.04 | .955 | 2.2deg | 26ms |
| Post | Extra | 560ms | (.34,2.0,.5,1) | 1.14 | .86 | 5deg | 34ms |
| Riso | Calm | 300ms | = e-out | 1.0 | 1 | 0deg | 0ms |
| Riso | Standard | 460ms | (.34,1.8,.5,1) | 1.09 | .90 | 3.6deg | 26ms |
| Riso | Extra | 560ms | (.34,2.0,.5,1) | 1.14 | .86 | 5deg | 34ms |
| Tide | Calm | 300ms | = e-out | 1.0 | 1 | 0deg | 0ms |
| Tide | Standard | 720ms | (.25,.7,.25,1) | 1.0 | .985 | 0deg | 44ms |
| Tide | Extra | 560ms | (.34,2.0,.5,1) | 1.14 | .86 | 5deg | 34ms |
| Candy | any | 520ms | (.34,1.5,.5,1) | 1.07 | .925 | 2.6deg | 30ms |

Two consequences worth naming: Tide Extra is *faster* than Tide Standard on `--t-big` (560 vs
720) and gains a spring Tide was designed not to have; Tide Calm drops `--t-big` from 720 to 300
while `--t-move` stays 420. mailo, which ships Post only, applies the levels on `:root` with the
same values and has no look cascade to fight.

### 3.4 Named durations, delays and derived values

The prototypes write these as literal milliseconds, so they do not change with look or level
(except under Reduced, 3.2). Names are the plan's ("Token model").

| Name | Value | Kind | Used by | Source |
| --- | --- | --- | --- | --- |
| `--t-big-heavy` | `calc(var(--t-big) * 1.15)`; Post 483ms, Riso 529ms, Tide 828ms, Candy 598ms, Calm 345ms, Extra 644ms | derived | unread row exits | S:329, C:1089 |
| `--t-spark` | 520ms | CSS | star sparks | S:307, C:420 |
| `--t-curl` | 560ms | CSS | snooze exit | S:328, C:715 |
| `--t-send` | 620ms | CSS | compose-send; also Tide `dissolve` 620ms (C:733) | S:376, S:572, C:509 |
| `--t-float` | 900ms | CSS | zZ floater; also `dest` period 900ms (S:447) | S:335 |
| `--t-hc-out` | 120ms | CSS | hover card out | S:402 |
| `--t-scene` | 380ms | CSS | Space layer cross-fade | S:86 |
| `--t-shake` | 420ms | CSS | `shake-x` (S). C's `shake` is 560ms (C:546) | S:590 |
| `--d-fly` | 350ms | CSS delay | strip preview label | S:445 |
| (park) | 420ms | CSS | `park` | S:570 |
| (nudge) | 520ms | CSS, C only | outbox retry | C:545 |
| (C shake) | 560ms | CSS, C only | outbox needs-sign-in | C:546 |
| (sail) | 1150ms | CSS, C only, orphaned | boat | C:535 |
| (boat-return) | 900ms | CSS, C only, orphaned | boat | C:536 |
| (spin) | 1.1s linear | CSS, C only | busy halo | C:291 |
| (send ring) | 5s linear | JS-set transition; quire's `--t-send-ring`, a hold (Reduced keeps 5 s: it is the undo window, not motion; mailo gaps 3) | undo-send countdown ring | S:2339 |
| `--t-flash` | 1200ms linear | CSS (quire's `chip-flash` keyframe, wave 1 amendment, proposed) | mentioned person chip's ring (04-COMPONENTS §10) | S:2119 |
| `--e-shake` | `cubic-bezier(.36,.07,.19,.97)` | easing | `shake-x`, C `shake` | S:590, C:546 |
| `--e-linear` | `linear` | easing | spin, send ring | C:291, S:2339 |
| HoverOpen | 450ms | Rust-only | hover intent | S:1704 |
| HoverClose | 150ms | Rust-only | hover intent | S:1704 |
| HoverWarm | 400ms | Rust-only | hover intent | S:1704 |
| ToastHold | 5200ms | Rust-only | undo toast | S:1552, C:1920 |
| HealStep | 18ms per row | Rust-only (and CSS delay) | heal ripple | S:330 |
| SentHold | 1600ms | Rust-only | send pill after "Sent" | S:2342 |
| `FRAME_SLACK` | 34ms | Rust-only | added to every `settle()` | plan, `time.rs` |
| `--t-fill` | 800ms | Rust-driven (no keyframe) | a battery ring's fill and its counting figure (4.12) | design/23 section 1.1 |
| `FRAME_TICK` | 16ms | Rust-only | the sampling rate of a motion driven from Rust (`use_level_run`), not a design duration | `time.rs` |
| `--t-sweep` | 700ms (Calm 500, Extra 900; Reduced: no sweep, the primitive shows the target at once) | CSS and Rust | an arc or bar sweeping in on Appear, and the count in step with it (`ds::detail::Sweep`, `CountUp`) | design/26 §3.4 |
| `--t-count-step` | 33ms, every level | Rust repaint floor (a hold: Reduced keeps it) | a counting number repaints its text at most this often | design/26 §3.4 |
| `--t-pending-step` | 300ms (Calm 360), linear | CSS and Rust | one step of a bounded pending loop (`use_pending`); a four-layer cycle is 1200ms | design/26 §3.4 |
| PendingGrace | 400ms, every level | Rust-only | a pending loop shows only if the operation is still running after this | design/26 §3.4, R4 |
| PendingCap | 10s, every level | Rust-only | after this a pending loop holds its still frame; the most a `PendingToken`'s deadline can be | design/26 §3.4, R4 |
| SettleHold | 900ms, every level | Rust-only | how long a success check stays drawn | design/26 §3.4, R14 |
| `--t-idle-dim` | 2000ms (proposed; Reduced: 60ms by this table, but the primitive never plays it — it snaps straight to the level instead, the way `--t-sweep` does) | Rust-driven (no keyframe) | the pre-screen-off idle overlay's fade in towards its own level; waking is never timed, it snaps (`ds::detail::use_idle_dim`, `IdleDim`) | design/22 §3.24 `idle.dim_s`/`idle.dim_level_pct`; Q447 |

`--tilt` has no Reduced/Calm meaning beyond 0deg; the drag ghost simply does not tilt.

## 4. Keyframe catalogue

Moved to `archive/05-MOTION-keyframes.md` (history). The surviving motion is `30-CATALOGUE.md` section 1.3: Presence, Roster, Spring, cross-fade, collapse, the spinner and the shake; keyframes that 30 does not name are dead code, removed in the clean-up phase.

## 5. Assignments

Which element plays which keyframe. "exit" = `cubic-bezier(.55,0,.75,.2)` (`--e-exit`). Fill and
iteration are as written; blank iteration = 1. "Stagger" names the custom property that
multiplies the delay token. Rows marked **C** exist only in C; rows marked *orphaned* are
defined but never triggered by the prototype's script.

| # | Element / state | Keyframe | Duration | Easing | Fill | Iteration | Stagger | Source |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | `.slide.in-r` (Space switch, forward) | `slide-r` | `--t-big` | `--e-spring` | none | | | S:102 |
| 2 | `.slide.in-l` (Space switch, back) | `slide-l` | `--t-big` | `--e-spring` | none | | | S:103 |
| 3 | `.list.entering .row` (list first shown) | `rise` | `--t-move` | `--e-out` | backwards | | `--i` x `--stagger` | S:294 (S:293 sets `.row{animation:none}`, overriding S:181) |
| 4 | `.reader-body > *` | `rise` | `--t-move` | `--e-out` | backwards | | `--i` x `--stagger` (no child sets `--i`, so 0) | S:214 |
| 5 | `.scrim` (S) | `fade` | `--t-move` | `--e-out` | none | | | S:222 |
| 6 | `.peek` | `peek-in` | `--t-big` | `--e-spring` | none | | | S:225 |
| 7 | `.cmdk-wrap` (S backdrop) | `fade` | `--t-quick` | `--e-out` | none | | | S:231 |
| 8 | `.cmdk` (S) | `peek-in` | `--t-big` | `--e-spring` | none | | | S:234 |
| 9 | `.star.pop .ic` | `star-pop` | `--t-big` | `--e-spring` | none | | | S:304 |
| 10 | `.sparks.go i` (starring only) | `spark` | 520ms | `--e-out` | forwards | | per-spark `--a` angle | S:307 |
| 11 | `.row:hover .strip button` | `pop-in` | `--t-move` | `--e-spring` | forwards | | `--j` x `--stagger` (C uses `--i`, C:439) | S:319 |
| 12 | `.row.going[data-op="archive"]` | `fold` | `--t-big` | exit | forwards | | | S:327 |
| 13 | `.row.going[data-op="snooze"]` | `curl` | 560ms | exit | forwards | | | S:328 |
| 14 | `.row[data-read="unread"].going` | (duration only) | `--t-big-heavy` | | | | | S:329 |
| 15 | `.row.healing` | `heal` | `--t-move` | `--e-spring` | backwards | | `--d` x 18ms (HealStep) | S:330 |
| 16 | `.floater` (the "zZ") | `floatup` | 900ms | `--e-out` | forwards | | | S:335 |
| 17 | `.item.gulp` | `gulp` | `--t-big` | `--e-spring` | none | | | S:339 |
| 18 | `.count.bump`, `.pin .n.bump` | `bump` | `--t-move` | `--e-spring` | none | | | S:341 |
| 19 | `.item[data-place][aria-current="true"]::before` (seal) | `seal-pop` | `--t-big` | `--e-spring` | forwards | | | S:346 |
| 20 | `.today-item.entering` | `tab-in` | `--t-big` | `--e-spring` | none | | | S:350 |
| 21 | `.today-item.leaving` | `tab-out` | `--t-move` | exit | forwards | | | S:351 |
| 22 | `.composer` (legacy, *orphaned* in S) | `compose-rise` | `--t-big` | `--e-spring` | none | | | S:375 |
| 23 | `.composer.sending` (legacy, *orphaned* in S) | `compose-send` | 620ms | exit | forwards | | | S:376 |
| 24 | `.hc` | `hc-in` | `--t-move` | `--e-spring` | none | | | S:401 |
| 25 | `.hc.out` | `hc-out` | 120ms | `--e-out` | forwards | | | S:402 |
| 26 | `.linkpill` | `hc-in` | `--t-quick` | `--e-out` | none | | | S:433 |
| 27 | `.item.dest` | `dest` | 900ms | `--e-out` | none | infinite alternate | | S:447 |
| 28 | `.win.no-side.side-peek .side` | `slide-r` | `--t-move` | `--e-spring` | none | | | S:454 |
| 29 | `.b-quote .inner` (unfolded quote) | `rise` | `--t-move` | `--e-out` | none | | | S:486 |
| 30 | `.cpage` (composer page) | `page-in` | `--t-big` | `--e-spring` | none | | | S:568 |
| 31 | `.cpage.parking` (origin `0% 60%`) | `park` | 420ms | exit | forwards | | | S:570 |
| 32 | `.cpage.sending` | `compose-send` | 620ms | exit | forwards | | | S:572 |
| 33 | `.prop-row.shake` | `shake-x` | 420ms | `--e-shake` | none | | | S:590 |
| 34 | `.pchip` | `chip-in` | `--t-move` | `--e-spring` | none | | | S:593 |
| 35 | `.t-replyq .rq-body` (legacy `.t-*`) | `rise` | `--t-move` | `--e-out` | none | | | S:657 |
| 36 | `.fmenu` | `menu-pop` | `--t-move` | `--e-spring` | none | | | S:666 |
| 37 | `.bubble` | `menu-pop` | `--t-quick` | `--e-spring` | none | | | S:684 |
| 38 | `.c-warn` | `rise` | `--t-move` | `--e-out` | none | | | S:696 |
| 39 | `.plain` | `rise` | `--t-move` | `--e-out` | none | | | S:702 |
| 40 | `.inline-reply` | `page-in` | `--t-big` | `--e-spring` | none | | | S:720 |
| 41 | `.cmdk .fmenu` (menu embedded in the command menu) | none (`animation:none`) | | | | | | S:796 |
| 42 | restored rows after undo (inline style) | `rise` | `--t-big` | `--e-spring` | backwards | | | S:1560 |
| 43 | **C** `.acct[data-sync="idle"] .halo` | `breathe` | `--t-ambient` | `ease-in-out` | none | infinite | | C:288 |
| 44 | **C** `.acct[data-sync="busy"] .halo` | `spin` | 1.1s | `linear` | none | infinite | | C:291 |
| 45 | **C** `.view[aria-current="true"]::before` | `seal-pop` | `--t-big` | `--e-spring` | forwards | | | C:318 |
| 46 | **C** `.view.is-gulping` | `gulp` | `--t-big` | `--e-spring` | none | | | C:324 |
| 47 | **C** `.row.is-entering` (arrival, undo) | `row-in` | `--t-big` | `--e-spring` | backwards | | | C:373 |
| 48 | **C** `.row.is-parting` (*orphaned*) | `part` | `--t-move` | `--e-out` | none | | | C:375 |
| 49 | **C** `.chip.is-landing` | `chip-land` | `--t-big` | `--e-spring` | none | | | C:402 |
| 50 | **C** `.frame`, `.img-ph` (opaque body) | `rise` | `--t-move` | `--e-out` | none | | | C:472, C:483 |
| 51 | **C** `.stagger > *` (*orphaned*) | `rise` | `--t-move` | `--e-out` | backwards | | `--i` x `--stagger` | C:496 |
| 52 | **C** `.composer` (floating composer) | `compose-rise` | `--t-big` | `--e-spring` | none | | | C:506 |
| 53 | **C** `.composer.is-sending` | `compose-send` | 620ms | `--e-exit` | forwards | | | C:509 |
| 54 | **C** `.boat.sail` / `.boat.return` (*orphaned*) | `sail` / `boat-return` | 1150ms / 900ms | `--e-out` | forwards | | | C:535-536 |
| 55 | **C** `.outbox.nudge` | `nudge` | 520ms | `--e-out` | none | | | C:545 |
| 56 | **C** `.outbox.shake` | `shake` | 560ms | `--e-shake` | none | | | C:546 |
| 57 | **C** `.row.is-going[data-op="archive"]` | `fold` | `--t-big` | `--e-exit` | forwards | | | C:713 |
| 58 | **C** `.row.is-going[data-op="trash"]` | `crumple` | `--t-big` | `--e-exit` | forwards | | | C:714 |
| 59 | **C** `.row.is-going[data-op="snooze"]` | `curl` | 560ms | `--e-exit` | forwards | | | C:715 |
| 60 | **C** `.row[data-read="unread"].is-going` | (duration only) | `--t-big-heavy` | | | | | C:1089 |
| 61 | **C** `.floater` (*orphaned* in C) | `floatup` | 900ms | `--e-out` | forwards | | | C:747 |
| 62 | **C** `.menu` (origin `88% 0`) | `menu-in` | `--t-move` | `--e-spring` | none | | | C:1007 |
| 63 | **C** `.shell[data-peek="center"] .reader` | `peek-in` (C text) | `--t-big` | `--e-spring` | none | | | C:1047 |
| 64 | **C** `.shell[data-peek="full"] .reader` | `peek-in` (C text) | `--t-move` | `--e-out` | none | | | C:1050 |
| 65 | **C** `.scrim` | `fade-in` | `--t-move` | `--e-out` | none | | | C:1055 |
| 66 | **C** `.cmdk-wrap` | `fade-in` | `--t-quick` | `--e-out` | none | | | C:1060 |
| 67 | **C** `.cmdk` | `cmdk-in` | `--t-big` | `--e-spring` | none | | | C:1064 |

Look-scoped exits (C:731-743) are in section 8. The C catalogue cards (C:2268-2332) replay
several of these inline; they are demonstrations, not assignments.

**Canonical choice where S and C differ** (S wins): the command menu enters with `peek-in` at
`--t-big`/`--e-spring` (row 8), not `cmdk-in`; the scrim is `fade` to opacity 1 over
`--scrim` (row 5), not ink at .16 via `fade-in`; floating menus use `menu-pop` (row 36) with the
placement rules of `06-INTERACTIONS.md#4-menu-placement`, and `menu-in` survives only if the
port keeps C's trigger-anchored view menus (section 12).

## 6. Transitions

Every `transition` declaration. `quick` = `--t-quick`, `tap` = `--t-tap`, `move` = `--t-move`,
`big` = `--t-big`; easing `out` = `--e-out`, `spring` = `--e-spring`.

| Element | Properties | Duration | Easing | Source |
| --- | --- | --- | --- | --- |
| `.win` (sidebar hide/show) | `grid-template-columns`, `padding` | move | out | S:83 |
| `.layer` (Space colour cross-fade) | `opacity` | 380ms | out | S:86 |
| `.cmd` (command pill) | `background-color` / `transform` | quick / tap | out | S:96 |
| `.pin` (account tile) | `background-color` / `transform` | quick / tap | out | S:111 |
| `.pin .av` | `transform` | quick | spring | S:357 |
| `.item` (sidebar item) | `background-color`, `color` / `transform` | quick / tap | out | S:130 |
| `.item .x` | `opacity` | quick | out | S:141 |
| `.sp` (Space dot) | `transform` / `border-color` | quick | spring / out | S:148 |
| `.mini` | `transform` / `background-color`, `box-shadow` | tap / quick | out | S:169 (C:353 adds `color` quick) |
| `.row` | `transform`, `box-shadow`, `background-color`, `border-color` | quick | out | S:179 (C:366 adds `opacity`) |
| `.tool` | `background-color`, `color` | quick | out | S:205 (C:997 adds `transform` tap) |
| `.seg button` | `background-color`, `color` | quick | out | S:267 (C:233 adds `transform` tap) |
| `.dot` (unread mark) | `transform` / `opacity` | quick | spring / out | S:295, C:380 |
| `.star` | `opacity` | quick | out | S:300, C:408 |
| `.star .ic` | `stroke`, `fill` | quick | out | S:302, C:412 |
| `.strip` | `opacity` | quick | out | S:314, C:428 |
| `.strip button` | `background-color`, `color` | quick | out | S:318, C:435 |
| `.strip .fly` | `opacity` / `transform` | quick | out / spring | S:444; on hover `transition-delay: var(--fly-delay, 350ms)`, 0ms under `.win.warm` (S:445-446) |
| `.toast` | `transform` | big | spring | S:363 (C:563 adds `opacity` quick out) |
| `.toast .tab` | `transform` / `background-color` | move / quick | spring / out | S:368, C:570; set to `none` while dragging (S:1565) |
| `.btn` | `transform` / `box-shadow` | tap / quick | out | S:388 (C:527 adds `filter` quick) |
| `.composer input, textarea` (legacy S; live C) | `border-color`, `box-shadow` | quick | out | S:383, C:517 |
| `.b-quote .fold` | `background-color`, `color` | quick | out | S:484 |
| `.att` | `transform`, `box-shadow` | quick | out | S:528 |
| `.c-top .state .pip` | `background-color` | quick | out | S:577 |
| `.prop-row` | `background-color` | quick | out | S:585 |
| `.gutter` (legacy) | `opacity` | quick | out | S:614 |
| `.content:empty::before` (legacy) | `opacity` | quick | out | S:621 |
| `.sendpill` | `transform` | big | spring | S:708 |
| `.sendpill .run` (ring) | `stroke-dashoffset` 0 -> 57 | 5s | linear | S:2339 (set from script) |
| `.ograb` | `opacity`, `background-color` | quick | out | S:748 |
| `.inp` | `border-color`, `box-shadow` | quick | out | S:782 |
| **C** `body` | `background-color`, `color` | move | out | C:184 |
| **C** `.tabs button` / `::after` underline | `color` / `transform` scaleX 0 -> 1 | quick / move | out / spring | C:247, C:255 |
| **C** `.view` | `background-color`, `color` / `transform` | quick / tap | out | C:304 |
| **C** `.consent button` | `transform` / `background-color` | tap / quick | out | C:468 |
| **C** `.outbox` | `transform` | big | spring | C:542 |
| **C** `.outbox .run` (ring) | `stroke-dashoffset` | per call (900, 4000, 0 ms) | linear | C:2147-2153 |
| **C** `.reader` below 1000px (slide-over) | `transform` | big | spring | C:754 |
| **C** `.specimen .ic` | `transform` / `color` | quick | spring / out | C:981 |
| **C** `.menu button` | `background-color`, `color` | quick | out | C:1017 |

State changes with no transition (they snap): `.presets button:hover` scale 1.1 (S:272),
`.handle.on` scale 1.15 (S:257), `.view.is-drop-target` scale 1.045 rides the `.view` transform
transition (C:320-323, C:304).

## 7. JS-driven timing as Rust timers

### 7.1 The rule

Blitz dispatches no `animationend` or `transitionend` (plan, "Findings: Blitz", #863). Every
state that the prototypes advanced on `animationend` or a `setTimeout` is advanced by a Rust
timer whose length comes from the same token table the stylesheet is generated from, so CSS and
Rust cannot drift (plan: "`onanimationend`-driven state … Rust timers from a Rust timing table;
generate mailo's tokens.css from that table").

```
settle(anim, level, index) = duration(anim, level) + index * stagger(level) + FRAME_SLACK
FRAME_SLACK = 34 ms            // two frames at 60 Hz
index is capped at 12          // StaggerIndex; plan: "stagger capped at 12"
```

Timers start in event handlers, not in render (plan, Blitz risk table: "timers -> start in
handlers, host bridges waker to calloop ping"). A test (`animation-declaration-matches-Rust-
recipe`, `settle table`) proves each CSS declaration equals the Rust recipe.

Worked values, Post, Standard: `settle(Fold)` = 420 + 34 = 454 ms; `settle(FoldHeavy)` = 483 +
34 = 517 ms; `settle(Curl)` = 560 + 34 = 594 ms; `settle(Heal, k)` = 250 + 18k + 34 ms (heal uses
HealStep, not `--stagger`); `settle(Rise, i)` = 250 + 26i + 34, at most 250 + 312 + 34 = 596 ms;
`settle(HcOut)` = 154 ms; `settle(TabOut)` = 284 ms; `settle(Gulp)` = 454 ms; `settle(Floatup)` =
934 ms; `settle(ComposeSend)` = 654 ms; `settle(Park)` = 454 ms. Calm: `settle(Fold)` = 334 ms.
Extra: 594 ms. Reduced: every `settle` = 60 + 34 = 94 ms (stagger 0).

### 7.2 Timer table

"Prototype" is what the script did; "quire timer" is what replaces it. Names in *italics* are
proposed here; the rest are the plan's.

| What | Prototype | quire timer | Source |
| --- | --- | --- | --- |
| Hover card opens after rest | `setTimeout(show, 450)`; 0 when warm | HoverOpen 450 ms (0 when warm) | S:1704, S:1795 |
| Hover card closes after absence | `setTimeout(closeCard, 150)` | HoverClose 150 ms | S:1783, S:1802 |
| Hover stays warm after a close | `warmUntil = now + 400` | HoverWarm 400 ms | S:1711 |
| `.win.warm` removed (fly label instant while warm) | `setTimeout(…, WARM_MS + 20)` = 420 ms | HoverWarm expiry (no +20; the warm flag is Rust state) | S:1712 |
| Hover card node removed after `.out` | `setTimeout(remove, 130)` | `settle(HcOut)` = 154 ms | S:1713 |
| Undo toast hides | `setTimeout(hide, 5200)`, reset on every new undo | ToastHold 5200 ms | S:1552, C:1920 |
| Row exit completes | `animationend` or fallback `setTimeout(finish, 900)` | `settle()` of Fold, FoldHeavy, Curl or Crumple | S:1545, C:1892-1893 |
| Heal ripple class removed | `setTimeout(…, 600)` (S), 700 (C, and C has no heal CSS) | `settle(Heal, k)` per row | S:1544, C:1859 |
| A batch of rows leaves (a Clear in the notification center) | none (S and C remove one row at a time) | the batch's longest `settle(exit, i)`, `i` its place in the batch capped at 12; then the rows drop together and the rows below heal by the dropped rows' summed measured heights (design/04 section 54, sill Q510) | quire |
| Gulp class removed | `setTimeout(…, 460)` | `settle(Gulp)` = 454 ms | S:1520, C:1851 |
| Today entry removed after close | `animationend` or `setTimeout(done, 400)` | `settle(TabOut)` = 284 ms | S:1485 |
| List re-render after read toggle | `setTimeout(renderList, 260)` | *ReadReflow* 260 ms (the dot's transition is 170 ms) | S:1533 |
| zZ floater removed | `setTimeout(remove, 950)` | `settle(Floatup)` = 934 ms | S:1536 |
| Sparks class removed (C) | `setTimeout(…, 560)` | `settle(Spark)` = 554 ms; S never removes it and restarts by reflow | C:1909 |
| Undo/arrival `row-in` class removed (C) | 520 ms / 540 ms | `settle(RowIn)` (Post 454, Candy 554) | C:1932, C:2112 |
| Space colour cross-fade | CSS transition 380 ms, no timer | none needed (a transition, not a state) | S:86, S:1178-1186 |
| Composer page send: reader re-renders | `setTimeout(…, 560)` while `compose-send` runs 620 ms | `settle(ComposeSend)` = 654 ms (S cut the animation 60 ms short) | S:2333 |
| Undo-send pill appears | next animation frame | next frame (ds `request_redraw`) | S:2339 |
| Undo-send countdown | `setInterval(…, 1000)` x 5; ring 5 s linear | *SendCountdown* 5 s with 1 s ticks | S:2340-2342 |
| "Sent" pill hides | `setTimeout(hide, 1600)` | SentHold 1600 ms | S:2342 |
| Composer park: reader re-renders | `setTimeout(…, 380)` (0 when silent) while `park` runs 420 ms | `settle(Park)` = 454 ms (S cut it 40 ms short) | S:1993 |
| Draft autosave | debounce `setTimeout(save, 700)` | *AutosaveDebounce* 700 ms | S:1979 |
| Focus To field after the page opens | `setTimeout(focus, 60)` | *FocusAfterMount* 60 ms, or focus on mount if the host allows | S:1967 |
| Mentioned person chip flash | `setTimeout(remove .flash, 1200)` | `Anim::ChipFlash` over `--t-flash` 1200 ms (a pulse; *FlashHold* stays for a consumer that times it) | S:2119 |
| Selection bubble placement | `requestAnimationFrame(showBubble)` on `selectionchange` | next frame | S:2160 |
| C sync busy | `setTimeout(…, 1200)` | not specified (demo) | C:2093 |
| C status line clears | `setTimeout(…, 3400)`; `.status` has no CSS | not specified (A8 #3) | C:1644 |
| C floating composer send | `setTimeout(…, 560)` while `compose-send` runs 620 ms | `settle(ComposeSend)` | C:2157 |
| C outbox accent class removed | `setTimeout(…, 700)` | `settle(Nudge)` = 554 / `settle(Shake)` = 594 ms | C:2144 |
| C outbox sent | ring 900 ms; hide at 1800 ms | not specified (C-only outbox) | C:2160-2163 |
| C outbox retry | ring 4000 ms; nudge again at 2000 ms; "On its way" at 4200 ms; hide 1500 ms later | not specified | C:2164-2172 |
| C outbox fixed | ring 900 ms; hide at 1700 ms | not specified | C:2181-2186 |
| C catalogue replay | 50 ms delay; heal demo 320 ms | not ported (demo) | C:2268-2332 |

Two prototype timers were shorter than their animations (send 560 < 620, park 380 < 420) and one
was a no-op (C heal). `settle()` fixes all three by construction.

## 8. Exits per operation

"Acting gets an exit" (principle 4). The exit plays on the leaving row; then the rows below
heal (S:1543-1544), the destination gulps, the undo toast appears. S has no trash op (A8 #9).

| Operation | Post (S; C Post and Candy) | Riso (C) | Tide (C) | Extra effect | Source |
| --- | --- | --- | --- | --- | --- |
| Archive | `fold` `--t-big` exit forwards | `yeet` `--t-big` exit | `dissolve` 620ms exit | destination "Archive" gulps | S:327, C:713, C:738, C:733 |
| Trash, Delete forever (C) | `crumple` `--t-big` exit | `crumple` | `dissolve` 620ms | "Trash" gulps (trash only) | C:714, C:1871, C:1888 |
| Snooze | `curl` 560ms exit, plus the "zZ" floater (S only) | `curl` | `dissolve` 620ms | "Snoozed" gulps | S:328, S:1535-1536, C:715 |
| Read / Unread | none: the dot scales out with its transition; list re-renders at 260 ms | none | none | toast only | S:1532-1533, S:295-296 |
| Restore, Wake, Remove label, Unstar in Starred (C) | treated as "archive": `fold` | `yeet` | `dissolve` | "Inbox" gulps (restore, wake) | C:1874-1878, C:1888 |
| Star / Unstar | not an exit: `star-pop`; sparks only when starring | same | same | "Starred" gulps (S) | S:1521-1528 |
| Label (toggle) | not an exit (opening then acting in place): `gulp` on the label | same | same | toast (C) | S:1509, C:1993-2000 |

Duration override: an unread row exits at `--t-big-heavy` (t-big x 1.15). Because the override
is written as a bare `animation-duration` rule of equal specificity after the snooze rule (S:329
after S:328; C:1089 after C:715), it also replaces snooze's 560 ms: an unread snooze curls in
483 ms (Post), *faster* than a read snooze. Tide's look-scoped rule is more specific, so Tide
exits stay 620 ms read or unread. See section 12.

The zZ floater: `div.floater` with text "zZ", `position:fixed`, placed at `left = row.right -
70px`, `top = row.top + 8px`, removed after 950 ms (S:1535-1536, S:334-335). Display face 800,
14px (S), 15px (C:746-747), colour `--accent`.

Blitz: `dissolve` needs `filter:blur()` and is excluded (section 11); the Tide exit in the port is
not specified (section 12). `curl` must drop its `filter` (section 9).

## 9. Rules for the Blitz port

Each rule names its source in the plan's Blitz findings or risk table.

1. **No `animationend`, so settle timers.** Every class that the prototype removed on
   `animationend` or a timeout is Rust state that ends on `settle()` (section 7). Exits run with
   `animation-fill-mode: forwards` so the node holds its last frame until the `Roster` drops it.
2. **Restart by keyframe alias, not by reflow.** The prototype restarted an animation with
   `classList.remove(x); void el.offsetWidth; classList.add(x)` (S:1520, S:1525, S:1526, S:2319;
   C:1850, C:1905-1908, C:2143). Blitz has no forced reflow. The stylesheet emits every keyframe
   twice, `X` and `X--b`, identical; `use_pulse(anim)` toggles `data-pulse="a|b"` and the rule
   for each value names the other alias, so the animation name changes and Stylo restarts it
   (plan, `use_pulse`: "restarts via keyframe alias swap"). Fallback if spike S5 fails: one
   frame with the class removed.
3. **Stagger index capped at 12.** `StaggerIndex` saturates at 12 (plan, `use_roster`). This
   replaces C's cap of 8 (C:1709, C:1721; stated as "capped at eight" at C:2295) and S's
   uncapped `--i` (S:1277). Reason: transforms on more than about 200 nodes lag in Blitz (#595).
4. **No `filter` in keyframes or transitions.** Plain `anyrender_vello` does not paint `filter`;
   vello_hybrid does, but the lint bans it regardless (plan, risk table: "filter -> banned by
   lint"). Consequences: `curl` loses `filter:saturate(.2)` (S:332, C:729) and keeps its
   transform and opacity; C's `.btn:hover` `filter:saturate(1.1)` (C:529) is dropped; the
   unpressed account tile's `filter:saturate(.55)` (S:551) becomes opacity only; `dissolve`
   (blur) is excluded.
5. **Transform performance.** Lift only on hover (one node at a time), never on every row;
   never transform an absolutely positioned element inside an inline box (#840); keep animated
   node counts under the stagger cap.
6. **`color-mix()` becomes a token.** `dest` animates to `color-mix(in oklab, var(--accent) 35%,
   transparent)` (S:448); the port uses the precomputed `--accent-ring` (plan, token model).
7. **Layout-property animations need the spike.** `tab-in`/`tab-out` animate `max-height` and
   `padding-block` (S:352-353); `.win` transitions `grid-template-columns` (S:83). Stylo
   interpolates them; whether Blitz relayouts per frame fast enough is unverified (spike items
   S3/S4). If not, Today entries animate transform and opacity only and the list reflows at
   `settle`.
8. **SVG stroke animation is unverified.** The send ring transitions `stroke-dashoffset` on an
   inline SVG circle (S:713, S:2339). Inline SVG is rasterised through usvg; CSS transitions on
   its attributes are not in the list of things known to work. Proposed: draw the ring as a ds
   custom paint (`blitz_dom::Widget`) driven by the SendCountdown timer.
9. **`var()` inside keyframes** (`--overshoot`, `--a`, `--dy`) is spike item S3; the fallback is
   generating one keyframe block per resolved value from the Rust table.
10. **Overlays render at the root.** `position:fixed` breaks inside transformed ancestors, so
    floaters, hover cards, menus and the drag ghost render through `OverlayHost` at the end of
    `.ds` (plan, components).
11. **Reduced is explicit.** Never rely on `@media (prefers-reduced-motion)`; the portal value is
    resolved in Rust and written as `data-motion="reduced"` (plan, "Must be rewritten").
12. **Fullscreen surfaces never animate** (plan, shell risks, cosmic-comp #2777).

## 10. Shell motion

Tokens for the desktop surfaces. Status: **settled (user)** = decided with the user in the
plan's "UX decisions settled"; **settled (plan)** = in the approved plan's shell design;
**proposed** = this document's recommendation; **not specified** = no source gives a value.
Behaviour (what triggers what) is in `06-INTERACTIONS.md#20-desktop-interactions-settled`.

| Surface motion | Value | Easing | Status | Source |
| --- | --- | --- | --- | --- |
| Dock magnification while the pointer moves | Plank parabola: `offset = min(abs(cursor - center), zoomIconSize); p = offset / zoomIconSize; zoom = 1 + (1 - p²)(zoomPercent * progress - 1)`; icon 48 px up to 96 px; influence ±1 magnified icon width; neighbours slide apart, baseline fixed | **none**: the icon under the cursor tracks every move with no easing | settled (user) | plan "UX decisions settled"; Appendix C-A |
| Dock magnification grow-in on enter (`progress` 0 -> 1) | "short" (observed); no number | not specified | not specified | Appendix C-A |
| Dock magnification shrink on leave (`progress` 1 -> 0) | ~200 ms | not specified (proposed `--e-out`) | settled (user) for the duration; proposed for the easing | plan; Appendix C-A |
| Dock bounce, informational request | bounces for 1 s | decelerating up, accelerating down (proposed: up `--e-out`, down `--e-exit`) | proposed | Appendix C-A (H) |
| Dock bounce, critical request / launch | repeats until the app is active / launched | same | proposed; conflicts with "nothing loops" (section 12) | Appendix C-A (H) |
| Dock bounce amplitude | about one icon height | | proposed | Appendix C-A (L) |
| Dock auto-hide (off by default) | 0.2 s delay after the pointer rests at the edge, ~0.5 s slide | not specified | settled (user) | plan; Appendix C-A (M) |
| Launcher panel open | `peek-in`, `--t-big`, `--e-spring` (S's command menu); backdrop none (the catcher surface is never animated) | `--e-spring` | proposed for the keyframe; settled (plan) for "catcher never animated" | S:234; plan sill "Launcher v1" |
| Launcher open latency budget | p95 < 100 ms from `launcher toggle` to first frame, over 20 toggles | n/a (a latency, not a duration) | settled (plan) | plan sill; sill's `dev/accept/launcher.sh` |
| Undo toast (in apps) | spring in from `translateX(-50%) translateY(160%)` over `--t-big`; hold ToastHold 5200 ms | `--e-spring` | settled (prototype) | S:360-364, S:1552 |
| Notification banner | spring in (as the toast); hold ~5 s; top-right; swipe right dismisses | `--e-spring` | proposed; entry direction decided: from the right, `notifications.banner_entry_direction` selects from below (section 12 item 7) | Appendix C-D (M) |
| Workspace switch: frame tint cross-fade | 380 ms (`--t-scene`) opacity cross-fade of two layers | `--e-out` | settled (user) | plan; S:86, S:1178-1186 |
| Wallpaper light/dark cross-fade | `--t-big` | not specified | settled (plan) | plan sill "Wallpaper" |
| Bar menus (tray, clock, volume, network, battery, workspace) | `menu-pop`, `--t-move`, `--e-spring` on first open | `--e-spring` | proposed; conflicts with macOS "no open animation" (section 12) | S:666; plan sill "every menu … with ds Menu"; Appendix C-D |
| Switching between open bar menus by hover | instant, no animation | | proposed (from macOS L) | Appendix C-D |
| Bar menu close | fade | not specified (proposed `--t-quick` `--e-out`) | proposed | Appendix C-D (L) |
| Dock context menu, dock hover label | not specified | | not specified | Appendix C-A: hover label delay/size UNKNOWN |
| Minimize to dock | genie/scale ~0.5 s on macOS | | not specified (compositor-owned) | Appendix C-A |

## 11. Excluded keyframes

Moved to `archive/05-MOTION-keyframes.md` (history).

## 12. Open decisions

1. **Motion levels vs looks.** The prototype cascade makes Calm/Extra inert on Candy and lets
   them override Riso/Tide's own curves (3.3). Proposed: levels apply uniformly on top of any
   look (mailo's behaviour), and Tide under Extra keeps `--t-big` at least its Standard value.
2. **Reduced details.** Easing under Reduced, `--lift` under Reduced, and whether HealStep and
   the hover-intent delays shrink under Reduced are not specified. Proposed: `--e-spring` =
   `--e-out`, `--lift` 0px, HealStep 0, hover-intent delays unchanged (they measure intent, not
   motion).
3. **Unread snooze runs faster than read snooze** (section 8). Proposed: `--t-curl-heavy` =
   560 x 1.15 = 644 ms, so the unread rule means what principle 6 says. Implemented (FINDINGS "W2
   integration", heavy exits): the roster settles `CurlHeavy` and `CrumpleHeavy` for an unread
   row and the row stylesheet plays them; the 644 ms value stays proposed.
4. **Loops.** `dest` (S:447), `breathe`/`spin` (C:288-291) and the dock's critical bounce
   (section 10) contradict "nothing loops". Proposed: keep `dest` (it lasts only while the
   pointer is on the button), keep `spin` only while a sync runs (mailo already says so), drop
   `breathe` (an idle loop), keep the critical bounce (it is the macOS signal for "needs you").
5. **Tide's exit in the port.** `dissolve` needs `filter:blur()`. Proposed: a filter-free
   variant (opacity, `translateY(-8px) scale(.99)`), if Tide ships at all (see
   `07-LOOKS.md`).
6. **Menu open animation on the desktop.** ds `Menu` uses `menu-pop`; macOS opens menus with no
   animation (Appendix C-D). Proposed: bar menus pop on first open, switch instantly while one is
   open, fade on close.
7. **Notification banner entry direction.** The toast rises from below (translateY 160%);
   banners sit top-right. **Decided (2026-09-26):** banners enter from the right edge
   (translateX) by default, with the same spring and hold 5200 ms (the hold is the caller's);
   `notifications.banner_entry_direction` (`BannerEntry::{FromRight, FromBelow}`, design/22)
   selects rising from below instead, and the banner leaves back the way it came. quire's `BannerStack { entry }` writes `data-entry`, which
   sets the `--banner-dx`/`--banner-dy` that the one `banner-in`/`banner-out` pair reads (4.9).
   Swipe right dismisses, and a swiped banner leaves to the right whatever the entry edge.
8. **Hover-card spring.** `hc-in` springs 450 ms after rest, which is not "the 200 ms after you
   touched something". Keep as is (it is contact of a kind) or use `--e-out`. Not decided.
9. **`menu-in` vs `menu-pop`.** Two menu entrances exist (C:1009, S:667). Proposed: `menu-pop`
   everywhere; drop `menu-in` from the canonical set.
10. **Heal step.** S uses 18 ms per row (S:330); C's catalogue says "a 15ms-per-row ripple"
    (C:2308) and C has no heal rule at all. Proposed: HealStep 18 ms (S wins).
11. **Seal and bump replay on every re-render.** S rebuilds the sidebar with `innerHTML`, so the
    seal pops on every render, not only when the place changes, and `.bump` is never removed.
    The port keys both to the change (seal: place changed; bump: count changed), via `use_pulse`.
12. **Spring durations exceed 200 ms.** `--t-big` springs (420 ms Post) run past the 200 ms the
    principle names; the principle is about *when* overshoot may start, and this reading should
    be confirmed.

## 13. Sources

- `~/mailo-design/mailo-spaces.html` (S): tokens S:7-46; motion CSS S:83-454, S:486, S:568-598,
  S:614-721, S:782, S:796, S:807-809; timers S:1485, S:1520-1571, S:1704-1843, S:1967-1993,
  S:2119, S:2160, S:2316-2344.
- `~/mailo-design/mailo-charm.html` (C): tokens and looks C:13-176, C:777-924; motion CSS
  C:184-584, C:653-774, C:997-1089; notes C:1114-1417; timers C:1639-2187; catalogue
  C:2268-2332.
- `~/mailo-design/mailo-charm.bak.html` (B): the Mochi look, B:748-800.
- `~/mailo/crates/mail-app/src/ui/style/tokens.css`: mailo's port of the Post motion tokens and
  levels (same values as C).
- `~/.claude/plans/vast-toasting-peach.md`: "Design: `<ds>` design-system repo" (Anim enum,
  settle, use_pulse, Roster, HoverIntent, placement, token model, risk table), "Findings:
  Blitz / Dioxus Native", "Design: `<shell>` repo" (dock, launcher, wallpaper), "UX decisions
  settled with the user", Appendix A5 (motion), A8 (bugs), Appendix C-A and C-D.

## 14. Driven motion (wave H1, 2026-09-27)

design/27-HIG-PARITY.md section 3.12 and [FLUID] (WWDC 2018 session 803): a motion a hand drives,
or can interrupt, is a spring integrated in Rust on the frame clock, not a keyframe of fixed
length. Keyframes stay for arrivals no hand touched (`rise`, `fold`, `curl`, `gulp`, `bump`,
`seal-pop`, `shake-x`, `pop-in`, a sheet's or a panel's first entrance on mount).

### 14.1 The spring

`ds::motion::Spring { damping: Ratio, response: Millis }`, SwiftUI's parameters: `damping` is
the damping ratio in thousandths (1000 critical; below it, it overshoots), `response` the period
it would swing at undamped, which reads as how quick it feels. It has no duration; it ends when
it rests (within 0.25 px of its target and slower than 4 px/s, measured through the caller's
`PxPerUnit`). Each leg (`Leg`) is solved in closed form (critically damped `(A + Bt)e^(-wt)`,
under-damped `e^(-zwt)(A cos + B sin)`), so the state at any instant is exact however frames fall.

Only `SpringSpec::for_touch(Touch)` makes a spring, so no caller writes a raw damping:

| Touch | Damping | Velocity handed on |
| --- | --- | --- |
| `Touch::Remote` (a service, a timer, another window) | 1.0 | none |
| `Touch::Contact` with no velocity (a click, a key) | 1.0 | none |
| `Touch::Contact` released at ≥ 50 px/s toward the target | 0.8 | the release velocity |
| `Touch::Contact` released away from the target | 1.0 | the release velocity |
| any, under Reduced | 1.0 | none |

### 14.2 Tokens (proposed; to be tuned beside macOS)

| Token | Response | For |
| --- | --- | --- |
| `--spring-quick` (`SpringResponse::Quick`) | 300 ms | a toggle knob, a selection thumb or ring, a slider release, a leaving sheet or panel |
| `--spring-move` (`SpringResponse::Move`) | 450 ms | a pane switch, a sheet or panel arriving, a swiped card or dragged tile returning |

They are Rust values (`SpringResponse::millis`), not stylesheet variables: no stylesheet reads a
spring.

### 14.3 The driver

`use_spring(target, SpringSpec, PxPerUnit) -> SpringFrame` follows a target: it stands there on
mount and, each time the target changes, starts a new leg from the exact position and velocity
the old leg has at that instant (retargeting; nothing restarts, nothing waits). A contact that
carries a velocity (`Contact::with_velocity`, from a drag's `onpointerup`) starts the leg at that
velocity instead. `use_spring_motion(at, PxPerUnit) -> SpringMotion` is the imperative form:
`go(target, spec)`, `track(at)` (1:1 under a hand, no frames of its own), `snap(at)`. Two dimensions (sill Q432, a widget springing into
its snap cell): `use_spring_point(target: Point, spec)` and `use_spring_point_motion(at)`, one
spring per axis, `go(target, touch, Release { x, y }, response)` handing each axis the hand's
velocity along it; `PointThrow::landing` picks the cell nearest the projection. The frame
task sleeps `FRAME_TICK` (16 ms) on `ds::time`, so a harness on `Clock::Virtual` drives it
exactly; it ends the moment the spring rests, and asks for nothing after (design/26 R3). A
component writes the position into a custom property on its own element (`--knob-x`, `--seg-dx`,
`--switcher-at`, `--pane-p`, `--present-p`, `--swipe-dx`, `--drag-dx`/`--drag-dy`, the slider's
`--f`), and its stylesheet draws from it with no transition.

### 14.4 Throws

A release projects to where the scroll view's deceleration would stop it:
`p + v·r/(1-r)` with `r = 0.998` per ms, about `p + 0.5 s × v` (`Throw::projected`), and lands
on the endpoint nearest the projection, not nearest the release (`Throw::landing`). The slider
throws its value (a release at ≥ 600 px/s, the swipe's fling speed) and springs its thumb there
at the hand's velocity. `VelocityMeter` measures a release from a drag's last two moves (nothing
when the pointer stopped more than 100 ms before letting go).

### 14.5 Reduced

Every `Anim` declares its form under Reduced (`Anim::reduced() -> ReducedForm`): `Same` for a
keyframe that moves nothing (a fade, a colour, a ring draining), `CrossFade(In)` (`fade`) for an
arrival, `CrossFade(Out)` (`menu-out`) for a departure, `Still` (`hold`) for an emphasis (a bump,
a shake, a pop). The pulse classes play the form under `.ds[data-motion=reduced]`, and a data
test (`every_moving_keyframe_has_a_still_reduced_form`) fails any moving keyframe without a still
form. Springs under Reduced are critically damped and take no thrown velocity; tracking stays
1:1. The sheet, the edge panel and the pane switcher only cross-fade under Reduced (their
transforms are dropped); the knob, the thumb and the ring still move, critically damped.

A component that writes a moving keyframe in its own stylesheet (not through a pulse class) is
not reached by the table yet; those keep 60 ms under Reduced (section 3.2) until each is given a
Reduced rule.

### 14.6 What converted in H1

| Part | Spring | Touch | Was |
| --- | --- | --- | --- |
| Toggle knob | `--knob-x` px, Quick | the click (1.0) | `transform` transition `--t-move --e-spring` |
| Segmented thumb (new: equal segments, one sliding thumb) | `--seg-dx` segments from the selected cell (`--seg-col`), Quick | the click (1.0) | the pressed fill jumped |
| App switcher ring | `--switcher-at` cells, Quick | remote (the shell's keys, 1.0) | transition `--t-quick --e-spring` |
| Slider | `--f`, Quick | the release (0.8 when thrown toward) | 1:1 only; the release dropped its speed |
| Notification card (and screenshot thumbnail) swipe return | `--swipe-dx` px, Move | the release, with its velocity | transition `--t-move --e-spring` |
| Pane switcher | `--pane-p` 0..1, Move | remote (1.0) | `slide-r`/`slide-l`, `pane-out-*`, restarted on reversal |
| Sheet hide, show again | `--present-p` 0..1, Quick out, Move in | remote (1.0) | `sheet-out`; a re-show restarted `peek-in` |
| Edge panel hide, show again | `--present-p` 0..1, Quick out, Move in | remote (1.0) | `panel-out`, `hold` on a taken-back hide |
| Dock tile drag return (`DragReturnFrame`, sill wires the drag) | `--drag-dx`/`--drag-dy` px, Move | the release, per axis | none (sill had no return) |

Not converted: the launcher's open and close (27 section 3.12 lists it last; it keeps its
keyframes), the dock's bounce and magnification (27 section 8 decision 4), sheet and panel drag
(neither takes a drag yet), workspace swipe (12 section 12.3.7, sill's). The swipe's dismiss
decision keeps its settings-backed thresholds (80 px, 600 px/s) rather than the projection.
