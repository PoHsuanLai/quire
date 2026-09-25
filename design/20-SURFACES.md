# 20 Surfaces

One section per shell surface and per app: what it is built from. Every row names a
Material (`ds::Material`), components (04-COMPONENTS), motion (05-MOTION token and keyframe
names), behaviours (06, 10-13), keyboard, blur and input regions, and the milestone.

Status: **S** = settled (PLAN, SPEC or prototypes), **P** = proposed.
Material enum (PLAN "Design: `<ds>`"): `Window, Bar, Dock, Popover, Sheet, Toast, Osd, Widget`.
Layer names are wlr-layer-shell layers; `KeyboardMode` and `RegionSpec`/`BlurSpec` are
shell-host types (PLAN "Design: `shell-host`"). Milestones are SPEC "Milestones" 0-13.
Chrome that sits on a Space tint uses the `--f-*` frame tokens (21-SPACES).

## 1. Shell surfaces

### 1.1 Bar (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | layer-shell `Top`, one per output, anchor `TOP\|LEFT\|RIGHT`, `ExclusiveZone::Reserve(h)` | S |
| Height | height token (value in 01-LAYOUT) | S |
| Material | `Bar`, tinted by the workspace SpaceLook (`--f-*`); `Ds` draws the gradient at the bar's tint and the frame ground (bar gaps) | S |
| Content | left: app name, workspace indicator (drag reorder); right: tray, volume, network, battery, clock | S |
| Components | `IconButton{Status}` per status item (settled, bar gaps: box and glyph from `bar.status_*` through `StatusMetrics`), `MenuBarItem` around the app name, titles and clock (the 4 px hover and open pill, 13/500 text; settled 2026-09-24), `WorkspacePills` for the workspace indicator (one segmented group on the frame; settled 2026-09-24), `MenuEntry::Info` for status lines, `Glyph` (`IconSize::Bar` 22, P), `Menu{Dropdown}` + `MenuEntry` for every menu (22 px rows, 13 px text), `Count`, `Tooltip{Fly}` (12 px) | S / P |
| Motion | menus `menu-pop` `--t-move` `--e-spring`; hover bg `--t-quick` `--e-out`; press `--squish` `--t-tap`; tint cross-fade `--t-scene` 380 ms (21-SPACES §5) | S |
| Behaviours | 13-BEHAVIOUR-menus-windows (menu bar, menus: open delay, safe triangle), 06 (menus, Escape), 12 (workspace swipe updates indicator) | S |
| Keyboard | `KeyboardMode::None`; a grabbing popup switches it to `OnDemand` until close | S |
| Blur / input | blur `Whole`; input `Whole` | S |
| Popups | `open_popup(anchor Element(btn), FitContent, grab Seat)` | S |
| Milestone | M1 | S |

### 1.2 Dock (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Top`, anchor `BOTTOM`, height = base x max magnification + padding, `Reserve(base + margin)` | S |
| Material | `Dock`, tinted by SpaceLook; drawn by `Ds` like the bar (bar gaps) | S |
| Components | app tiles (08-ICONS plate; `IconView { plate }` as the placeholder), `RunningDot`, optional `DockFloor`, `Count` badge (LauncherEntry), progress ring, `Tooltip{Fly}` hover label, `Menu{Context}` (windows, desktop actions, Keep in Dock, Quit), `Popover` for folder stacks; geometry from `DockMetrics` (48 tiles, 8 gaps, 6 padding; settled 2026-09-24) and the pill a `Corner::Squircle` | S / P |
| Motion | magnification: no easing while tracking, ~200 ms shrink on leave (10); launch bounce and attention bounce (10); badge change `bump`; label `hc-in`/`hc-out` P; menu `menu-pop` | S |
| Behaviours | 10-BEHAVIOUR-dock (Plank parabola, 48 → up to 96, neighbours slide apart, click/right-click, stacks, drag-out, badges); auto-hide off by default (0.2 s delay, ~0.5 s slide when on) | S |
| Keyboard | `None` | S |
| Blur / input | blur `Element("dock-pill")`; input `Element("dock-hit")`; both follow the pill during magnification | S |
| Milestone | M3 | S |

### 1.3 Launcher (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | two surfaces kept warm: catcher (`Overlay`, anchor all, painted once, never animated) + panel (`Overlay`, centred, ~680 x 460) | S |
| Material | panel `Popover` with SpaceLook tint on its chrome, result list on `--raise` | P |
| Components | `CommandPalette<T>` (`host: Surface`, sized to its content, the shell scale's 22 px field, `corner: Squircle(14)`; settled 2026-09-24), `SearchField`, `ListRow`, `Kbd`, `SectionHeader` (groups), `Menu{Rich}` for actions (Ctrl+K / Alt+K), `Chip{Accent}` operator tokens | S |
| Motion | panel `peek-in` `--t-big` `--e-spring` (S uses peek-in for the command menu; C `cmdk-in`); rows `rise` stagger on first show only, cap 12; close `fade` `--t-quick` P | S / P |
| Behaviours | 06 (command menu search, up/down clamped, Enter, Esc, Tab cycles providers), 13 (Spotlight-style launcher appearance); open p95 < 100 ms | S |
| Keyboard | panel `Exclusive`; closes on Esc, `Focus::Lost`, catcher click, `launcher toggle` | S |
| Blur / input | panel blur `Element("panel")`; catcher blur none, input `Whole` | S |
| Milestone | M4 (v1), M9 (v2 providers) | S |

### 1.4 Wallpaper (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Background`, one per output, `ExclusiveZone::Ignore` | S |
| Material | none (opaque image) | S |
| Components | none (one image element) | S |
| Motion | light/dark cross-fade `--t-big`; renders only on change | S |
| Relation to Spaces | independent of SpaceLook (21-SPACES §8) | P |
| Keyboard | `None` | S |
| Blur / input | blur none; input `Empty` | S |
| Cache | `$XDG_CACHE_HOME/sill/wallpaper/<hash>-<w>x<h>.png`, Lanczos3 | S |
| Milestone | M2 | S |

### 1.5 Control center (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | xdg popup from the bar | P |
| Material | `Popover` | P |
| Content | Wi-Fi, Bluetooth, volume, brightness, Do Not Disturb, power profile, Now Playing (MPRIS), BT device batteries, focus modes | S |
| Components | `Toggle`, `Slider`, `SegmentedControl` (power profile), `ListRow` (networks, devices), `AppearancePicker` (the one picker), `SectionHeader`, `Button{Mini}` | S |
| Motion | open `menu-pop` `--t-move` `--e-spring`; toggle knob spring `--t-quick` `--e-spring`; sub-page slide `slide-r`/`slide-l` P | S / P |
| Behaviours | 13 (control center), 06 (Escape, menus) | S |
| Keyboard | popup grab → bar `OnDemand` | S |
| Blur / input | blur and input = popup surface `Whole` | P |
| Milestone | M5 | S |

### 1.6 Notifications (toasts) and notification center (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | toasts: `Overlay`, anchor `TOP\|RIGHT` under the bar; center: `Overlay` panel anchored right, or xdg popup from the clock | P |
| Material | toasts `Toast`; center `Popover` | P |
| Server | `org.freedesktop.Notifications` (cosmic-notifications absent) | S |
| Components | toast card (`HoverTarget` for actions, `Button{Mini}` actions, app icon 08 plate at 20 P), `ToastHost` stack; center: `ListRow` groups by app, `SectionHeader`, `Button{Quiet}` Clear | S / P |
| Motion | toast in `slide-l` `--t-big` `--e-spring` P; out `fade` with `--e-exit` P; stack closes gap with `heal` 18 ms/row; center rows `rise` on first show, dismiss `fold` `--t-big` `--e-exit` | P |
| Timers | hold 5200 ms (`ToastHold`), paused while hovered P | S / P |
| Behaviours | 13 (notifications), 06 (hover intent 450/150/400) | S |
| Keyboard | toasts `None`; center `OnDemand` | P |
| Blur / input | per toast `Element("toast-<id>")`; input same | P |
| Milestone | M6 | S |

### 1.7 OSD (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, anchor `TOP\|RIGHT` under the bar's reserve (`osd.position = TopRight`, the default, as current macOS shows its volume and brightness panel), `osd.margin_px` from the bar's reserve; `BottomCentre` keeps the old bottom-centred placement above the dock; no exclusive zone | S (user, 2026-09-25) |
| Material | `Osd` | S |
| Components | `Osd` (owns the card and its presence): `Glyph` (volume/brightness at `Bar` 22), a title line, level bar (`Slider { mode: Level }`: no thumb, not focusable, fill transition); a small panel styled like a control-center slider module (§1.5 grid metrics, `--r-tile`) | S |
| Motion | in `Anim::OsdIn` (a short drop-and-fade from above at TopRight; a rise at BottomCentre; the direction follows the anchor) `--t-quick` `--e-out`; level change transition `--t-quick` `--e-out`; hold `osd.hold_ms` (1500) then `Anim::OsdOut` `--t-move` `--e-exit`, the host unmaps at settle | S |
| Behaviours | 13 (UI sounds: volume pop from freedesktop sound theme) | S |
| Keyboard / blur / input | `None`; blur `Element("osd")`; input `Empty` | P |
| Milestone | M6 | S |

### 1.8 Power menu (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, full-output scrim + centred panel | P |
| Material | `Sheet` over `Scrim` | P |
| Content | Log out, Restart, Shut down, Suspend (logind-zbus) | S |
| Components | `Scrim`, `Sheet`, `Button{Primary, Secondary, Danger}`, `Kbd` hints | P |
| Motion | panel `peek-in` `--t-big` `--e-spring`; scrim `fade` `--t-move` | P |
| Behaviours | 06 (Escape, focus ring), 13 (focus/raise) | S |
| Keyboard | `Exclusive`; arrows move, Enter confirms, Esc closes | P |
| Blur / input | blur `Element("panel")`; input `Whole` | P |
| Milestone | M5 P (launcher system commands already exist at M4) | P |

### 1.9 Lock screen (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `ext-session-lock-v1` lock surface per output (not layer-shell) | S |
| Material | `Window` (opaque); background = pre-blurred wallpaper crop | P |
| Components | clock (display face), `Avatar`, `TextInput{Boxed}` password, `Button{Primary}` | P |
| Motion | password error `shake-x` 420 ms `(.36,.07,.19,.97)` once; unlock `fade` `--t-move` `--e-exit` | S / P |
| Behaviours | lock before sleep on logind `PrepareForSleep` (SPEC); 06 (Escape clears field) | S |
| Keyboard | lock surfaces always receive keyboard | S |
| Milestone | borrowed (swaylock/hyprlock) at M7; own at M11 | S |

### 1.10 Polkit prompt (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, scrim + centred panel | P |
| Material | `Sheet` over `Scrim` | P |
| Components | `Avatar`, `TextInput{Boxed}`, `Button{Primary, Secondary}`, details as `HoverCard` P | P |
| Motion | `peek-in` in; wrong password `shake-x` once | P |
| Keyboard | `Exclusive` | P |
| Blur / input | blur `Element("panel")`; input `Whole` | P |
| Milestone | borrowed (lxqt-policykit / polkit-gnome) at M7; own at M11 | S |

### 1.11 App switcher (SPEC Tier 2)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, centred; check what cosmic-comp provides first (SPEC) | S |
| Material | `Osd` | P |
| Components | app tiles (08 plate at 64 P), `Tooltip{Fly}` name under selection | P |
| Motion | show after 150 ms hold P, `fade` `--t-quick`; selection ring transition `--t-quick` `--e-spring` | P |
| Behaviours | 13 (Cmd+Tab: apps not windows) | S |
| Keyboard | `Exclusive` while shown (must see modifier release) | P |
| Milestone | M11 | S |

### 1.12 Calendar popup (SPEC Tier 2)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | xdg popup from the bar clock | P |
| Material | `Popover` | P |
| Components | month grid (new ds component `MonthGrid`, add to 04 first), `ListRow` events, `IconButton{Tool}` prev/next | P |
| Motion | `menu-pop`; month change `slide-l`/`slide-r` | P |
| Data | Calendar app (CalDAV) feeds it (SPEC) | S |
| Milestone | M10 P (Tier 2) | P |

### 1.13 Screenshot thumbnail (SPEC integrated experience, priority pick)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, anchor `BOTTOM\|RIGHT` | P |
| Material | `Toast` | P |
| Components | thumbnail image, `HoverTarget` actions (Mark up, Copy text via OCR, Delete), drag source | P |
| Motion | in `rise` `--t-big` `--e-spring` P; auto-dismiss `slide-r` out `--e-exit` after 5200 ms (`ToastHold`) P | P |
| Behaviours | 06 (drag), 13 (UI sounds: screenshot grab) | S |
| Keyboard / blur / input | `None`; blur and input `Element("thumb")` | P |
| Milestone | M10 | S |

### 1.14 Desktop widgets (SPEC Tier 2)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Bottom` layer (above wallpaper, below windows), one surface per widget P | P |
| Material | `Widget` | S |
| Content | calendar, weather, battery (SPEC) | S |
| Components | `SectionHeader`, `Count`, `ListRow` | P |
| Motion | none in steady state (nothing loops); value change `bump` | S |
| Keyboard / blur / input | `None`; blur `Element("widget")`; input `Element("widget")` | P |
| Milestone | M10 | S |

### 1.15 Quick note (SPEC Notes)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | layer-shell `Top`, `KeyboardMode::OnDemand` (SPEC: normal windows cannot stay on top) | S |
| Material | `Window` (paper: apps stay on paper, 21-SPACES §7) | P |
| Components | editor (Notes' block editor), `CommandPalette` on Cmd+K, `SearchField`, `IconButton{Tool}` | S / P |
| Motion | show `page-in` `--t-big` `--e-spring`; hide `fade` `--t-quick` `--e-exit` | P |
| Behaviours | toggle on shortcut, Esc hides, never loses text (SPEC); 06 (Escape, command menu) | S |
| Blur / input | none; input `Whole` | P |
| Milestone | M12 with Notes P | P |

### 1.16 Hot corners (SPEC integrated experience)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, one tiny surface per enabled corner, anchor corner, 2 x 2 px P | S / P |
| Material | none (fully transparent) | S |
| Motion | none | S |
| Behaviours | 13 (dwell time and action table) | P |
| Keyboard / blur / input | `None`; blur none; input `Whole` | P |
| Milestone | M10 | S |

### 1.17 Surface summary

| Surface | Layer | Material | Keyboard | Blur | M |
| --- | --- | --- | --- | --- | --- |
| Bar | Top | Bar | None | Whole | 1 |
| Wallpaper | Background | none | None | none | 2 |
| Dock | Top | Dock | None | dock-pill | 3 |
| Launcher | Overlay x2 | Popover | Exclusive | panel | 4 |
| Control center | popup | Popover | OnDemand | Whole | 5 |
| Power menu | Overlay | Sheet | Exclusive | panel | 5 P |
| Notifications | Overlay | Toast / Popover | None / OnDemand | per toast | 6 |
| OSD | Overlay | Osd | None | osd | 6 |
| Lock | session-lock | Window | always | none | 7 / 11 |
| Polkit | Overlay | Sheet | Exclusive | panel | 7 / 11 |
| App switcher | Overlay | Osd | Exclusive | whole | 11 |
| Calendar popup | popup | Popover | OnDemand | Whole | 10 P |
| Screenshot thumb | Overlay | Toast | None | thumb | 10 |
| Widgets | Bottom | Widget | None | widget | 10 |
| Hot corners | Overlay | none | None | none | 10 |
| Quick note | Top | Window | OnDemand | none | 12 P |

## 2. Apps

All apps are xdg toplevels through shell-host (`spawn_toplevel`), `Material::Window`, on
paper (Post surface tokens). Only mail's frame shows the Space colour (21-SPACES §9).

### 2.1 Mail (first; exists)

| Field | Value | St |
| --- | --- | --- |
| Role | toplevel; migrates webview → Blitz (Phase A path dep, Phase B Blitz) | S |
| Material | `Window` with Space frame (`.ds-layer` A/B cross-fade + `.ds-grain`) around the Post card | S |
| Layout | S `.win` grid 232 px + card, card inset 8 (01-LAYOUT) | S |
| Components | `CommandPill`, `SidebarItem`, `SectionHeader`, `ListRow`, `HoverStrip`, `HoverCard`, `Tooltip{Fly}`, `Menu{Dropdown, Context, Rich, Slim}`, `CommandPalette`, `ToastHost`, `Peek`, `Scrim`, `Tabs`, `Chip`, `Avatar`, `Count`, `Button*`, `TextInput`, `AppearancePicker`, Space editor pieces | S |
| Motion | full A5 assignment set: `rise`, `fold`, `curl`, `heal`, `gulp`, `bump`, `seal-pop`, `tab-in`/`tab-out`, `star-pop`, `spark`, `pop-in`, `peek-in`, `menu-pop`, `hc-in`/`hc-out`, `compose-rise`, `compose-send`, `page-in`, `park`, `shake-x`, `slide-r`/`slide-l`, layer 380 ms | S |
| Behaviours | 06 (all), 11 (scroll), 12 (back/forward swipe) | S |
| Launcher | compose and search actions; reports verification codes to the shell | S |
| Milestone | after quire W2 (Phase A), Blitz Phase B | S |

### 2.2 Settings

| Field | Value | St |
| --- | --- | --- |
| Components | sidebar `SidebarItem` list, `SearchField`, `ListRow`, `Toggle`, `Slider`, `SegmentedControl`, `AppearancePicker`, Space editor (21-SPACES §6), `Menu{Dropdown}` | S / P |
| Motion | page change `page-in`; sidebar seal `seal-pop` | P |
| Behaviours | every page deep-linkable from the launcher (SPEC); 06, 11 | S |
| Milestone | M12 | S |

### 2.3 Files

| Field | Value | St |
| --- | --- | --- |
| Components | `SidebarItem` (places, tags), `ListRow` / grid tiles, `Chip{Label}` tags, `Menu{Context}`, `SearchField`, `Tabs` P | S / P |
| Motion | `rise` on first show; delete `fold`; drop target `gulp` | P |
| Behaviours | Quick Look on Space (SPEC); 06 (drag), 11, 12 | S |
| Milestone | M12 | S |

### 2.4 Quick Look

| Field | Value | St |
| --- | --- | --- |
| Role | standalone previewer service; toplevel window P | S / P |
| Material | `Window` | P |
| Components | toolbar `IconButton{Tool}`, pdfrum page widget (`blitz_dom::Widget`) | S |
| Motion | `peek-in` in, `fade` out | P |
| Keyboard | Space and Esc close | P |
| Milestone | M12 | S |

### 2.5 Terminal

Open decision (2026-09-24): the terminal core is undecided until the app-suite milestone; candidates `alacritty_terminal`, `wezterm-term` (Rust), libghostty (Zig, C ABI). ghostty is the daily terminal until then; cosmic-term is not part of this program.

| Field | Value | St |
| --- | --- | --- |
| Components | terminal grid as custom `Widget` (alacritty_terminal), `Tabs`, `Menu{Context}` | S |
| Motion | chrome only; grid never animates | P |
| Milestone | M12 | S |

### 2.6 Media player

| Field | Value | St |
| --- | --- | --- |
| Components | libmpv video on a Wayland subsurface; controls overlay on `Material::Osd`, `Slider`, `IconButton` | S / P |
| Motion | controls `fade` in/out, hide after 2000 ms idle P | P |
| Milestone | M12 | S |

### 2.7 Document and image viewer

| Field | Value | St |
| --- | --- | --- |
| Components | pdfrum page `Widget`, thumbnail sidebar `ListRow`, markup toolbar (shared with screenshot tool) | S |
| Milestone | M12 | S |

### 2.8 Notes

| Field | Value | St |
| --- | --- | --- |
| Components | library: `SidebarItem` folders, `ListRow` notes, editor (mail composer blocks: `/` menu, selection bubble), `CommandPalette` | S / P |
| Motion | composer set from mail (`page-in`, `chip-in`, `menu-pop`) | P |
| Milestone | M12 | S |

### 2.9 Photos

| Field | Value | St |
| --- | --- | --- |
| Components | grid tiles, `SidebarItem`, `Peek` viewer, `Slider` edits | P |
| Motion | viewer `peek-in`; grid `rise` on first show only | P |
| Milestone | M12 (medium priority) | S |

## 3. Open decisions

1. Launcher panel Material: `Popover` (proposed) or a dedicated `Palette` variant.
2. Notification toast position and entrance (top-right `slide-l`, proposed) vs mailo's bottom
   toast (`translateY(160%)` spring).
3. OSD hold 1500 ms and position (bottom centre) need 13's macOS numbers.
4. Power menu milestone (M5 proposed; SPEC names no milestone).
5. Calendar popup milestone (M10 proposed) and a new `MonthGrid` component in 04.
6. Quick note milestone (M12 with Notes, proposed) and its shortcut (SPEC: key to be chosen).
7. Hot corner surface size 2 x 2 px and dwell (13).
8. App switcher: build ours or use cosmic-comp's (SPEC says check first).

## 4. Sources

- PLAN "Design: `<shell>` repo (sill)" (bar, wallpaper, dock, launcher layer configs, regions,
  keyboard modes), "Design: `shell-host`" (`LayerConfig`, `KeyboardMode`, `RegionSpec`,
  `BlurSpec`, popups), "Design: `<ds>`" (Material enum, components, Anim), "UX decisions
  settled" (dock, gestures, Spaces), Appendix A4-A6.
- SPEC "Shell surfaces" (Tier 1-3), "Integrated experiences", "Bundled native apps",
  "Notes" (quick note), "Milestones".
- 04-COMPONENTS, 05-MOTION, 06-INTERACTIONS, 10-13 BEHAVIOUR docs, 21-SPACES.
