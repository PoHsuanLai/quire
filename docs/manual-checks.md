# Manual checks queued for the user

Things only a person at the machine can verify. While the user is away they are assumed to
work; each is ticked off with the date and what was seen. sill keeps its own queue at
`sill/docs/manual-checks.md`; shell-host's item f at 1.5 is listed there.

## Waiting on you (picks and checks, queued 2026-09-26)

- [ ] **mailo fcitx5 check**: mailo's native-only flip is committed on a branch and waits on it.
  Type Chewing (ㄋㄧˇㄏㄠˇ → 你好) into mailo's native build and confirm the preedit shows and
  commits; the mailo session has the exact steps.
- [ ] **Widget picks** (progress page "Widgets matched to the reference"): percent weight, ring
  track, low-battery colour, dark World Clock card, filled device glyphs, Space tint default.
- [ ] **Level-control look** (progress page, OSD): Capsule (recommended) / CapsuleKnob / Segments.
- [ ] **palmrest live steps**: section below.
- [ ] **A real sill login** (`dist/sill-session`): the first run of the whole desktop on real
  hardware; sill's queue lists what to look at (M7 login, M11 lock screen and polkit agent).
- [ ] **Real-mouse and real-keyboard re-runs**: every live-input result before shell-host F47
  used injected input; sill's queue lists them.

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

## palmrest (Magic Mouse daemon, M8; repo ~/palmrest, FINDINGS F1–F21)

Nothing here has run on the real mouse yet; every fixture is synthesised. The Python
`magic-mouse-kde.service` stays in charge until you switch.

1. **Record real sessions** (safe, read-only): `cd ~/palmrest && cargo run --release -- record /tmp/mm.bin`, then scroll slow, fast, repeated flicks, diagonal and change finger count for about 20 s, Ctrl+C. Five such files replace the synthesised parity fixtures (F9) and verify the +y sign, the rear band side and the ~90 Hz report rate.
2. **Switch services when ready**: `~/palmrest/install.sh` disables the Python service, enables palmrest with your current feel (`SCROLL_SPEED=32`, `SCROLL_ACCEL_MAX=1`, F6) and rolls back if palmrest does not stay up. Under KDE it sets `foreign_output = "wheel"` so scroll direction does not flip (F7).
3. **Live acceptance** after switching: design/12 §12.8 items 8 (libinput sees finger scroll with an axis stop, no motion, no taps), 9 (calibrate the virtual touchpad's mm/px; at 0.25 it re-lands every ~120 px, F8), 10 (suppression over shell surfaces), 11 (latency on the device), 12 (timestamp calibration).
