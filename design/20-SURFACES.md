# 20 Surfaces

One section per shell surface and per app: what it is built from. Every row names a
Material (`ds::Material`), components (04-COMPONENTS), motion (05-MOTION token and keyframe
names), behaviours (06, 10-13), keyboard, blur and input regions, and the milestone.

Status: **S** = settled (PLAN, SPEC or prototypes), **P** = proposed.
Material enum (design/30 §3.2): `Window, Bar, Dock, Menu, Popover, Sheet, Sidebar, Toast, Osd, Widget`.
Layer names are wlr-layer-shell layers; `KeyboardMode` and `RegionSpec`/`BlurSpec` are
shell-host types (PLAN "Design: `shell-host`"). Milestones are SPEC "Milestones" 0-13.
The Space colour reaches chrome per Look: behind the material in Mac, as `--f-*` frame tokens in Arc (design/30 §3.3, 21-SPACES).

## 1. Shell surfaces

### 1.1 Bar (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | layer-shell `Top`, one per output, anchor `TOP\|LEFT\|RIGHT`, `ExclusiveZone::Reserve(h)` | S |
| Height | height token (value in 01-LAYOUT) | S |
| Material | `Bar`, over the workspace SpaceLook's colour where the Look puts it (design/30 §3.3); `Ds` draws it (bar gaps) | S |
| Content | left: app name, workspace indicator (drag reorder); right: tray, then the module items `control_center.menu_bar_*` shows (sound, network, battery by default; each opens its module's detail pane), the control center item, clock (user direction 2026-09-25: controls at the top right as on macOS) | S |
| Components | `MenuBarItem` per status item (box and glyph from `bar.status_*` through `StatusMetrics`) and around the app name, titles and clock (the 4 px hover and open pill, 13/500 text), `WorkspacePills` for the workspace indicator (one segmented group on the frame), `MenuItem` rows (status lines included), `Glyph` (`IconSize::Bar` 22, P), `Menu` with `Placement::Bar` for every menu (22 px rows, 13 px text), `Badge`, `Tooltip` (12 px) | S / P |
| Motion | menus open Instant and close with a Fade `--t-quick`; hover background `--t-quick`; tint cross-fade `--t-big` (21-SPACES §5); no press scale | S |
| Behaviours | 13-BEHAVIOUR-menus-windows (menu bar, menus: open delay, safe triangle), 06 (menus, Escape), 12 (workspace swipe updates indicator) | S |
| Keyboard | `KeyboardMode::None`; a grabbing popup switches it to `OnDemand` until close | S |
| Blur / input | blur `Whole`; input `Whole` | S |
| Popups | `open_popup(anchor Element(btn), FitContent, grab Seat)` | S |
| Milestone | M1 | S |

### 1.2 Dock (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Top`, anchor `BOTTOM`, height = base x max magnification + padding, `Reserve(base + margin)` | S |
| Material | `Dock`, over the SpaceLook's colour per Look; drawn by `Ds` like the bar (bar gaps) | S |
| Components | `DockTile` (08-ICONS plate; `IconView { plate }` as the placeholder; `Badge` from LauncherEntry; `ProgressIndicator{Bar}`), `RunningDot`, optional `DockFloor`, `DockLabel` hover label, `Menu` with `Placement::Context` (windows, desktop actions, Keep in Dock, Quit), `Popover` for folder stacks; geometry from `DockMetrics` (48 tiles, 8 gaps, 6 padding) and the pill a `Corner::Squircle` | S / P |
| Motion | magnification: no easing while tracking, ~200 ms shrink on leave (10); launch bounce and attention bounce (10); a badge change swaps at once; the label fades `--t-quick`; the menu opens Instant | S |
| Behaviours | 10-BEHAVIOUR-dock (Plank parabola, 48 → up to 96, neighbours slide apart, click/right-click, stacks, drag-out, badges); auto-hide off by default (0.2 s delay, ~0.5 s slide when on) | S |
| Keyboard | `None` | S |
| Blur / input | blur `Element("dock-pill")`; input `Element("dock-hit")`; both follow the pill during magnification | S |
| Milestone | M3 | S |

### 1.3 Launcher (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | two surfaces kept warm: catcher (`Overlay`, anchor all, painted once, never animated) + panel (`Overlay`, centred, ~680 x 460) | S |
| Material | panel `Popover` over the SpaceLook's colour per Look, result list on `--raise` | P |
| Components | `CommandPalette<T>` (`host: Surface`, sized to its content, the shell scale's 22 px field, `corner: Squircle(14)`), `TextField{Search}`, `Row`, `KeyEquivalent`, `SectionHeader` (groups), `Menu` for actions (Ctrl+K / Alt+K), `Chip{Accent}` operator tokens | S |
| Motion | panel open: undecided (13 §13.9 item 9); rows appear with the panel, no stagger; close Fade `--t-quick` | S / P |
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
| Components | `ModuleGrid` of `ModuleTile`s (a module's disc, title, status and detail chevron; Off/On/Busy; Half or Full span), `PaneSwitcher` (the grid and a module's detail pane), `Row` (networks, devices, outputs: 44 px, the text menu's type, a check, toggle, chevron, value or glyph at the end), `Toggle`, `Slider`, `SegmentedControl` (power profile), `RadioGroup` with image labels (Appearance), `SectionHeader`, `Button` at `ControlSize::Mini`; glyphs from `Icon::CONTROL` (Now Playing's Play/Pause/SkipBack/SkipForward, device Headphones/Speaker/Mouse/Gamepad/Phone); `ModulePanel` for the modules with content (levels, Now Playing, Appearance, Battery) on the tile's frame, `ModuleGrid { columns, gap, padding }` from the settings, the bar item glyph `Icon::Switches`; the light level well is .18 (fill/well >= 1.6:1) (CONSUMING "Control center parts") | S |
| Motion | open and close Fade `--t-quick` (a shell popover); toggle knob by a `Spring`, track tint fade `--t-quick`; a module's detail pane slides in by `PaneSwitcher` (a `Spring` on `--pane-p`, reversing mid-slide) | S / P |
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
| Components | toast card (`HoverTarget` for actions, `Button` at `ControlSize::Mini` actions, app icon 08 plate at 20 P), `ToastHost` stack; center: `Row` groups by app in a `List`, `SectionHeader`, `Button` (Inline bezel) Clear | S / P |
| Motion | toast in Slide(Right) `--t-move` `--e-out`; out Slide(Right) `--t-quick` `--e-exit`, a swipe dismiss continuing with the release velocity; the stack closes the gap by `Roster`; center rows enter and leave by `Roster` | S |
| Timers | hold 5000 ms (`ToastHold`), paused while hovered | S |
| Behaviours | 13 (notifications), 06 (hover intent, design/30 §1.2) | S |
| Keyboard | toasts `None`; center `OnDemand` | P |
| Blur / input | per toast `Element("toast-<id>")`; input same | P |
| Milestone | M6 | S |

Components (CONSUMING.md "Notification parts"): the banner is `ds::NotificationCard` (app icon at
32, summary 13/700, age in the data face, a `Rich` body clamped two lines to six on hover,
`CardAction` Mini buttons, the 18 px close button at the top left, a `GroupCount` chip with up to
three offset layers, dismissed by `use_swipe`), in the Toast material by default from inside its own
transparent scope; the stack is `ds::BannerStack` (`BannerPosition::{TopRight, BottomRight}`,
`on_hidden` per key once the exit ends); the center is a `SidePanel` (right edge,
`notifications.center_width_px`, Popover material, `shown` and `on_hidden`) of `ds::GroupHeader`s
over cards. The center's material is Popover. `NotificationMetrics` writes the geometry keys.

### 1.7 OSD (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, anchor `TOP\|RIGHT` under the bar's reserve (`osd.position = TopRight`, the default, as current macOS shows its volume and brightness panel), `osd.margin_px` from the bar's reserve; `BottomCentre` keeps the old bottom-centred placement above the dock; no exclusive zone | S (user, 2026-09-25) |
| Material | `Osd` | S |
| Components | `Osd` (owns the card and its presence): `Glyph` (volume/brightness at `Bar` 22), a title line, a `LevelIndicator` (no thumb, not focusable); a small panel styled like a control-center slider module (§1.5 grid metrics, `--r-tile`) | S |
| Motion | in Fade `--t-quick`; a level change tweens linearly over `--t-move`; hold `osd.hold_ms` (1500) then Fade `--t-move`, the host unmaps after the exit | S |
| Behaviours | 13 (UI sounds: volume pop from freedesktop sound theme) | S |
| Keyboard / blur / input | `None`; blur `Element("osd")`; input `Empty` | P |
| Milestone | M6 | S |

Components: `ds::Osd` (the card and its presence, `on_hidden` once the exit ends, inside one
transparent Osd root), `ds::LevelIndicator` (the level with its glyph inside), `ds::OsdPosition`
and `ds::OsdMetrics` (`osd.position`, `osd.margin_px`); CONSUMING.md "OSD parts".

### 1.8 Power menu (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, full-output click catcher (dims nothing) + centred panel | P |
| Material | `Sheet` | P |
| Content | Log out, Restart, Shut down, Suspend (logind-zbus) | S |
| Components | `Ds { material: Sheet, extent: RootExtent::Viewport }` holding `Sheet { attach: Attach::Centre, shown, on_hidden }`; `Button` Cancel, `Button` with role Destructive Restart at `ControlSize::Regular`, the default `Button` Shut Down, an unavailable action `Availability::Disabled`; `KeyEquivalent{Cap}` hints | P |
| Motion | panel in Slide(Top) with a `Spring` (response Move), out Slide(Top) `--t-move` `--e-exit` (design/30 §1.3); the host unmaps after the exit | P |
| Behaviours | 06 (Escape, focus ring), 13 (focus/raise) | S |
| Keyboard | `Exclusive`; arrows move, Enter confirms, Esc closes | P |
| Blur / input | blur `Element("panel")`; input `Whole` | P |
| Milestone | M5 P (launcher system commands already exist at M4) | P |

### 1.9 Lock screen (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `ext-session-lock-v1` lock surface per output (not layer-shell) | S |
| Material | `Window` (opaque); background = pre-blurred wallpaper crop | P |
| Components | clock (display face), `Avatar` (the user's animated emoji or photo through `UserPicture`, design/25 section 7; the asset's own animation, no moods), `TextField{Secure}` password, the default `Button` | P |
| Motion | password error `use_shake` (`--t-shake`, `--e-shake`) once; unlock Fade `--t-move` `--e-exit` | S / P |
| Behaviours | lock before sleep on logind `PrepareForSleep` (SPEC); 06 (Escape clears field) | S |
| Keyboard | lock surfaces always receive keyboard | S |
| Milestone | borrowed at M7 (`session.locker`, design/22 §3.19: cosmic-greeter, else swaylock, else hyprlock); own at M11, and sill's own lock is the default since the user's pick of 2026-09-27 (`auto` keeps the borrowed order) | S |

### 1.10 Polkit prompt (SPEC Tier 1)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, click catcher (dims nothing) + centred panel | P |
| Material | `Sheet` | P |
| Components | `Avatar`, `TextField{Secure}`, `Button` (default and Cancel), details as `HoverCard` P | P |
| Motion | the sheet's enter and exit (design/30 §1.3); wrong password `use_shake` once | P |
| Keyboard | `Exclusive` | P |
| Blur / input | blur `Element("panel")`; input `Whole` | P |
| Milestone | borrowed at M7 (`session.polkit_agent`, design/22 §3.19: polkit-kde, polkit-gnome, lxqt-policykit, polkit-mate, else cosmic-osd); own at M11 | S |

### 1.11 App switcher (SPEC Tier 2)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, centred; check what cosmic-comp provides first (SPEC) | S |
| Material | `Osd` | P |
| Components | app tiles (08 plate at 64 P), the selected app's name as a `Label` under the selection | P |
| Motion | show after 150 ms hold P, Fade `--t-quick`; the selection square slides by a `Spring` | P |
| Behaviours | 13 (Cmd+Tab: apps not windows) | S |
| Keyboard | `Exclusive` while shown (must see modifier release) | P |
| Milestone | M11 | S |

### 1.12 Calendar popup (SPEC Tier 2)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | a widget in the notification center (M6's center replaced the clock's popup) | P |
| Material | `Popover` | P |
| Components | month grid (`MonthGrid`), `Row` events, `Button{Toolbar}` prev/next | P |
| Motion | Fade `--t-quick` (a Popover); month change `slide-l`/`slide-r` | P |
| Data | the Calendar app (2.10, its own repo) feeds it; until then local only | S |
| Milestone | M10 P (Tier 2) | P |

### 1.13 Screenshot thumbnail (SPEC integrated experience, priority pick)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | `Overlay`, anchor `BOTTOM\|RIGHT` | P |
| Material | `Toast` | P |
| Components | thumbnail image, `HoverTarget` actions (Mark up, Copy text via OCR, Delete), drag source | P |
| Motion | in Slide(Right) like a Toast; auto-dismiss after `ToastHold` (5000 ms), Slide(Right) `--t-quick` `--e-exit` | P |
| Behaviours | 06 (drag), 13 (UI sounds: screenshot grab) | S |
| Keyboard / blur / input | `None`; blur and input `Element("thumb")` | P |
| Milestone | M10 | S |

### 1.14 Desktop widgets (SPEC Tier 2)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | one `Bottom` surface per output (above wallpaper, below windows), the widgets on a grid inside it | P |
| Material | `Widget` | S |
| Content | calendar, weather, battery (SPEC) | S |
| Components | `SectionHeader`, `Badge`, `Row` | P |
| Motion | none in steady state (nothing loops); a value change is instant | S |
| Keyboard / blur / input | `None`; blur `Element("widget")`; input `Element("widget")` | P |
| Milestone | M10 | S |

### 1.15 Quick note (SPEC Notes)

| Field | Value | St |
| --- | --- | --- |
| Layer / role | layer-shell `Top`, `KeyboardMode::OnDemand` (SPEC: normal windows cannot stay on top) | S |
| Material | `Window` (paper: apps stay on paper, 21-SPACES §7) | P |
| Components | editor (Notes' block editor), `CommandPalette` on Cmd+K, `TextField{Search}`, `Button{Toolbar}` | S / P |
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
| Behaviours | `13-BEHAVIOUR-menus-windows.md#13-3-12-hot-corners` (dwell, re-arm, modifier, action table) | P |
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

### 1.17 Accounts (system service)

One place for the user's online accounts, like the reference platform's Apple Account and
Internet Accounts (user, 2026-09-26): sign in once, and every native app (Mail, Calendar,
Photos, Files/Drive, Notes, Contacts) uses that account; several accounts side by side.

| Field | Value | St |
| --- | --- | --- |
| Shape | a user-session service with a D-Bus API, its own repo, portable (the account core also runs in-process on non-Linux, so mailo stays cross-platform) | P |
| Providers | Google, Microsoft, Fastmail, iCloud where possible, and "custom server": IMAP/SMTP, JMAP, CalDAV, CardDAV, WebDAV, Nextcloud (a self-hosted server gives the whole iCloud-like set) | P |
| Services per account | mail, calendar, contacts, files, photos, notes; each switchable per account (the reference's per-service toggles) | P |
| Secrets | OAuth refresh tokens and passwords in the Secret Service (keyring), never in config files; apps get short-lived access tokens from the service, scoped to what they asked for | P |
| Compatibility | also answer GNOME Online Accounts' D-Bus API if cheap, so foreign apps that read it see the same accounts (research) | P |
| Seeds | mailo already has account discovery (autoconfig), OAuth, IMAP/SMTP/JMAP and CalDAV/CardDAV (`mail-domain`, `mail-proto`, `mail-pim`): extract the account core from mailo the way latchkey was extracted, coordinated with the mailo session | P |
| UI | a Settings pane (accounts list, add account sheet, per-service toggles, sign-out), and the first-run "sign in" step | P |
| Risk | Google's restricted scopes (Gmail, Drive) need a verified OAuth client and a security assessment; decide per provider whether we ship our own client ID or ask the user for one | P |
| Milestone | before the app pass's first sync-using app; the design doc (28-ACCOUNTS) and research first | P |

## 2. Apps

All apps are xdg toplevels through shell-host (`spawn_toplevel`), `Material::Window`, on
paper (`--paper`). Only mail's frame shows the Space colour (21-SPACES §9).

### 2.1 Mail (first; exists)

| Field | Value | St |
| --- | --- | --- |
| Role | toplevel; migrates webview → Blitz (Phase A path dep, Phase B Blitz) | S |
| Material | `Window` with Space frame (`.ds-layer` A/B cross-fade + `.ds-grain`) around the card | S |
| Layout | S `.win` grid 232 px + card, card inset 8 (01-LAYOUT) | S |
| Components | `CommandPill`, `Row` (`List{SourceList}` for the sidebar), `SectionHeader`, `HoverCard`, `Tooltip`, `Menu`, `CommandPalette`, `ToastHost`, `EdgePeek`, `SegmentedControl`, `Chip`, `Avatar`, `Badge`, `Button`, `TextField`, `RadioGroup`, Space editor pieces | S |
| Motion | design/30 §1.3 only; the A5 keyframe set is retired (design/30 Part 4); the Space layer cross-fades `--t-big` | S |
| Behaviours | 06 (all), 11 (scroll), 12 (back/forward swipe) | S |
| Launcher | compose and search actions; reports verification codes to the shell | S |
| Milestone | after quire W2 (Phase A), Blitz Phase B | S |

### 2.2 Settings

| Field | Value | St |
| --- | --- | --- |
| Components | sidebar `Row`s in a `List{SourceList}`, `TextField{Search}`, `Row`, `Toggle`, `Slider`, `SegmentedControl`, `RadioGroup`, Space editor (21-SPACES §6), `Menu` | S / P |
| Motion | page change `page-in` | P |
| Behaviours | every page deep-linkable from the launcher (SPEC); 06, 11 | S |
| Milestone | M12 | S |

### 2.3 Files

| Field | Value | St |
| --- | --- | --- |
| Components | `Row` in a `List{SourceList}` (places, tags), `Row` / grid tiles, `Chip` tags, `Menu` with `Placement::Context`, `TextField{Search}`, `TabView` P | S / P |
| Motion | list changes by `Roster`; the drop target shows the `List` drop line | P |
| Behaviours | Quick Look on Space (SPEC); 06 (drag), 11, 12 | S |
| Milestone | M12 | S |

### 2.4 Quick Look

| Field | Value | St |
| --- | --- | --- |
| Role | standalone previewer service; toplevel window P | S / P |
| Material | `Window` | P |
| Components | toolbar `Button{Toolbar}`, pdfrum page widget (`blitz_dom::Widget`) | S |
| Motion | `peek-in` in, `fade` out | P |
| Thumbnails | a PDF's first page (and the page strip's pages) reuse the launcher preview's part, `ds_native::PdfFileThumb` over `ds::PdfThumb` (design/04 section 45): same raster, same cache | S |
| Keyboard | Space and Esc close | P |
| Milestone | M12 | S |

### 2.5 Terminal

Open decision (2026-09-24): the terminal core is undecided until the app-suite milestone; candidates `alacritty_terminal`, `wezterm-term` (Rust), libghostty (Zig, C ABI). ghostty is the daily terminal until then; cosmic-term is not part of this program.

| Field | Value | St |
| --- | --- | --- |
| Components | terminal grid as custom `Widget` (alacritty_terminal), `TabView`, `Menu` with `Placement::Context` | S |
| Motion | chrome only; grid never animates | P |
| Milestone | M12 | S |

### 2.6 Media player

| Field | Value | St |
| --- | --- | --- |
| Components | libmpv video on a Wayland subsurface; controls overlay on `Material::Osd`, `Slider`, `Button{Toolbar}` | S / P |
| Motion | controls `fade` in/out, hide after 2000 ms idle P | P |
| Milestone | M12 | S |

### 2.7 Document and image viewer

| Field | Value | St |
| --- | --- | --- |
| Components | pdfrum page `Widget`, thumbnail sidebar `Row`, markup toolbar (shared with screenshot tool) | S |
| Milestone | M12 | S |

### 2.8 Notes

| Field | Value | St |
| --- | --- | --- |
| Components | library: `Row` folders and notes in a `List`, editor (mail composer blocks: `/` menu, context menu), `CommandPalette` | S / P |
| Motion | design/30 §1.3 only | P |
| Milestone | M12 | S |

### 2.9 Photos

The reference platform's Photos app is the target (user, 2026-09-26): a library, not a file
browser. Its own repo; in the app pass (M12). §2.7 stays the Preview-like document viewer and
Quick Look (2.4) stays the space-bar peek; all three share the image and PDF parts.

| Field | Value | St |
| --- | --- | --- |
| Library | one library over the user's photo folders and every account's photo service (1.17); Library by Years / Months / Days / All, Albums, Favourites, Recently Deleted, Hidden (locked), imports | P |
| Viewer | full-window viewer with swipe between photos, pinch/scroll zoom, info panel (date, place, camera, lens, exposure), Live Photo / motion playback, video playback | P |
| Edits | non-destructive (original kept, edit list stored beside it): crop, straighten, rotate, exposure/colour adjustments, filters, markup (shared with Capture 2.11 and the viewer 2.7), revert | P |
| Search and grouping | on-device only: places (EXIF GPS, offline geocoder), dates, people and objects through a local model, never a cloud service | P |
| Sync | through the system account (1.17): photos from a cloud provider appear in the same library; shared albums where the provider has them | P |
| Components | grid tiles with a zoomable time grid, `Row` in a `List{SourceList}`, the viewer, `Slider` edits, `Sheet` for import and share | P |
| Motion | the grid-to-viewer zoom (the tile grows into the viewer and shrinks back) | P |
| Formats | JPEG, PNG, WebP, AVIF, HEIC/HEIF, RAW previews, GIF, video (through Capture's decode path) | P |
| Milestone | app pass (M12); research on the reference's Photos behaviour first | P |

### 2.10 Calendar

Its own app and its own repo (user, 2026-09-26: "not going to be so simple"), like mail: a
mailo-like client that connects several cloud calendar providers (CalDAV, Google, Microsoft),
with UI borrowed from good calendar apps. It is NOT part of the widgets pass; the M10 calendar
widget and popup read from it once it exists and stay local-only until then.

| Field | Value | St |
| --- | --- | --- |
| Repo | new (name open), depends on quire; account and sync core modelled on mailo's | P |
| Components | `MonthGrid` (04 §39), week/day views (new, quire first), `Row` events, `Sheet` editor | P |
| Milestone | after M12's first apps; scoped on its own (research pass on calendar UIs first) | P |

### 2.11 Capture

Its own app and its own repo (user, 2026-09-26), deferred to the app pass (M12). One app for
stills and video, aimed at the best tools (Screen Studio, OBS), not the reference platform's
recorder. The M10 screenshot thumbnail (1.13) becomes the shared after-capture surface for both.

| Field | Value | St |
| --- | --- | --- |
| Repo | new (name open); MIT OR Apache-2.0, no GPL/AGPL code (libobs, Cap, gpu-screen-recorder are read for ideas only), no FFmpeg | S |
| Capture | stills and windows through sill's privileged compositor capture (no prompt); portal + PipeWire (`ashpd`, `pipewire`) for other sessions | P |
| Compose | scenes, webcam, overlays, cursor smoothing, click-following zoom, window backgrounds on our own wgpu/vello stack | P |
| Encode | a **vendor-neutral encoder trait** (user: "not nvidia specific"). Primary backend `gpu-video` (MIT, Software Mansion; Vulkan Video, `wgpu::Texture` in, H.264 + HEVC encode, AV1 in progress; needs wgpu 30, ours is 29: take it at a toolchain bump; NOT the rest of smelter, whose real-time licence is restrictive). Fallbacks: VA-API (`cros-codecs`) where a driver has no Vulkan Video encode (this machine's RADV iGPU exposes the encode queue but no codec extension, 2026-09-26), `rav1e` software. The app never names a vendor; backend chosen by probing, overridable | P |
| Frames | zero-copy: DMA-BUF in, GPU compose, GPU encode; no CPU copy on any backend | P |
| Audio | per-source PipeWire capture, `nnnoiseless`, mixer, multi-track | P |
| Output | `muxide` MP4; replay buffer; streaming (`webrtc` WHIP, `srt-tokio`, `rml_rtmp`) | P |
| Milestone | app pass (M12). First step: a proof that DMA-BUF frames reach a hardware encoder with no CPU copy through the trait, on NVIDIA AND on the AMD iGPU of this machine, before any UI | P |

## 3. Open decisions

1. Launcher panel Material: `Popover` (proposed) or a dedicated `Palette` variant.
2. Notification toast position (top-right, 13.3.6) vs mailo's bottom toast. The entrance is
   settled: Slide(Right) (design/30 §1.3).
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
