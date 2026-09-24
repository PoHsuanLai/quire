# icongen

Stdlib-only client of ComfyUI's HTTP API for the app-icon pipeline (design/08-ICONS.md 3).
ComfyUI itself lives in `~/comfy` (start it as `~/comfy/README.md` says, under `limit16g`).

```bash
cd tools/icongen
uv run icongen bakeoff --model klein --out ~/comfy/out/bakeoff/klein      # 5 subjects x 4 seeds
uv run icongen bakeoff --model qwen  --out ~/comfy/out/bakeoff/qwen
uv run icongen bakeoff --model klein --style flat --out ~/comfy/out/round2/klein   # round two (08 3.3)
uv run icongen bakeoff --model klein --style flat-trigger --out ~/comfy/out/round2/klein-trigger
uv run icongen vary --model klein --hero <raw.png> --hero-subject mail --out <dir>   # reference pass
uv run icongen free                                                        # unload models
uv run icongen dump-workflows --out workflows                              # graphs, for the record
```

- `--style`: `3d` is round one's brief (kept to reproduce it), `flat` round two's abstract
  paper-and-ink brief, `flat-trigger` the same with a flat-illustration style word in front.
- `icongen/brief.py`: the style briefs, the negative prompt, the five subjects with their plate
  and object palettes, the seeds, the reference-pass prompt.
- `icongen/graphs.py`: one pure builder per model and mode (Klein text-to-image and
  reference-latent edit; Qwen-Image-2512; Qwen-Image-Edit-2511).
- `icongen/client.py`: upload, `/prompt`, `/history` polling, `/view`, `/free`.
- Every render appends one line to `<out>/runs.jsonl`: seconds, VRAM before and peak
  (`nvidia-smi`, 0.5 s sampling), prompt, workflow sha256.
- Renders stay under `~/comfy/out/`; they are never committed. `tools/icons` plates them and
  builds the contact sheets (`tools/icons/bakeoff.sh`, `tools/icons/vary.sh`).
