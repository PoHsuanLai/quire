# 13 BEHAVIOUR: Menus, switcher, notifications, control center, focus, launcher, sounds

Status legend: **settled** = decided with the user or fixed by the design prototypes (plan,
2026-09-23); **proposed** = chosen here, tune in the gallery, the user may override. Confidence
of the basis: **H** primary source or code, **M** reliable secondary source, **L** observed or
inferred, **UNKNOWN** nothing published. "Plan" = `~/.claude/plans/vast-toasting-peach.md`;
"C-D" = its Appendix C section D; "A1..A7" = its Appendix A (design prototype extraction, S =
mailo-spaces.html).

## 13.1 What this governs

The bar's menus and every ds `Menu` (menu tracking, submenus, item geometry), the app switcher
(Cmd+Tab), notification banners, the control center popover, focus and raise rules for our
windows, the launcher's appearance and the UI sound mapping. Bar layout and content per surface
are in `20-SURFACES.md`; the menu markup and tokens are in `04-COMPONENTS.md`; this document is
behaviour and numbers.

## 13.2 Reference behaviour (macOS)

| # | Fact | Value | Conf. | Source |
| --- | --- | --- | --- | --- |
| R1 | Menu bar height | 24 pt since Big Sur (22 before; 37 on notched laptops) | H | C-D |
| R2 | Status items | <= 22 pt; icons 16 pt; disabled 35 % opacity | H | C-D |
| R3 | Menu tracking | click toggles; press-drag-release works; hover switches between open menus instantly; opens with no animation; fades on close | L | C-D |
| R4 | Submenus | safe triangle | H | C-D |
| R5 | Submenu delay, item height (~22 pt), font 13 pt, radii | observed only | L / UNKNOWN | C-D |
| R6 | Cmd+Tab | quick tap switches without UI; show delay not published; Tab / ` step; Q quit; H hide; release switches; Esc cancels | M / L / UNKNOWN | C-D |
| R7 | Mission Control | `expose-animation-duration` default not published; Space swipes follow the trackSwipe model | UNKNOWN | C-D |
| R8 | Notifications | banners top-right; dismiss after ~5 s; alerts persist; swipe right dismisses; duration not user-changeable since Big Sur | M | C-D |
| R9 | Control Center | no published numbers | UNKNOWN | C-D |
| R10 | First click | on an inactive window only activates, unless the view opts in via `acceptsFirstMouse` | H | C-D |
| R11 | Background drag | Cmd-drag moves a background window without activating it | M | C-D |
| R12 | Spotlight / Launchpad | `springboard-show/hide/page-duration` keys exist; Spotlight position and row height not published | H / UNKNOWN | C-D |
| R13 | Sounds | CoreAudio SystemSounds: `finder/empty trash.aif`, `system/Grab.aif` (screenshot), dock poof; volume pop location not published | H / UNKNOWN | C-D |

## 13.3 Our behaviour (specification)

### 13.3.1 Bar geometry

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Bar height (`--bar-h`) | 32 px | proposed (L; plan M0 spike item a: maximised height = `output_h - 32`); macOS 24 (R1 H) |
| Exclusive zone | `Reserve(32)` | settled (plan Bar surface) |
| Title / status item hit height | 24, vertically centred | proposed |
| Title padding | 0 / 10 px; app name ui 13/700, other titles ui 13/500 | proposed (R5 13 pt, A2 menu items 13) |
| Open-title highlight | pill 24 high (`--shell-bar-item`), radius 4 (`--r-shell-bar-item`), `--f-pill` while its menu is open and `--f-pill-hover` under the pointer; no transition; ds `MenuBarItem` for text items, `IconButton{Status}` draws the same pill | settled 2026-09-24 (the macOS polish pass) |
| Title text | 13 px (`--fs-shell-bar`) at 500 (`--fw-shell-bar`), the app name 700 | settled 2026-09-24 (R5 13 pt) |
| Workspace indicator | one segmented group on the frame (ds `WorkspacePills`): a `--f-pill-hover` track 24 high, radius 4, the current workspace on `--f-pill` with `--shadow-current` | settled 2026-09-24 |
| Status item | max 22 px glyph box, Lucide glyph 16 px (`one weight, one colour`), gap 4 | proposed values (R2 H); `20-SURFACES.md` §1.1 proposes `IconSize::Bar` 22, see open decision 8. Mechanism settled (bar gaps): ds `IconButton{Status}` reads `--bar-status-box`/`--bar-status-glyph`, written by `ds::StatusMetrics` from the three settings keys |
| Disabled | opacity .35 | R2 (H) |
| Material / tint | `Material::Bar` over compositor blur, `--f-*` tokens of the workspace SpaceLook, cross-fade 380 ms | settled (plan Spaces); drawn by `Ds` (bar gaps: `data-frame="tinted"`, the gradient at the bar's tint alpha, `data-ground="frame"`) |

### 13.3.2 Menu tracking (bar menus; the same machine drives every ds `Menu`)

| Behaviour | Specification | Status |
| --- | --- | --- |
| Open | on **press** of a bar title or status item | R3 (L) |
| Release on the originating title | menu stays open (click mode) | R3 |
| Press-drag-release | while the button is held, items highlight under the pointer; release on an enabled item picks it; release on a separator, header, disabled item or outside after the pointer entered the menu closes without picking | R3 (L); settled in ds `Menu` (bar gaps: a release after a press that began outside picks; `on_hover` and `on_release` report to an external tracker) |
| Click toggles | in click mode, a press on the open menu's title closes it | R3 |
| Hover switch | while any bar menu is open, the pointer entering another bar title or status item opens that menu and closes the current one in the **same frame**, 0 ms delay, no open or close animation | settled delay (R3); no animation settled in ds `Menu` (`entrance: MenuEntrance::Instant`; the owner removing a menu is immediate) |
| Outside press | closes (xdg_popup `popup_done`); the press is not delivered to the other client | proposed (compositor grab semantics) |
| Pick | the menu closes, then the action runs (A6 "Click .it closes then picks") | settled (design); ds `Menu` calls `onpick` then `onclose` in the same handler, so both land before the next frame (bar gaps, sill Q11) |
| First open animation | `menu-pop --t-move --e-spring` (A5: y -4, s .97, fade -> 0) | settled for ds menus (A5) |
| Close animation | `fade` over `--t-quick` with `--e-exit` | settled for Escape and outside click (bar gaps: `Anim::MenuOut`, `data-presence="leaving"`, `onclose` after it settles); a ds `Popover` does the same since 2026-09-24 |
| Highlight | selected item background `--accent-soft` (A3), no transition, follows the pointer and the keyboard | settled (design colour), timing proposed |
| Keyboard | Up/Down move with **wrap** (A6); Left/Right switch to the adjacent bar menu or close/open a submenu; Enter, Space or Tab pick; Esc closes one level; typing filters (A6) | settled (A6) |
| Position (bar menus) | design `placeFloat`: `x = title.left - 8`, `y = title.bottom + 6`; flip and clamp 8 px from output edges; status menus right-aligned to `item.right + 8` | settled (A6) |
| Implementation | one reusable `xdg_popup` per bar; hover switch = `xdg_popup.reposition` + content swap, so no new grab (and no new input serial) is needed | proposed |

### 13.3.3 Menu item geometry (text menus: bar, context, dock)

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Variant | ds `Menu{Dropdown}` for bar menus (`20-SURFACES.md` §1.1), `Menu{Context}` for dock and context menus, `Menu{Slim}`; all text menus share the item metrics below | settled (coherence rule: one menu component) |
| Menu padding | 5 px | settled (A4 `.fmenu` pad 5) |
| Menu radius | `--r-menu` 12 | settled (plan tokens) |
| Material | `Material::Popover` over blur for bar, dock and tray menus (`03-COLOR.md`) | settled by name |
| Menu width | min 220 (A4 `.slim` 220), max 420, sized to content | settled min, max proposed |
| Item height | 22 px (`--shell-menu-row`, `menus.item_height_px`) | settled 2026-09-24 (the macOS polish pass; R5 ~22 pt L; was 24 proposed) |
| Item text | 13 px (`--fs-shell-menu`) at 400 | settled 2026-09-24 (R5 13 pt) |
| Selected-row highlight | inset by the 5 px panel padding, radius 6 (`--r-shell-highlight`), `--accent-soft` (Dropdown `--surface-2`) | settled 2026-09-24 |
| Item radius | `--r-item` 9 | settled token |
| Columns | check 22 px (A4 `.slim` `22px 1fr auto`); optional glyph 16 + gap 6; label `1fr`, ui 13/400, single-line truncate; shortcut `auto`, data 10 `--ink-faint` (A4 `.sc`), rendered with the ds `Shortcut` vocabulary (⌘⇧⌥⌃); submenu chevron 12 px | settled columns (A4), sizes R5 13 pt |
| Checkmark | 14 px accent check in the 22 px column (A4 "checked = 14 accent check") | settled |
| Separator | 1 px `--line-soft` hairline, 5 px margin above and below (11 px row, `--shell-menu-sep`, `menus.separator_margin_px`), inset 8 px to the text | settled 2026-09-24 (the macOS polish pass; was 4 proposed) |
| Section header | data 9.5, .14em, upper (A4 `.g`), 22 px row, not selectable | settled style, height proposed |
| Status line | an item's row (padding 6 / 8), title ui 13 / 600 `--ink`, detail 11.5 `--ink-faint`, no eyebrow, not selectable, skipped by keys | settled (bar gaps: `MenuEntry::Info`), sizes proposed |
| Disabled item | opacity .35, not selectable, skipped by arrow keys | R2 (H); settled in ds `Menu` (`MenuEntry::Item { availability }`, tray gaps Q7) |
| Rich menus | `Menu{Rich}` keeps the design's 34 px tile rows (A4 `.fmenu .it`) | settled |

### 13.3.4 Submenus

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Open delay (pointer rests on a submenu item) | 200 ms | proposed (R5 UNKNOWN) |
| Open immediately | Right arrow, Enter, or click on the item | settled (ds `Menu`, tray gaps Q7; `MenuTrackEvent::Expand`) |
| Placement | right of the parent menu, first item aligned with the parent item (`y = item.top - 5`, the panel padding: 6 for Dropdown), gap 2; flips left when it would cross the output edge - 8 | settled for ds `Menu` (through `place()`, tray gaps Q7) |
| Safe triangle | while a submenu is open and the pointer moves inside the triangle formed by the previous pointer sample and the submenu's near-edge top and bottom corners (inflated 4 px), the parent's selection does not change | R4 (H) |
| Triangle timeout | if the pointer stays inside the triangle without motion for 300 ms, the item under the pointer is selected (and its submenu, if any, follows the 200 ms rule) | proposed |
| Close | leaving the triangle onto another parent item closes the submenu immediately; Left arrow or Esc closes one level | settled (ds `Menu`, tray gaps Q7) |
| Animation | submenus open with no animation (only the first menu of a tracking session pops) | settled for ds `Menu` (`[data-depth]` has no entrance) |

### 13.3.5 App switcher (Cmd+Tab)

Bound through COSMIC's shortcuts (`system_actions` `WindowSwitcher` / `WindowSwitcherPrevious`
-> `sill switcher next|prev`; Toshy turns physical Cmd+Tab into the switcher chord). The surface
is an Overlay layer with `KeyboardMode::Exclusive`, kept warm (plan launcher pattern).

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Unit | applications (not windows), MRU order of each app's most recently activated toplevel; the current app first; apps whose toplevels are all minimized included | R6 (M) |
| Initial selection | index 1 (the previous app) | R6 |
| Show delay | 150 ms after the chord; if the modifier is released before, switch to index 1 with no UI | proposed (R6 quick tap M, delay UNKNOWN) |
| Modifier release detection | on `wl_keyboard.enter`, the modifier state tells whether the chord's modifier is still held; afterwards, key events | proposed |
| Tab / Shift+Tab | next / previous, wrap | R6 |
| ` (grave) | previous | R6 (L) |
| Left / Right | previous / next | proposed |
| Q | quit the selected app (close its toplevels); its tile leaves with `fold` | R6 |
| H | hide the selected app (minimize its toplevels) | R6 |
| Esc | cancel: close, no switch | R6 |
| Release | activate the selected app's most recent toplevel (unminimize if needed), close | R6 |
| Pointer | hover selects, click activates | proposed |
| Panel | centred on the output under the pointer; `Material::Osd` (radius 18, `03-COLOR.md`), padding 16; cells 112 px with 96 px icons, gap 8; selected cell `--f-pill`, `--r-tile` 12; app name ui 13/600 centred under the row | proposed (L) |
| Overflow | icons shrink to fit `output_w - 64`, minimum 48, then the row scrolls to keep the selection visible | proposed |
| Animation | appears with `fade --t-quick`, no pop; closes instantly on release | proposed |

### 13.3.6 Notifications

`sill` owns `org.freedesktop.Notifications` (cosmic-notifications absent, plan COSMIC findings).

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Position | top-right: right 8, top `--bar-h + 8` | R8 (M) |
| Banner | width 360, min height 64, padding 12; `Material::Toast` over blur, radius 16 (`03-COLOR.md` materials table: notification banners = Toast) | proposed (L) |
| Content | app icon 32 left; title ui 13/700; body ui 13, 2 lines clamped (`clip_chars`); time data 10 top-right | proposed |
| Enter | slide from `translateX(calc(100% + 8px))` to 0, `--t-big --e-spring` (as the design toast) | proposed (A4 toast) |
| Banner lifetime | 5000 ms; the app's `expire_timeout` is ignored except 0 (persist) | R8 (M: fixed ~5 s) |
| Alert (persistent) | urgency Critical, `resident` hint, or `expire_timeout == 0`: stays until acted on or closed | R8 (M) |
| Hover | pauses the timer (resumes with max(remaining, 1500 ms)); expands: body up to 6 lines, action buttons (ds `Button{Mini}`) appear, a close button (18 px circle) at the top-left corner; height animates `--t-move --e-out` | R8 hover expand (M), numbers proposed |
| Click | body: `ActionInvoked("default")` + close; action button: that action + close | proposed |
| Swipe right to dismiss | pointer drag: follows 1:1 to the right, left motion damped x 0.25; release dismisses if `dx >= 80 px` or velocity `>= 600 px/s`, else springs back `--t-move --e-spring`. Horizontal scroll over the banner (Magic Mouse, touchpad): same thresholds on the summed px at Ended | R8 (M), thresholds proposed |
| Exit | timeout and dismiss both slide right, `--t-move --e-exit`; banners below move up `--t-move --e-spring` | proposed |
| Stack | at most 3 banners visible, newest on top, gap 8; older ones queue | proposed |
| Grouping by app | a banner from an app whose banner is visible replaces its content and shows a count chip; two offset layers (4 px each) beneath indicate the group; the history groups by app, newest first, collapsed to the newest with "N more" | R8 grouping (M), geometry proposed |
| Do Not Disturb | no banners and no sounds except urgency Critical; everything still enters the history | proposed |
| Sound | the `sound-name` or `sound-file` hint, else `message-new-instant`; `suppress-sound` honoured | proposed |

### 13.3.7 Control center

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Anchor | a popover from its bar status item, which sits at the right end of the status area just left of the clock (as on macOS); participates in bar hover-switch (13.3.2) | settled (user, 2026-09-25) |
| Module bar items | any module can also be shown as its own bar status item (macOS "Show in Menu Bar"): a click on that item opens the module's detail pane as a popover directly, not the whole control center; the setting is per module (M5 freeze names the keys) | proposed (user direction, 2026-09-25) |
| Open / close | `menu-pop --t-move --e-spring` / `fade --t-quick --e-exit` | settled open (brief), close proposed |
| Size | width 320, height fits content, max `output_h - --bar-h - 16` then scrolls (no rubber band) | proposed (R9 UNKNOWN) |
| Grid | 2 columns, gap 8, padding 12; module tiles `--r-tile` 12; sliders span both columns | proposed |
| Modules (order) | Wi-Fi (network service), Bluetooth (bluez), Focus / Do Not Disturb, Display brightness slider, Sound volume slider + output device, Now Playing (MPRIS), Appearance (`AppearancePicker`, the one picker, plan), Battery (UPower) | proposed; derived from the plan's service and crate list, the Claude Doc spec's list wins where it differs |
| Detail | a tile's chevron opens its detail pane in place: `slide-l` / `slide-r` `--t-move --e-spring` (A5) | proposed |

### 13.3.8 Focus and raise rules

| Rule | Specification | Status |
| --- | --- | --- |
| First click (our apps) | a pointer press that activates an inactive ds-native toplevel is consumed (not delivered to Blitz) unless the element under it or an ancestor carries `data-first-mouse` (ds opts in: window drag areas, scrollbar thumbs, the traffic-light buttons). "Activating" = the surface's `xdg_toplevel` `activated` state or `wl_keyboard.enter` arrived within 100 ms before the press | R10 (H), detection proposed |
| Hover and scroll | work in inactive windows without activating them (Wayland default); our scroll engine latches normally | proposed |
| Foreign apps | toolkit and compositor behaviour; not controlled | limit |
| Cmd-drag a background window | compositor feature; cosmic-comp moves windows with Super+drag (activation behaviour is cosmic-comp's) | R11 (M); limit |
| Raise | activating an app raises one toplevel (cctk `activate`); macOS raises all of the app's windows; not reproducible | limit |
| Layer surfaces | bar and dock never take keyboard focus; their popups switch the layer to `OnDemand` until closed (plan shell-host rule); launcher and switcher use `Exclusive` | settled (plan) |

### 13.3.9 Launcher appearance (Spotlight-like)

| Value | Number | Status | Basis |
| --- | --- | --- | --- |
| Panel | fixed 680 x 460, Overlay layer, `KeyboardMode::Exclusive`, blur `Element("panel")` | settled (plan) |
| Horizontal | centred on the output under the pointer | settled (plan "centred panel") |
| Vertical | panel top = `max(--bar-h + 24, round(0.4 x output_h - 230))` (sits slightly above centre) | proposed (R12 UNKNOWN) |
| Component | ds `CommandPalette` ("the command menu is the same menu, just bigger and centred", A0 S:792) | settled |
| Radius | `--r-panel` 14, drawn as a squircle of 14 | settled (A4 cmdk r14; squircle 2026-09-24) |
| Input | ui 22 at 500 (`--fs-shell-field`, `--fw-shell-field`), search glyph 20 (`--shell-field-glyph`), padding 12 / 16, row about 56 px | settled 2026-09-24 (the macOS polish pass, Spotlight-like; was ui 16 with a 16 px glyph) |
| Result row | 34 px tile, gap 9, padding 5 / 7, radius 8; title 14/600 (`--fs-shell-row`), detail 12 faint (`--fs-shell-detail`), trailing data 10 | sizes settled 2026-09-24, geometry proposed |
| Card height | as tall as its content up to the panel (no empty box under the last row); the shell's blur region follows the card's measured rect | settled 2026-09-24 |
| Corner and shadow | `Corner::Squircle(14)`; the Sheet material's stack v2 (hairline, highlight, contact and ambient shadows, 03-COLOR §17.4) | settled 2026-09-24 |
| Section header | data 9.5 .14em upper, 24 px | settled style |
| Selection | `--accent-soft`, moves instantly with keys and hover | settled (A3) |
| Visible rows | `floor((460 - 56 - 10) / 44)` = 8 | derived |
| Open | `peek-in --t-big --e-spring` (s .95, y 12 -> 1, A5) | settled (brief, A5) |
| Close | `fade --t-quick --e-exit`; the catcher is never animated or dimmed (plan) | settled catcher, fade proposed |
| Open latency | p95 < 100 ms over 20 toggles (`accept-launcher.sh`) | settled (plan) |

### 13.3.10 UI sounds

Played from the freedesktop sound theme (`sound.theme`, default `freedesktop`) by
`sill-services` `sound` (libpulse-binding, already in the pinned block: samples uploaded once,
`play_sample` for low latency).

| Event | macOS | Our sound name (freedesktop naming spec) | Status |
| --- | --- | --- | --- |
| Screenshot taken | `system/Grab.aif` (H) | `screen-capture` | proposed |
| Empty Trash | `finder/empty trash.aif` (H) | `trash-empty` | proposed |
| Volume step | volume pop (location UNKNOWN) | `audio-volume-change`, once per key press (not on repeat), after the level changes, not when muted | proposed |
| Dock item removed | dock poof (H) | `item-deleted` | proposed |
| Notification banner | app-defined | hint sound, else `message-new-instant` | proposed |
| Critical alert | | `dialog-warning` | proposed |
| Invalid key in our apps | alert beep | `bell` | proposed |
| Settings | | `sound.ui_sounds = On | Off` (default On), `sound.volume_feedback = On | Off` (default On) | proposed |

### 13.3.11 Window frame (our client-decorated windows; settled 2026-09-25)

The frame quire draws for a window that asks for no server decorations
(`Ds { window: WindowFrame::Titlebar }`, design/04 "Window frame"). It acts through the host seam
`ds::HostWindow`: ds-native over winit, sill over shell-host's `SurfaceHandle`.

| Behaviour | Specification | Status |
| --- | --- | --- |
| Move | a primary press on the titlebar's empty area, then travel of more than `window.move_threshold_px` (4) on either axis: `begin_move()` once, while the button is still down (Wayland's `xdg_toplevel.move` takes the press's serial). Not on a light; not while maximized or fullscreen | settled |
| Double-click the titlebar | `zoom(Toggle)`: maximized restores, anything else maximizes (macOS's "double-click a window's title bar to zoom", its default) | settled |
| Resize | a primary press on an edge zone (4 px sides, 12 px corners): `begin_resize(edge)` at once. No zones while maximized or fullscreen | settled |
| Green light | a click is `zoom(Toggle)`; its mark is "restore" while maximized | settled |
| Tiling menu | held `window.tile_menu_press_ms` (500), rested on `window.tile_menu_hover_ms` (800 = the 450 ms hover intent + 350), right-click, or ArrowDown on the focused light: Move & Resize with Fill (= maximize), Left half, Right half, Centre; a placement the host reports `Support::No` for is unavailable. The hold that opened it does not also zoom | settled shape (macOS Sequoia), delays proposed |
| First click | the titlebar, the lights and the edge zones carry `data-first-mouse` (13.3.8) | settled |
| Keyboard | Tab reaches close, minimize, zoom; Enter or Space presses; Escape closes the menu | settled |
| Placement on Wayland | Left half, Right half and Centre are unavailable: a toplevel can neither read nor set its position (FINDINGS "Window frame") | limit |

## 13.4 State machines

Pure; `now` is an argument; effects are returned.

Menu tracking: **settled** (implemented in quire `crates/ds/src/overlay/menu_track.rs`,
2026-09-24, sill gap Q1; the table below is its test list). The shipped shape differs from the
first sketch in names only: generic over the caller's menu key, timings carried in the machine.

```rust
// quire: crates/ds/src/overlay/menu_track.rs  (drives bar menus and every ds Menu)
pub enum Held { Held, Released }
pub enum Entered { Entered, NotYet }
pub struct MenuTrack<K> { pub timing: MenuTiming, pub phase: MenuPhase<K> } // timing from menus.*
pub enum MenuPhase<K> { Closed, Tracking(Session<K>) }
pub struct Session<K> { menu: K, held: Held, entered: Entered, hot: Option<ItemPath>,
                        sub: Submenu, pointer: Point, under: MenuTarget<K> }
pub enum Submenu {
    None,
    Pending { item: ItemPath, since: Instant },                    // 200 ms
    Open    { item: ItemPath, guard: Option<SafeTriangle> },
}
pub struct SafeTriangle { from: Point, top: Point, bottom: Point, still_since: Instant }
pub enum MenuTrackEvent<K> { PressTitle(K), Release(MenuTarget<K>), Move(Point, MenuTarget<K>),
                             Key(MenuKey), OutsidePress, SubPlaced { top: Point, bottom: Point }, Tick }
pub enum MenuTrackEffect<K> { Open(K, MenuAnim), Switch(K), Close(MenuAnim), OpenSub(ItemPath),
                              CloseSub, Highlight(Option<ItemPath>), Pick(ItemPath),
                              Adjacent(MenuDirection), RequestTick(Instant) }
impl<K: Clone + PartialEq> MenuTrack<K> {
    pub fn step(self, ev: MenuTrackEvent<K>, now: Instant) -> (Self, Vec<MenuTrackEffect<K>>);
}
```

| From | Event | To | Effects |
| --- | --- | --- | --- |
| Closed | PressTitle(a) | Tracking{a, Held, NotYet} | Open(a, MenuPop) |
| Tracking{held: Held} | Release on title a | Tracking{Released} (click mode) | |
| Tracking{held: Held} | Release on enabled item | Closed | Close(Fade), Pick |
| Tracking{held: Held, entered: Entered} | Release elsewhere | Closed | Close(Fade) |
| Tracking{held: Held, entered: NotYet} | Release outside menu and title | Closed | Close(Fade) |
| Tracking{Released} | PressTitle(a) (same) | Closed | Close(Fade) |
| Tracking{Released} | Release on enabled item | Closed | Close(Fade), Pick |
| Tracking | Move onto title b != a | Tracking{b, same held} | Switch(b) (no animation) |
| Tracking | Move onto item i (outside any Guard) | hot = i; if i has a submenu: Sub::Pending{i, now} | Highlight(i) |
| Tracking{sub: Open{guard}} | Move inside the triangle | unchanged, `guard.from` = this point | |
| Sub::Pending | Tick, `now - since >= 200 ms` | Sub::Open | OpenSub |
| Sub::Open{guard} | Tick, `now - still_since >= 300 ms` | re-evaluate hot under pointer | |
| Tracking | Key Esc | one level up / Closed | CloseSub / Close(Fade) |
| Tracking | OutsidePress | Closed | Close(Fade) |

```rust
// sill: surfaces/switcher/machine.rs
pub enum Switcher {
    Hidden,
    Armed   { since: Instant, sel: usize },     // chord seen, UI not shown (150 ms)
    Shown   { sel: usize },
}
pub enum SwIn { Chord(Dir), ModifierReleased, Key(SwKey), Hover(usize), Click(usize), Tick }
pub enum SwOut { Show, Hide, Select(usize), Activate(AppId), Quit(AppId), HideApp(AppId) }
pub fn step(s: Switcher, ev: SwIn, now: Instant, apps: &[AppId]) -> (Switcher, Vec<SwOut>);
// Armed + ModifierReleased -> Hidden + Activate(apps[sel]); Armed + Tick >= 150 ms -> Shown + Show;
// Shown + ModifierReleased -> Hidden + Activate; Esc -> Hidden (no Activate); Q/H act on sel and stay.

// sill: surfaces/notifications/banner.rs
pub enum Banner {
    Entering { since: Instant },
    Shown    { deadline: Deadline },              // Deadline::At(Instant) | Deadline::Never
    Hovered  { remaining: Duration },
    Dragging { dx: Px, samples: Samples },
    Leaving  { since: Instant, why: Leave },       // Timeout | Dismissed | Acted
}
pub fn step(b: Banner, ev: BannerIn, now: Instant) -> (Banner, Vec<BannerOut>);
```

## 13.5 Formulas

```
safe triangle      point q is guarded iff q is inside triangle(p_prev, A - (0,4), B + (0,4)),
                   A, B = submenu near-edge top and bottom corners; barycentric sign test
submenu open       now - rest_since >= 200 ms
switcher           show iff modifier held at t_chord + 150 ms
banner lifetime    deadline = t_shown + 5000 ms; on hover leave: deadline = now + max(remaining, 1500 ms)
banner swipe       x(dx) = dx if dx >= 0 else 0.25 dx ; dismiss iff dx >= 80 or v >= 600 px/s
launcher top       max(bar_h + 24, round(0.4 H - 230))
first mouse        activating iff (t_press - t_activated) <= 100 ms
```

## 13.6 Configuration

| Key | Type | Default | Status |
| --- | --- | --- | --- |
| `menus.submenu_delay_ms` | ms | 200 (0..1000) | proposed |
| `switcher.show_delay_ms` | ms | 150 (0..500) | proposed |
| `window.move_threshold_px` | px | 4 (1..16) | proposed |
| `window.tile_menu_press_ms` | ms | 500 (200..2000) | proposed |
| `window.tile_menu_hover_ms` | ms | 800 (450..3000) | proposed |
| `notifications.dnd` | `On | Off` | `Off` | proposed |
| `notifications.banner_style` per app | `Banner | Alert | None` | `Banner` | R8 (M: macOS per-app style) |
| `sound.theme` | name | `freedesktop` | proposed |
| `sound.ui_sounds` | `On | Off` | `On` | proposed |
| `sound.volume_feedback` | `On | Off` | `On` | proposed |

Banner duration is not a setting (R8: not user-changeable since Big Sur).

## 13.7 Integration

| Concern | Owner |
| --- | --- |
| Menu tracking machine, safe triangle, `Menu` component | quire `overlay/menu_track.rs` (settled, shipped), `components/menu.rs` |
| Popup surfaces, reposition, grabs | shell-host `popup` (`PopupConfig`, `xdg_positioner`, `xdg_popup.reposition`) |
| Bar, switcher, notifications, control center, launcher surfaces | sill `surfaces/{bar,switcher,notifications,control_center,launcher}` |
| Notification server | sill-services (pattern: cosmic-notifications `subscriptions/notifications.rs`) |
| Toplevel activation / close / minimize | sill-services `cosmic_wl` (cctk) |
| Shortcuts | `dist/cosmic/.../system_actions` (Launcher, WindowSwitcher) |
| Sounds | sill-services `sound` (libpulse-binding) |
| First-mouse filter | shell-host `input/pointer.rs` (activation-time tracking) + ds `data-first-mouse` |

Platform limits:

- `xdg_popup.grab` needs the serial of a recent user action; hover switch therefore reuses one
  popup with `reposition` (xdg_wm_base v3+). If a compositor lacks reposition, the fallback
  is close + reopen with the last button serial, which the compositor may refuse (then the
  menu closes and the next press opens the other one).
- Nested cosmic-comp does not grab the keyboard (cosmic-comp#9): KDE eats Super and Cmd+Tab
  chords; switcher and launcher are tested through `sill` CLI/IPC in the nested session.
- cosmic-comp #2230: a popup or layer under a still pointer gets no enter; menus open by press
  so this affects only the first hover highlight.
- No foreign-toplevel protocol on KWin: switcher and window lists are empty there.
- No client placement on Wayland: `xdg_toplevel` has no position request, so our windows'
  Left half, Right half and Centre are unavailable there (neither cosmic-comp's
  `zcosmic_toplevel_manager_v1` nor KWin's `org_kde_plasma_window` sets a geometry either;
  FINDINGS "Window frame").
- Toshy's Cmd mapping on app_id-less layer surfaces is unverified (plan shell risks); the
  switcher accepts both Alt+Tab and Super+Tab chords.

## 13.8 Acceptance tests

Driven by `ds-native::Harness` (pure menus) and `sill debug` in nested cosmic-comp.

1. **Open on press**: a press on a bar title shows its menu on the next frame; release on the
   title leaves it open; a second press on the title closes it.
2. **Press-drag-release**: press title, move to item 3, release: item 3's action runs once and
   the menu closes; release on a separator: nothing runs, the menu closes.
3. **Hover switch**: with menu A open, moving onto title B shows B and hides A in the same
   frame (0 frames where both or neither are visible), no `menu-pop` on B.
4. **Submenu delay**: resting on a submenu item opens the submenu at 200 ms +- 1 frame, not at
   190 ms; Right arrow opens it immediately.
5. **Safe triangle**: with a submenu open, a diagonal path from the parent item toward the
   submenu that crosses two other parent items keeps the submenu open and the parent selection
   unchanged; stopping inside the triangle for 300 ms selects the item under the pointer.
6. **Keyboard**: Down from the last item wraps to the first; disabled items are skipped; Esc
   closes a submenu then the menu; typing filters.
7. **Geometry snapshot**: headless render of a bar menu with check, glyph, shortcut, separator,
   header, disabled and submenu items: item height 24, separator row 9, width >= 220, disabled
   opacity .35.
8. **Switcher quick tap**: chord and modifier release within 100 ms: the previous app is
   activated and no switcher frame is ever presented. Held 300 ms: the panel appears at 150 ms
   +- 1 frame with index 1 selected; Tab twice then release: index 3 activated; Esc: nothing
   activated; Q: the selected app's toplevels receive close.
9. **Banner lifetime**: a notification with `expire_timeout = -1` leaves at 5000 ms +- 1 frame;
   hovered at 4000 ms for 3 s: still present, leaves 1500 ms after the pointer leaves;
   `expire_timeout = 0` and urgency Critical: present after 60 s.
10. **Swipe dismiss**: drag 90 px right and release: `NotificationClosed(reason 2)`; drag 50 px
    slowly: springs back; horizontal scroll of 100 px on the banner: dismissed.
11. **Grouping**: 3 notifications from one app within 2 s: one banner card with count 3; 5 from
    5 apps: 3 visible cards, newest on top.
12. **First mouse**: clicking a button in an inactive ds-native window activates the window and
    the button's handler does not run; a second click runs it; a `data-first-mouse` element
    runs on the first click.
13. **Launcher**: opens with `peek-in` settling at 420 ms (`settle(PeekIn)`), rows 44 px, 8
    rows visible, p95 open < 100 ms over 20 toggles.
14. **Sounds**: screenshot, Empty Trash, volume key, dock remove each trigger exactly one
    `play_sample` of the mapped name; none when `sound.ui_sounds = Off` (volume feedback follows
    its own key).

## 13.9 Open decisions

1. Menu item height: 24 (macOS-like, proposed) vs the design's Slim 30. Whichever wins applies
   to every text menu (coherence rule).
2. Bar height: 32 (plan spike) vs 24-28 (macOS 24).
3. Pick feedback: macOS blinks the chosen item once before closing; the design closes then
   picks with no blink.
4. Control center module list against the Claude Doc spec (rev 31) list.
5. Launcher vertical position formula (Spotlight sits high; the plan says centred).
6. Switcher icon size 96 vs macOS ~128.
7. Notification banner material: `Toast` (03-COLOR proposal, used here) vs inverse ink/paper like the mail toast (03-COLOR open decision 12).
8. Bar glyph size: 16 (macOS status icons, used here) vs `IconSize::Bar` 22 (`20-SURFACES.md` §1.1).

## 13.10 Sources

- Plan Appendix C-D (all R rows), "Design: sill" (Bar, Launcher, notifications server,
  shortcuts), "Design: shell-host" (popups, keyboard modes), "Findings: COSMIC" (shortcuts,
  cosmic-notifications, cosmic-comp #2230, #9), M0 spike item a.
- Sibling docs: `03-COLOR.md` materials, `05-MOTION.md` (`menu-pop`, `peek-in`, `--d-fly`),
  `20-SURFACES.md` §1.1, `01-LAYOUT.md` §13.1 (bar height open decision).
- Plan Appendix A: A0 (principles, S:792), A2 (type sizes), A3 (colour usage), A4 (`.fmenu`,
  `.slim`, cmdk, toast), A5 (keyframes and assignments), A6 (menus, keys, Esc).
- Apple Human Interface Guidelines (menus, menu bar), Apple `NSView.acceptsFirstMouse(for:)`,
  freedesktop Desktop Notifications Specification 1.2, freedesktop Sound Naming Specification,
  Wayland `xdg_popup` (grab, reposition) (as cited in C-D and plan).
