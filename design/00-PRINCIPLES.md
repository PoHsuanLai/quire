# 00 Principles

## 1. What this governs

This file is the stance every other design doc applies: what the design is for, and the rules that
decide between two options when no number settles it. It holds no pixel values; those live in
`01-LAYOUT.md`, `02-TYPE.md`, `03-COLOR.md` and the later files. `30-CATALOGUE.md` is the source of
truth and wins over this file; when a brief and this file disagree, the brief is wrong.

The direction (settled, `30-CATALOGUE.md` section 3.4): **one quiet Look, pre-Liquid-Glass macOS**,
with Dia as the reference browser for structure only (a calm sidebar beside the content). Arc is
history: some of its ideas survive as features in their quieter form (30 section 3.3); none of its
values do. The Arc-era text of this file is `archive/00-PRINCIPLES-arc-era.md`.

Source keys used in the older design docs (the numbers they cite are history, not targets):

| Key | File | Status |
| --- | --- | --- |
| `S` | `~/mailo-design/mailo-spaces.html` | the Arc/Post-era prototype; a source of behaviour ideas, not of values |
| `C` | `~/mailo-design/mailo-charm.html` | older prototype; the source of the four rules in section 3 |
| `P` | `~/.claude/plans/vast-toasting-peach.md` | the orchestration plan |

`S:123` means line 123 of `S`. A number with no source key and no doc section is a mistake.

## 2. The stance

Quiet. The interface looks like a Mac and gets out of the way:

- The window is flush: a sidebar and the content side by side, as in any standard Mac window. No
  frame, no inset card, no card shadow (30 section 3.4).
- A Space gives the sidebar one flat tint. Nothing else in the window changes colour with it; the
  content stays on paper, so every contrast rule we test still holds.
- Chrome is greyscale. Hue appears where it states a fact: unread, selection, focus, status,
  destructive, which account, which app.
- One system face, sentence-case headings, short copy, four shadows, no ornament that needs a
  second look.
- The trust rule from `C` stands: same inbox, same behaviour, every time. Every animation is a
  pure function of state, and the interface behaves identically on identical input.

## 3. The four rules

They decide every open question about ornament. They come from `C`'s four cards.

| Rule | Reading now |
| --- | --- |
| One weight, one colour | Glyphs are line glyphs (24 grid, 2 px stroke, round caps and joins), one colour (the ink token), no fills. App icons are the one coloured glyph family, and each app has its own colour so apps are told apart as on the Mac. |
| Springs only on contact | Nothing loops and nothing bounces on its own; the spinner is the one loop. A spring belongs to something a hand touched (30 section 1.3). The Dock's attention bounce is the Mac's own. |
| Variation with a reason | Whatever differs between two items states a fact about them. |
| Colour is a claim | Chrome is greyscale; a hue states a fact (unread, starred, destructive, status, which account, which app). The Space tint is the one decorative-but-owned colour: it says which Space you are in, on the sidebar ground only. |

Supporting line from `C:1162-1163`: the softness is in the corner radius and the round stroke caps,
not in a drawing.

## 4. Motion principles

The Mac's motion model and nothing else; tables and tokens are `30-CATALOGUE.md` section 1 (the
surviving parts of `05-MOTION.md` apply where 30 is silent). Principles that stay:

1. **Keyed to state.** Every animation names a state change; nothing plays for its own sake or on
   first show.
2. **Acting gets an exit; opening does not promise a commit.** A removed row fades and slides away
   and the rows below close the gap; opening a menu or popover changes nothing in the list.
3. **No randomness.** No rare flourishes, rotating copy, time-of-day tinting or hash-seeded
   variation.
4. **Do not drop what is leaving.** The node stays until its exit ends.
5. **Concessions are dull.** Allowing something the user once refused gets no celebration.
6. **Small motions change appearance, not layout.** A Space switch cross-fades the tint; it does not
   slide content.

The small state details every stateful element shows are `26-DETAILS.md`: one grammar, so a detail
is never hand-made for one item.

## 5. Interaction principles

Hover waits, stays warm and never has side effects: it opens after the profile's delay, closes
after its own, and stays warm for 400 ms so the next one opens at once (30 section 1.2). Nothing a
hover does marks read or fetches.

| Stance | Reading |
| --- | --- |
| Truth over trust | A link's real destination shows in a status pill, like Safari's status bar; it is loud when the text lies. |
| Preview the result | Actions preview their result: snooze names the time. |
| One field, one menu | Every input and every list of choices uses the same components; one menu shape at every size. |
| Handles are for objects | Only objects have a drag handle. |
| Closing is not deleting | Dropping a Today tab closes the tab only; it never archives mail. |

State machines for all of these are `06-INTERACTIONS.md`.

## 6. Where each element comes from

The UX is Mac and so is the UI: quiet. This table sorts the kept ideas by lineage so a surface
author knows which convention to follow when a detail is missing. What is not listed was dropped.

| Lineage | Elements |
| --- | --- |
| Mac | Window structure, sidebar, list and reader order, toolbars, sheets, segmented controls, menus, the bottom toast, the Dock and its bounce, the menu bar, Cmd+Tab, Settings window, `⌃1-9` Desktop switching |
| Dia (structure only) | A calm sidebar beside the content; a command pill; no chrome competing with the page |
| Arc (kept as features, quieter) | Pinned tiles, Today tabs, edge peek, link status pill, grouped launcher commands, a flat Space tint on the sidebar ground |
| Notion | Composer as a page with `/` and `@` menus; views with group-by and per-view hover actions |
| Dropped (history) | The gradient frame and grain backdrop, the sidebar drawn on colour, the inset card, the hue-by-chroma Space editor with up to three dots, content-slide Space switching, 4-up tiles, caps-tracked labels |

On the desktop the Mac lineage adds the menu bar, dock, sheets, Cmd+Tab and focus rules; Spaces add
the flat tint per workspace. See section 8.

## 7. The coherence rule

One design system draws every surface, and a port means the whole look. Every surface, native app
and widget must be visually and behaviourally identical: same tokens, same menus, same
micro-animations. Tokens alone are not a port; type, layout, motion and component markup must all
be shared.

1. No stylesheet outside the design system holds a colour, a raw duration, a raw curve, a
   keyframe, a font family or a `:root` selector.
2. No surface or app writes raw button, input or menu markup; it uses the design system's
   components.
3. A missing component or token is added to the design system first, never patched locally.
4. Motion state is driven by the design system's timers, never by ad-hoc sleeps.

## 8. The desktop reading

Every app window is flush paper; the shell chrome (bar, dock, launcher, control centre) is the
Mac's translucent materials. The Space tint reaches the sidebar ground and the workspace indicator
and nothing else.

### 8.1 Settled decisions this reading rests on

The dated decision log is `30-CATALOGUE.md` section 3.4. In short: one flat tint per Space, flush
window, `⌃1-9` switching with a cross-fade, muted widgets with rings only in the control centre,
per-app icon colours, Mac Dock bounce with magnification capped at 72 px, eight accents with Blue
the default and the accent set in Settings.

### 8.2 The zones on the desktop

| Element | Desktop | Rule |
| --- | --- | --- |
| Shell chrome (bar, dock, launcher) | Mac materials over the wallpaper's blur | No Space inks; greyscale glyphs. |
| App window content | Paper tokens | The Space never enters the content. |
| Sidebar ground | The Space's flat tint (target, clean-up phase) | The only place the tint is drawn. |
| Workspace indicator in the bar | Space dot and name; drag reorder | Its Space editor lives in the bar's workspace menu and in Settings. |

### 8.3 The four rules on the desktop

| Rule | Desktop reading |
| --- | --- |
| One weight, one colour | Every glyph in bar, dock menus, control centre, notifications and OSD is a line glyph in the ink token; tray icons are recoloured to ink. |
| One coloured thing | App icons are the one full-colour element in the chrome, each in its own colour. |
| Springs only on contact | Dock magnification tracks the pointer with no easing; the spring belongs to contact (press, drop, open). |
| Variation with a reason | A badge, an indicator or a bounce states a fact about that item. |
| Colour is a claim | In app content, hue states a fact. In the chrome, hue is only status, account, app icon, and the Space name's dot. |

### 8.4 Surface by surface

Values are in `20-SURFACES.md`, `01-LAYOUT.md` section 13 and `03-COLOR.md`.

| Surface | Governing principles |
| --- | --- |
| Bar | Mac material; line glyphs; one menu shape for every bar menu |
| Dock | Mac material; app icons are the coloured thing; magnification without easing, capped at 72 px; Mac bounce |
| Launcher | The command menu, bigger and centred; one menu shape |
| Control centre | One field, one menu; rings allowed here only; Dark Mode toggle, no accent picker |
| Notifications | Acting exits, opening does not; no loops |
| OSD | Springs only on contact; line glyphs |
| Widgets | Muted: neutral plates, colour only where it means something |
| Wallpaper | Behind everything; keyed to state; no loops |
| App windows | Flush paper; the Space tints the sidebar ground only |

### 8.5 Mac behaviour, Mac surface

Behaviour follows macOS numbers where they exist, and so does the look. Where macOS behaviour
conflicts with a principle above, the conflict is listed under Open decisions rather than resolved
here. Behaviour numbers live in `10-BEHAVIOUR-dock.md`, `11-BEHAVIOUR-scroll.md`,
`12-BEHAVIOUR-gestures.md` and `13-BEHAVIOUR-menus-windows.md`.

## Open decisions

1. **Which Space colours mailo when a mail Space and a workspace Space differ.** Not specified.
2. **Hover intent on the shell.** Whether dock labels, bar items and tray items use the 450/150/400
   ms hover intent is not specified (30 section 1.2 gives the three profiles).
3. **Notifications versus the undo toast.** macOS banners sit top-right and dismiss after about
   5 s; the toast sits bottom-centre. Position and timing for shell notifications are not
   specified beyond 30 section 1.3.
4. **Wallpaper and the Space.** The wallpaper is a picture; the Space tint never reaches it.

(Settled by the 2026-10-02 decisions and removed from this list: the Space frame on apps, Dock
bounce, loops, the icon-colour boundary, control-centre material.)
