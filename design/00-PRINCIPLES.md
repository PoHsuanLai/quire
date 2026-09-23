# 00 Principles

## 1. What this governs

This file is the stance every other design doc applies: what the design is for, the rules that
decide between two options when no number settles it, which parts come from Arc and which from
macOS, and how the mail prototype's rules read on the desktop shell. It holds no pixel values;
those live in `01-LAYOUT.md`, `02-TYPE.md`, `03-COLOR.md` and the later files. When a brief and
this file disagree, the brief is wrong. Quotes are verbatim from the prototypes.

Source keys used in every design doc:

| Key | File | Status |
| --- | --- | --- |
| `S` | `~/mailo-design/mailo-spaces.html` | newest prototype; wins every conflict |
| `C` | `~/mailo-design/mailo-charm.html` | older prototype; source for looks, candy hues, motion levels |
| `P` | `~/.claude/plans/vast-toasting-peach.md` | the orchestration plan; source for settled desktop decisions |

`S:123` means line 123 of `S`. A number with no source key is a mistake; report it.

## 2. The stance

The mail stays on paper and the colour lives around it. `S` states the thesis in its title and
lede:

> A Space colours the frame; the mail stays on paper.
> The colour fills the window **around** the Post card, never inside it, so every contrast rule we
> already test still holds. Brightness is not a control: it is fixed per theme, and chroma is
> capped until the sidebar's text passes.

`S:816-820`

`C` states the trust rule that sits under it:

> Same inbox, same behaviour, every time.
> an inbox is a trust surface, and an app that behaves differently on Tuesday than it did on
> Monday is one you stop trusting. **Every animation here is a pure function of state.** The charm
> has to come from the shapes, the icon language and the timing curves instead, which is harder
> and holds up better.

`C:1115-1119`

And the mechanism that makes the looks swappable:

> One shell, three motion dialects. Tokens are the whole trick: a "look" swaps colour + radius +
> shadow + easing, nothing else.

`C:8-9`

## 3. The four rules

These four rules decide every open question about ornament. They come from `C`'s four cards.

| Rule | Verbatim | Source |
| --- | --- | --- |
| One weight, one colour | "Lucide geometry throughout: a 24px grid, 2px stroke, round caps and joins. No illustration, no fills, nothing with more than one colour in it." | `C:1121-1122` |
| Springs only on contact | "Nothing loops and nothing bounces on its own. The only overshoot in the app is in the 200ms after you touched something." | `C:1123-1124` |
| Variation with a reason | "The one thing that differs between rows is legible: an unread thread folds 15% slower than a read one, because it weighed more." | `C:1125-1126` |
| Colour is a claim | "The chrome is greyscale. A hue only appears where it states a fact — unread, starred, destructive, or which label this is. Nothing is tinted for decoration, so the colour that *is* there is worth looking at." | `C:1127-1129` |

Two supporting lines from `C`:

- "The cuteness is in the corner radius and the round stroke caps — **not** in a drawing."
  `C:1162-1163`
- Icon base: "Lucide geometry: 24 grid, 2px stroke, round caps and joins, one colour, no fills.
  The roundness is the whole personality; nothing here is an illustration." `C:968-972`

How `S` changes the fourth rule: `S` does not carry "Colour is a claim" as `C` wrote it. In `S`
the frame chrome is coloured by the Space (`S:77-87`), every label chip is `--accent-soft`
(`S:198-199`) rather than a per-label hue, and there is no candy shelf. The rule that survives in
`S` is narrower: colour inside the card is still a claim (unread dot, selection, focus, star,
danger), and the Space colour stays outside the card. `S` wins; see `03-COLOR.md` for the exact
split.

## 4. Motion principles

Every animation names a state change and nothing plays for its own sake. `S` labels its motion
section:

> SMALL MOTION — every one keyed to a state change, none loops

`S:289-291`

The rules, each with its source:

1. **Keyed to state.** Rows rise "only when a list is first shown, not on every re-render"
   (`S:292`). Stagger lives on first show only.
2. **Acting gets an exit; opening gets a spring in place.** "Acting gets an *exit*. The row folds,
   the sidebar gulps, the undo toast appears. Something left." / "Opening gets a *spring in
   place*. Nothing moves in the list, nothing is undoable yet, and the row stays exactly where it
   was until you choose. An exit animation here would be a lie — it would promise a commit that
   hasn't happened." (`C:1338-1343`)
3. **Leaving has a shape per operation.** "leaving: archive folds, snooze curls away; the rows
   below close the gap" (`S:325`).
4. **The receiver answers.** "the place that received it gulps; a count that changed bumps"
   (`S:338`). "the seal: a dot that pops beside the place you are in" (`S:344`).
5. **Tabs behave like tabs.** "Today: an entry slides in like a new tab and collapses when closed"
   (`S:349`).
6. **No randomness.** "no rare flourishes, no rotating copy, no ink splashes, no time-of-day
   tinting, and no hash-seeded variation either" (`C:1381-1384`). "an interface that behaves
   differently on identical input is one you cannot form a model of" (`C:1378-1379`).
7. **The body is opaque.** The message body "is now animated as *one opaque block*, and all the
   staggering moved to the chrome around it" (`C:1356-1358`).
8. **Restyle, never move.** "The three reading modes … must be implemented by **restyling one
   container, never by moving the node**" (`C:1367-1368`).
9. **Do not drop what is leaving.** "don't drop the row from the signal until the exit animation
   ends, or the node vanishes mid-flight" (`C:1402-1403`).
10. **Concessions are dull.** Allowing remote images gets "the dullest [motion] in the app: no
    celebration when you allow images, because allowing them is a concession, not an
    achievement" (`C:1363-1364`).

Exact timings, curves and keyframes are `05-MOTION.md`.

## 5. Interaction principles

Hover waits, stays warm and never has side effects. `S` states it as a section title:

> HOVER — wait for intent, stay warm, never mark read, never fetch

`S:396-398`; the timing is "Opens after 450ms of rest, closes after 150ms of absence (so the
pointer can travel into the card), and stays warm for 400ms after a close so the next card opens
at once. Nothing here marks read or fetches" (`S:1698-1702`).

The other interaction stances, verbatim:

| Stance | Quote | Source |
| --- | --- | --- |
| Truth over trust | "the real destination of a link, like a browser's status bar … instant, and loud when the text lies" | `S:430`, `S:1811` |
| Preview the result | "actions preview their result: snooze names the time, archive lights its destination" | `S:440` |
| One field, one menu | "ONE FIELD, ONE MENU — every input and every list of choices in the window uses these, so nothing feels bolted on." | `S:777-780` |
| One menu, many sizes | "the command menu is the same menu, just bigger and centred" | `S:792` |
| Every menu is one shape | "Every menu in the window is this: a title row, items with a tile, a name, a line of help, and a shortcut; arrows, Enter and Esc; typing filters. Only what the items are changes." | `S:2066-2067` |
| Handles are for objects | "objects are the only things with a handle" | `S:745` |
| Closing is not deleting | Today entries "drop off after 12 idle hours. Dropping off closes the entry only. **It never archives mail.**" | `S:921-922` |

State machines for all of these are `06-INTERACTIONS.md`.

## 6. Where each element comes from

The UX is Mac and the UI may be louder, Arc style. The plan states the requirement: "UX is
Mac-vibed; UI may be more aggressive, Arc-browser style" (`P:11`). The table sorts the prototype's
elements by lineage so a surface author knows which convention to follow when a detail is
missing.

| Lineage | Elements | Source |
| --- | --- | --- |
| Arc | OKLCH gradient frame plus grain, calibrated to Arc's own `--arc-palette-*` exports | `S:908-912`, `S:986-991` |
| Arc | Sidebar drawn straight on the colour with translucent white pills | `S:89-153` |
| Arc | Command pill "Search or run a command" with `Ctrl T` | `S:841` |
| Arc | Grouped command menu, "like Arc's" | `S:1590` |
| Arc | Space dots and name in the sidebar foot; `Ctrl 1..9` switches with a content slide | `S:145-153`, `S:1669` |
| Arc | 4-up pinned tiles | `S:107-118` |
| Arc | Today entries behave like tabs: slide in, close with x, drop after 12 h | `S:349-353`, `S:920-922` |
| Arc | Hide sidebar (`Ctrl S`) plus edge peek | `S:450-454`, `S:1841-1843` |
| Arc | Link pill as a status bar | `S:430-437` |
| Arc | Card inset 8 px inside the frame, with its own radius | `S:81`, `S:156-160` |
| Arc | Space editor: hue x chroma field, up to 3 dots, grain slider, presets | `S:243-279` |
| Mac / conventional | Sidebar, list, reader three-pane order | `S:840-860` |
| Mac / conventional | List toolbar; reader header with a tool row | `S:162-164`, `S:202-207` |
| Mac / conventional | Centred peek over a scrim (sheet) | `S:220-226` |
| Mac / conventional | Segmented controls; small-caps section headers; bottom toast | `S:265-268`, `S:120-125`, `S:359-370` |
| Notion | Composer as a page with `/` and `@` menus, selection bubble, draggable objects | `S:564-775` |
| Notion | Views with group-by and per-view hover actions | `C:987-1041`, `C:1302-1310` |

On the desktop the Mac lineage adds the menu bar, dock, sheets, Cmd+Tab and focus rules; the Arc
lineage adds Spaces per workspace. See section 8.

## 7. The coherence rule

One design system draws every surface, and a port means the whole look. The plan's load-bearing
constraint: "Every surface, every native app and every widget must be visually and behaviourally
identical: same tokens, same menus, same micro-animations" (`P:12-14`). The mailo memory rule it
cites: "design port means the whole look": tokens alone are not a port; type, layout, motion and
component markup must all be shared (`P:86-87`).

The plan's enforcement, restated as principles (mechanics in the plan, `P:225-235`):

1. No stylesheet outside the design system holds a colour, a raw duration, a raw curve, a
   keyframe, a font family or a `:root` selector.
2. No surface or app writes raw button, input or menu markup; it uses the design system's
   components.
3. A missing component or token is added to the design system first, never patched locally.
4. Motion state is driven by the design system's timers, never by ad-hoc sleeps.

The single appearance picker is shared: `AppearancePicker` is "THE one picker for mailo, control
center, settings" (`P:355`).

## 8. The desktop reading

The shell is the frame and every app window is a card on paper. This section maps each principle
above onto the shell surfaces and onto the decisions the plan records as settled with the user.

### 8.1 Settled decisions this reading rests on

| Decision | Verbatim or near-verbatim | Source |
| --- | --- | --- |
| Mac UX, Arc UI | "UX is Mac-vibed; UI may be more aggressive, Arc-browser style." | `P:11` |
| Spaces on the desktop | "each COSMIC workspace has a SpaceLook; bar, dock and launcher chrome use its `--f-*` frame tokens over compositor blur; apps stay on paper; workspace switch cross-fades the tint (380 ms)." | `P:792-794` |
| Space store and editor | "workspace <-> SpaceLook store, default presets per workspace index, editor lives in Settings and in the bar's workspace menu" | `P:794-796` |
| Dock | "magnification ON by default, bottom, no auto-hide. Plank parabola curve, icon 48 -> up to 96, neighbours slide apart, no easing while tracking, ~200 ms shrink on leave; toggle + sizes + auto-hide (0.2 s delay, ~0.5 s slide) available in Settings." | `P:784-786` |
| Small glyphs | Lucide (ISC) primary, Tabler (MIT) for gaps, 24 grid, 2 px stroke, round caps, through the `Icon` enum; tray and third-party symbolic icons recoloured to the ink token. | `P:806-809` |
| App icons | Our own, one template: continuous-curvature squircle plate, gradient families from the Candy palette (5 hues x base/deep/soft), edge highlight and shadow; third-party apps' icons sit inside our plate at a fixed inset so every dock tile has the same silhouette. | `P:810-825` |

### 8.2 The two zones on the desktop

The mail window's split between frame and card becomes the split between shell chrome and app
content.

| Mail prototype | Desktop | Rule |
| --- | --- | --- |
| The Space-coloured frame around the card (`S:73-87`) | Bar, dock and launcher chrome | Use the workspace's `--f-*` frame tokens over compositor blur (`P:792-793`). |
| The Post card, "untouched by the Space's hue" (`S:155`) | Every app window's content | Stays on paper tokens; the Space colour never enters it (`P:793`). |
| Space switch: two layers cross-fade (`S:86`, `S:1181-1186`) | Workspace switch | Cross-fades the tint over 380 ms (`P:793-794`). |
| Space dots and name in the sidebar foot (`S:145-153`) | Workspace indicator in the bar | The bar shows the workspace indicator with drag reorder (`P:574-575`); its Space editor lives in the bar's workspace menu (`P:796`). |

### 8.3 The four rules on the desktop

| Rule | Desktop reading |
| --- | --- |
| One weight, one colour | Every glyph in bar, dock menus, control center, notifications and OSD is a Lucide/Tabler line glyph in the ink token; tray icons are recoloured to ink (`P:806-809`). |
| One coloured thing | App icons are the one full-colour element in the shell chrome: our own squircle plates in the Candy gradient families (`P:810-825`). Everything else in the chrome is ink on the Space frame. |
| Springs only on contact | Dock magnification tracks the pointer with no easing and no spring (`P:785`); the spring belongs to contact (press, drop, open). |
| Variation with a reason | Any per-item difference in the shell (a badge, an indicator, a bounce) must state a fact about that item. |
| Colour is a claim | Inside app content, hue states a fact (unread, selection, focus, star, danger). In the chrome, the Space hue says which workspace you are in and nothing else. |

### 8.4 Surface by surface

Each row names which principle governs the surface and what is settled. Values are in
`01-LAYOUT.md` section 13 and `03-COLOR.md` sections 15-16; anything missing is open.

| Surface | Zone | Governing principles | Settled |
| --- | --- | --- | --- |
| Bar | Frame | Frame tokens over blur; one-colour glyphs; one menu shape for every bar menu | Per output, top, reserves its height, no keyboard focus, whole-surface blur; left: app name and workspace indicator; right: tray, volume, network, battery, clock; every menu is the design system's Menu in an xdg popup anchored to its button (`P:574-577`). |
| Dock | Frame | Frame tokens over blur; app icons are the coloured thing; magnification without easing | Bottom, magnification on, 48 to 96, no auto-hide by default (`P:784-786`); indicator dots, LauncherEntry badge and progress ring; context menu: windows, desktop actions, Keep in Dock, Quit (`P:581-583`). |
| Launcher | Frame | "the command menu is the same menu, just bigger and centred" (`S:792`); one menu shape | Centred Overlay panel, keyboard-exclusive, blurred, over a transparent full-screen catcher; closes on Esc, focus loss or catcher click (`P:586-589`). |
| Control center | not specified | One field, one menu; the one appearance picker (`P:355`) | Uses `AppearancePicker`. Material and zone not specified. |
| Notifications | not specified | Acting exits, opening springs; no loops | Nothing settled beyond owning `org.freedesktop.Notifications` (`P:181-182`). |
| OSD | not specified | Springs only on contact; one-colour glyphs | A `Material::Osd` exists (`P:315`); nothing else settled. |
| Widgets | not specified | Coherence rule: same tokens, same components | A `Material::Widget` exists (`P:315`); nothing else settled. |
| Wallpaper | Behind everything | Keyed to state; no loops | Per output, Background layer, no input; light/dark cross-fade over `--t-big`; renders only on change (`P:577-579`). |
| App windows | Paper | The Post card rules, unchanged | Apps stay on paper (`P:793`). |

### 8.5 Mac behaviour, Arc surface

Behaviour follows macOS numbers where they exist; the look follows the prototype. Where macOS
behaviour conflicts with a principle above, the conflict is listed under Open decisions rather
than resolved here. Behaviour numbers live in `10-BEHAVIOUR-dock.md`, `11-BEHAVIOUR-scroll.md`,
`12-BEHAVIOUR-gestures.md` and `13-BEHAVIOUR-menus-windows.md`.

## Open decisions

1. **"Colour is a claim" versus the Space frame.** `C` says "The chrome is greyscale"
   (`C:1127`); `S` colours the chrome by Space (`S:77-87`) and drops per-label hues
   (`S:198-199`). `S` wins for the mail window. Whether any shell chrome (control center,
   notifications, OSD) is greyscale or Space-tinted is not specified.
2. **Do app windows carry a Space frame?** In `S` the mail window itself is the frame (sidebar on
   colour around the card, `S:77-160`). The desktop decision says "apps stay on paper"
   (`P:793`). Whether an app's own window chrome (for example mailo's sidebar) takes the
   workspace SpaceLook, the app's own Space, or paper is not specified.
3. **Which Space colours mailo when a mail Space and a workspace Space differ.** Not specified.
4. **Dock bounce loops.** macOS repeats the launch bounce until launched and a critical request
   bounces until the app is active (`P:1394-1396`); "Nothing loops and nothing bounces on its
   own" (`C:1123`). Not resolved.
5. **"Nothing loops" already has two exceptions in the prototypes.** `S`'s `.item.dest` runs
   `dest 900ms … infinite alternate` (`S:447`); `C`'s sync halo breathes and spins infinitely
   (`C:288-291`). Whether loops are allowed while an operation is in progress (hover over
   archive, sync busy) is not specified.
6. **Hover intent on the shell.** Whether dock labels, bar items and tray items use the
   450/150/400 ms hover intent (`S:1704`) is not specified; macOS dock label delay is unknown
   (`P:1397`).
7. **Notifications versus the undo toast.** macOS banners sit top-right and dismiss after about
   5 s (`P:1476-1477`); the prototype's toast sits bottom-centre and hides after 5200 ms
   (`S:360`, `S:1552`). Position and timing for shell notifications are not specified.
8. **Launcher as the command menu.** `S:792` makes the command menu "the same menu, just bigger
   and centred"; the plan's launcher panel is a fixed ~680x460 (`P:587`) while `S`'s command menu
   is `min(540px, 88%)` wide (`S:793`). Whether the launcher is literally the command-menu
   component is not specified.
9. **Where the icon principle ends.** `C`'s rule is "nothing with more than one colour in it"
   (`C:1122`); the app-icon decision makes app icons full-colour gradients (`P:810-814`). The
   boundary (dock tiles and launcher results coloured; everything else one colour) is the reading
   in 8.3 but is not stated in the plan.
10. **Wallpaper and the Space.** Whether the wallpaper shows the Space gradient and grain, or a
    picture, or both, is not specified.
11. **Material and zone for control center, notifications, OSD and widgets** are not specified.

## Sources

- `S` `~/mailo-design/mailo-spaces.html`: title and lede `S:815-820`; notes `S:907-926`; motion
  section comments `S:289-353`; hover `S:396-454`; one field one menu `S:777-799`; menus
  `S:2066-2067`; hover timing `S:1697-1704`.
- `C` `~/mailo-design/mailo-charm.html`: header `C:6-10`; hero and four cards `C:1112-1164`;
  notes on acting vs opening, opaque body, reading modes, randomness, porting `C:1325-1417`.
- `P` `~/.claude/plans/vast-toasting-peach.md`: context `P:3-24`; coherence rules `P:225-235`;
  memory rule `P:86-87`; appearance picker `P:355`; shell surfaces `P:574-592`; settled UX
  decisions `P:779-796`; icons `P:801-843`; Appendix A0 `P:874-917`; Appendix C `P:1381-1485`.
