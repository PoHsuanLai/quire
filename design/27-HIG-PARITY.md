# 27 HIG parity: quire against the pre-2025 Human Interface Guidelines

Status: reviewed 2026-09-26 (§8 settled), branch `hig-parity` (quire). Docs only. Every rule this file
proposes is **proposed**; nothing here changes a settled value in another doc until the user
settles it (README §5). Confidence keys as in `13-BEHAVIOUR-menus-windows.md`: **H** the vendor's
own guidelines or documentation (here: an archived HIG page), **M** a reliable secondary source,
**L** observed on macOS 14/15 and not measured, **UNKNOWN** nothing found.

**Why this file exists.** The target is "feels like a Mac". The design docs so far were built
from the mail prototypes (`S`, `C`) and from measured macOS behaviour (10-13). Nobody has walked
the vendor's own guideline pages, page by page, and asked what we do about each. This file does,
for the guideline set that describes the macOS 14 / macOS 15 look we copy.

## 0. Source and era

### 0.1 Which HIG

The guidelines as they stood **before the June 2025 redesign** (the material the vendor calls
Liquid Glass, shipped with macOS 26). That is the macOS 14 Sonoma / macOS 15 Sequoia era. We
match it on purpose: its flat, bright, lightly translucent chrome is what 03 §17.4, 23 and the
icon work already target, and the later glass language would undo those decisions.

The live HIG pages now describe macOS 26, so every page here comes from an archived copy. The HIG
site renders in the browser from a JSON document per page; the archived HTML is an empty shell,
so this audit reads the archived JSON
(`developer.apple.com/tutorials/data/design/human-interface-guidelines/<page>.json`) and its
`primaryContentSections`. Snapshot rule: the capture closest to 2025-05-01 inside 2025-01-01 to
2025-06-08; where none exists, the closest inside 2023-06-06 (the macOS 14 announcement) to
2025-06-08. Every page used is inside the macOS 14/15 era. Raw downloads stay in the session's
scratch folder and are not committed; this file paraphrases and quotes only short phrases.

**71 pages retrieved**, captured between **2023-06-21 and 2025-05-31**. Each page's change log
was read to confirm the capture postdates the macOS 14 update of that page.

**Not retrieved** (no archived JSON capture before 2025-06-08 exists for these slugs): `images`,
`loading`, `printing`, `panels`, `toggles`, `steppers`, `token-fields`, `labels`, `combo-boxes`,
`image-wells`, `path-controls`. Their sections below say so and rest on observed behaviour (L).

### 0.2 Snapshots

URL form: `http://web.archive.org/web/<timestamp>/https://developer.apple.com/tutorials/data/design/human-interface-guidelines/<page>.json`.

| Page | Timestamp | Page | Timestamp |
| --- | --- | --- | --- |
| accessibility | 20241114144601 | menus | 20240331074052 |
| alerts | 20240325112651 | modality | 20240325112649 |
| app-icons | 20250422012110 | motion | 20250508092624 |
| boxes | 20250407102551 | multitasking | 20240225033236 |
| branding | 20240124163000 | notifications | 20250531072050 |
| buttons | 20250224003721 | offering-help | 20231102160907 |
| charts | 20250422213606 | onboarding | 20231007215409 |
| collaboration-and-sharing | 20231102161305 | outline-views | 20230922115555 |
| collections | 20240906234327 | playing-audio | 20240118231612 |
| color | 20250502205658 | pointing-devices | 20240612052007 |
| color-wells | 20240713082909 | pop-up-buttons | 20231113234319 |
| column-views | 20240331074803 | popovers | 20240904221418 |
| context-menus | 20250224003722 | privacy | 20241230014852 |
| dark-mode | 20230803023232 | progress-indicators | 20250224003751 |
| designing-for-macos | 20250211124836 | pull-down-buttons | 20231102154910 |
| disclosure-controls | 20240906234423 | rating-indicators | 20250224003810 |
| dock-menus | 20250407102757 | right-to-left | 20250108042624 |
| drag-and-drop | 20231005123829 | scroll-views | 20240918020229 |
| edit-menus | 20240401203827 | search-fields | 20230710220527 |
| entering-data | 20231102160906 | searching | 20240210073513 |
| feedback | 20231007004820 | segmented-controls | 20250416004743 |
| file-management | 20231102152004 | settings | 20231102161025 |
| focus-and-selection | 20231102152003 | sf-symbols | 20250508092734 |
| gauges | 20250224003909 | sheets | 20240906202937 |
| going-full-screen | 20240318022538 | sidebars | 20250228024414 |
| icons | 20250422012136 | sliders | 20240407152039 |
| inclusion | 20240124163402 | split-views | 20231124122124 |
| keyboards | 20250225175135 | tab-views | 20250407102552 |
| launching | 20231102161137 | text-fields | 20230621223927 |
| layout | 20250322040005 | text-views | 20231007215717 |
| lists-and-tables | 20230920195139 | the-menu-bar | 20241214023016 |
| managing-accounts | 20231102153032 | toolbars | 20240302055431 |
| managing-notifications | 20250502150210 | typography | 20241106035514 |
| materials | 20250207012039 | undo-and-redo | 20231007215656 |
| | | windows | 20240503161141 |
| | | writing | 20241106035134 |

### 0.3 Other sources

| Key | Source | What it gives | Conf. |
| --- | --- | --- | --- |
| [FLUID] | "Designing Fluid Interfaces", WWDC 2018 session 803, `https://developer.apple.com/videos/play/wwdc2018/803/` (transcript read 2026-09-26) | Interruption and redirection as a core property; springs described by **damping** and **response**, "we actually like to avoid using duration"; start at **100 % damping** (no overshoot); **80 % damping** only when the driving gesture has momentum (their example: tap-to-present at 100 %, swipe-to-dismiss at 80 %); hand the gesture's velocity to the animation; a thrown object goes to the endpoint nearest its **projection** (velocity through the scroll deceleration rate), not its release point; a drag begins after about **10 pt** of hysteresis on touch; 1:1 tracking; rubber-band at edges | H |
| [KIT] | Design resources page, snapshot `http://web.archive.org/web/20250502162612/https://developer.apple.com/design/resources/` | macOS Sequoia: Sketch library (2025-01-21, 36.8 MB), design templates for Figma and Sketch (2025-01-21, Sketch 21.8 MB; Figma community file 1251588934545918753), production templates (Sketch 2025-01-03, 2.3 MB; Photoshop 6.5 MB). Not downloaded: the kits' licence limits their use to designing for the vendor's platforms, so we cite them as the place a number would be measured, not as a source we copy from | H (existence) |
| [SCROLL] | `11-BEHAVIOUR-scroll.md` R5 | the scroll deceleration rate .998 per ms, which [FLUID]'s projection uses | H |

### 0.4 What the June 2025 redesign changed, which we do not adopt

- A refracting, reflecting glass material for bars, toolbars, sidebars and controls that float
  over content. We keep 03 §17's tint-over-blur materials with the stack v2 layers.
- Controls and toolbar groups drawn as floating capsules; sidebars as inset floating panes. We
  keep chrome attached to the window edge.
- A new, explicit concentric-shape rule and larger corner radii across windows and controls. We
  take the older, implicit form of the rule (section 3.9) without the larger radii.
- A transparent menu bar and layered, glass-edged app icons with clear and tinted variants. We
  keep a solid-tint bar (03 §17.2) and 08's plate icons.
- Scroll-edge fades in place of hairline separators. We keep hairlines.

## 1. Summary

### 1.1 Gaps, ranked by how much each hurts "feels like a Mac"

Rank is judged, not measured: how often a person meets the gap and how quickly it reads as "not a
Mac". **Enforce** names the cheapest mechanism that keeps it fixed: **Type** (a Rust type makes
the wrong thing unwritable), **Lint** (a `ds::lint` CSS or markup rule), **Test** (a harness or
data test), **Doc** (a reviewer's CHECKLIST line).

| # | Gap | Where it shows | Enforce | Section |
| --- | --- | --- | --- | --- |
| 1 | Motion is fixed-duration keyframes; a new input restarts or waits instead of redirecting from the current position and speed; release velocity is dropped | every drag, swipe, panel slide, sheet, knob, selection | Type + Test | 3.12 |
| 2 | No app menus in the bar: no App, File, Edit, View, Window, Help, no standard items | the top of every screen | Type + Test | 5.2 |
| 3 | Standard shortcuts are not reserved; prototype keys collide (Mod+S, Mod+T, Mod+K); no Cmd+Q/W/M/H/, map; modifier order unspecified | every app | Type + Test | 6.2 |
| 4 | Focus and selection read as web: one fixed ring (2.5 px, radius 4) on every control, lists ringed instead of highlighted, no emphasized/unemphasized selection, menus highlight in a pale wash | every keyboard user; every list and menu | Lint + Type | 6.4 |
| 5 | Accessibility settings are mostly unwired: high contrast is read and ignored; no reduced transparency; reduced motion still slides (at 60 ms); no "differentiate without colour"; no text size | anyone who sets them | Type + Test | 3.1 |
| 6 | CJK text has no specified face, fallback or line breaking; the shipped faces are Latin subsets | every Traditional Chinese string | Test + Doc | 3.14 |
| 7 | Pointer is the web's: `cursor:pointer` on every control; no copy, not-allowed or resize cursors from the drag rules | every hover | Lint | 6.3 |
| 8 | Type ramp is the prototype's: 15 px body (Mac 13), sizes under the 10 pt floor (9.5, 8), uppercase tracked section headers | every text-dense surface | Lint | 3.16 |
| 9 | Modality: the sheet is a centred peek over a scrim, not attached to its window; no focus trap; no alert component; destructive styling on deliberate actions | dialogs, power menu, polkit | Type + Test | 4.10, 5.7, 5.8 |
| 10 | Undo and redo exist only as mail's undo toast; no Cmd+Z / Shift+Cmd+Z in fields and lists, no Edit menu | every edit | Type + Test | 4.18 |
| 11 | No written style: title case for buttons and menus, the ellipsis rule, "Cancel" always, tooltip length | every label | Lint + Doc | 3.17 |
| 12 | Label hierarchy stops at three inks; text on a material uses card or frame inks instead of per-material vibrant inks | menus, bar, control center | Type (tokens) + Test | 3.11 |
| 13 | Menus: first open pops with a spring; context menus dim unavailable items and show shortcuts; titles and help lines in text menus | every menu | Type + Test | 5.1, 5.4 |
| 14 | No state restoration: windows, panes, sidebar expansion, the last settings pane | every relaunch | Type + Test | 4.6 |
| 15 | Sidebar metrics and icon colour: no Small/Medium/Large; icons in ink, not accent | every app sidebar | Type | 5.12 |
| 16 | Glyph weight does not follow the adjacent text's weight; no symbol scales | every icon beside text | Type | 3.15 |
| 17 | Nested corner radii are chosen by hand, not derived | cards, fields, highlights, focus rings | Type + Lint | 3.9 |
| 18 | No privacy indicators for camera, microphone, screen capture | the bar | Doc + service | 3.13 |
| 19 | Drag threshold 8 px (Mac: about 3 pt), no return-to-source on a failed drop, no count badge | drag in lists, dock | Test | 4.1 |
| 20 | Alert sounds and the beep are specified but unbuilt | errors | Test | 4.14 |

### 1.2 What can become types or lint

26-DETAILS (branch `detail-grammar`) proposes a `Detailed` trait (a component's own state
enum says what each transition means), `use_detail` (tracks a state, returns the moment),
`Touch::{Contact, Remote}` (only contact may spend an overshoot) and moment tables tested as data.
Most of this audit's motion and state rules fit that machinery without a second one:

| Rule | Mechanism | Where |
| --- | --- | --- |
| Contact carries the release velocity; springs retarget from position and velocity | `Touch::Contact(Velocity)` (extends 26 §4.1's `Touch`); `use_spring` next to 26's `use_tween` | `ds::detail`, `ds::motion` |
| Damping 1.0 unless the driving gesture had momentum; 0.8 then | `SpringSpec::for_touch(Touch)`; no public constructor takes a raw damping | `ds::motion` |
| Reduced motion swaps movement for a cross-fade | every keyframe declares a `ReducedForm` in the recipe table; a data test fails a moving keyframe without a still form | `ds::motion::recipe`, 05 §3.2 |
| A context menu never shows shortcuts and hides unavailable items | `MenuKind::Context` drops `shortcut` and filters `Availability::Disabled` at the type level | `ds::components::menu` |
| A default button is never destructive | `AlertButtons { default: Action<Safe>, cancel, other }`; `Action<Destructive>` cannot be the default | `ds::components::alert` |
| Reserved shortcuts | `Shortcut::custom(..)` returns an error for a combination in the standard table; `Shortcut::standard(StandardAction)` is the only way to bind one | `ds::components::vocab`, test |
| Inner radius follows outer radius | `Radius::inner(outer, inset)`; the CSS lint rejects a raw radius on an element the markup marks as nested | `ds::geometry`, `ds::lint` |

New CSS lint rules proposed (land as warnings first and tell the sill session: sill reads quire by path, so a new
Strict rule breaks sill's master at once): `PointerCursor` (no `cursor:pointer` outside link and
drag-handle rules), `MinFontSize` (no size token below `--fs-min` 10 px in UI text),
`FocusRingShape` (a `:focus-visible` outline radius must be `calc(var(--r-*) + var(--focus-gap))`),
`ReducedFormMissing` (data test, not CSS). New markup lint rules: `UnnamedControl` (every
interactive element has an accessible name), `ThreeDots` (a label contains "..." instead of
"…").

## 2. How to read the page sections

Each section names the HIG page and its snapshot date, then four rows:

- **Apple**: what the page says, paraphrased, with its numbers.
- **quire today**: what our docs specify, with the section, or "nothing".
- **Verdict**: **Adopt**, **Adapt** (how), **Skip** (why) or **Covered**.
- **Rule**: for Adopt and Adapt, the rule we would write, the doc and crate it lives in, and
  whether it is a type, lint, test or doc line.

## 3. Foundations

### 3.1 Accessibility (2024-11-14)

The page covers several settings; each has its own row.

| Setting | Apple | quire today | Verdict | Rule |
| --- | --- | --- | --- | --- |
| Contrast | Text contrast floors: 4.5:1 up to 17 pt, 3:1 at 18 pt and up or bold. Standard controls follow Increase Contrast; system colours ship an "Accessible" variant for it (color page: e.g. blue light `0,122,255` becomes `0,64,221`; dark `10,132,255` becomes `64,156,255`) | Gates at 4.5:1 (03 §6, legibility tests); `SystemPrefs.contrast: Contrast::High` is read from the portal and resolved to nothing (`appearance/resolve.rs`) | Adopt | A `High` token set: `--ink-soft`/`--ink-faint` pulled toward `--ink`, hairlines doubled in alpha, accent and status colours swapped for darker (light) or lighter (dark) variants, material tints at their solid alpha. Gate: body text 7:1 under High (dark-mode page asks for 7:1 on custom colours). 03 new §19; `ds::tokens`; test `every_pair_legible_under_high_contrast` |
| Reduce Transparency | "Make areas of blurred content and translucency mostly opaque", with a colour that differs from the translucent one | Nothing; `BlurState::Unavailable` already paints `--m-tint-solid` (alpha ≥ .94) | Adopt | `appearance.transparency = System \| Reduced` (the portal has no key for it, so ours is a setting); Reduced forces the solid path on every material and drops the vibrancy boost. 03 §17.1; `ds::material`; test over every `Material` |
| Reduce Motion | "Tighten springs … or track 1:1", no z-axis depth animation, no animating into or out of blurs, "replace a slide with a fade" | Reduced = every duration 60 ms, iteration count 1 (05 §3.2): slides still slide, scales still scale | Adapt | Keep 60 ms (settle timers depend on it) but each moving keyframe's Reduced form is a cross-fade with no transform; springs under Reduced are critically damped with no overshoot and track 1:1 while touched. 05 §3.2; `ds::motion::recipe`; data test `every_moving_keyframe_has_a_still_reduced_form` |
| Colour alone | "Avoid relying solely on color"; problem pairs: blue/orange, red/green, red/black, red or green with gray; example: red square for offline, green circle for online. Links may add an underline | 00 §3 "colour is a claim" says what a hue means, not that a second cue exists | Adopt | Every status carried by hue also carries a shape or a word: `StatusMark { hue, shape: Shape }` with no hue-only constructor; `appearance.differentiate_without_color = Off \| On` adds link underlines and the shapes in dots (unread dot, online dot). 03 §14; `ds::components::vocab`; type |
| Text size | macOS has no Dynamic Type (typography page). The sidebar size setting (Small/Medium/Large, see 5.12) is the Mac's text-size lever; use Regular through Bold, avoid Light and thinner | `display.scale` only (22 §3.15); weights 400-800 already | Adapt | `appearance.sidebar_size = Small \| Medium \| Large` drives the 5.12 metrics; no free text scale (the Mac has none). 22 §3.1; `ds::tokens` |
| VoiceOver (screen reader) | Every element reachable, labelled, grouped; decorative images hidden; announce layout and content changes; each page a unique title and headings | Components emit `role` and `aria-*` (04 shared vocabulary); Blitz's `accessibility` feature is on in the workspace; how shell surfaces (layer-shell through shell-host) reach AT-SPI is not specified | Adopt | Markup lint `UnnamedControl`; a live-region contract for toasts, banners, OSD and progress (`aria-live="polite"`, critical banners `assertive`); shell-host exports each surface's AccessKit tree over AT-SPI. 04 global rules; `ds::lint::markup`; shell-host; test: Orca reads the launcher and a banner (manual check, queued in sill/docs/manual-checks.md) |
| Full Keyboard Access | Keyboards page: with it on, every control is reachable; Ctrl+F1 toggles it; Ctrl+F2 menu bar, Ctrl+F3 Dock, Ctrl+F5 toolbar, Ctrl+F6 next panel, Ctrl+Tab next control group | Every control is in the Tab order (04 components, 13.3.11 "Tab reaches close, minimize, zoom") | Adapt | See 6.4: `appearance.keyboard_navigation = TextAndLists \| All` (default `All`, our Arc H5 stance); the Ctrl+F2/F3 shortcuts focus the bar and the dock. 06 §2; sill |
| Hit targets | "Even when people use a pointer", too-small targets frustrate; 44 pt on touch | 22 px menu rows, 12 px traffic lights (the Mac's own) | Covered | Mac sizes are the Mac's; nothing to add |

### 3.2 App icons (2025-04-22)

| | |
| --- | --- |
| **Apple** | macOS icons share a rounded-rectangle shape, front-facing view, level position and a uniform drop shadow from the template; "lifelike" rendering of a familiar tool, often floating just above the plate and past its edge; interior shadows and highlights lit from "just above center and tilted slightly downward"; never a silhouette other than the rounded rectangle; primary content inside the icon grid, everything inside the outer box. Sizes 16, 32, 128, 256, 512 at 1x and 2x, 1024 for distribution; PNG, sRGB or Gray Gamma 2.2 |
| **quire today** | 08 §2: squircle plate n = 5 on an 824/1024 grid, per-app gradient, abstract symbols pressed into the plate (the user's decision: abstract, not realistic) |
| **Verdict** | Adapt: keep the abstract language (settled); take the shape, grid, light direction and one shared drop shadow |
| **Rule** | The plate's highlight and inner shadow assume one light above centre, tilted down; the drop shadow is one recipe for every icon (08 §2.5). 08 §2.5; `quire icon` tool; test: the shadow parameters are one constant |

### 3.3 Branding (2024-01-24)

| | |
| --- | --- |
| **Apple** | Branding defers to content; use standard patterns; accent colour where the system allows; no logo repetition; no launch screen as branding |
| **quire today** | 07 looks and 21 Spaces carry the identity; no splash screens |
| **Verdict** | Covered |

### 3.4 Color (2025-05-02)

| | |
| --- | --- |
| **Apple** | Use colour sparingly; never one colour for two meanings; supply light and dark variants; semantic dynamic colours by purpose (35 on macOS: label, secondary/tertiary/quaternary label, control accent, keyboard focus indicator, selected content background, unemphasized selected content background, find highlight, separator, window background, under-page background …). Every macOS system colour has Default, Accessible, and Vibrant (on materials) variants; e.g. red `255,59,48` / dark `255,69,58` / accessible `215,0,21` |
| **quire today** | 03: Post palette, status inks, Space frame tokens, contrast gates; roles by token (03 §14) |
| **Verdict** | Adapt: keep our palette (the Space look is ours); take the missing semantic roles and the three variants per colour |
| **Rule** | Add roles `--ink-quaternary`, `--sel-bg` / `--sel-ink` (emphasized), `--sel-bg-quiet` (unemphasized), `--find-highlight`, `--focus-ring`, `--under-page`; each colour token carries `{normal, high, vibrant}` values (3.1, 3.11). 03 §3, §14; `ds::tokens`; test `every_role_has_three_variants` |

### 3.5 Dark Mode (2023-08-03)

| | |
| --- | --- |
| **Apple** | "Avoid offering an app-specific appearance setting"; test with Increase Contrast and Reduce Transparency on; 4.5:1 minimum, strive for 7:1 on custom colours; label colours at four levels adapt; soften white content backgrounds; desktop tinting under the graphite accent |
| **quire today** | `Theme::{System, Light, Dark}` per app and per Space (22 §3.1; `resolve` lets the Space, then the app, then the desktop decide) |
| **Verdict** | Adapt: the Space's theme is ours and stays; the per-app override leaves the app's settings |
| **Rule** | Apps never show their own appearance picker; `AppearancePicker` appears only in Settings and the control center (plan: "THE one picker"). A Space may still pin a theme. 22 §3.1; doc line in CHECKLIST §3 |

### 3.6 Icons (interface icons) (2025-04-22)

| | |
| --- | --- |
| **Apple** | Consistent size, detail, stroke and perspective; **match the icon's weight to the adjacent text**; **optical alignment**: pad an asymmetric icon so geometric centring of the asset centres it optically ("typically very small, but … a big impact"); selected-state variants only where the container does not show selection; vector formats; alt text; flip only direction-bearing icons in RTL. Document icons: folded top-right corner, 16-512 px, centre image half the canvas with a 10 % margin |
| **quire today** | 08 §1: Lucide/Tabler, 24 grid, stroke 2, round caps, one colour; `Icon` enum; sizes 11-22 |
| **Verdict** | Adapt |
| **Rule** | (a) Optical offsets: each `Icon` may carry an `OpticalNudge { dx, dy }` in 1/24 units, applied inside its viewBox, so every consumer centres it geometrically; test: a gallery row of all icons in 24 px circles for review. (b) Weight: see 3.15. (c) Document icons for Files: 08 new §2.12. 08 §1.3; `ds::icon`; type |

### 3.7 Images (not retrieved)

| | |
| --- | --- |
| **Apple** | Page not retrieved. Observed (L): assets at 1x and 2x, vector where possible |
| **quire today** | 01 §2.1 pixel snapping; 08 exports |
| **Verdict** | Covered as far as known |

### 3.8 Inclusion (2024-01-24)

| | |
| --- | --- |
| **Apple** | Address people as "you", not "users"; plain language, no idioms, humour used with care; no needless gender; approachable first run |
| **quire today** | 09 H7 onboarding voice; no writing guide |
| **Verdict** | Adopt, inside the writing rules (3.17) |

### 3.9 Layout, and concentric corners (2025-03-22)

| | |
| --- | --- |
| **Apple** | Reading order top-leading first; group with space, backgrounds or separators; align to ease scanning; keep critical controls away from a window's bottom edge (people push windows off-screen); respect the camera housing. This era has no explicit concentric-radius rule; the widgets page asks that inner corners follow the container (`ContainerRelativeShape`, 23 §8.1 W3) |
| **quire today** | 01: grid, spacing scale, radii table (01 §10). Radii are chosen per element: card 12/12/12/4 inside a window of 18 at an 8 px inset, fields 10, menu 12 with highlight 6 at a 5 px inset. Nothing derives one from another |
| **Verdict** | Adapt: add the concentric rule in its older, implicit form |
| **Rule** | `inner = max(outer - inset, floor)` with `floor = 4`: a highlight inside a 12 px menu at a 5 px inset is 7, a card inside the 18 px window at 8 px is 10 (the card's flap corner stays 4 by design). `Radius::inner(outer, inset)` in `ds::geometry`; tokens `--r-*` generated from it; lint `NestedRadius` flags a raw radius on an element the markup marks `data-nested`. The focus ring is the outward case: `outer + gap` (6.4). 01 §10; type + lint |

### 3.10 Inclusive colour and charts

Folded into 3.1 (colour alone) and 5.25 (charts).

### 3.11 Materials, vibrancy and the label hierarchy (2025-02-07)

| | |
| --- | --- |
| **Apple** | Materials blur and modify what is behind; **vibrancy** pulls colour from behind into foreground text, symbols and fills; "avoid using nonvibrant colors on top of" a material; choose by meaning (window, menu, popover, sidebar, title bar …), not by look; behind-window blending for menus, sheets, sidebars; within-window for toolbars over scrolling content. Menus are vibrant by default. Dark-mode page: primary, secondary, tertiary, quaternary label colours |
| **quire today** | 03 §17: eight materials, tint over compositor blur, stack v2, vibrancy baked into the tint (Blitz cannot blend what is behind, 03 §17.4). Inks: `--ink`, `--ink-soft`, `--ink-faint` on the card; `--f-ink*` on the frame (02 §8) |
| **Verdict** | Adapt: true vibrancy is out of reach; approximate it per material |
| **Rule** | `Surface(material)` redefines `--ink`, `--ink-soft`, `--ink-faint`, `--ink-quaternary` and the separator inside its scope to values fitted to that material's tint (the "vibrant" variant of 3.4), so components never pick inks per material. Four label levels everywhere: primary, secondary, tertiary, quaternary (watermarks, disabled glyphs). 03 §17.4 and 02 §8; `ds::material`; test: every ink on every material over black and white passes its level's floor |

### 3.12 Motion (2025-05-08) and [FLUID]

| | |
| --- | --- |
| **Apple** | "Add motion purposefully"; "make motion optional" (never the only carrier); feedback motion "follows people's gestures"; "brevity and precision"; avoid motion on frequent interactions; **"Let people cancel motion … don't make people wait for an animation to complete"**. [FLUID]: interruption and redirection at any moment; describe springs by damping and response, not duration; 100 % damping by default, 80 % when the gesture has momentum; carry the gesture's velocity into the animation; throw to the endpoint nearest the projected position (`p + v·r/(1-r)` with the scroll rate r = .998 per ms, about `p + 0.5 s × v`); track 1:1; rubber-band at limits |
| **quire today** | 05: CSS keyframes and transitions with duration tokens; Rust `settle()` timers; `use_pulse` restarts by alias swap. Built exceptions: scroll physics (11, velocity and momentum), dock magnification (10, no easing), swipe-to-dismiss (`motion/swipe.rs`: speed threshold 600 px/s, flies out "from where it is"), `PaneSwitcher` ("a switch mid-slide reverses"). 26 R10: a Sweep retargets from its current share; springs are `cubic-bezier` overshoots (`--e-spring`) of fixed length, e.g. 420 ms |
| **Verdict** | Adapt: keep keyframes for state moments no hand touches (Appear, Dismiss by timeout, bump, gulp); every motion a hand drives or can interrupt becomes a Rust spring |
| **Rule** | (1) `Spring { damping: Ratio, response: Millis }` integrated per frame in Rust, writing a `--x`/`--f` custom property on an HTML wrapper; retargeting keeps position and velocity (no jump, no restart). (2) `Touch::Contact(Velocity)`: a release hands its velocity to the spring; `SpringSpec::for_touch` gives damping 1.0 for a tap or key, 0.8 for a gesture with momentum toward the target ([FLUID]); response `--spring-quick` 300 ms and `--spring-move` 450 ms (proposed; to be tuned beside macOS). (3) Throws pick the endpoint nearest `p + 0.5 s × v`, not nearest `p`: notification swipe, dock drag-out return, sheet and panel drag, workspace swipe (12 §12.3.7), the slider knob released with speed. (4) Any new target mid-motion retargets; nothing blocks input (26 R10 generalised). (5) Under Reduced: damping 1.0, no projection overshoot, 1:1 tracking kept. Where: 05 new §14 "Driven motion"; `ds::motion::spring`; 26 §4.1 `Touch`. Tests: interrupt at 40 % toward a new target, assert position continuous and velocity continuous within 5 %; a throw at 1500 px/s from 30 % lands on the far endpoint; idle after settle paints 0 frames (26 R3) |

What converts, in order: sheet and panel present/dismiss, notification banner swipe and return,
control-center pane slide (`PaneSwitcher`), toggle knob, segmented and switcher selection
indicator, slider knob on release, dock icon drag return, launcher open/close. What stays
keyframes: `rise`, `fold`, `curl`, `gulp`, `bump`, `seal-pop`, `shake-x`, `pop-in` for arrivals
no one touched.

### 3.13 Privacy (2024-12-30)

| | |
| --- | --- |
| **Apple** | Ask only for what a feature needs, at the moment it is used, never at launch unless required; a purpose string that is one active sentence ("The app records during the night to detect snoring sounds", not "needed for a better experience"); a pre-alert screen has one button that opens the system alert; macOS: sandboxing, signing, "avoid making assumptions about who is signed in". The page does not describe the menu-bar recording indicators; on macOS 14/15 an orange dot marks microphone use, green marks camera, and a purple glyph marks screen recording, with the app named in Control Center (L) |
| **quire today** | Nothing |
| **Verdict** | Adopt |
| **Rule** | (a) sill shows an in-use indicator in the bar for camera (PipeWire video capture nodes), microphone (capture streams) and screen cast (portal sessions): one dot per kind, in the kind's colour **and** a glyph (3.1 colour alone), the app names in the control center; the dot cannot be hidden by settings. (b) Portal permission dialogs (our polkit-style prompt) carry a purpose string from the requesting app, shown as one sentence. 20 new §1.18; sill `privacy` service; test: a fake capture node shows the dot within one frame of the node appearing |

### 3.14 Right to left, localization, CJK and input methods (2025-01-08; layout, inclusion)

| | |
| --- | --- |
| **Apple** | RTL: mirror layout and progress direction, never mirror digits or logos, align paragraphs by their own language, flip direction-bearing icons only. Layout: handle locale formats and text length. Keyboards: Control+Space and Control+Option+Space switch input sources; shortcuts are localized to the keyboard. SF Symbols ship Chinese, Japanese and Korean variants |
| **quire today** | 02 §2: Inter, Space Mono, Noto Serif, cut to **Latin and Latin-ext subsets**; no CJK face, fallback order or line-break rule. `ds::edit` handles IME preedit and commit (`edit/composition.rs`). No RTL |
| **Verdict** | Adopt for CJK (the desktop is used with zh-TW text and input); Skip RTL for now (no RTL locale in use; revisit when one ships) |
| **Rule** | (a) A CJK fallback per script in `--font-ui`: Noto Sans CJK TC (or Source Han Sans TC) for zh-TW, then SC, JP, KR by locale, registered with the renderer like the Latin faces; tracking 0 for CJK runs (the Latin tracking tokens must not space Han characters). (b) Line breaking: CJK breaks between ideographs; no truncation mid-cluster; `clip_chars` counts grapheme clusters. (c) Preedit: the composing text draws underlined in the field, the candidate window anchors to the caret rect the field reports (text-input-v3 `set_cursor_rectangle`); Escape during composition cancels the composition, not the layer (06 §18 gets a priority 0 row). (d) Dates and numbers through the locale (the clock, calendar widget). 02 new §12; `ds::fonts`, `ds::edit`; tests: a zh-TW string renders with no `.notdef` glyph; `clip_chars` never splits a cluster; Escape with an active preedit leaves the menu open |

### 3.15 SF Symbols (2025-05-08)

| | |
| --- | --- |
| **Apple** | Nine weights matching the text weights, so a symbol matches adjacent text; three scales (small, medium, large) relative to the text's cap height; rendering modes: monochrome, hierarchical (one colour, opacity per layer), palette, multicolour; variable colour for a level (layers light at thresholds; the speaker's waves); design variants: outline in toolbars and lists, fill for selection, slash for unavailable, enclosed for small sizes; animations: appear, disappear, bounce (once), scale (persists), pulse, variable colour (cumulative or iterative), replace (down-up, up-up, off-up), magic replace, wiggle, breathe, rotate; "apply symbol animations judiciously" |
| **quire today** | 08 §1: one stroke weight (2 on the 24 grid) at every size and beside every text weight; one colour. 26 §2 and §3 take the animation vocabulary (Appear, Pending as bounded variable colour, Replace as `MorphStyle::{DownUp, OffUp, CrossFade, Slash}`) |
| **Verdict** | Adapt: weight tracking and scales Adopt; rendering modes Skip beyond monochrome and a two-level hierarchy (00 §3 "one weight, one colour" is ours); animations Covered by 26 |
| **Rule** | (a) `Glyph { icon, size, weight: GlyphWeight }` where `GlyphWeight` comes from the adjacent text's weight token: 400 → stroke 1.75, 500 → 2.0, 600 → 2.25, 700 → 2.5 (on the 24 grid; proposed; 00 §3's "2 px stroke" becomes the 500 row). A component passes its label's weight; no free stroke. (b) `SymbolScale::{Small, Medium, Large}` sizes an inline glyph from the text's size token (proposed: 1.0, 1.2, 1.4 × the font size), replacing hand-picked `IconSize` next to text. (c) Hierarchical: a glyph may mark one layer secondary, drawn at the `--ink-soft` level of the same colour. 08 §1.2, §1.4; `ds::icon`; type |

### 3.16 Typography (2024-11-06)

| | |
| --- | --- |
| **Apple** | Minimum size **10 pt on macOS**; avoid Light and thinner; few typefaces; macOS text styles (size/line height, weight): Large Title 26/32 Regular, Title 1 22/26, Title 2 17/22, Title 3 15/20, Headline 13/16 **Bold**, **Body 13/16 Regular**, Callout 12/15, Subheadline 11/14, Footnote 10/13, Caption 1 10/13, Caption 2 10/13 Medium; emphasized weights one step up (Bold, Semibold, Heavy for Headline). Tracking tightens with size: +12/1000 em at 10 pt, +6 at 11, 0 at 12, −6 at 13, −11 at 14, −16 at 15, −20 at 16, −26 at 17, then easing back to +14 at 28-30 and 0 by 80. Standard control fonts by role: control content, label, menu, menu bar, message, palette, tooltips |
| **quire today** | 02: Inter System typeface (settled 2026-09-26); base 15 px / 1.55 (`S`); sizes down to 9.5 (eyebrow `--fs-micro`) and 8 (`--fs-dial`); tracking 0 at body; uppercase tracked caps for section headers (0.06em under System); menus 13 (13.3.3) |
| **Verdict** | Adapt: take the Mac ramp for the System typeface's UI roles; keep the Editorial ramp for mail's opt-in voice |
| **Rule** | (a) `--fs-min` = 10: lint `MinFontSize` rejects a UI text size below it (widget dial numerals, which are drawings, carry an exemption marker). (b) Under System, the UI roles map to the Mac styles: body and controls 13/16, secondary lines 11/14, section headers 11 Bold in `--ink-soft` in **title case, no uppercase, no tracking** (Mac sidebar headers, L), window titles 13/600 (built), menu 13 (built). (c) Tracking under System follows the table above by size (Inter's own metrics are close; the token table records the size-to-tracking map, not per-role guesses). Open decision 1: this moves the desktop's density from the prototype's 15 px to the Mac's 13 px. 02 §4.1, §5; `ds::tokens::type`; lint |

### 3.17 Writing (2024-11-06), with alerts, menus, buttons, help

| | |
| --- | --- |
| **Apple** | Decide a voice and keep it; plain words; action labels start with a verb; "Next" over "Let's do this!"; pick title case or sentence case per element and keep it; "you" (inclusion); empty states say what to do next; errors near the problem, no blame, say how to fix ("Choose a password with at least 8 characters"), no "oops". Component pages set the Mac pattern: **title case** for buttons, menu items, menu titles, segment labels, column headings and tab labels; **sentence case** with punctuation for alert informative text and notification bodies; box and slider labels in sentence case ending with a colon in settings panes; **"…" when an item opens a view that asks for more input**; alerts: "Cancel" always titles the cancel button, avoid "OK" except in purely informational alerts, never "Yes"/"No"; help tags (tooltips) 60-75 characters at most, start with a verb, do not repeat the control's name |
| **quire today** | Nothing beyond 09 H7 ("short, warm, no exclamation marks") |
| **Verdict** | Adopt |
| **Rule** | A writing section in 02 (new §13): the case table per element, the ellipsis rule, the alert button vocabulary, the tooltip limit, "you" not "the user". Enforced where text is data: markup lint `ThreeDots` ("..." in any label), `TitleCaseLabel` warns on a lower-case second word in `Button`, `MenuEntry` and segment labels (an allowlist for articles and short prepositions), a test that `Tooltip` text is ≤ 75 characters. `ds::lint::markup`; doc + lint + test |

## 4. Patterns

### 4.1 Drag and drop (2023-10-05)

| | |
| --- | --- |
| **Apple** | Show the drag image once the pointer moves **about three points**; translucent drag image; highlight a destination only if it accepts; on a failed drop the item returns to its source or "evaporates"; a count badge for multi-item drags; Option at drop time copies; auto-scroll near edges; keep the dropped content selected; drag from an inactive window without activating it; drag cursors (copy, link, not allowed, disappearing item); offer a menu alternative; undo a drop |
| **quire today** | `DRAG_THRESHOLD` 8 px Manhattan (`motion/drag.rs`, 04 §34); `DragGhost`, `DropLine`; dock drag-out (10 §10.3.8) |
| **Verdict** | Adapt |
| **Rule** | Threshold 3 px Euclidean for content drags (keep 4 px for window move, 13.3.11); a failed drop springs the ghost back to its source with the release velocity (3.12); `DragGhost { count: Option<Count> }` shows the badge; drop outcome sets the cursor (6.3); Option held at release copies. 06 §6; `ds::motion::drag`; test: 3 px starts a drag, 2 px does not; a failed drop settles at the source rect |

### 4.2 Entering data (2023-11-02)

| | |
| --- | --- |
| **Apple** | Gather from the system instead of asking; clear hints; secure fields for secrets; never prefill passwords; choices over typing; validate as people type; disable Next until required data exists; macOS: an **expansion tooltip** shows a truncated field's full text on hover |
| **quire today** | `TextInput`, `SearchField`, `LockPrompt`; 26 G53 proposes `FieldState::Invalid` |
| **Verdict** | Adopt the expansion tooltip; the rest Covered |
| **Rule** | A truncated `.ds-truncate` field or cell gets a `Tooltip{Fly}` with its full text after the hover intent (06 §3). 04 §6, §18; test |

### 4.3 Feedback (2023-10-07)

| | |
| --- | --- |
| **Apple** | Match the delivery to the significance; status near the item; alerts only for critical, actionable information; warn only on unexpected irreversible loss (the Finder does not warn on every delete); confirm only significant completions; say why a command cannot run |
| **quire today** | 00 §4 motion principles; 26 moments; undo toast |
| **Verdict** | Covered |

### 4.4 File management (2023-11-02)

| | |
| --- | --- |
| **Apple** | New/Open/Open Recent in the File menu; save automatically; "Untitled" for new documents; hide extensions by default; when autosave is off, a dot on the close button and beside the name in the Window menu marks unsaved changes |
| **quire today** | Files app planned (20 §2.3); no document model |
| **Verdict** | Adopt when the first document app lands |
| **Rule** | `WindowFrame::Titlebar { edited: Edited::{Saved, Unsaved} }` draws the dot in the close light only when the app declares `Autosave::Off`. 04 window frame; type |

### 4.5 Going full screen (2024-03-18)

| | |
| --- | --- |
| **Apple** | Full screen when it serves the task; people choose when to leave; the Dock stays revealable; the toolbar can hide until the pointer reaches the top; Mission Control always reachable; resume where people left off |
| **quire today** | 05 §9 rule 12: fullscreen surfaces never animate; 13.3.11 zoom |
| **Verdict** | Adopt: bar reveal in fullscreen |
| **Rule** | In a fullscreen workspace, pointer at the top edge for the hover intent (450 ms) reveals the bar over the app; the dock reveals at the bottom edge with 10's auto-hide numbers. 13 §13.3.1; sill |

### 4.6 Launching, and state restoration (2023-11-02)

| | |
| --- | --- |
| **Apple** | No launch screen on macOS; no setup before value; permissions at use; **"Restore the previous state when your app restarts … Restore granular details"** (scroll position included). Settings page: reopen the last pane. Outline views: remember expansion |
| **quire today** | Nothing (focus hand-back exists, 06 §17) |
| **Verdict** | Adopt |
| **Rule** | `trait Restorable { type State: Serialize + DeserializeOwned; fn save(&self) -> Self::State; fn restore(state) }` on window-level state: window size (placement is the compositor's on Wayland), open panes and their widths, sidebar expansion, selected item, scroll offset, last settings pane. Stored per app under `$XDG_STATE_HOME/<app>/restore.toml`, written on change (debounced 700 ms, as the draft autosave) and on quit. 06 new §23; `ds::restore`; test: save, relaunch the harness, same state |

### 4.7 Loading (not retrieved)

| | |
| --- | --- |
| **Apple** | Page not retrieved. Progress indicators page (5.21) covers the substance |
| **quire today** | 26 Pending, R4 grace 400 ms |
| **Verdict** | Covered by 26 |

### 4.8 Managing accounts (2023-11-02)

| | |
| --- | --- |
| **Apple** | Ask for an account only when the core needs it; delay sign-in; name the method; never "passcode"; deletion as easy as creation |
| **quire today** | mailo accounts; no shell accounts |
| **Verdict** | Covered for now (mailo owns it) |

### 4.9 Managing notifications, and the notifications page (2025-05-02; 2025-05-31)

| | |
| --- | --- |
| **Apple** | Interruption levels: Passive, Active (default), Time Sensitive, Critical; only Time Sensitive and Critical break through a Focus; Critical also through silence. Content: short title, sentence case, full punctuation, no app name (the icon is shown), no truncation by the app; up to **four** actions, title case, no "Open" action, prefer non-destructive; badges count unread notifications only; macOS notification sounds mix with other audio |
| **quire today** | 13.3.6: banners, 5 s, hover expand, actions as `Button{Mini}`, swipe, grouping, Do Not Disturb passes only urgency Critical |
| **Verdict** | Adapt |
| **Rule** | Map freedesktop urgency and hints to levels: `low` → Passive (history only, no banner), `normal` → Active, `critical` → Critical; an `x-quire-time-sensitive` hint → Time Sensitive, which passes a Focus. Cap actions at 4. 13.3.6; sill; test per level against DND on and off |

### 4.10 Modality (2024-03-25)

| | |
| --- | --- |
| **Apple** | Modal only with a clear benefit; short and simple; title the task; always an obvious dismissal; confirm before losing content; one modal at a time; alerts are the one thing that may appear over a popover |
| **quire today** | `Scrim`, `Sheet`, `Peek` (04 §24); `LayerStack` Escape order (06 §18); O-15: Peek and Sheet neither move nor trap focus |
| **Verdict** | Adopt |
| **Rule** | `ModalScope` owns focus: on open, focus the first field or the default button; Tab cycles inside; on close, focus returns (the existing `Focus::Controlled` hand-back); a second modal request while one is open is refused (`Result`), except an alert. 04 §24 (closes O-15); `ds::focus`; type + test |

### 4.11 Multitasking (2024-02-25)

| | |
| --- | --- |
| **Apple** | Pause attention-bound work on switch-away, resume on return; finish user-started tasks in the background; duck or pause for audio interruptions |
| **quire today** | Not specified |
| **Verdict** | Adopt for media: the media player pauses on screen lock and on a call (MPRIS); the rest Covered by the compositor |

### 4.12 Onboarding (2023-10-07)

| | |
| --- | --- |
| **Apple** | Fast, fun, optional; teach by doing; inline first-use tips over a separate flow; skippable and findable later; no licences |
| **quire today** | 09 H7 |
| **Verdict** | Covered |

### 4.13 Offering help (2023-11-02)

| | |
| --- | --- |
| **Apple** | Tips for simple features only (≤ 3 steps), one or two sentences; macOS help tags describe only the control under the pointer, start with a verb, 60-75 characters, do not repeat the label |
| **quire today** | `Tooltip{Fly, Card}` (04 §18), hover intent 450 ms |
| **Verdict** | Adopt the text rules (3.17) |

### 4.14 Playing audio, and UI sounds (2024-01-18)

| | |
| --- | --- |
| **Apple** | The system volume governs output; use the system volume view; respect headphone routing (pause on unplug); don't repurpose media keys; macOS notification sounds mix with other audio |
| **quire today** | 13.3.10: freedesktop sound theme, screenshot, trash, volume step (once per press), dock poof, banner, critical alert, `bell`; settings `sound.ui_sounds`, `sound.volume_feedback` |
| **Verdict** | Covered; add pause-on-unplug |
| **Rule** | Media player pauses when the default sink changes from headphones to speakers (the Mac does). 20 §2.6; sill audio service |

### 4.15 Printing (not retrieved)

| | |
| --- | --- |
| **Apple** | Page not retrieved. Keyboards page: Cmd+P prints, Shift+Cmd+P page setup |
| **quire today** | Nothing |
| **Verdict** | Skip until a document app ships; reserve the shortcuts (6.2) |

### 4.16 Searching (2024-02-10)

| | |
| --- | --- |
| **Apple** | Index content for system search; custom file types describe their metadata; use the system open/save panels (they have search) |
| **quire today** | Launcher (13.3.9) searches apps and commands |
| **Verdict** | Adapt later: the launcher gains a provider interface for app content (mail, notes) |

### 4.17 Settings (2023-11-02)

| | |
| --- | --- |
| **Apple** | Task options in context, app-wide options in Settings; few settings; don't duplicate system settings in apps; macOS: Settings opens from the App menu with **Cmd+,**; a toolbar of panes that always shows the active one; title "<App> Settings" or the pane's name; minimize and zoom dimmed; reopen the last pane |
| **quire today** | 22: every proposed value is a key (the user's rule, H6); Settings app planned with a sidebar (09 H2) |
| **Verdict** | Adapt: our Settings is one system app with a sidebar (the macOS 13+ System Settings shape), and apps' own settings follow the pane rules |
| **Rule** | App settings windows: Cmd+, opens, minimize and zoom lights disabled, last pane restored (4.6), window title tracks the pane. 04 window frame (`TrafficLights` per-light availability); 22 §5 |

### 4.18 Undo and redo (2023-10-07)

| | |
| --- | --- |
| **Apple** | Undo many times, no needless limit; show what an undo changed, scrolling to it if needed; name the target ("Undo Paste and Match Style", "Undo Typing"); batch related micro-changes; undo and redo at the top of the Edit menu with **Cmd+Z** and **Shift+Cmd+Z**; buttons only where needed |
| **quire today** | Mail's undo toast and pull tab for list operations (06 §9); `ds::edit` fields have no undo stack specified |
| **Verdict** | Adopt |
| **Rule** | `UndoStack<Op>` in `ds::edit` for every `TextInput` and editable list, typing coalesced per word or 1 s pause; `Op::title()` names it for the Edit menu; Cmd+Z / Shift+Cmd+Z bound through the reserved table (6.2); the toast stays as the visible affordance for list operations and uses the same stack. 06 §9, new §24; test: type, undo, redo round-trips; titles match |

### 4.19 Collaboration and sharing (2023-11-02)

| | |
| --- | --- |
| **Apple** | System share sheets and collaboration popovers |
| **quire today** | Nothing |
| **Verdict** | Skip (no sharing service on this desktop yet) |

## 5. Components

### 5.1 Menus (2024-03-31)

| | |
| --- | --- |
| **Apple** | Items are verbs, title case, no articles; "…" when more input follows; toggles by changing title (Show/Hide), by a checkmark, or by a pair; dim unavailable items but keep the menu openable; group with separators, most-used first; submenus one level, about five items, marked with a chevron |
| **quire today** | 04 §20 `Menu` (Rich, Slim, Dropdown, Context); 13.3.2-13.3.4 tracking, geometry (22 px rows, 13 px text, 22 px check column, separators, safe triangle, 200 ms submenu delay); first open `menu-pop` spring; highlight `--accent-soft`; menus radius 12 |
| **Verdict** | Adapt |
| **Rule** | (a) Text menus (Dropdown, Context, bar) open with **no animation** and close with the `--t-quick` fade (macOS R3; closes 05 §12 item 6 and 06 open decision 16 in favour of the Mac); Rich keeps `menu-pop`. (b) Highlight: accent fill with `--accent-ink` text and glyphs (L; `selectedMenuItemTextColor`), not `--accent-soft`. (c) Pick blink: the chosen item flashes once (off 60 ms, on 60 ms) before the menu closes (L; 13.9 item 3). (d) Radius: highlight 4-5, menu about 6 on the Mac (L, measure against [KIT]); ours follows 3.9 from whatever outer radius is settled. 13.3.2-13.3.3; `ds::components::menu`; test per kind |

### 5.2 The menu bar (2024-12-14)

| | |
| --- | --- |
| **Apple** | Leading side: system menu, then the active app's menus in order **App (name in bold), File, Edit, Format, View, app-specific, Window, Help**; trailing side: menu bar extras, which open a **menu, not a popover**, unless the content is too complex. App menu: About, Settings… (Cmd+,), Services, Hide (Cmd+H), Hide Others (Opt+Cmd+H), Show All, Quit (Cmd+Q). Edit: Undo, Redo, Cut, Copy, Paste, Paste and Match Style, Delete, Select All, Find, Spelling, Substitutions, Transformations, Speech, Dictation, Emoji & Symbols. View: toolbar, sidebar, tab bar, Enter Full Screen. Window: Minimize, Zoom, tabs, Bring All to Front, window list. Help: search and the app's help. Every toolbar command also lives in a menu. Dynamic items change with Option. One-word menu titles |
| **quire today** | Bar left: app name and workspace indicator (00 §8.4, 20 §1.1); right: tray and status items; every status item opens a ds `Menu` or a popover (control center, calendar) |
| **Verdict** | Adopt for our own apps; Adapt for foreign apps |
| **Rule** | (a) `ds::commands::MenuModel`: the standard menus and items as data (`StandardMenu`, `StandardItem` with their reserved shortcuts, 6.2), plus app sections; ds-native apps export it over a session-bus interface (the `com.canonical.dbusmenu` format existing global-menu implementations read, so foreign Qt and GTK apps that export one also appear). (b) sill's bar renders the active window's model with the one menu machine (13.3.2 hover switch applies). (c) Foreign apps without a model show only the App menu (Hide, Quit, which sill performs through the toplevel protocol). (d) Status items open menus; the control center and calendar stay popovers (they are too complex for a menu, the page's exception). 13 new §13.3.13; `ds::commands`; sill bar; test: a harness app's Edit menu lists Undo with its title |

### 5.3 Dock menus (2025-04-07)

| | |
| --- | --- |
| **Apple** | Open windows and a few high-value actions; every item also available elsewhere |
| **quire today** | 10 §10.3.7: windows, desktop actions, Keep in Dock, Quit |
| **Verdict** | Covered |

### 5.4 Context menus (2025-02-24)

| | |
| --- | --- |
| **Apple** | Relevant items only, few, at most about three groups; available elsewhere too; **hide** unavailable items (not dim); **no keyboard shortcuts** in context menus; rarely a title; one submenu level |
| **quire today** | `Menu{Context}` shares the text menu metrics; 09 H5 "every menu item shows its shortcut" |
| **Verdict** | Adopt; this overrides H5 for context menus only |
| **Rule** | `MenuKind::Context` takes items without a `shortcut` field and filters `Availability::Disabled` out when built. 04 §20; 09 H5 note; type |

### 5.5 Edit menus (2024-04-01)

| | |
| --- | --- |
| **Apple** | On macOS the Edit menu in the bar plus a context menu on selected content; offer only applicable commands; selectable static text; support undo |
| **quire today** | `SelectionBubble` in the composer (mail) |
| **Verdict** | Adopt: a right click in any `TextInput` opens a Context menu with Cut, Copy, Paste, Select All (and Undo); labels and static text are selectable where useful (error text, addresses). 04 §6; test |

### 5.6 Popovers (2024-09-04)

| | |
| --- | --- |
| **Apple** | Small amounts of content; arrow points at the source; don't cover the source; save work on outside close; **one popover at a time**, never a cascade; nothing over a popover except an alert; switching between bar-button popovers with one click; animate size changes; can detach into a panel |
| **quire today** | 04 §21 `Popover`, `place()`; 13.3.7 control center; bar hover switch |
| **Verdict** | Adapt: no arrow (the Mac's menu-bar popovers have none since macOS 11, L); one-at-a-time as a rule; animate size |
| **Rule** | `OverlayHost` holds at most one popover; opening another closes the first in the same frame; height changes animate with the spring (3.12) as `PaneSwitcher` does. 04 §21; test |

### 5.7 Sheets (2024-09-06)

| | |
| --- | --- |
| **Apple** | Always modal on macOS; a card with rounded corners floating on its parent window, which dims; people may still use other windows of the app; reasonable default size, resizable when useful; dismiss buttons (Done, OK, Cancel) at the bottom, trailing corner; one sheet at a time; use a panel for repeated input |
| **quire today** | `Sheet` derived from `Peek`: centred over a scrim across the surface (04 §24; 20 §1.8 power menu, 1.10 polkit) |
| **Verdict** | Adapt |
| **Rule** | Two kinds: `SheetKind::Window` (attached to its parent window's top edge below the titlebar, only that window dimmed, other windows usable; for app dialogs) and `SheetKind::Session` (centred over the output, for power and polkit where no parent window exists). Buttons bottom-trailing, default rightmost (5.8). Present and dismiss by the spring (3.12). 04 §24; type |

### 5.8 Alerts (2024-03-25)

| | |
| --- | --- |
| **Apple** | Sparingly; not merely informational; not for common undoable actions; not at launch. Title says what happened; informative text only if it adds value, sentence case; up to **three** buttons; one- or two-word verbs; **"Cancel"** always titles cancel; avoid "OK" unless informational; default button on the **trailing** side of a row (top of a stack); Cancel leading; **destructive style only for an action people did not deliberately choose** (Empty Trash's own confirmation does not style Empty Trash as destructive); never make Cancel the default; Esc and **Cmd+.** cancel; macOS: app icon, optional suppression checkbox, help button, accessory view; caution symbol sparingly |
| **quire today** | No alert component; power menu (20 §1.8) uses `Button{Danger}` for Restart and `Primary` for Shut Down |
| **Verdict** | Adopt |
| **Rule** | `Alert { icon: AppIcon, title, informative: Option<Text>, buttons: AlertButtons, suppression: Option<Suppress> }`, `AlertButtons { default: Action<Safe>, cancel: Cancel, other: Option<Action<Any>> }` (at most three; `Cancel`'s label is fixed "Cancel"; a destructive action cannot be the default). Keys: Return = default, Esc and Cmd+. = cancel. Power menu: Restart and Shut Down are deliberate choices, so neither is styled destructive. 04 new §45; 20 §1.8; type + test |

### 5.9 Panels (not retrieved)

| | |
| --- | --- |
| **Apple** | Page not retrieved. Sheets page: a panel for repeated input with visible results (find and replace). Observed (L): floating utility windows, smaller titlebar, hide when the app deactivates |
| **quire today** | Nothing |
| **Verdict** | Skip until an app needs one |

### 5.10 Windows (2024-05-03)

| | |
| --- | --- |
| **Apple** | Frame (title bar, toolbar, tab bar, rare bottom bar) and body; states **main**, **key**, **inactive** each look different: the key window's lights are coloured, others grey; a title unless content makes it obvious; document name or "Untitled", numeric suffixes from 2; no paths in titles; unsaved dot only without autosave; bottom bars for small status, never critical |
| **quire today** | 04 window frame and 13.3.11: 28 px titlebar, 12 px lights, reveal rule, zoom menu, first-click rules (13.3.8) |
| **Verdict** | Covered, one addition |
| **Rule** | Inactive windows also dim their body's accent: selections go to the unemphasized colour (6.4), the titlebar text to `--f-ink-faint` (built). 04 window frame |

### 5.11 Toolbars (2024-03-02)

| | |
| --- | --- |
| **Apple** | Top of the window, integrated with or below the title bar; frequent commands only; grouped; symbols without bezels, a hover background appears only on hover or press; no persistent selected look (exceptions: view toggles); every toolbar item is also a menu command; customizable and hideable (Opt+Cmd+T); custom icons 19×19 px (38 @2x); search collapses to a button when narrow |
| **quire today** | `IconButton{Tool}` (04 §2); 09 H5 caps toolbar item counts per surface |
| **Verdict** | Adapt |
| **Rule** | `Toolbar` component: unified with the 28 px titlebar into a 52 px frame (L) when an app has one; items are `IconButton{Tool}` with no rest background; each item names the `MenuModel` command it mirrors (type: `ToolbarItem { command: CommandId }`, so no toolbar-only action exists). 04 new §46; type |

### 5.12 Sidebars (2025-02-28)

| | |
| --- | --- |
| **Apple** | Full window height on macOS; rounded selection highlight; three sizes from the General setting: **Small** row 24 pt, 16 px icon, 11 pt text; **Medium** row 28, 20 px icon, 13 pt text; **Large** row 32, 24 px icon, 15 pt text; 17 pt horizontal spacing between cells, 0 vertical; **icons use the accent colour** by default (fixed colours only where the colour means something); at most two levels; hide it by a known command; auto-collapse when the window narrows; no edit buttons at the bottom edge |
| **quire today** | 01 §4 sidebar and 04 §19 `SidebarItem` from `S` (the Arc lineage: drawn on the Space colour, pinned tiles, Today); icons in `--f-ink-soft`; Mod+S hides it |
| **Verdict** | Adapt: the Arc sidebar stays (00 §6); take the size setting and the auto-collapse |
| **Rule** | `appearance.sidebar_size` (3.1) selects row 24/28/32, icon 16/20/24, text 11/13/15 for every sidebar; default Medium. Icon colour stays ink on the Space frame (the frame carries the hue; accent-coloured icons would put two hues on it), recorded as a deliberate difference. Auto-collapse below a width set per app. Hide/Show Sidebar moves to the View menu with Ctrl+Cmd+S (the Mac's), freeing Mod+S (6.2). 01 §4; `ds::components::sidebar_item`; type |

### 5.13 Split views (2023-11-24)

| | |
| --- | --- |
| **Apple** | Persistent selection in each pane leading to the detail; drag between panes; min and max pane sizes that keep the divider visible; hide panes with a command and shortcut; **thin divider, 1 pt** |
| **quire today** | 01 §3 panes; 09 H3 split |
| **Verdict** | Adopt the divider: 1 device-independent px hairline (`--hair`), a 6 px invisible drag zone, resize cursor (6.3), pane widths restored (4.6). 01 §3 |

### 5.14 Tab views (2025-04-07)

| | |
| --- | --- |
| **Apple** | Closely related panes; nouns as labels; at most six tabs; control on any side; inset from the window edge |
| **quire today** | `Tabs` (04 §12); 09 H2 prefers vertical lists to tab strips |
| **Verdict** | Covered (H2 already limits tabs to settings-like panes); add the six-tab cap as a debug assertion |

### 5.15 Segmented controls (2025-04-16)

| | |
| --- | --- |
| **Apple** | Closely related choices, not actions; equal segment widths; text or images, not both; nouns, title case; at most five to seven; icon sizes Regular 17×17, Small 14×13, Mini 12×11 px @1x; use a tab view, not a segmented control, for switching main-area views |
| **quire today** | 04 §3 `SegmentedControl` (2-4 segments) |
| **Verdict** | Adapt: equal widths and the icon sizes; the selection indicator slides by the spring (3.12) |

### 5.16 Toggles (not retrieved)

| | |
| --- | --- |
| **Apple** | Page not retrieved. Observed (L): macOS prefers a **checkbox** for most on/off settings and in lists of options, and a switch for a single prominent setting; switch sizes regular, small, mini |
| **quire today** | `Toggle` (switch) only; O-4 size open; no checkbox, no radio group |
| **Verdict** | Adopt a checkbox and a radio group |
| **Rule** | `Checkbox { label, value: Check }` and `RadioGroup<T>`; settings lists use checkboxes, a pane's master switch uses `Toggle`. 04 new §47-48 |

### 5.17 Sliders (2024-04-07)

| | |
| --- | --- |
| **Apple** | Minimum leading, maximum trailing; optional end icons; macOS: tick marks (directional thumb when present, round otherwise), circular sliders, live feedback, a label in sentence case with a colon; pair with a text field and stepper for exact values |
| **quire today** | 04 §5 `Slider`; drag with no tween, rubber band, press swell, `LevelTick` (26 §5.10.3) |
| **Verdict** | Adapt: add `ticks: Ticks::{None, Every(step)}` with snapping; the knob's release uses the spring (3.12) |

### 5.18 Steppers (not retrieved)

| | |
| --- | --- |
| **Apple** | Page not retrieved. Observed (L): paired up/down arrows beside a numeric field |
| **quire today** | Nothing |
| **Verdict** | Adopt when Settings needs numeric keys (22 has many): `Stepper` bound to a `TextInput{numeric}` |

### 5.19 Pickers, date pickers (2024-07-31)

| | |
| --- | --- |
| **Apple** | Predictable, ordered values; show in context; coarser minute steps where they divide 60; macOS: textual and graphical date pickers |
| **quire today** | `MonthGrid` (04 §39) for the calendar widget |
| **Verdict** | Adapt later: `DatePicker{Textual, Graphical}` from `MonthGrid` when Settings needs one |

### 5.20 Text fields, search fields, token fields, text views, combo boxes (2023-06-21; 2023-07-10; 2023-10-07; token fields and combo boxes not retrieved)

| | |
| --- | --- |
| **Apple** | Placeholder as a hint, not a label; size matches expected text; logical Tab order; validate at the right time; number formatters; expansion tooltip; combo box for text plus choices. Search: descriptive placeholder (not "Search"), clear button, search while typing or on Return, no label needed in content areas, toolbar placement usual. Text views: selectable useful text |
| **quire today** | `TextInput`, `SearchField`, `Chip{Token}` (04 §6, §7, §10) |
| **Verdict** | Covered, plus the clear button and the combo box |
| **Rule** | `SearchField` shows a clear button when non-empty (Esc also clears, then a second Esc leaves); `ComboBox` = `TextInput` + `Menu{Slim}` with typing filter (06 §2.4 already filters). 04 §7 |

### 5.21 Buttons, pop-up and pull-down buttons (2025-02-24; 2023-11-13; 2023-11-02)

| | |
| --- | --- |
| **Apple** | One or two prominent buttons per view; style, not size, marks the preferred choice; roles Normal, Primary (answers Return), Cancel, Destructive (red; never primary); title-case verbs; "…" when a view opens. macOS: push buttons; gradient buttons (symbols only) under tables; help button (circle with "?", one per window, bottom corner opposite the dismiss buttons); image buttons with about 10 px padding. Pop-up: mutually exclusive choices, shows the current one. Pull-down: commands, at least three, no title unless useful |
| **quire today** | 04 §1 Button (Primary, Secondary, Mini, Quiet, Danger); `Menu{Dropdown}` |
| **Verdict** | Adapt |
| **Rule** | `ButtonRole::{Normal, Primary, Cancel, Destructive}` separate from visual variant; Primary answers Return in its window or sheet; a `Destructive` role cannot be `Primary` (type). `PopUpButton<T>` (shows the choice, `Menu{Dropdown}` with a check) and `PullDownButton` (commands). 04 §1, new §49; type |

### 5.22 Disclosure controls, boxes (2024-09-06; 2025-04-07)

| | |
| --- | --- |
| **Apple** | Triangle points trailing when closed, down when open, with a descriptive label; one disclosure button per view. Boxes: small relative to the view, sentence-case title above, colon in settings panes, no nesting |
| **quire today** | `SectionHeader` fold (04 §13) |
| **Verdict** | Adopt `Disclosure` with the triangle rotation by the spring; boxes Covered by `ModulePanel` |

### 5.23 Progress indicators and gauges (2025-02-24)

| | |
| --- | --- |
| **Apple** | Determinate when possible; switch indeterminate to determinate when the length becomes known; **never switch circular to bar**; keep moving; context text, not "Loading"; cancel when safe; macOS: spinners for background work, unlabeled; bar indeterminate style exists. Gauges and level indicators: capacity (continuous or discrete, green by default), rating, relevance |
| **quire today** | `Spinner` (loops, 26 G15/G45), `SendPill`, `BatteryLevel`; 26 R9 determinate first |
| **Verdict** | Covered by 26 (R4 bounds the loop, R9 the switch rule); add `LevelIndicator{Continuous, Discrete}` for storage and battery lists |

### 5.24 Lists and tables, outline views, column views, collections (2023-09-20; 2023-09-22; 2024-03-31; 2024-09-06)

| | |
| --- | --- |
| **Apple** | Text in rows; sortable column headings (click again reverses), title-case nouns without colons; resizable columns; alternating row colours in wide multi-column tables; outline views: hierarchy only in the first column, Option-click expands all, remember expansion, **centred ellipsis** for long cell text, single click to edit a name; column views: root in the first column, a preview when a leaf is selected; collections: standard grid, animate insert, delete, reorder |
| **quire today** | `ListRow` (mail rows), `.ds-truncate` fades the end |
| **Verdict** | Adopt when Files lands |
| **Rule** | `Table<Row>` with `Column { title, sort: Sort, width: Resizable }`, alternating rows via `--row-alt`, `Truncate::{End, Middle}` (middle for file names). 04 new §50; `ds::text::clip` gains middle clipping; test |

### 5.25 Charts (2025-04-22)

| | |
| --- | --- |
| **Apple** | Describe the chart; accessible to screen readers and keyboard; never colour alone; separators between adjacent colour areas |
| **quire today** | Nothing (widgets draw rings, not charts) |
| **Verdict** | Skip until a chart exists; 3.1's colour rule applies |

### 5.26 Scroll views (2024-09-18)

| | |
| --- | --- |
| **Apple** | System gestures and keys; show that content scrolls; no same-axis nesting; auto-scroll only as far as needed; scroll bars transient unless the setting says always; don't shift content when they appear |
| **quire today** | 11 (host physics, overlay scrollbar, `scroll.scrollbars`) |
| **Verdict** | Covered |

### 5.27 Color wells, rating indicators, image wells, path controls (2024-07-13; 2025-02-24; the last two not retrieved)

| | |
| --- | --- |
| **Apple** | Colour wells open the system picker and accept drag and drop; ratings show whole stars only, evenly spaced |
| **quire today** | `SpaceEditor` field (its own picker) |
| **Verdict** | Skip (no need yet) |

## 6. Platform and input

### 6.1 Designing for macOS (2025-02-11)

| | |
| --- | --- |
| **Apple** | Large displays: more content, fewer nested levels, less modality; resizable windows and full screen; **the menu bar gives access to all commands**; precise pointer input; **keyboard shortcuts** and keyboard-only work; personalization (toolbars, windows, colours, fonts); viewing distance 1-3 ft |
| **quire today** | 00 §6 "UX is Mac" |
| **Verdict** | Covered as a stance; 5.2 and 6.2 carry the two gaps it names |

### 6.2 Keyboards and standard shortcuts (2025-02-25)

| | |
| --- | --- |
| **Apple** | Support Full Keyboard Access; **respect standard shortcuts**, never repurpose one; a table of about 100 standard shortcuts (selection: Cmd+Space Spotlight, Cmd+Tab and Shift+Cmd+Tab, Cmd+\` next window of the app, Cmd+, settings, Cmd+. cancel, Cmd+? help, Cmd+A/C/V/X/Z, Shift+Cmd+Z, Cmd+F/G/Shift+Cmd+G, Cmd+H, Opt+Cmd+H, Cmd+M, Cmd+N, Cmd+O, Cmd+P, Cmd+Q, Cmd+S, Shift+Cmd+S, Cmd+T Fonts, Opt+Cmd+T toolbar, Cmd+W, Opt+Cmd+W, Ctrl+Cmd+F full screen, Opt+Cmd+D Dock, Shift+Cmd+3/4 screenshots, Ctrl+Space input source, Ctrl+F1-F7 keyboard access, Opt+Cmd+Esc Force Quit); custom shortcuts only for frequent commands; **Command is the main modifier**, Shift secondary, Option sparingly, **avoid Control** (the system uses it); list modifiers in the order **Control, Option, Shift, Command** (⌃⌥⇧⌘); no Shift for the upper character of a key (Cmd+? not Shift+Cmd+/); don't make a new shortcut by adding a modifier to an unrelated one |
| **quire today** | 06 §2.1 from `S`: Mod+T and Mod+K open the command menu, **Mod+S toggles the sidebar** (Cmd+S is Save), Mod+1..9 switch Spaces; "Mod" = Ctrl or Cmd, Toshy maps Cmd to Ctrl; `Shortcut` renders ⌘⇧⌥⌃ (04 O-2) with no stated order; the launcher's shortcut is the Arc `Ctrl T` (Cmd+T is Fonts / New Tab) |
| **Verdict** | Adopt |
| **Rule** | (a) `StandardAction` enum with the table's bindings as data; `Shortcut::custom(keys)` returns `Err(Reserved(StandardAction))` for any reserved combination; `Shortcut::standard(StandardAction)` is the only way to bind one; unit test over the whole table. (b) Rendering order ⌃⌥⇧⌘ then the key; the upper character shown as itself. (c) Re-map the prototype keys: sidebar Ctrl+Cmd+S (Mac's Show/Hide Sidebar), command menu Cmd+K (Cmd+T stays New Tab in apps with tabs), Space switching Ctrl+1..9 (the Mac's "Switch to Desktop N" keys, L). (d) The launcher answers Cmd+Space. (e) Toshy's Cmd-to-Ctrl mapping is below us: our bindings are written in Mac terms and the `Mod` layer resolves them. 06 §2, 04 O-2; `ds::components::vocab`; type + test |

### 6.3 Pointing devices and pointers (2024-06-12)

| | |
| --- | --- |
| **Apple** | Gestures behave the same everywhere; never redefine system gestures; modifier-drag gives the same result by any input. Pointers: **arrow** for selecting and interacting with interface elements; **pointing hand** only when the content is a link; I-beam for text; open and closed hand for dragging content within a view; crosshair; resize (up, down, left, right, both); drag copy (Option), drag link, disappearing item, operation not allowed, contextual menu (Control held) |
| **quire today** | 12 gestures (Magic Mouse set, covered); CSS uses `cursor:pointer` on every button, row, item, scrim and segment (04, 14 rules in `components/*.css`) |
| **Verdict** | Adopt |
| **Rule** | Controls use the arrow (`cursor:default`); `pointer` only on `a[href]` and the link pill; `text` on editable text; `grab`/`grabbing` on drag handles and the toast pull tab; resize cursors on split dividers and window edges (built); during a drag the drop outcome sets copy, not-allowed or disappearing. CSS lint `PointerCursor` rejects `cursor:pointer` in any rule whose selector is not a link. 04 global rules; `ds::lint`; lint |

### 6.4 Focus and selection (2023-11-02)

| | |
| --- | --- |
| **Apple** | Use the system focus effects; never move focus without the person's action (except directional keyboard moves); **by default Tab reaches content elements (text fields, lists, search fields), not buttons, sliders and toggles; Full Keyboard Access adds those**; a focus ring for a text or search field, a **row highlight** for a list or collection; focused list: white text on an accent-coloured highlight; unfocused list: standard text on a grey highlight. Color page: `keyboardFocusIndicatorColor`, `selectedContentBackgroundColor`, `unemphasizedSelectedContentBackgroundColor` |
| **quire today** | 04 global rules: `:focus-visible{ outline:2.5px solid var(--accent); outline-offset:2px; border-radius:4px }` on everything; list rows show focus with the ring; selected rows `--raise` with an accent border (`S`); menus and launcher `--accent-soft`; every control in the Tab order |
| **Verdict** | Adapt |
| **Rule** | (a) Ring: `--focus-ring` = the accent at .55 alpha (L; the Mac's ring is translucent accent), width 3 px, gap 1 px, radius = the element's radius + gap (3.9), so a pill gets a pill ring; it appears with a `--t-quick` fade (the Mac's ring settles in, L). Lint `FocusRingShape`. (b) `FocusStyle::{Ring, Highlight}` per component: `ListRow`, `SidebarItem`, menu and launcher results use `Highlight` (never a ring). (c) Selection colours: `--sel-bg` (accent) with `--sel-ink` (accent ink) when the list has focus in the active window; `--sel-bg-quiet` (neutral grey) and `--ink` otherwise. The mail list's raised selected card stays in mailo (its own look); shell lists and menus use the Mac colours. (d) `appearance.keyboard_navigation = TextAndLists \| All`, default `All` (open decision 3): `TextAndLists` removes buttons, toggles, sliders and segments from Tab. 04 global rules, 06 §17; `ds::focus`; lint + type |

## 7. Proposed waves

In priority order. Each wave is one quire lane (and a sill lane where named), gated as usual. Waves
H0 and H1 can run in parallel; H2 needs H0's lint rules; H4 needs H1's springs for its sheets.

**H0: Guardrails (quire; small).** The cheap, mechanical rules that stop new drift before the
larger waves: the CSS lint rules `PointerCursor`, `MinFontSize` and `FocusRingShape` (landing as
warnings, with sill told first), the markup rules `UnnamedControl` and `ThreeDots`, the
`StandardAction` table with `Shortcut::custom` refusing reserved keys, and the modifier render
order. Fix the violations inside quire (cursor rules, "..." strings, prototype shortcuts
Mod+S/Mod+T). Doc: 02 §13 writing rules, 06 §2 re-mapped keys, CHECKLIST lines for writing,
pointer and shortcuts.
sill's side (Whopper, 2026-09-26): 0 `cursor:pointer`, no literal font sizes (all `var(--fs-*)`),
real ellipses already, no Mod+S/Mod+T bindings. So `MinFontSize` is a **token** rule in quire:
`--fs-micro`, `--fs-nano`, `--fs-help` and `--fs-dial` must resolve to >= 10 px under System (fix in
the token table, not in sill); the lint also rejects literal sizes below 10 in any stylesheet.
`FocusRingShape` and `UnnamedControl` run as warnings over sill's surface markup tests and the
list goes to sill before they turn Strict.

**H1: Driven motion (quire).** `Spring { damping, response }`, `use_spring`, `Touch::Contact(Velocity)`
extending 26's `Touch`, `SpringSpec::for_touch`, projection with the .998 rate, retargeting;
Reduced forms per keyframe (cross-fade) and critically damped springs under Reduced. Convert the
contact motions listed in 3.12 (sheet, panel slide, notification swipe return, toggle knob,
segmented and switcher indicator, slider release, dock drag return). Tests: interruption
continuity, throw endpoint, idle 0 frames, every moving keyframe has a still Reduced form.
Doc: 05 new §14.

**H2: Accessibility settings (quire + sill).** High-contrast token set (with the colour page's
Accessible variants as the pattern) and its 7:1 gate; `appearance.transparency`; "differentiate
without colour" with `StatusMark`; `appearance.sidebar_size`; `appearance.keyboard_navigation`;
AccessKit names and live regions on every component; sill: shell-host exports surfaces over
AT-SPI, Ctrl+F2/F3 focus the bar and dock. Manual checks queued (Orca on the launcher and a
banner).

**H3: Focus, selection, pointer, menus (quire).** The shape-following translucent ring,
`FocusStyle::{Ring, Highlight}`, emphasized and unemphasized selection colours, menus highlight
in accent with accent ink, text menus open without animation and blink on pick, context menus
drop shortcuts and hide unavailable items, the one-popover rule, the arrow cursor everywhere
but links. Gallery contact sheet reviewed beside macOS screenshots.

**H4: Commands, undo, modality (quire + sill).** `MenuModel` with the standard menus and items;
ds-native apps export it; sill's bar renders the active app's menus (App, File, Edit, View,
Window, Help) and the App menu for foreign apps; toolbar items bound to commands; `UndoStack`
in `ds::edit` with titled Undo/Redo and Cmd+Z / Shift+Cmd+Z; `ModalScope` focus trap;
`SheetKind::{Window, Session}`; the `Alert` component and its button types; power menu buttons
restyled (no destructive style on deliberate actions).

**H5: Type ramp and writing (quire; needs the user's decision on density).** Under the System
typeface, UI roles on the Mac text styles (body 13/16, secondary 11/14, section headers 11 Bold
title case), the 10 px floor, size-based tracking, glyph weight following text weight and
`SymbolScale`. Editorial keeps the prototype's ramp. Gallery before and after, side by side with
macOS.

**H6: CJK and restoration (quire + sill).** CJK faces and fallback order per locale, CJK
tracking 0, cluster-safe clipping, preedit drawing and caret-anchored candidates, Escape
cancelling a composition first; `Restorable` and the per-app restore file (window size, panes,
sidebar expansion, scroll, last settings pane).

**H7: Remaining components and the bar's privacy dots (quire + sill).** Checkbox, radio group,
pop-up and pull-down buttons, stepper, disclosure, `Table` with sorting, resizing and middle
truncation, `LevelIndicator`; `Radius::inner` and the `NestedRadius` lint with the radii table
regenerated; drag threshold 3 px, return-to-source, count badge, drag cursors; sill's camera,
microphone and screen-cast indicators with the portal purpose string; notification interruption
levels.

## 8. Decisions (settled by the user 2026-09-26)

1. **Density.** SETTLED: 13 px body for shell surfaces and system apps under System (H5); mailo
   keeps 15 px under Editorial.
2. **Sidebar icon colour.** SETTLED: keep ink on the Space colour (5.12).
3. **Keyboard navigation default.** SETTLED: `All` by default (Tab reaches every control, 09 H5);
   the Mac's `TextAndLists` is a setting.
4. **Dock and menu motion under the spring model.** SETTLED (proposal taken): the dock's bounce and
   magnification keep their curves (10); not converted in H1.
5. **Space switching keys.** SETTLED (proposal taken): Ctrl+1..9, with Cmd+1..9 left to apps.
6. **Global menus for foreign apps.** SETTLED (proposal taken): apps that do not export menus get
   only the App menu.

## 9. Sources

- Archived HIG pages, 71, listed with timestamps in 0.2 (JSON form of each page).
- [FLUID] WWDC 2018 session 803 page and transcript, fetched 2026-09-26.
- [KIT] design resources page, snapshot 20250502162612.
- Our docs: 00, 01, 02, 03, 04, 05, 06, 08, 09, 10, 11, 12, 13, 20, 22, 23, and 26 on branch
  `detail-grammar`; code read: `crates/ds/src/appearance/{system,resolve}.rs`,
  `motion/{drag,swipe}.rs`, `edit/composition.rs`, `components/*.css` cursor rules, the
  workspace `Cargo.toml` Blitz features.
