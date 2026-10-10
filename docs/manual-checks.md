# Manual checks queued for the user

Things only a person at the machine can verify. While the user is away they are assumed to
work; each is ticked off with the date and what was seen. sill keeps its own queue at
sill's `docs/manual-checks.md`; shell-host's item f at 1.5 is listed there.

## Keys as actions (queued 2026-10-10)

- [ ] **Clipboard and Space chords per session**: in a quire app's edit surface and secure field,
  copy, cut and paste with the platform's own chord (Ctrl on KDE and GNOME, Command arriving as
  Super on our desktop) and confirm Ctrl+C does nothing on our desktop; on KDE change Copy in
  System Settings (kdeglobals `[Shortcuts]` `Copy=`) and confirm a restart picks it up. The
  harness checks the mapping, not a real session.
- [ ] **Wheel zoom**: hold the platform's primary modifier (Command on our desktop, Ctrl on KDE)
  over a zoomable view and scroll; a plain wheel scrolls.
- [ ] **A keycap source**: when the desktop's launcher passes keycap's `KeymapSource`, change a
  standard chord in keycap and watch an open app's menu text follow without a restart (the
  reload runs inside `Ds`'s render and has not been run live).

## Waiting on you (picks and checks, queued 2026-09-26)

- [ ] **mailo fcitx5 check**: mailo's native-only flip is committed on a branch and waits on it.
  Type Chewing (ㄋㄧˇㄏㄠˇ → 你好) into mailo's native build and confirm the preedit shows and
  commits; the mailo session has the exact steps.
- [x] **Widget picks** (settled 2026-09-27: battery percent 500, track = plate darkened, one red; World Clock follows the scheme; filled device glyphs; cards tinted by the Space [superseded 2026-10-02: neutral plates, no Space tint, design/23 §1.0]; the widget card becomes an interface) (progress page "Widgets matched to the reference"): percent weight, ring
  track, low-battery colour, dark World Clock card, filled device glyphs, Space tint default.
- [x] **Level-control look** (settled 2026-09-27: Capsule) (progress page, OSD): Capsule (recommended) / CapsuleKnob / Segments.
- [ ] **palmrest live steps**: section below.
- [x] **Spotlight settings** (settled 2026-09-27: clipboard history on, web search on, default skin tone) (launcher v2, design/22 §5 Spotlight page, all proposed): clipboard
  history on (memory only, never password-manager copies) or off; web search as the last result
  on or off (engine DuckDuckGo by default); the default emoji skin tone.
- [ ] **Launcher v2 real-input look** (progress page "Launcher v2"): Space / Cmd+Y / Right-at-end
  open the preview, arrow keys wrap in the emoji grid, holding Down through PDFs stays smooth.
- [x] **Lock decisions** (settled 2026-09-27: sill's own lock by default, 5 s grace after the screen sleeps, no lockout delay) (sill M11, done on nested): the lockout policy after failed passwords,
  `lock_grace` (how long after the screen sleeps the password is still not asked), and whether
  sill's own lock becomes the default over the borrowed locker.
- [x] **Widget follow-ups** (settled 2026-09-27: drag to move yes, dim under a window no): dim desktop widgets under a focused window (the reference
  does), drag-to-move desktop widgets.
- [ ] **A real sill login** (`dist/sill-session`): the first run of the whole desktop on real
  hardware; sill's queue lists what to look at (M7 login, M11 lock screen and polkit agent).
- [ ] **Real-mouse and real-keyboard re-runs**: every live-input result before shell-host F47
  used injected input; sill's queue lists them.

## quire

- [ ] **EditSurface IME in a real window** (v0.1.7, FINDINGS "Edit surface"): with fcitx5 on
  Chewing, run `cargo run -p ds-blitz --example edit`, click into the surface, type ㄋㄧˇㄏㄠˇ
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
- [ ] **A real file drop from a file manager** (branch `drop-and-windows`, FINDINGS "File drops
  and a second window"): `cargo run -p ds-blitz --example file_drop`, then drag two files from
  Dolphin onto the window. Expect the box's dashed outline to turn accent as the drag enters,
  solid accent and a copy cursor while over the box, a refused cursor off it, and on release over
  it "Attached 2 file(s)" with both absolute paths listed and printed. Drag a link out of a
  browser: nothing lights, the cursor refuses, nothing prints. Repeat once under X11
  (`WAYLAND_DISPLAY= cargo run …`): there the cursor may trail one move behind. No test drives
  winit's data-transfer events.
- [ ] **A second window's compositor close and raise** (same branch): `cargo run -p ds-blitz
  --example second_window`, press "Open message" twice. Close one message window with its
  title-bar button (or Alt+F4): it goes, the terminal prints its `VirtualDom dropped`, the other
  windows stay. Close the first window with a message window open: both go and the process
  exits. The autopilot run (`QUIRE_AUTOPILOT=1`) covered closes from the app's side only, and
  never looked at whether `handle.focus()` raised the window.

- **Spelling (branch `spellcheck`, design/04 section 50).** `cargo run -p ds-gallery`, Edit
  page: `noet` and `speling` carry red round dots just under the glyphs, like TextEdit's; a
  right-click on one lists suggestions, Ignore and Learn and the menu looks right at 1x and at a
  fractional scale; click into a word and type (no mark until you leave it); a Chinese or
  Japanese line typed through fcitx5 is never marked. Learn writes
  `~/.local/share/quire/spelling/en_US.dic`.

## mailo

Kept by the mailo session in its own repo.

## palmrest (Magic Mouse daemon, M8; repo ~/palmrest, FINDINGS F1–F21)

Nothing here has run on the real mouse yet; every fixture is synthesised. The Python
`magic-mouse-kde.service` stays in charge until you switch.

1. **Record real sessions** (safe, read-only): `cd ~/palmrest && cargo run --release -- record /tmp/mm.bin`, then scroll slow, fast, repeated flicks, diagonal and change finger count for about 20 s, Ctrl+C. Five such files replace the synthesised parity fixtures (F9) and verify the +y sign, the rear band side and the ~90 Hz report rate.
2. **Switch services when ready**: `~/palmrest/install.sh` disables the Python service, enables palmrest with your current feel (`SCROLL_SPEED=32`, `SCROLL_ACCEL_MAX=1`, F6) and rolls back if palmrest does not stay up. Under KDE it sets `foreign_output = "wheel"` so scroll direction does not flip (F7).
3. **Live acceptance** after switching: design/12 §12.8 items 8 (libinput sees finger scroll with an axis stop, no motion, no taps), 9 (calibrate the virtual touchpad's mm/px; at 0.25 it re-lands every ~120 px, F8), 10 (suppression over shell surfaces), 11 (latency on the device), 12 (timestamp calibration).
4. **After the sensor change (palmrest 7abdae3, F25–F29)**: use the Magic Mouse on the new build for a minute (scroll, swipe, double-tap); it should feel exactly as before, since thresholds are now in mm but convert back to the same values. Optional: `[device] source = "evdev"` in gestures.toml and compare (README "Manual checks"). Other mice and the Magic Mouse 1/USB-C are marked `TODO(untested)`; no check needed.

## focus_with_token on a real Wayland compositor (queued 2026-10-03)

- [ ] **Activation of an existing window**: once an app (mailo's handoff) calls
  `WindowHandle::focus_with_token` with the token of a D-Bus `Activate` or a notification click,
  put another window in front, click the notification, and confirm the existing window comes to
  the front and takes the keyboard. The tests drive the request against a fake compositor only;
  whether KDE, COSMIC and GNOME honour the token is this check.
