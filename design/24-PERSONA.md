# 24 Persona

Status: built 2026-09-26 on branch `persona` (quire), for review. The request (user, 2026-09-26):
"a user icon, mac has these cute emoji like thing that moves, see if we can add that". The
reference desktop shows the user's animated character on its lock and login screen. We make
our own: a cute, abstract, procedural character drawn from a handful of flat shapes, never the
reference platform's assets, style or names. Code, classes and assets say **persona**.

## 1. What this governs

`ds::Persona` (the character), `ds::PersonaSpec` (what it looks like, user data), `ds::Mood`
(what it is doing, set by the caller), `ds::UserPicture` / `ds::UserPortrait` (a surface that
takes either a letter `AvatarFace` or a persona), the five persona keyframes and the two
duration tokens they need, and the persona's exceptions to design/00 section 4 and design/05
section 2.

## 2. References looked up

What was read, and what we took. Nothing was copied: no shapes, paths, palettes or code.

| Reference | What it does | Licence | Taken |
| --- | --- | --- | --- |
| The reference desktop's animated lock-screen character ([AppleInsider](https://appleinsider.com/articles/22/11/07/how-to-set-an-animated-memoji-on-your-lock-screen-in-macos-ventura), [MacRumors](https://www.macrumors.com/how-to/make-mac-user-profile-animated-memoji-monterey/), [Apple Community](https://discussions.apple.com/thread/255177602)) | Waves when shown, falls asleep if nobody logs in, smiles on a successful login, grows more annoyed with each wrong password; since the 2023 release it is small at the bottom of the screen | proprietary | The behaviour table's shape only (section 4): sleep, a reaction to a wrong password, a reaction to unlock. Not its escalating anger (one wince, the same every time: principle 6), not its wave on show (nothing plays on mount), not its 3D rendered style. |
| Phone-platform stickers and keyboard mashups ([Emojipedia on Emoji Kitchen](https://emojipedia.org/emoji-kitchen), [Blob emoji](https://en.wikipedia.org/wiki/Blob_emoji)) | Flat, round, gumdrop-shaped faces with a few features; the blob as a character with no species | proprietary | The idea that a character need not be human (the blob, the round animals). |
| [DiceBear](https://www.dicebear.com/licenses/) (fun-emoji, thumbs) | Avatars generated from a seed by picking one variant per part | code MIT; each style its artist's (fun-emoji CC BY 4.0, thumbs CC0) | The parts-as-variants model and seeded generation, which are ideas, not expression. No style, no SVG. |
| [Boring Avatars](https://github.com/boringdesigners/boring-avatars) (beam) | A face on a disc from a username and a palette | MIT | Nothing beyond the disc behind the face, which our `Avatar` already has. |
| Notion-style faces ([notion-avatar](https://github.com/Mayandev/notion-avatar)) | Hand-drawn black-line faces assembled from parts | illustrations CC0 | Nothing: line illustration is the opposite of our flat, colour-carried rule. |

## 3. The character system

### 3.1 Shapes (settled for review)

A 100 x 100 canvas on a disc. Every shape is flat: one colour, no outline on a filled shape
(a filled shape may carry a same-colour stroke to round its corners), no gradient, no shading,
no highlight but the one eye fleck. Strokes (eyes shut, brows, mouths, whiskers, glasses) are
round-capped ink, as the glyphs are.

- **Head**: a superellipse `|x/rx|^n + |y/ry|^n = 1` at (50, 55-57): `Round` (29.5 x 28.5, n 2),
  `Soft` (30.5 x 27.5, n 3), `Tall` (26.5 x 30.5, n 2.3), `Wide` (33 x 26.5, n 2.5). Features are
  placed as shares of the head's half widths, so every part fits every head.
- **Creature**: `Person` (no ears), `Bear` (round ears, light muzzle, ink nose), `Cat` (rounded
  triangle ears, ink nose, two whiskers a side), `Bunny` (two tall ears leaning out, a pink
  nose), `Blob` (no ears; always wears its sprout).
- **Eyes**: `Dot`, `Oval`, `Shine` (larger, one light fleck), `Wide` (dots set further apart).
- **Brows**: `Hidden`, `Soft` (arcs), `Straight`.
- **Mouth** (Idle's): `Smile`, `Cat` (a small w), `Small`, `Grin` (open, filled ink).
- **Top**: `Bare`, `Tuft` (one curl), `Crop` (a cap whose hairline lifts at the middle),
  `Fringe` (three rounded locks), `Bob` (length behind the head, a swept fringe), `Bun`,
  `Curly` (seven curls round the crown). Hair follows the head's own outline 2 units out.
- **Cheeks**: `Blush` (two soft ovals at .85), `Plain`.
- **Accessory**: `Plain`, `Glasses` (two rings and a bridge round the eyes), `Freckles` (three
  dots a cheek, in the face's deeper tone), `Bow` (the disc's hue, saturated).
- **Backdrop**: the disc, one of the icon palette's eight hues.
- At **Small** (28 px) brows, whiskers, freckles and the eye fleck are dropped and eyes and
  mouth are drawn 1.25x heavier (as design/08 drops details at 16 and 32 px).

### 3.2 Palette (derived, OKLCh)

On the icon palette's eight hues (design/08 section 2.10: clay 40, ochre 85, sage 130, jade 175,
teal 220, slate 265, plum 310, rose 355), computed in Rust (`persona/palette.rs`), never a
literal in CSS:

| Role | Value |
| --- | --- |
| Fur (animals, blobs) | L .79, C .12, the hue |
| Skin | Peach L .89 C .05 h 55; Sand .83 / .07 / 72; Honey .74 / .09 / 66; Umber .61 / .085 / 52; Cocoa .51 / .07 / 45 |
| Inner ear, muzzle | the face, L +.09, chroma x .45 |
| Freckles | the face, L -.16 |
| Blush, a rabbit's nose, a tongue | L face -.06 (at least .45), C .12, h 12 |
| Ink (eyes, brows, mouth, nose, glasses) | L .24, C .015, h 50: design/08's ink, warmed |
| Hair | Ink .30/.02/50, Cocoa .42/.06/50, Auburn .53/.12/40, Honey .78/.11/82, Silver .87/.01/250, Rose .70/.13/355, Teal .62/.10/220, Plum .52/.11/310 |
| Bow | L .62, C .16, the backdrop's hue |
| Disc, light | L .91, C .055 (L .80, C .07 when the face is within .08 of .91) |
| Disc, dark | L .46, C .06 (L .34 when the face is within .08 of .46) |

Every face is at least .25 lighter than the ink (so 28 px reads) and at least .05 away from its
disc in lightness in both schemes (tested). The chroma is **brighter than the icons' 0.07 cap**:
the brief asks for bright and colour-carried. `PersonaFinish::Muted` (a prop) takes every colour
to .55 of its chroma, the icons' Muted ratio, for a surface that follows `icons.style = Muted`.

**Offered for the user's pick** (gallery page Persona, "Two finishes", and the sheets in the
report): **A, Colour** (default) and **B, Muted**.

### 3.3 From a seed

`PersonaSpec::from_seed(u64)` (SplitMix64) weights people 4 : bear 2 : cat 2 : rabbit 2 : blob 2.
People take a skin tone and always hair (a bare head read as unfinished in the 48-seed grid);
animals take a fur hue and a tuft or nothing (hair on an animal read as a wig); a blob its
sprout. A fur's disc is at least 90 degrees of hue away from it. Any part may still be set
explicitly on any creature (the gallery's "Every top on every creature" grid checks those).
`PersonaSpec::default()` is an ochre blob with a teal sprout on a teal disc: no skin tone or
hair is assumed for a person who has not made their own.

### 3.4 Persistence

`PersonaSpec` is serde (Settings will save it): fieldless enums in `snake_case`, the struct with
`#[serde(default)]` so a partial or older spec loads, never `deny_unknown_fields`. The frozen
fixture is the default spec's JSON in `crates/ds/tests/persona_spec.rs`. `seed: PersonaSeed`
is stored so the blink rhythm survives a save.

## 4. Behaviour

The caller sets the mood; the persona plays it. Mounting, a new `wake: WakeStamp` and every mood
change **wake** it: `persona-breathe` runs once across a 20 s window in every mood, and in a mood
with open eyes a Rust timer blinks at gaps of 3 to 6 s taken from the spec's seed. When the
window closes nothing is left playing.

| Mood | Face | Motion on change | Lock screen use |
| --- | --- | --- | --- |
| Idle | the spec's eyes, brows and mouth | blinks and breathes for 20 s after a wake, then still | at rest; wake on pointer move or key |
| Attentive | eyes (and brows) a little lower, a small mouth | the glance down, `transform` over `--t-quick --e-out` | while the user types |
| Wince | `>` `<` squint, inner brow ends raised, a wobbly mouth | `persona-wince` once, `--t-shake --e-shake` (shake-x's beats in shares of the width, a small turn) | wrong password |
| Happy | `^` `^` eyes, brows up, open smile with a tongue | `persona-hop` once, `--t-big --e-spring` | unlocked |
| Asleep | closed eyes, a small `o`, head leaning 7 degrees | one `z` rises and fades (`persona-drift`, `--t-drift` 2400 ms); the lean over `--t-big --e-out` | display off |

Nothing plays on mount except the breathing window (the persona is shown, not acted on), and a
wake that is not a mood change plays no mood motion. Reduced motion: no pulse is fired, no timer
runs and no transition applies, so a mood changes at once and nothing blinks or breathes.

## 5. Motion rules and their exceptions

- **Nothing loops (design/00 section 4, design/05 principle 7), exception.** Idle blinks and
  breathes, which is motion no state change asked for. It is allowed only inside the 20 s awake
  window after a wake or a mood change, as finite plays: each blink is one `persona-blink`
  pulse fired by a Rust timer (never `infinite`), and the breathing is one `persona-breathe` run
  of 20 s (`--t-awake`) with four breaths inside its keyframes. After the window the persona
  rests fully and paints 0 frames: **the idle-frame rule wins** (design/CHECKLIST, "Idle surface
  paints 0 frames"). Tested: `ds-native/tests/persona_life.rs` sees a blink within 6 s of the
  wake and, 21 s after it, no pulse class and `Harness::is_animating() == false`.
- **No randomness (design/00 section 4 rule 6), exception.** The blink gaps vary, because two
  eyes blinking on a metronome read as mechanical and two personas side by side should not blink
  in step. The variation is deterministic: the same spec blinks at the same gaps after every wake
  (a sequence from its stored seed), so identical input still behaves identically.
- **Springs only on contact.** Happy's hop springs: it answers the user's own unlock, the
  contact. Nothing else in the persona overshoots; the glance, the lean and the drift ease out.
- **Errors shake once and hold still** (design/05 principle 7). Wince shakes once and holds its
  squint until the caller sets another mood. It does not escalate on repeated failures.
- **Motion is in the persona's own units.** Every keyframe moves in shares of the persona's box,
  so 28, 64 and 128 px move alike.

## 6. Built (quire)

- `crates/ds/src/components/persona/`: `spec.rs` (the parts), `seed.rs` (from_seed, blink
  gaps), `palette.rs`, `geometry.rs` (the superellipse head), `head.rs` (the still layer),
  `face.rs` (eyes, brows, mouth, worn, the z), `mark.rs`, `mood.rs`, `life.rs` (the pulses and
  the blink timer), `picture.rs` (`UserPicture`, `UserPortrait`); `persona.css`.
- Markup: `div.ds-persona[data-size][data-mood]` holding nested wrappers (hop, shake, breath,
  tilt, look, blink) and one `svg.ds-persona-layer[data-part]` per part (`still`, `eyes`,
  `brows`, `mouth`, `worn`), plus `svg.ds-persona-z` asleep. The brief asked for parts as groups
  inside one SVG; on Blitz stylesheet rules and animations do not reach inside an SVG (spike S6,
  FINDINGS "Glyph follows the level"), so each moving part is its own stacked SVG and the HTML
  wrapper around it is what moves. Colours are SVG attributes computed in Rust.
- Anims `PersonaBlink`, `PersonaBreathe`, `PersonaWince`, `PersonaHop`, `PersonaDrift`; tokens
  `--t-awake` 20 s and `--t-drift` 2400 ms (design/05 section 4.11).
- Gallery page **Persona** (`--page persona`) and `ds-gallery --persona-frames DIR` (eight
  frames through each mood's motion, taken through a `Harness`).
- The lock prompt (a separate branch) takes `LockUser { avatar: AvatarFace }` today; switching
  it to `UserPicture` and `UserPortrait` is its follow-up. Nothing in the lock files changed here.

## 7. Open decisions

1. **Finish**: A (Colour, bright) or B (Muted). Default A.
2. **Settings keys**: the spec (`account.persona`, a `PersonaSpec`) and whether the lock screen
   shows the persona or the letter; not in design/22 yet.
3. **Escalation**: the reference escalates its annoyance; ours winces the same every time
   (principle 6). Confirm.
4. **Wake sources on the lock screen**: pointer movement and key presses are assumed; the lock
   screen owner decides.
