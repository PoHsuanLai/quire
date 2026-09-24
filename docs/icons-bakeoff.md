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
