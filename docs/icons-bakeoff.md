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
