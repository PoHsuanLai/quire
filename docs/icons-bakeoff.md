# App-icon bake-off: FLUX.2 Klein 4B vs Qwen-Image-2512 (2026-09-24)

This is design/08-ICONS.md 3.4 run once, local models only. **The winner is not picked
here: the user picks it from the sheets.** Below are the facts and the agent's own reading of
them.

## Contact sheets (`tools/progress/shots/icons/`)

| Sheet | What it shows |
| --- | --- |
| `klein-raw.png`, `qwen-raw.png` | 5 subjects (rows) x 4 seeds (11, 22, 33, 44), 1024 px renders as the model gave them |
| `klein-plated.png`, `qwen-plated.png` | the same renders through `tools/icons`: keyed, fitted, on the subject's plate, master with shadow; under each tile the 16, 32 and 48 px exports at 1:1 on a light (`#F1F3EE`) and a dark (`#1D211B`) ground |
| `klein-reference-raw.png`, `klein-reference-plated.png` | reference pass: hero Mail (Klein seed 22), then the other four subjects conditioned on it |
| `qwen-reference-raw.png`, `qwen-reference-plated.png` | the same with Qwen-Image-Edit-2511, hero Mail (Qwen seed 11) |

Rebuild: `tools/icons/bakeoff.sh <model> <raw> <plated> tools/progress/shots/icons` and
`tools/icons/vary.sh ...` (see `tools/icongen/README.md` for the renders). Raw renders and
plated masters stay in `~/comfy/out/bakeoff/`, not in git.

## Setup

- Brief: design/08 3.3 text, plus one sentence the 2026-09-24 smoke finding asked for ("A
  clean, simple 3D illustration of ..., not a photograph"). Verbatim in
  `tools/icongen/icongen/brief.py`. No platform names, no "icon".
- Subjects (08 3.4): Mail envelope, Files folder, Terminal (screen block with a prompt caret,
  no letters), Notes sticky pad, Photos instant camera.
- Plate per subject (proposal, 08 says "decided per icon at review") and the object's palette
  from a neighbouring family so it contrasts: Mail blue plate / amber object; Files amber /
  blue; Terminal violet / green; Notes green / amber; Photos red / blue.
- Klein: `flux-2-klein-4b-Q4_K_M.gguf`, 4 steps, euler, cfg 1, zeroed negative (the distilled
  recipe; Klein takes no negative prompt). Qwen: `qwen-image-2512-Q3_K_M.gguf`, 20 steps,
  euler/simple, cfg 2.5, shift 3.1, the 08 3.3 negative prompt. Both 1024 x 1024.
- Graphs: `tools/icongen/workflows/*.json`; each render's workflow sha256, seed, prompt,
  time and VRAM are in `~/comfy/out/bakeoff/<model>/runs.jsonl`.

## Timings and VRAM (RTX 5070 Ti 16 GB, ComfyUI 0.37.0, `limit16g`)

VRAM is `nvidia-smi memory.used` sampled every 0.5 s; the desktop alone holds 1.6-2.9 GB,
so "model share" is peak minus that baseline. Host RAM was tight throughout (a 74 GB job
held the machine at 5-9 GB available); nothing swapped into a failure.

| | Klein 4B Q4_K_M | Qwen-Image-2512 Q3_K_M | Qwen-Image-Edit-2511 Q3_K_M |
| --- | --- | --- | --- |
| Cold first render (load + sample) | 37.7 s | 98-114 s | 489 s |
| Warm render, 1024 px | 3.0-4.1 s (median 3.3) | 69-91 s (typical 70-75) | 118-126 s |
| Peak VRAM on the card | 12.0 GB | 14.7 GB | 14.5 GB |
| Model share over desktop | ~9.3 GB | ~12-13 GB | ~12 GB |
| Whole 20-render bake-off | ~70 s | ~26 min | (4 edits: ~14 min) |

Klein is about 20x faster per image. Qwen at Q3_K_M leaves ~1.2 GB of the 16 GB free, so it
cannot run next to any other GPU job; a larger quant would not fit.

## Assessment (the agent's reading; the user decides)

**Object readability at 48 px.** Both are good at 48 and readable at 32 for Mail, Files,
Notes and Photos. Qwen is slightly better: its objects are fuller in the frame, higher
contrast, and the camera and folder silhouettes are cleaner. At 16 px only Mail and Files
survive for both models; Photos becomes a blue blob with a dot, Notes an orange square,
Terminal a dark square. Terminal is the weakest subject in both: Klein draws a checkmark on
a green screen (reads as "done", not "terminal"), Qwen draws a mouse pointer and a checkmark
on a screen. Neither drew a prompt caret; the brief's "no letters" wins over "prompt caret".
Terminal needs a different subject phrasing (or the symbolic glyph plan of 08 2.7) whichever
model wins.

**Style consistency across subjects.** Both are consistent within a sheet: same matte clay
material, soft top-left light, gentle contact shadow. Qwen's set reads more like one family
(same three-quarter angle and scale across all five subjects); Klein varies the camera more
(Notes lies flat and rotated in three seeds, the envelope is frontal in one and tilted in
others) and its objects vary more in size, which the fit step evens out.

**Background cleanliness.** Both give the plain light grey ground asked for, with a soft
vignette and floor shadow. Qwen's ground has a lit floor band that the key initially left as
white slabs; the key now allows ground up to 0.08 L lighter than the corners and both sheets
key cleanly except a few edge cases: Klein seed 33 Mail and the white-framed Klein
terminals lose a sliver of their white rim (white object on a light ground is the key's hard
case), and Qwen Terminal seed 33 has a faint shadow ghost. The key is a colour-distance
region grow, not segmentation (08 3.7), and it is the post-process's weakest step.

**Artefacts.** Klein: occasional soft or melted edges (Mail s11's corners, Notes s11's warped
sheet), a slightly lumpy terminal bezel. Qwen: cleaner geometry and edges, crisp folds and
paper, no melting; its pointer-and-check glyphs on Terminal are the only invented symbols.
Neither drew any text or frame; neither drew a rounded tile behind the object.

**Reference pass (08 3.5, 3.6).** This is where the two differ most.
- Klein with the hero as a reference latent keeps one strong visual DNA (frontal view,
  identical footprint and scale, the same clay and light) but copies the hero's shape too
  hard: "files" came back as a blue envelope, "notes" as a flat card. Terminal (a play
  button in a screen) and Photos (a camera) are good and clearly sibling objects.
- Qwen-Image-Edit-2511 at Q3_K_M with the settings above did badly: every edit got a gold
  squiggle-patterned background (which the key cannot remove, so the plated tiles are
  dirty), "notes" came back as a copy of the envelope, "terminal" as a green envelope. Files
  and Photos kept the hero's DNA and are good objects. The likely causes are this run's
  choices, not the model's ceiling: an empty negative prompt at cfg 4, 20 steps instead of
  the template's 40, the Q3 quantisation of a 20B edit model, and the hero passed as a raw
  render with its grey ground. Not retried here.
- So neither reference pass is usable as-is; both carry the hero's look, both over-copy its
  shape. Text-to-image with the fixed brief already gives a consistent set for both models,
  which makes the reference pass less necessary than 08 3.6 assumes.

**Summary for the choice.** Qwen-Image-2512 gives the better-looking, more consistent raw
set; Klein is 20x faster, fits beside another GPU job, and is close behind on four of the
five subjects. Neither is at the "too low, go hosted" line on this evidence: the Terminal
subject and the reference pass are the open problems, and both are prompt and workflow
problems as much as model ones.

## Post-process notes (tools/icons)

- Plate: n = 5 superellipse; its measured area is 95.0 % of the square (test
  `squircle_area_is_95_percent`), as 08 2.1 computes. 08 2.2's grid holds at every size
  (test `grid_matches_the_table`).
- 08 2.5's shadow scaled by plate/48 at 1024 gives offset 103 px and blur 275 px: more than
  the 100 px canvas margin, so the baked shadow is clipped at the canvas bottom in the
  master. It looks acceptable on the sheets, but 08 2.5 should state whether the shadow
  scales with the plate or is capped to the margin (open).
- The key's thresholds differ from 08 3.7's proposal (ΔE_OK 0.03, 2 px feather): the renders
  have a vignette and a soft floor shadow, so a single-colour ΔE threshold either eats white
  objects or keeps the shadow. The key now grows the ground from the border with a chroma
  gate (0.025), lighter (0.08) and darker (0.35) L limits and a 0.012 ΔE step limit between
  neighbours, drops specks below 2 % of the object's area, and keeps the model's floor shadow
  as a translucent dark matte cut at the safe square with a 4 % feather (08 2.4). All values
  are in `Template` (proposed, like the rest).
- Small exports (16/24/32) are downscaled from the 824 px plate region in linear light and
  re-masked at the target size (08 2.6); the rim and highlight are drawn at 1024 and shrink
  with it, so at 16 px they vanish.

## What was not run

- HiDream-O1-Image (08's third candidate, "only if 1 or 2 disappoints"): not downloaded.
- A second reference pass with better edit settings (negative prompt, 40 steps, hero object
  on a flat ground) for Qwen-Image-Edit; and a Klein reference pass with a weaker reference
  (the hero at lower resolution or as the second of two references).
- Speed LoRAs (Qwen Lightning 4/8-step): not used, so the Qwen timings are the plain model's.

## Round two: abstract, in our language (2026-09-24)

The user on round one: "too realistic, I want it to be more abstract, and try to make it speak
our design language." Two directions were built side by side; **the user picks**. The language
as it applies to an app icon is written down in design/08-ICONS.md 2.8 (paper cut-outs, one
stroke weight with round caps, four colours, Post's `12 12 12 4` corner, the frame grain), and
08 3.3's brief is rewritten for it.

### Sheets (`tools/progress/shots/icons/`)

| Sheet | Direction | What it shows |
| --- | --- | --- |
| `abstract-plated.png` | B, procedural | the five specs, each at 512, 48, 32 and 16 px 1:1, on the light (`#F1F3EE`) and dark (`#1D211B`) ground; every size drawn natively from the shapes |
| `qwen2-plated.png` | A, generated | Qwen-Image-2512, round-two brief, 5 subjects x 4 seeds, plated by `tools/icons` (256 px tile of the master, 16/32/48 strip on both grounds under each) |
| `klein2-plated.png` | A, generated | FLUX.2 Klein 4B, same brief and layout |
| `klein2-trigger-plated.png` | A, variation | Klein with the flat-illustration trigger in front of the brief, 4 seeds |
| `qwen2-trigger-plated.png` | A, variation | Qwen with the trigger, seeds 11 and 22 only (time) |

Rebuild: B is `target/release/icons abstract --spec tools/icons/specs/*.toml --out-dir <dir>
--sheet <png>` (about 3 s for all five, every export size and the sheet). A is
`uv run icongen bakeoff --model <m> --style flat|flat-trigger --out ~/comfy/out/round2/<m>`
then `tools/icons/bakeoff.sh <name> <raw> <plated> <sheets>`. Renders stay in
`~/comfy/out/round2/`, not in git.

### What ran

- Brief: 08 3.3's round-two text, verbatim in `tools/icongen/icongen/brief.py` (`FLAT_BRIEF`,
  `FLAT_NEGATIVE`, `FLAT_TRIGGER`, `FLAT_WORDS`). The object now uses its own plate family's
  two tones (words and hex) plus paper white and ink; the ground is mid grey so white and ink
  both key. Klein ignores the negative prompt (distilled, cfg 1); Qwen gets it.
- Terminal wording: first "a prompt chevron and a cursor block, side by side, inside a rounded
  dark rectangle"; Klein drew down-pointing double chevrons and a hand cursor. Rewritten to "a
  prompt chevron and a cursor block: an ink right-pointing chevron like a greater-than sign,
  then a solid paper white bar, side by side on one line" and re-run; the sheets show only the
  rewritten wording.
- Settings as round one (Klein Q4_K_M, 4 steps; Qwen-Image-2512 Q3_K_M, 20 steps, cfg 2.5),
  1024 px, seeds 11 22 33 44. Timings on the free card: Klein 3.0 s warm (8.0 s cold), peak
  11.1 GB; Qwen 67-78 s (median 70), peak 15.0 GB. 24 Klein + 20 Qwen + 10 Qwen trigger renders.
- Plating unchanged from round one (`tools/icons plate`): the key handled the mid-grey ground
  cleanly for every render; no white-rim losses this time.
- B: `tools/icons abstract` plus five specs in `tools/icons/specs/`, written by hand (mail: a
  deep-blue flap on a paper rectangle; files: a soft back folder and a paper front folder with
  an ink label bar; terminal: an ink chevron and a paper cursor block on the bare plate; notes:
  a paper sheet with a turned-down corner and two ink lines; photos: a paper frame with a deep
  hill and a base-red circle).

### Comparison (the agent's reading)

**Readability at 16/32/48.**
- *Procedural*: all five read at 48 and 32; at 16 mail, terminal and photos are still clear,
  files and notes read as "a paper thing with a line" (the label bar and the fold survive, the
  back folder is a tint). Edges stay crisp at 16 because each size is drawn from the shapes, not
  downscaled. It is also the smallest object on the plate (about 67 % against the generated
  sets' 72 % fit), which costs a little at 16 and buys the "generous negative space".
- *Qwen*: mail, files and terminal read at every size; its terminal is the first model terminal
  that is a terminal (a big `>` and a bar, all four seeds). Notes fails: every seed is an
  all-green sheet on the green plate, no paper and no lines, so at 16 it is a green square.
  Photos reads at 48, blurs to a red frame at 16.
- *Klein*: mail and photos read at every size; photos at 16 reads as a person (a circle over
  a hump), a real misreading risk. Files is two brown slabs (no tabs in three seeds), terminal
  has a violet chevron on a violet plate (two seeds drew a diamond), notes is a white square
  with a green corner that at 16 is only a white square.

**Coherence across the five.**
- *Procedural*: coherent by construction: one stroke, one corner family, one lift, the same
  four colours and the same scale in every icon. Nothing varies that was not written down.
- *Klein*: a consistent sticker style (black outline on every shape), but that outline is a
  second weight next to our 2-unit stroke and is not in the language. Seeds vary a lot in what
  they draw.
- *Qwen*: the least coherent of the three this round: mail and terminal are flat, files is
  classic folder clip art, and several seeds reintroduce depth the brief forbade (extruded
  envelope edges, bevelled chevrons, offset drop shadows on notes, grey ground left inside the
  photo frames).

**How much each speaks the language.**
- *Procedural*: the most: paper is Post `--surface`, ink is Post `--ink`, the stroke is the
  glyph stroke with round caps, the corners are the card's, the grain is the frame's tile, and
  colour appears only as the plate family. It looks like the shell's own glyphs grown up.
- *Generated*: both moved decisively away from round one (no clay, no studio light), and Klein's
  mail and photos are close in spirit. But neither keeps the rules the language is made of:
  outlines at their own weight, hues outside the four, square caps and sharp corners, and
  (Qwen) creeping 3D. The flat-illustration trigger changed little on Klein (cleaner mail folds,
  slightly better chevrons, no gain on files or notes). On Qwen it fixed notes (both seeds now
  draw a white paper sheet with a turned corner instead of a green one) but kept the offset
  drop shadows and the grey ground inside the photo frames.

**Effort per new icon.**
- *Procedural*: writing a 20-40 line TOML spec and looking at the sheet; about 10-20 minutes by
  hand per icon, a few seconds to render, fully reproducible, reviewable as text in a diff.
  The cost is taste: someone has to design each icon's arrangement, and the vocabulary (five
  shapes plus polygon) limits how illustrative an icon can get; that limit is the language's
  own ("nothing here is an illustration").
- *Generated*: 20 renders per subject (1 min on Klein, 25 min on Qwen), then a human picks
  one and usually still wants a colour or shape fixed, which cannot be done by editing the
  result; the next icon may not match the last (08 3.6's reference pass over-copied in round
  one). A LoRA (08 3.9) could lock the style, at the cost of a training run and ~8 approved icons
  first.

**Summary for the choice.** The procedural route is the only one that follows the language
rule for rule, is coherent without effort, and keeps its 16 px exports crisp; its limit is that
each icon is designed by hand from a small vocabulary. The generated route now produces flat
icons, and Klein's mail and photos are usable ideas, but neither model holds the stroke, corner
and colour rules, and each set would need a picking pass plus fixes. A plausible middle: use
the models as sketchpads for arrangements, then write the chosen arrangement as a spec.

### Not run

- HiDream-O1-Image, a Klein LoRA, and a second reference pass: out of scope for this round.
- Qwen with the trigger on seeds 33 and 44 (time: 70 s per render).
- A per-size hinting pass for the procedural 16 px (snapping strokes to whole pixels); the
  analytic anti-aliasing is used as is.

## Round three: pressed into the plate (2026-09-24)

The user sent a reference and asked for its details, not its look: no frosted glass, colours
free per app, but the symbol crafted into the plate (shallow emboss, soft internal geometry, no
borders, outlines or strokes), a plate with thickness whose bevel catches light in a few narrow
spots, matte diffusion, and a soft two-hue gradient ground per app. The rules are
design/08-ICONS.md 2.9; they replace 2.8's one-stroke rule for app icons only (the glyph set
keeps Lucide's stroke). Again two directions; **the user picks**.

### Sheets (`tools/progress/shots/icons/`)

| Sheet | Direction | What it shows |
| --- | --- | --- |
| `round3-procedural.png` | procedural | the five specs at 512, 48, 32 and 16 px 1:1, light and dark ground; every size drawn natively |
| `round3-klein-ground.png` | generated, face | Klein draws the embossed symbol on a full-bleed two-hue surface; `tools/icons face --mode ground` takes it as the plate face and adds our bevel. 5 subjects x 4 seeds, 256 tile plus 16/32/48 on both grounds |
| `round3-klein-ground-picks.png` | generated, face | one pick per subject at 512, 48, 32, 16 on both grounds (mail s33, files s22, terminal s11, notes s33, photos s11) |
| `round3-klein-tile.png` | generated, whole tile | Klein draws the whole bevelled tile on mid grey; `face --mode tile` keys it and re-masks it with our squircle, keeping its own bevel |
| `round3-klein-tile-picks.png` | generated, whole tile | one pick per subject (mail s22, files s44, terminal s22, notes s22, photos s33) |
| `round3-qwen-ground.png` | generated, face | Qwen-Image-2512 on the two best Klein subjects, mail and terminal, 4 seeds, face mode |

Rebuild: `target/release/icons abstract --spec tools/icons/specs/*.toml --out-dir <d> --sheet
<png>`; `uv run icongen bakeoff --model klein --style emboss|emboss-tile --out <raw>`, then
`tools/icons/round3.sh <sheet> <raw> <plated> ground|tile <sheet-dir>` and `icons strips` for the
picks. Renders in `~/comfy/out/round3/`, not in git.

### What ran

- Procedural: the vocabulary lost its strokes (chevron and bar are filled round-ended bands);
  each layer is `raised`, `recessed` or `flush` with a fill and an opacity; the plate gained a
  per-icon two-hue ground in OKLCh, matte diffusion and a bevel finish (top arc, bottom shade,
  one specular point). At 16 and 32 px the emboss and bevel details are dropped and recessed
  fills are strengthened. The five specs were rewritten for it.
- Klein 4B: 20 face renders, 20 tile renders, 3.0 s each warm, peak about 10 GB. Photos was
  first worded "a circle above a gentle rounded hill, inside a rounded frame" and every seed in
  both modes read as a person (a head over shoulders); it was reworded (the sun small and off
  to the upper right) and re-run, and the sheets show only the new wording.
- Qwen-Image-2512 Q3_K_M: mail and terminal only, face mode, 4 seeds each, 70-87 s per render,
  peak 15.1 GB.

### Comparison (the agent's reading)

**Readability at 16/32/48.**
- *Procedural*: all five read at 48 and 32 and hold their silhouettes at 16 (the flat fallback
  keeps the V of the flap and the notes lines visible). Highest contrast of the three: the
  symbol is near-white paper on a saturated but softened ground.
- *Klein, face mode*: mail, terminal and photos read at 48 and 32; at 16 they blur to a pale
  blob on a pastel plate. Files and notes are weak at every small size: pale grey symbol on a
  ground Klein washed to near-grey in the middle. Klein's grounds come out pastel and patchy,
  not the two clear hues asked for.
- *Klein, tile mode*: bigger symbols, so mail and files read better at 32, but the key fails
  where the tile's own face is pale grey against the mid-grey ground: holes in notes s11/s44
  and terminal s33, black specks at 16. Only usable after picking.
- *Qwen*: the most "crafted into" of all (symbol in the plate's own colour, tone on tone), which
  is exactly what makes it the weakest at 16 and 32; its bevels are chiselled rather than soft
  and one terminal seed drew a dark outline.

**Coherence.**
- *Procedural*: coherent by construction, the same relief, bevel, light and scale everywhere;
  only the ground pair and the shapes change.
- *Klein, face mode*: surprisingly coherent: the same clay-matte material, light and emboss depth
  across all five and across seeds. The symbol size and the ground saturation vary.
- *Klein, tile mode*: the tile's own proportions, bevel and colour blocks vary per seed, so the
  set looks less like one family once re-masked.
- *Qwen*: two subjects only; within them consistent.

**How close to the reference details.**
- Emboss without outlines: Klein face mode is the most convincing soft emboss; the procedural
  one is cleaner but more graphic (a light top edge and a shade band on a flat fill, no real
  curvature); Qwen is embossed but chiselled.
- Plate thickness and narrow bevel light: procedural and face mode get our bevel (thin arc, one
  soft point); tile mode keeps Klein's own, which is softer and wider.
- Two-hue ground: procedural gives the clearest two hues; Klein's are pastel and uneven.

**Effort per new icon.** Unchanged from round two: a spec (10-20 minutes by hand, seconds to
render) versus 20 renders and a picking pass per subject (1 minute on Klein, 25 on Qwen), with
face mode needing no keying and tile mode needing a key that fails on pale tiles.

**Summary for the choice.** Procedural is the most legible and the only one guaranteed
coherent; Klein face mode comes closest to the reference's soft clay emboss and is itself
consistent, at the cost of 16/32 legibility and weak grounds. A workable hybrid: Klein face
renders as the material study, the procedural renderer's ground pairs and bevel as the plate,
or the procedural route with a deeper relief pass if the user prefers the clay look.

### Not run

- Qwen on files, notes and photos, and Qwen in tile mode (time: 70 s per render).
- Compositing a keyed Klein symbol onto our own procedural two-hue plate (the soft emboss has
  no hard edge for the key to stop at; face mode was used instead).

## Round four: dialects in one muted palette (2026-09-25)

The user on round three: fewer gradients, less vibrant colour, and not every icon from the same
recipe; mix in other design languages, e.g. monochrome. Round three's picks stay the starting
point (procedural terminal and notes; Klein face-mode mail, files and photos). The rules are
design/08-ICONS.md 2.10: one shared skeleton (squircle, light, bevel arc, bottom shade, drop
shadow, emboss depth, grain), one muted palette in OKLCh under a **chroma cap of 0.07**, no
gradients beyond a ±0.012 L tonal shift, and four dialects: Monochrome, Graphite, Paper, Solid.
**The user picks a dialect per app.**

### Sheets (`tools/progress/shots/icons/`)

| Sheet | What it shows |
| --- | --- |
| `round4-dialects.png` | rows mail, files, terminal, notes, photos; columns Monochrome, Graphite, Paper, Solid (procedural, every size drawn natively), and for mail, files and photos a fifth column "Klein monochrome": the round-three Klein face-mode pick (mail s33, files s22, photos s11) retinted into the Monochrome dialect. Each cell is the 512 px icon with its 16, 32 and 48 px at 1:1 on the light (`#F1F3EE`) and dark (`#1D211B`) ground underneath |
| `round4-monochrome-space.png` | the whole set in Monochrome tinted by the Space, one row for Work (hue 268) and one for Home (152), the tint taken from `ds::space::derive`'s accent so it matches the frame |

Rebuild: `target/release/icons face --input ~/comfy/out/round3/klein-ground/<pick>.png --mode
ground --retint monochrome --tint <hue> --out-dir <klein-dir> --name <app>` for the three Klein
picks, then `target/release/icons dialects --spec tools/icons/specs/*.toml --klein-dir
<klein-dir> --out <png>` and `target/release/icons space --spec tools/icons/specs/*.toml --out
<png>`. No model was run this round: the Klein variants are round three's renders, retinted.

### What changed

- `tools/icons`: the spec names a recommended `dialect` and a `tint` from the palette, and each
  layer a role (`symbol`, `secondary`, `detail`, `spot`) instead of a colour; the plate is one
  colour with the tonal shift (Solid: flat); `dialect.rs` holds the palette, the cap and the
  role table; `retint.rs` brings a model face into a dialect (a lightness plane fitted to the
  border is the ground, the relief above it is scaled to the dialect's symbol lightness, and
  mottling under 0.02 L is quietened so the gain lifts the symbol, not the model's texture);
  `board.rs` builds the cells and derives the Space tint through `ds`.
- The five specs keep round three's shapes, recoloured by role. Recommended dialects in the
  specs: mail Monochrome slate, files Solid ochre, terminal Graphite, notes Paper (ochre spot on
  the fold), photos Monochrome clay.
- `ds-settings`: `icons.style` (Colour | Muted | Monochrome, default Colour) and
  `icons.monochrome_tint` (Space | Accent | Neutral, default Space), both Basic on the Appearance
  page; design/22 3.3 and 5 carry the rows. In Monochrome, third-party icons inside our plates are
  desaturated and re-tinted to the same hue. Only the keys and the sheets exist; the runtime
  waits for the pick.

### Reading (the agent's)

- **Colour.** Every plate now sits at or under C 0.07 against round three's 0.15-0.20; next to
  the Klein retints the procedural plates are the same family of tones, so a mixed set reads as
  one palette. No icon carries a gradient you can see.
- **Legibility.** All four dialects keep the symbol at least 0.26 L off its plate, and every
  procedural cell reads at 32 and 48. At 16: Graphite and Solid are the clearest (light on dark or
  on colour), Monochrome is close behind, Paper's white plate nearly disappears on the light
  ground (the ink symbol carries it, the plate edge does not) but is the strongest on dark.
- **Klein variants.** Mail and photos retint very well: the clay emboss survives, the colour is
  now the palette's, and at 16/32 they read as well as the procedural Monochrome. Files is
  weaker: Klein's ground under that pick is mottled, and the gain needed to lift its pale folder
  also lifts the mottling (a cleaner seed or a re-render would fix it).
- **Monochrome vs Solid** differ less than the other pairs (both a coloured plate with a light
  symbol); the difference is tone-on-tone symbol and a tonal shift against a white symbol on a
  flat plate. If the user wants Solid louder, it is the one dialect that could take a slightly
  higher cap.
- **Space tint.** Work and Home give a blue-violet and a green set that match the frames' hues;
  the symbols stay legible at every size in both.

### Per-app recommendation

| App | Dialect | Why |
| --- | --- | --- |
| Mail | Monochrome, the Klein variant | the softest, most crafted emboss of the set, reads at 16 |
| Files | Solid ochre (procedural) | the Klein files is mottled; Solid's white folder is the clearest at 16 |
| Terminal | Graphite | a terminal is a dark surface; the light chevron reads at every size |
| Notes | Paper, ochre spot | the one icon on paper, like a page; the spot marks the fold |
| Photos | Monochrome clay, the Klein variant | as mail; the off-centre sun keeps it from reading as a person |

That mixes three dialects plus Paper across five apps while keeping one palette and one skeleton.

## Round five: colourways (2026-09-25)

The user: "Can we try more colors for the icons", read as more colourways within round four's
language. Procedural only; the Klein variants are round three's faces retinted as in round four.
**The user picks a colourway per app** (and whether to go bolder).

### Sheets (`tools/progress/shots/icons/`)

| Sheet | What it shows |
| --- | --- |
| `round5-colourways.png` | each app in its round-four dialect in all eight hues at C 0.07: mail Monochrome (Klein retint of mail s33), files Solid, terminal **Monochrome** (its recommended Graphite has no hue, so it cannot show colourways), notes Paper (only the spot on the fold changes), photos Monochrome (Klein retint of photos s11). Each cell is the 512 px icon with its 16/32/48 at 1:1 on light and dark |
| `round5-bolder.png` | the five apps in clay, sage and slate (hues 40, 130, 265, a third of the wheel apart) at C 0.07, 0.11 and 0.15; same skeleton, no gradients |
| `round5-palette.png` | the eight hues as the Solid plate colour (L 0.62) at the three caps, on the light and the dark ground, each labelled with its name, OKLCh values and hex; `CLIP` marks a colour outside sRGB |

Rebuild: the Klein retints with `icons face --mode ground --retint monochrome --tint <hue>
--chroma <cap> --name <app>-<hue>-<cap>` for mail s33 and photos s11 (48 files), then
`icons colourways --spec tools/icons/specs/*.toml --as terminal=monochrome --klein-dir <dir>
--out <png>`, `icons bolder ... --hues clay,sage,slate --out <png>` and `icons palette --out
<png>`. Each sheet renders in a few seconds.

### What changed

- The palette grew from six uneven hues to eight, 45 degrees apart: clay 40, ochre 85, sage 130,
  jade 175, teal 220, slate 265, plum 310, rose 355 (design/08 2.10). Sage moved from 150 to 130,
  teal from 200 to 220, slate from 255 to 265 and plum from 320 to 310; the specs still name the
  same hues, so the round-four icons shift by those few degrees.
- `tools/icons`: a `ChromaCap` (0.07 default, 0.11 and 0.15 for comparison) threads through the
  roles and `face --chroma`; `colourways`, `bolder` and `palette` subcommands, and `--as
  app=dialect` to show an app in another dialect. The specs are unchanged.

### Reading (the agent's)

- **Eight hues read as eight.** At C 0.07 every neighbour pair is distinct at 512 and at 48; at
  16 the closest pairs are clay/rose and teal/slate, which separate by hue only a little at that
  size.
- **Against the dock.** Every hue, at every cap, sits at a WCAG ratio of 3.0-4.1 against the dock
  pill on the Work and Home frames in both schemes (the pill is white at .72 over the light frame
  L 0.94 and white at .10 over the dark frame L 0.22; the plate is L 0.62), so no colourway fails
  the pill; lightness does that work, not hue.
- **Against the Space presets' frames.** Work's frame is hue 268 and Home's 152, both at very low
  chroma (0.04 and 0.03 light). *Sit well*: on Work, slate (the same hue, the icon reads as part
  of the Space), teal and plum (neighbours, 42-48 degrees away); on Home, sage and jade (22-23
  degrees away) and ochre and teal. *Clash*: the near-complements, ochre and clay on Work (132-177
  degrees away) and plum and rose on Home (157-158 degrees). At C 0.07 the clash is mild (the
  frame is nearly grey); at 0.15 the complements vibrate against the tinted frame and the dock
  stops reading as one Space. In Monochrome with `icons.monochrome_tint = space` the question does
  not arise: the icons take the Space's hue.
- **Bolder.** 0.11 still reads as the same family as round four and makes the set noticeably
  livelier, especially clay and slate; 0.15 turns the plates into candy again (close to round
  three's saturation), clips ochre, jade and teal out of sRGB, and loses the matte clay look in the
  Klein retints. If the user wants more colour, 0.11 is the step to take; 0.15 is the upper bound
  this sheet exists to rule out.

### Per-app colourway recommendation

| App | Colourway | Why |
| --- | --- | --- |
| Mail | slate | the suite's anchor hue, sits on the Work frame, and the Klein envelope is at its best in cool blue |
| Files | ochre | warm folder colour; clearest white-on-colour of the eight; the one warm Solid plate in the dock |
| Terminal | jade | a green-teal terminal reads as a terminal, and it separates from mail's slate |
| Notes | clay spot | a warm mark on the paper page; ochre is taken by files |
| Photos | rose | a warm dusty pink suits a photo frame and keeps it apart from files' ochre and notes' clay |

Five distinct hues, each app a different part of the wheel, all at C 0.07; move to 0.11 as a set
if the user wants them louder.

## Round six: shipped (2026-09-25)

The user: "go with your suggestion for now". The shipped set is round four's dialect and round
five's hue per app (design/08-ICONS.md 2.11): mail Monochrome slate (Klein retint), files Solid
ochre, terminal Monochrome jade, notes Paper with a clay spot, photos Monochrome rose (Klein
retint), all at chroma 0.07.

- **Assets.** `assets/icons/apps/<app>/<px>.png` (Colour), `<app>/muted/` (cap 0.04) and
  `<app>/monochrome/` (neutral grey, tinted at run time), 13 sizes each, 195 PNGs, 6.1 MB. Written
  by `tools/icons ship` from `tools/icons/ship.toml`; the Klein source renders stay out of git.
- **Consistency.** Both kinds of face go through one finish (squircle, bevel arc, bottom shade,
  specular point, rim), one grain and one drop shadow at every size.
  `tools/icons/tests/shipped.rs` measures the exported 512s in every style: the silhouette, the
  bevel shade and arc, and the shadow's reach match across the five. One exception it records:
  the Paper plate is too close to white for the white arc to show.
- **The run-time half.** `ds::icon::retint` (bytes in, bytes out, no renderer) re-colours our
  Monochrome set and third-party icons; `ds::icon::Tint::space` takes the Space's accent through
  `ds::space::derive`, as the round-four sheet did. `sill` picks the file per style and calls
  `retint` after loading, before caching (CONSUMING.md "App icons and the icon style").
- **Sheet.** `tools/progress/shots/icons/round6-shipped.png`: the shipped set, the Muted set, and
  the Monochrome set tinted through `ds::icon::retint` for Work (268) and Home (152), each at 512
  with 16/32/48 on light and dark, all read back from `assets/icons/apps/`, so the sheet is
  exactly what ships.
