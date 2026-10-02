# 05 Motion: the keyframe catalogue and excluded keyframes (archived 2026-10-02)

> **History, read-only.** Moved from `design/05-MOTION.md` sections 4 and 11. Only the keyframes
> `30-CATALOGUE.md` section 1.3 names survive; the dead ones (`slide-r/l`, `page-in`, `shake`, `morph`,
> the `--b` duplicates if unused) are removed in the clean-up phase (30 section 3.4). Mac motion only.

## 4. Keyframe catalogue

### 4.1 How to read this catalogue

This is the canonical set `ds-motion/motion.css` is generated from (plan: "motion.css = canonical
mockup keyframes; spaces.html wins on name clashes; mascot/look-scoped keyframes out"). Every
`@keyframes` in S and C is here, quoted verbatim from its source line. When both files define a
name, S's text is quoted and C's difference is noted. Each entry names its `Anim` variant from
the plan's enum and says whether the prototype actually plays it ("live") or only defines it
("orphaned"). Mascot keyframes (`pip-*`, `zzz`, `heart-up`) and the look-scoped exits
(`dissolve`, `yeet`) are quoted in section 11 and are not part of the canonical set.

Count: S defines 27, C defines 32 (of which 7 are excluded). Shared: 12. Canonical set: 27 (S)
+ 13 (C-only) = 40 names. The plan's `Anim` enum has 39 variants; it has no variant for `part`
or `fade-in`, and adds `FoldHeavy` (a duration, not a keyframe).

### 4.2 Defined in both files (S text quoted)

#### 4.2.1 `rise`

Source S:218. `Anim::Rise`. Live in S and C. C:664 is the same motion written `0%{ transform:translateY(7px); opacity:0; } 100%{ transform:translateY(0); opacity:1; }`.

```css
@keyframes rise{ from{ transform:translateY(7px); opacity:0; } to{ transform:none; opacity:1; } }
```

#### 4.2.2 `star-pop`

Source S:308. `Anim::StarPop`. Live in S and C. C:665-669 identical values, split over lines.

```css
@keyframes star-pop{ 0%{ transform:scale(1) rotate(0); } 30%{ transform:scale(1.5) rotate(-14deg); } 55%{ transform:scale(.88) rotate(9deg); } 78%{ transform:scale(1.1) rotate(-3deg); } 100%{ transform:scale(1) rotate(0); } }
```

#### 4.2.3 `spark`

Source S:309. `Anim::Spark`. Live in S and C. C:670-673 identical. `--a` is set per spark `i` to `k*60deg`, k = 0..5 (S:1285, C:1690-1691).

```css
@keyframes spark{ 0%{ opacity:1; transform:translate(-50%,-50%) rotate(var(--a)) translateY(0) scale(1); } 100%{ opacity:0; transform:translate(-50%,-50%) rotate(var(--a)) translateY(-17px) scale(.3); } }
```

#### 4.2.4 `pop-in`

Source S:323. `Anim::PopIn`. Live in S and C. C:657 identical.

```css
@keyframes pop-in{ 0%{ transform:scale(.55); opacity:0; } 68%{ transform:scale(var(--overshoot)); opacity:1; } 100%{ transform:scale(1); opacity:1; } }
```

#### 4.2.5 `fold`

Source S:331. `Anim::Fold, FoldHeavy`. Live in S and C. C:716-720 identical. FoldHeavy is the same keyframes at `--t-big-heavy` (t-big x 1.15, S:329, C:1089).

```css
@keyframes fold{ 0%{ transform:scaleY(1) translateX(0); opacity:1; } 30%{ transform:scaleY(1) translateX(10px) rotate(.6deg); } 100%{ transform:scaleY(.02) translateX(-26px) rotate(-2deg); opacity:0; } }
```

#### 4.2.6 `curl`

Source S:332. `Anim::Curl`. Live in S and C. C:726-730 adds `filter:saturate(1)` at 0%. Both end on `filter:saturate(.2)`, which Blitz cannot paint on the default backend; see section 9.

```css
@keyframes curl{ 0%{ transform:translateY(0) scale(1); opacity:1; } 35%{ transform:translateY(2px) scale(.985); } 100%{ transform:translateY(22px) scale(.93); opacity:0; filter:saturate(.2); } }
```

#### 4.2.7 `floatup`

Source S:336. `Anim::Floatup`. Live in S (the snooze "zZ"). Orphaned in C: C:746-749 defines `.floater` and the keyframes but no C script creates a floater. C:748-749 identical values.

```css
@keyframes floatup{ 0%{ opacity:0; transform:translateY(0) scale(.6); } 20%{ opacity:1; } 100%{ opacity:0; transform:translateY(-38px) scale(1.25) rotate(8deg); } }
```

#### 4.2.8 `gulp`

Source S:340. `Anim::Gulp`. Live in S and C. C:656 identical.

```css
@keyframes gulp{ 0%{ transform:scale(1,1); } 34%{ transform:scale(1.07,.84); } 62%{ transform:scale(.97,1.07); } 100%{ transform:scale(1,1); } }
```

#### 4.2.9 `seal-pop`

Source S:347. `Anim::SealPop`. Live in S and C. C:655 identical.

```css
@keyframes seal-pop{ 0%{ transform:translateY(-50%) scale(0); } 60%{ transform:translateY(-50%) scale(1.5); } 100%{ transform:translateY(-50%) scale(1); } }
```

#### 4.2.10 `compose-rise`

Source S:392. `Anim::ComposeRise`. Orphaned in S (only the legacy `.composer`, S:373-375, which no S script creates). Live in C (C:506, the floating composer). C:675-679 identical values.

```css
@keyframes compose-rise{ 0%{ transform:translateY(26px) scale(.9); opacity:0; } 62%{ transform:translateY(-4px) scale(var(--overshoot)); opacity:1; } 100%{ transform:none; opacity:1; } }
```

#### 4.2.11 `compose-send`

Source S:393. `Anim::ComposeSend`. Live in S (`.cpage.sending`, S:572) and C (`.composer.is-sending`, C:509). C:680-684 identical values.

```css
@keyframes compose-send{ 0%{ transform:none; opacity:1; } 35%{ transform:translate(0,4px) scale(.97,.94); opacity:1; } 100%{ transform:translate(-40px,-120px) scale(.06) rotate(-8deg); opacity:0; } }
```

#### 4.2.12 `peek-in`

Source S:226. `Anim::PeekIn`. Live in S (peek, command menu) and C (centre/full peek); C's full peek plays it at `--t-move --e-out` as `Anim::PeekFullIn` (section 5 row 64, W2 integration). C:1052-1053 is gentler: `0%{ opacity:0; transform:scale(.97) translateY(10px); } 100%{ opacity:1; transform:scale(1) translateY(0); }`. S wins.

```css
@keyframes peek-in{ 0%{ opacity:0; transform:scale(.95) translateY(12px); } 100%{ opacity:1; transform:none; } }
```

### 4.3 Defined only in S

#### 4.3.1 `slide-r`

Source S:104. `Anim::SlideR`. Live: Space switch forward (S:102), sidebar edge peek (S:454).

```css
@keyframes slide-r{ from{ transform:translateX(26px); opacity:0; } to{ transform:none; opacity:1; } }
```

#### 4.3.2 `slide-l`

Source S:105. `Anim::SlideL`. Live: Space switch backward (S:103).

```css
@keyframes slide-l{ from{ transform:translateX(-26px); opacity:0; } to{ transform:none; opacity:1; } }
```

#### 4.3.3 `fade`

Source S:227. `Anim::Fade`. Live: scrim (S:222), command-menu backdrop (S:231, at `--t-quick` as `Anim::PaletteFade`, W2 integration). C has no `fade`; it has `fade-in` to .16 (section 4.3).

```css
@keyframes fade{ from{ opacity:0; } to{ opacity:1; } }
```

#### 4.3.4 `heal`

Source S:333. `Anim::Heal`. Live (S:330). C adds `.is-healing` (C:1858) but has no rule for it: a no-op (A8 #2).

```css
@keyframes heal{ from{ transform:translateY(var(--dy)); } to{ transform:none; } }
```

#### 4.3.5 `bump`

Source S:342. `Anim::Bump`. Live (S:341).

```css
@keyframes bump{ 0%{ transform:translateY(0) scale(1); } 40%{ transform:translateY(-3px) scale(1.25); } 100%{ transform:none; } }
```

#### 4.3.6 `tab-in`

Source S:352. `Anim::TabIn`. Live: Today entry opens (S:350). Animates `max-height`; see section 9.

```css
@keyframes tab-in{ 0%{ transform:translateX(-14px) scale(.96); opacity:0; max-height:0; } 60%{ opacity:1; max-height:40px; } 100%{ transform:none; opacity:1; max-height:40px; } }
```

#### 4.3.7 `tab-out`

Source S:353. `Anim::TabOut`. Live: Today entry closes (S:351), `Presence::Leaving(Exit::TabOut)` (W2 integration). Animates `max-height` and `padding-block`; see section 9.

```css
@keyframes tab-out{ 0%{ opacity:1; max-height:40px; } 100%{ opacity:0; max-height:0; padding-block:0; transform:translateX(-10px); } }
```

#### 4.3.8 `hc-in`

Source S:403. `Anim::HcIn`. Live: hover card (S:401), link pill (S:433, at `--t-quick --e-out` as `Anim::LinkPillIn`, W2 integration).

```css
@keyframes hc-in{ from{ opacity:0; transform:translateY(4px) scale(.98); } to{ opacity:1; transform:none; } }
```

#### 4.3.9 `hc-out`

Source S:404. `Anim::HcOut`. Live (S:402).

```css
@keyframes hc-out{ to{ opacity:0; transform:translateY(2px); } }
```

#### 4.3.10 `dest`

Source S:448. `Anim::Dest`. Live (S:447). The only S keyframe that loops (`infinite alternate`). `color-mix()` must become the precomputed `--accent-ring` (plan, token model).

```css
@keyframes dest{ from{ box-shadow:0 0 0 1.5px var(--accent) inset; } to{ box-shadow:0 0 0 3px color-mix(in oklab, var(--accent) 35%, transparent) inset; } }
```

#### 4.3.11 `page-in`

Source S:569. `Anim::PageIn`. Live: composer page (S:568), inline reply (S:720).

```css
@keyframes page-in{ from{ opacity:0; transform:translateY(10px) scale(.985); } to{ opacity:1; transform:none; } }
```

#### 4.3.12 `park`

Source S:571. `Anim::Park`. Live (S:570).

```css
@keyframes park{ to{ opacity:0; transform:translateX(-60%) scale(.35); } }
```

#### 4.3.13 `shake-x`

Source S:591. `Anim::ShakeX`. Live: To row when sending with no recipient (S:590, S:2319).

```css
@keyframes shake-x{ 20%{ transform:translateX(-6px); } 40%{ transform:translateX(5px); } 60%{ transform:translateX(-3px); } 80%{ transform:translateX(2px); } }
```

#### 4.3.14 `chip-in`

Source S:598. `Anim::ChipIn`. Live: person chip (S:593).

```css
@keyframes chip-in{ 0%{ transform:scale(.6); opacity:0; } 70%{ transform:scale(var(--overshoot)); opacity:1; } 100%{ transform:none; } }
```

#### 4.3.15 `menu-pop`

Source S:667. `Anim::MenuPop`. Live: every floating menu (S:666), selection bubble (S:684, at `--t-quick` as `Anim::BubblePop`, W2 integration).

```css
@keyframes menu-pop{ from{ opacity:0; transform:translateY(-4px) scale(.97); } to{ opacity:1; transform:none; } }
```

### 4.4 Defined only in C

#### 4.4.1 `breathe`

Source C:653. `Anim::Breathe`. C-only. Live: idle sync halo (C:288). Loops (`infinite`).

```css
@keyframes breathe{ 0%,100%{ opacity:0; transform:scale(.85); } 50%{ opacity:.45; transform:scale(1.06); } }
```

#### 4.4.2 `spin`

Source C:654. `Anim::Spin`. C-only. Live: busy sync halo (C:291). Loops (`infinite`).

```css
@keyframes spin{ to{ transform:rotate(360deg); } }
```

#### 4.4.3 `row-in`

Source C:658-662. `Anim::RowIn`. C-only. Live: arrival (C:2111) and undo (C:1931) via `.row.is-entering` (C:373).

```css
@keyframes row-in{
  0%{ transform:translateY(-14px) scale(.94); opacity:0; }
  58%{ transform:translateY(3px) scale(var(--overshoot)); opacity:1; }
  100%{ transform:translateY(0) scale(1); opacity:1; }
}
```

#### 4.4.4 `part`

Source C:663. No `Anim` variant. C-only. Orphaned in the shell: `.row.is-parting` (C:375) is never set by script; only the catalogue card plays it inline (C:2306). Not in the plan's `Anim` enum.

```css
@keyframes part{ 0%{ transform:translateY(-6px); } 100%{ transform:translateY(0); } }
```

#### 4.4.5 `chip-land`

Source C:674. `Anim::ChipLand`. C-only. Live: label chip after a drag-drop (C:402, C:2078).

```css
@keyframes chip-land{ 0%{ transform:scale(0) rotate(-12deg); } 62%{ transform:scale(1.22) rotate(4deg); } 100%{ transform:scale(1) rotate(0); } }
```

#### 4.4.6 `sail`

Source C:685-690. `Anim::Sail`. C-only. Orphaned: `.boat.sail` (C:535) is never created by script.

```css
@keyframes sail{
  0%{ transform:translate(0,0) rotate(0) scale(1); opacity:0; }
  12%{ opacity:1; }
  45%{ transform:translate(-90px,-70px) rotate(-7deg) scale(1.05); }
  100%{ transform:translate(-210px,-190px) rotate(-14deg) scale(.4); opacity:0; }
}
```

#### 4.4.7 `boat-return`

Source C:691-695. `Anim::BoatReturn`. C-only. Orphaned: `.boat.return` (C:536) is never created by script.

```css
@keyframes boat-return{
  0%{ transform:translate(-150px,-140px) scale(.5) rotate(-10deg); opacity:0; }
  25%{ opacity:1; }
  100%{ transform:translate(0,0) scale(1) rotate(6deg); opacity:0; }
}
```

#### 4.4.8 `nudge`

Source C:696. `Anim::Nudge`. C-only. Live: outbox retry (C:545, C:2165-2167). Bakes in `translateX(-50%)` for the centred pill; only valid on an element centred that way.

```css
@keyframes nudge{ 0%,100%{ transform:translateX(-50%) translateY(0); } 40%{ transform:translateX(-50%) translateY(-6px); } 70%{ transform:translateX(-50%) translateY(1px); } }
```

#### 4.4.9 `shake`

Source C:697. `Anim::Shake`. C-only. Live: outbox needs-sign-in (C:546, C:2174). Bakes in `translateX(-50%)`, like `nudge`. S uses `shake-x` for the same idea on an uncentred row.

```css
@keyframes shake{ 0%,100%{ transform:translateX(-50%); } 20%{ transform:translateX(calc(-50% - 7px)); } 40%{ transform:translateX(calc(-50% + 6px)); } 60%{ transform:translateX(calc(-50% - 4px)); } 80%{ transform:translateX(calc(-50% + 2px)); } }
```

#### 4.4.10 `crumple`

Source C:721-725. `Anim::Crumple`. C-only. Live: trash and purge exits (C:714, C:1888).

```css
@keyframes crumple{
  0%{ transform:scale(1) rotate(0) skewX(0); opacity:1; }
  40%{ transform:scale(.88) rotate(-4deg) skewX(-6deg); }
  100%{ transform:scale(.12) rotate(16deg) skewX(10deg) translateY(16px); opacity:0; }
}
```

#### 4.4.11 `menu-in`

Source C:1009-1010. `Anim::MenuIn`. C-only. Live: view-chrome menus and the in-row picker (C:1007).

```css
@keyframes menu-in{ 0%{ opacity:0; transform:scale(.94) translateY(-6px); }
  100%{ opacity:1; transform:scale(1) translateY(0); } }
```

#### 4.4.12 `fade-in`

Source C:1056. No `Anim` variant. C-only. Live: C scrim (C:1055) and C command-menu backdrop (C:1060). Ends at opacity .16, which is right for the scrim (whose resting opacity is .16) and wrong for `.cmdk-wrap`, whose resting opacity is 1: the backdrop animates to .16 and then snaps to 1. Not in the plan's `Anim` enum; S's `fade` replaces it.

```css
@keyframes fade-in{ from{ opacity:0; } to{ opacity:.16; } }
```

#### 4.4.13 `cmdk-in`

Source C:1066-1068. `Anim::CmdkIn`. C-only. Live: C command menu (C:1064). S uses `peek-in` for its command menu (S:234), and S wins.

```css
@keyframes cmdk-in{ 0%{ opacity:0; transform:scale(.93) translateY(-12px); }
  62%{ opacity:1; transform:scale(var(--overshoot)) translateY(2px); }
  100%{ opacity:1; transform:scale(1) translateY(0); } }
```


### 4.5 Added by quire (mailo gaps 3)

Settled for the port; the catalogue above is unchanged except where named.

- `fade-in` (4.4.12) is played after all, as `Anim::FadeIn`, ending on `--veil` (.16, a new
  opacity token) instead of a literal: C's ink veil rests at that opacity, so nothing snaps.
  S's `fade` still serves the scrim that rests at 1.
- `pill-up`: `from{ transform:translate(-50%,160%) } to{ transform:translate(-50%,0) }` at
  `--t-big --e-spring`, the entrance of a pill centred by `translateX(-50%)` (mailo's toast and
  send pill). quire's `SendPill` and `Toast` keep their `data-shown` transition, which also
  carries them down.
- `ring-drain`: `stroke-dashoffset` 0 to 57 at `--t-send-ring` linear, forwards, where CSS reaches
  the ring (the webview; on Blitz the SendPill writes the offset as an attribute, spike S6).
- `busy`: `0%,100%{ opacity:1 } 50%{ opacity:.45 }` at `--t-ambient --e-in-out`, infinite: a busy
  word's pulse, which stays legible where `breathe` fades to nothing. A loop, like `breathe` and
  `spin` (principle 7's listed exceptions); Reduced plays it once.
- `gulp`, `bump` and `seal-pop` scale their departure from rest by
  `min(1, (--overshoot - 1) x 25)`: 1 at Standard and Extra, 0 at Calm and Reduced. Calm (3.2:
  "no overshoot") now flattens them as it flattens `pop-in`; Standard is the text above.

### 4.6 Added by quire (control center parts, 2026-09-25)

- `slide-r` and `slide-l` are also played at `--t-move --e-spring` (`Anim::PaneInR`,
  `Anim::PaneInL`): a control-center detail pane arriving and the root pane coming back
  (design/13 section 13.3.7). The catalogue's Space-switch rows stay at `--t-big`.
- `pane-out-l`: `from{ transform:none; opacity:1 } to{ transform:translateX(-26px); opacity:0 }`
  and `pane-out-r`, its mirror to `+26px`, at `--t-move --e-exit`, forwards: the outgoing pane
  leaves the other way from the one arriving, over the same duration, so the pair reads as one
  push and one timer settles both (`PaneSwitcher`, sill Q80). An exit, so it does not spring
  (principle 2).

### 4.7 Added by quire (OSD parts, 2026-09-25)

- `osd-in`: `from{ opacity:0; transform:translateY(var(--osd-dy)) scale(.96) } to{ opacity:1;
  transform:none }` at `--t-quick --e-out` (design/20 §1.7's "in"), `Anim::OsdIn`: `pop-in`'s
  entrance with no overshoot.
- `osd-out`: `from{ opacity:1; transform:none } to{ opacity:0;
  transform:translateY(calc(var(--osd-dy) * .5)) }` at `--t-move --e-exit`, forwards (§10's exit
  rule), `Anim::OsdOut`.
- `--osd-dy` is the card's signed offset for its anchor, declared by `ds_shell::prelude::Osd` per position:
  `-8px` at the top right (drops in from above, lifts back out), `8px` at the bottom centre
  (rises in, drops away). Played outside the card, the fallback is the top right's.

### 4.8 Added by quire (sheet and modal parts, 2026-09-25)

- `sheet-out`: `from{ opacity:1; transform:none } to{ opacity:0; transform:scale(.98)
  translateY(8px) }` at `--t-move --e-exit`, forwards (section 10's exit rule), `Anim::SheetOut`:
  `peek-in` reversed and quieter. A sheet its host hides plays it and reports `on_hidden` at
  `settle(SheetOut)`; its scrim plays `menu-out` at `--t-quick` meanwhile.

### 4.9 Added by quire (notification parts, 2026-09-25)

- `banner-in`: `from{ transform:translate(var(--banner-dx,calc(100% + var(--s-16))),
  var(--banner-dy,0px)) } to{ transform:none }` at `--t-move --e-spring`, `Anim::BannerIn`: a
  banner slides in from past the surface's right edge, or from below its place when the stack's
  `data-entry=below` sets the vector to `0px, calc(100% + var(--s-16))` (section 12 item 7),
  opaque from its first frame. design/13 §13.3.6 proposed `--t-big`; the stack plays it at the
  `--t-move` of the exit and the heal it may arrive beside. It is played on the card's wrapper
  as it mounts, not on a presence attribute (Blitz keeps the last animated value when an
  animation is taken off an element).
- `banner-out`: `from{ opacity:1; transform:translateX(var(--swipe-dx,0px)) } to{ opacity:0;
  transform:translate(var(--banner-dx,calc(100% + var(--s-16))),var(--banner-dy,0px)) }` at
  `--t-move --e-exit`, forwards, `Anim::BannerOut` (and `Exit::BannerOut` for a roster row): a banner leaves by its
  entry edge from where a swipe left it (a swiped row, `data-flight=swipe`, always to the right);
  the rows below then `heal`.
- `panel-in`: `from{ transform:translateX(calc(100% + var(--s-16))) } to{ transform:none }` at
  `--t-move --e-out`, and `panel-out`, its reverse at `--t-move --e-exit`, forwards
  (`Anim::PanelIn`, `Anim::PanelOut`): the notification center's edge panel. No spring: opening
  it is not contact with it (principle 2), and an overshoot would lift it off its edge.
- A swiped card springs back with a `transform` transition at `--t-move --e-spring` (principle
  2: it is the release of a touch), none under Reduced, where it snaps back.
- The same panel at the bottom edge (`PanelEdge::Bottom`, Edit Widgets' sheet, sill Q521) moves
  as a sheet, not as an edge panel: its first showing plays `peek-in` at `--t-move --e-out`
  (`Anim::PeekFullIn`'s recipe), and every hide or show after it is the sheet's spring on
  `--present-p` (opacity, 12 px down, 95 %; Reduced: opacity only). No new keyframe.

### 4.10 Added by quire (screenshot thumbnail, 2026-09-26)

- `rise` at `--t-big --e-spring` (`Anim::ShotIn`): the screenshot thumbnail's entrance
  (design/20 section 1.13). The catalogue's `rise` row plays at `--t-move --e-out` for rows; the
  thumbnail is a surface arriving, so it takes the toast's `--t-big --e-spring`.
- `shot-out`: `from{ opacity:1; transform:none } to{ opacity:0; transform:translateX(calc(100% +
  var(--s-16))) }` at `--t-move --e-exit`, forwards (`Anim::ShotOut`): the thumbnail slides out
  to the right, past its own width, when dismissed or when its hold ends, holding its last frame
  until the host unmaps it at `settle(ShotOut)`. `slide-r` is an entrance (from 26 px right to
  rest); this is its exit, carried off the edge the card sits at.

### 4.11 Added by quire (the user's picture, 2026-09-26)

The persona's five keyframes (`persona-blink`, `-breathe`, `-wince`, `-hop`, `-drift`) and its
`--t-drift` token were removed with the persona. What remains:

- `picture-accept`: `0%{ transform:none } 35%{ transform:translateY(-8%) scale(1.03) }
  100%{ transform:none }` at `--t-big --e-spring` (`Anim::PictureAccept`), on
  `div.ds-user-picture`, from its base: when the password was right, the user's picture, whatever
  its kind, lifts once and lands (the unlock answers the user's contact, principle 2). Played
  once through a pulse and taken off at its `settle`; a lock screen unlocks at that settle. Not
  fired on mount or under Reduced.
- `--t-awake` (20 s) stays: an animated emoji's awake window (design/25 section 6).

**Exception to principle 7 (nothing loops).** An animated emoji plays its loop with no state
change asking for it. Allowed because it is bounded: whole loops only, only for 20 s after a wake
(mount, a new `WakeStamp`, a mood change), driven by a Rust task that then ends, never a CSS
`infinite`; then the picture rests on frame 0 and paints 0 frames (the idle-frame rule wins;
tested in `ds-conformance/tests/emoji_animation.rs` with `Harness::is_animating`). Under Reduced only still
frames are shown.

### 4.12 Added by quire (battery fill, 2026-09-26)

No keyframe: a battery ring's arc is an SVG path, which Blitz's stylesheet cannot animate, so
the fill is driven from Rust (`ds::use_level_run`, as the animated emoji drives its frames). On mount and on
each new `WakeStamp` the arc sweeps from empty to the level over `--t-fill` (800 ms, a new token)
at `--e-out`; on a new level, from the level drawn last. The path is recomputed on a 16 ms tick
and written only when it changes; the task ends when the sweep and its tail are done, so a ring
at rest re-renders nothing and paints 0 frames (tested in `ds-conformance/tests/battery_fill.rs`
with `Harness::is_animating`). Its tail: a charging bolt fades in over `--t-quick`, linear,
after an entrance fill. `BatteryFigure` counts the percentage in step. Under Reduced there is no
sweep at all (not a 60 ms one): the first frame is the final state. This replaces the ring's
`bump` on a new percentage (section 5 row 18 still applies to a figure a host wraps in
`Bumped`).

### 4.13 Added by quire (small-state details, 2026-09-26)

The primitives of design/26-DETAILS.md section 4. Six keyframes and nine `Anim` rows, each
played once on an HTML wrapper (Blitz's stylesheet cannot reach inside an SVG) and taken off at
its `settle`. Nothing here loops: the bounded pending loop is a Rust step timer over CSS
transitions (`--t-pending-step`), not a keyframe.

- `morph-in`: `from{ opacity:0; transform:scale(.7) } to{ opacity:1; transform:none }` at
  `--t-quick --e-out`, backwards (`Anim::MorphIn`): `MorphGlyph`'s incoming glyph (DownUp, OffUp).
- `morph-out`: the reverse, forwards (`Anim::MorphOut`): DownUp's outgoing glyph.
- `fade` at `--t-quick --e-out` (`Anim::MorphFadeIn`) and `morph-fade-out` (`from{ opacity:1 }
  to{ opacity:0 }`, forwards, `Anim::MorphFadeOut`): a cross-fade's two glyphs.
- `roll-in`: `from{ opacity:0; transform:translateY(60%) } to{ opacity:1; transform:none }` and
  `roll-out`: `to{ opacity:0; transform:translateY(-60%) }`, both `--t-quick --e-out`
  (`Anim::RollIn`, `Anim::RollOut`): `RollDigits`, one column per changed digit.
- `gulp` at `--t-big --e-out` (`Anim::SealOut`): a success seal (`SettleStyle::LockIn`) that
  nobody touched; the spring row `Anim::Gulp` plays only with `Touch::Contact` (design/26 R5).
- `nudge-up`: `0%,100%{ transform:none } 40%{ translateY(-6px) } 70%{ translateY(1px) }` at
  `--t-nudge --e-out` (`Anim::NudgeUp`): attention once per request (`use_nudge`, R6). `nudge`
  keeps the outbox pill's `translateX(-50%)` and so cannot move an element that is not centred
  that way.
- `slide-r` at `--t-move --e-out` (`Anim::PaneInROut`, sill Q370): the preview pane's entrance
  when nothing the person touched showed it; `Anim::PaneInR` (the spring) plays only when a key
  or press did (design/26 R5).
- `morph-in` at `--t-quick --e-spring` (`Anim::MorphInSpring`, design/26 D2): `MorphGlyph`'s
  incoming glyph when the person's own press caused the change (play/pause, a Focus disc); every
  other morph grows in at `--e-out` (`Anim::MorphIn`, design/26 R5).
- Edit Widgets' "Added" (sill Q523, design/23 section 9.7) is built from these, with no new
  keyframe: the Add button's check draws on (`SettleStyle::Check`) and grows in with
  `Anim::MorphInSpring` for the person's press (`Anim::MorphIn` otherwise); the placed row that
  arrives plays `Anim::RowIn` (`row-in`, `--t-big --e-spring`) for the press, `Anim::Rise`
  otherwise, taken off at its `settle`.

### 4.14 Added by quire (a widget card's exit, 2026-09-28)

- `widget-out`: `from{ opacity:1; transform:none } to{ opacity:0; transform:scale(.85) }` at
  `--t-move --e-exit`, forwards (`Anim::WidgetOut`, sill G423): a desktop widget the person
  removes in Edit Widgets shrinks about its centre and fades (principle 3: exits accelerate;
  principle 4: removing is acting, so it gets an exit), holding its transparent last frame until
  the host drops the card at `settle(WidgetOut)` (principle 10). Played on the card's pulse class
  when its host passes `CardPresence::Leaving` to `WidgetCard`/`WidgetFrame`, which calls
  `on_gone` at that settle. Reduced: `CrossFade(Out)` (`menu-out`), over Reduced's settle. A
  leave taken back plays `hold` so no half-faded value stays (sill G295). Not a roster row: a
  desktop card leaves nothing to heal.

## 11. Excluded keyframes

These are defined in C and deliberately left out of the canonical set: the mascot (Pip has CSS but no markup or script, A4) and the two look-scoped exits, which only apply under `body[data-look="riso"|"tide"]` and, for `dissolve`, need `filter:blur()` that Blitz cannot paint on the default backend. Quoted verbatim so nothing is lost if a look is revived.

### 11.1 `pip-float`

Source C:698. Mascot (Pip). CSS only; Pip has no markup or script in C (A4).

```css
@keyframes pip-float{ 0%,100%{ transform:translateY(0) scale(1,1); } 50%{ transform:translateY(-4px) scale(.995,1.01); } }
```

### 11.2 `pip-hop`

Source C:699-703. Mascot (Pip).

```css
@keyframes pip-hop{
  0%{ transform:translateY(0) scale(1,1); } 22%{ transform:translateY(2px) scale(1.1,.86); }
  52%{ transform:translateY(-16px) scale(.93,1.12); } 78%{ transform:translateY(1px) scale(1.07,.93); }
  100%{ transform:translateY(0) scale(1,1); }
}
```

### 11.3 `pip-wiggle`

Source C:704-707. Mascot (Pip).

```css
@keyframes pip-wiggle{
  0%,100%{ transform:rotate(0); } 18%{ transform:rotate(-7deg); } 42%{ transform:rotate(6deg); }
  66%{ transform:rotate(-4deg); } 86%{ transform:rotate(2deg); }
}
```

### 11.4 `zzz`

Source C:708. Mascot flourish; orphaned (no rule or script uses it).

```css
@keyframes zzz{ 0%{ opacity:0; transform:translate(0,0) scale(.7); } 25%{ opacity:.9; } 100%{ opacity:0; transform:translate(9px,-20px) scale(1.15); } }
```

### 11.5 `heart-up`

Source C:709. Mascot flourish; orphaned.

```css
@keyframes heart-up{ 0%{ opacity:0; transform:translateY(0) scale(.5); } 25%{ opacity:1; } 100%{ opacity:0; transform:translateY(-26px) scale(1.2); } }
```

### 11.6 `dissolve`

Source C:734-737. Look-scoped: Tide's exit for archive, trash and snooze (C:731-733). Uses `filter:blur()`.

```css
@keyframes dissolve{
  0%{ opacity:1; transform:translateY(0) scale(1); filter:blur(0); }
  100%{ opacity:0; transform:translateY(-8px) scale(.99); filter:blur(5px); }
}
```

### 11.7 `yeet`

Source C:739-743. Look-scoped: Riso's archive exit (C:738).

```css
@keyframes yeet{
  0%{ transform:translateX(0) rotate(0) scale(1); opacity:1; }
  22%{ transform:translateX(16px) rotate(2deg) scale(1.03,.94); }
  100%{ transform:translateX(-140%) rotate(-12deg) scale(.8); opacity:0; }
}
```


