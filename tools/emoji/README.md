# emojitool

The animated emoji pipeline (design/25-EMOJI.md section 4): fetch Noto Animated Emoji
(CC BY 4.0), find each loop's rest pose, resample, and pack each emoji into 128 px and 256 px
sprite sheets plus `manifest.json`. Never run by cargo; its output is committed.

```bash
cd tools/emoji
uv run emojitool build --out ../../crates/ds/assets/emoji          # the shipped set
uv run emojitool build --out /tmp/trial --only wink --step 66       # a trial, no manifest
```

- `emojitool/curated.py`: the set, in `ds::EmojiId::ALL`'s order (keep the two in step).
- `emojitool/fetch.py`: downloads into `.cache/` (gitignored) once; later runs read the cache.
- `emojitool/frames.py`: WebP decode with ANMF durations, the rest pose, rotation, resampling.
- `emojitool/sheet.py`: the 8-wide grid, the shared palette, PNG output.
- Options: `--step` ms between samples (80), `--sizes` (128 256), `--colours` (256; 0 keeps
  RGBA). The total must stay near 8 MB (design/25 section 6).
