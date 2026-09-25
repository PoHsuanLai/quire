# Manual checks queued for the user

Things only a person at the machine can verify. While the user is away they are assumed to
work; each is ticked off with the date and what was seen. sill keeps its own queue at
`sill/docs/manual-checks.md`; shell-host's item f at 1.5 is listed there.

## quire

- [ ] **EditSurface IME in a real window** (v0.1.7, FINDINGS "Edit surface"): with fcitx5 on
  Chewing, run `cargo run -p ds-native --example edit`, click into the surface, type ㄋㄧˇㄏㄠˇ
  and press Enter. Expect the printed inputs `Composition(Start)`, `Update("ㄋ")` …,
  `Update("")`, `End("你好")`, in that order, and no `Key`/`Text` while the preedit shows.
  No automated test drives winit's IME; re-check after every toolchain bump.
- [ ] **Clipboard HTML paste** (v0.1.6/7): copy a formatted snippet from a browser, paste into
  the same example window. Expect `Paste(Html { html, text })`, not `Paste(Text)`. On Wayland
  the read goes through XWayland like blitz-shell's text clipboard.
- [ ] **Pixel snapping on screen at 2.0 and, once a 1.5 output exists, at 1.5** (v0.1.4):
  `cargo run -p ds-gallery --release` (the `window` path), Controls page: the card borders,
  menu separators and the 16 px glyph strokes should be single crisp device rows; compare with
  the headless measurements in FINDINGS "Pixel snapping".
- [ ] **Password/Secret fields**: in the gallery's Fields page, type into the Secret field:
  dots only, and the caret follows; Ctrl+A/Ctrl+C then paste into a Text field pastes the
  secret (the clipboard is the app's own; documented).

## mailo

Kept by the mailo session in its own repo.
