# 06 Interactions

Status: draft for review, 2026-09-23. `S` = `~/mailo-design/mailo-spaces.html` (newest; wins
every conflict), `C` = `~/mailo-design/mailo-charm.html`. `S:123` means line 123 of S. "The
plan" means `~/.claude/plans/vast-toasting-peach.md`; "Appendix A/C" are its appendices.
Durations named here (HoverOpen, ToastHold, `settle()`, …) are defined in `05-MOTION.md`.
**Bug** marks prototype behaviour the port must not copy; each is repeated in section 21.

## 1. What this governs

Every behaviour a quire surface performs in response to input: the keyboard maps, the
hover-card intent machine, where menus and floating surfaces are placed and how they are
dismissed, drag and drop, the sidebar and its edge peek, Space switching, the undo toast and its
pull tab, undo send, the command menu's search and ranking, the selection bubble, the link pill,
destination preview, peek modes, the composer's lifecycle, focus, and what Escape does in each
context; and, in section 20, what the desktop shell must do (dock clicks and menus, launcher
keys, menu bar, click-through, notifications, Magic Mouse gestures). Each behaviour is written
as a state machine or a rule table so that it can be implemented as a pure `step(state, event)
-> (state, effect)` function (CONVENTIONS §0) and tested with a table. Motion values live in
`05-MOTION.md`; colours in `07-LOOKS.md`.

## 2. Keyboard maps

"Mod" = Ctrl or Cmd (`e.ctrlKey || e.metaKey`, S:1666). On this desktop Toshy maps Cmd to Ctrl.

### 2.1 Global (S, window listener)

| Keys | Action | Guard | Source |
| --- | --- | --- | --- |
| Mod+T, Mod+K | open the command menu | no-op if already open | S:1667, S:1632 |
| Mod+S | toggle the sidebar (`setSide(!hidden)`) | none | S:1668 |
| Mod+1 … Mod+9 | switch to Space n-1 | no-op if it is the current Space or does not exist | S:1669, S:1477 |
| Escape | close the command menu if open, else close the peek | none | S:1670 |
| Space | if a *thread* hover card is open: close it and open the peek for that thread | target not inside `input, textarea` | S:1804-1808 |

These listeners run for every keydown in the window, including keys typed in the composer
(section 2.5), because the composer handler does not stop propagation. **Bug** (S): Mod+K in the
composer both starts a link (S:2201) and opens the command menu (S:1667); Mod+Shift+S both
strikes through (S:2202) and toggles the sidebar (S:1668, `e.key` is "S"); Mod+1..9 while
composing switches Space, which re-renders the reader and silently parks the draft (S:1385);
the Space-key guard checks `input, textarea` but not `contenteditable`, so Space typed in the
composer body while a thread card is open opens a peek and eats the space. Rule for the port:
a focused text field owns every key it handles, and handled keys stop propagating.

### 2.2 Global (C)

| Keys | Action | Source |
| --- | --- | --- |
| Mod+K | open the command menu | C:2259 |
| Escape | close the command menu if open, else close the open view menu | C:2260 |

C has no Mod+T, no sidebar key, no Space keys. The in-row picker (C:1971) has no keyboard.

### 2.3 Command menu (focus in its input)

| Keys | Action | Source |
| --- | --- | --- |
| Down | `sel = min(sel + 1, count - 1)` (clamped, no wrap) | S:1653, C:2244 |
| Up | `sel = max(sel - 1, 0)` | S:1654, C:2245 |
| Enter | close the menu, then run the selected item | S:1655, S:1650 |
| Escape | close | S:1656 |
| typing | recompute results; `sel = 0` | S:1651 |
| pointer down on the backdrop itself | close | S:1660 |
| mouse down inside the list | prevented, so focus stays in the input | S:1658 |
| click an item | close, then run it | S:1659 |

**Bug** (S and C): Escape in the input closes the menu, then the same event reaches the global
handler, which now sees no menu and closes the peek (S) or the view menu (C). One Escape must
dismiss exactly one layer (section 18).

### 2.4 Floating menus (S `menu()`: slash, mention, people, turn-into, from, when, snooze, labels, object)

Handled by `menuKey` (S:2089-2098) only when it is called: from the composer's key handler
(S:2195) and from a people input (S:2264).

| Keys | Action |
| --- | --- |
| Down | `sel = (sel + 1) mod n` (wraps) |
| Up | `sel = (sel - 1 + n) mod n` (wraps) |
| Enter or Tab | close the menu, then pick the selected item |
| Escape | close the menu |
| any of the above | `preventDefault` and `stopPropagation` |
| typing (slash, mention) | query = text after the last `/` or `@`; filter; close if the query contains two spaces in a row or exceeds 24 characters (S:2186-2190) |
| typing (people input) | re-open the people menu filtered by the input; close when empty or no match (S:2256-2262) |

**Bug** (S): the snooze and label menus opened from the list (S:1504-1509) are never passed to
`menuKey`, so they cannot be driven by keyboard and Escape does not close them (the global
Escape closes a peek instead). Rule: every ds `Menu` handles its own keys wherever it opens.

### 2.5 Composer (focus inside the composer root)

Order matters: the first matching row wins (S:2194-2212).

| # | Keys | Condition | Action |
| --- | --- | --- | --- |
| 1 | menu keys | a floating menu is open | section 2.4 |
| 2 | Mod+Enter | | send (`trySend(later = false)`) |
| 3 | Mod+Shift+Enter | | schedule: if "Sends" is "Right away", set it to "Tomorrow · 08:00"; then send (S:2317-2318) |
| 4 | Mod+Shift+F | page composer only | toggle focus mode |
| 5 | Escape | | if focus mode: leave focus mode; else park the draft ("Keep for later") |
| 6 | Mod+B / I / U / E / K (no Shift) | focus in body or title | bold / italic / underline / inline code / link (section 12) |
| 7 | Mod+Shift+S | focus in body or title | strikethrough |
| 8 | Enter (no Shift) | caret line is a code block | insert a newline |
| 9 | Enter (no Shift) | caret line is an empty list item | turn it into a paragraph (leave the list) |
| 10 | Enter (no Shift) | caret at the end of a heading or quote | insert a new paragraph after it, caret there |
| 11 | Backspace | caret collapsed at the start of a line that is not a paragraph | turn the line into a paragraph |
| 12 | Tab / Shift+Tab | caret in a list item | indent / outdent |

Markdown as you type (S:2168-2185), only when no menu is open:

| Typed at line start | Becomes | Typed inline, once closed | Becomes |
| --- | --- | --- | --- |
| `# ` | heading (stored as `h2`) | `**text**` | bold |
| `## ` | subheading (stored as `h3`) | `*text*` or `_text_` | italic |
| `- ` or `* ` | bulleted list | `` `text` `` | inline code |
| `1. ` or `1) ` | numbered list | `~~text~~` | strikethrough |
| `[] ` or `[ ] ` | to-do list | | |
| `> ` | quote | | |
| ```` ``` ```` | code block | | |
| `---` (in a paragraph) | divider object | | |

`/` or `@` typed at the start of a line or after whitespace opens the slash menu or the mention
menu at the caret (S:2182-2183). Picking deletes the trigger and query, then turns the line,
inserts an object, or inserts a mention chip and adds the person to Cc with a 1200 ms flash
(S:2108-2122). Paste inserts plain text only, double newlines becoming paragraphs (S:2216-2222).
A click within 24 px of a to-do item's left edge toggles it done (S:2226).

### 2.6 People input (To, Cc)

| Keys | Action | Source |
| --- | --- | --- |
| Enter or `,` with text | add the typed address as a recipient | S:2265 |
| Backspace on an empty input | remove the last recipient, keep focus | S:2266 |
| menu keys while the people menu is open | section 2.4 | S:2264 |

### 2.7 Selection bubble link input

| Keys | Action | Source |
| --- | --- | --- |
| Enter | restore the saved selection; if the URL is non-empty, create the link; hide the bubble | S:2154-2155 |
| Escape | hide the bubble | S:2155 |

### 2.8 Space editor handles (colour field)

| Input | Action | Source |
| --- | --- | --- |
| Left / Right on a handle | hue -5° / +5° (mod 360) | S:1455 |
| Up / Down on a handle | chroma +0.05 / -0.05, clamped to 0..1 | S:1456 |
| pointer down / move on the field | the active dot follows: `h = x/width * 360`, `c = 1 - y/height`, clamped | S:1435-1451 |
| after a key | the handle keeps focus | S:1457 |

## 3. Hover-card intent machine

"HOVER — wait for intent, stay warm, never mark read, never fetch" (S:397). One manager serves
every card (S:1697-1703). Cards read only what the list already holds.

**Kinds and targets** (the innermost `[data-hc]` under the pointer wins, S:1787-1788): `thread`
(a list row), `sender` (the name in a row or the reader), `time` (a row's time; a small tip),
`acct` (an account tile), `pin` (a pinned sender), `today` (a Today entry).

**State** (Rust `HoverIntent<K>`, plan): `Idle`, `Pending{key, due}`, `Open{key}`,
`Closing{key, due}`, plus an independent `warm_until: Instant`. "Warm" = `now < warm_until` or a
card is open (S:1794).

| From | Event | Guard | To | Effect |
| --- | --- | --- | --- | --- |
| any | PointerOver(target) | target inside a leaving row (`.going`), or a peek is open, or the command menu is open | unchanged | none (S:1790) |
| Open{k} or Closing{k} | PointerOver(target k) | same key | Open{k} | cancel close timer (S:1791-1792) |
| any | PointerOver(target j) | j differs from the open or pending key | Pending{j} | cancel open timer; start open timer: 0 ms if warm, else HoverOpen 450 ms (S:1793-1795) |
| Pending{j} | open timer fires | | Open{j} | remove any open card instantly (no out animation); build card j; place it (below); play `hc-in` (S:1774-1784) |
| Pending{j} | PointerOut to a non-card, non-same target | | Idle | cancel open timer (S:1801) |
| Open{k} | PointerOut to anything but the card or the same target | | Closing{k} | start close timer HoverClose 150 ms (S:1802) |
| Closing{k} | pointer enters the card | | Open{k} | cancel close timer (S:1782) |
| Open{k} | pointer leaves the card | | Closing{k} | start close timer 150 ms (S:1783) |
| Closing{k} | close timer fires | | Idle | `warm_until = now + HoverWarm 400 ms`; set the window's warm flag (strip labels show without their 350 ms delay); play `hc-out`; remove the node at `settle(HcOut)` (S:1707-1714) |
| any | click in the list (capture phase) | a card is open | Idle | remove the card instantly, no warm, no out animation (S:1809) |
| Open{thread k} | Space key | focus not in a text field | Idle | close the card (warm), open the peek for thread k (S:1804-1808) |
| warm | `warm_until` passes | | | clear the warm flag (S:1712) |

Rules: a card never takes focus, never marks a thread read ("stays unread while you look",
S:1735), never fetches.

**Positioning** (S:1715-1726), in the window's coordinates, card size measured after mount:

| Kind | x | y |
| --- | --- | --- |
| thread | list column's right edge + 10 | row top - 4 |
| pin, today, acct | target right + 10 | target top - 6 |
| sender, time (and any other) | target left | target bottom + 6 |

Then `y = clamp(y, 8, window_height - card_height - 8)` and `x = clamp(x, 8, window_width -
card_width - 8)`. **No flip**: a card that would overflow slides along the edge instead. In ds
this is `place(anchor, content, bounds, want, gap)` with flipping disabled for hover cards
(the plan's `place` flips by default; section 21).

Card widths: 300 px (`.hc`, S:399), 260 px side cards (acct, pin, today; S:428), auto up to
260 px tips (time; S:426).

## 4. Menu placement

### 4.1 Floating menus (S `placeFloat`, S:2060-2065)

Anchor rect = the trigger's bounding rect, or the caret rect for slash and mention menus
(S:2053-2059, a zero-width span inserted at the caret when the range has no rect).

1. `x = anchor.left - 8`, `y = anchor.bottom + 6`.
2. Flip: if `y + menu_height > window_height - 8`, then `y = anchor.top - menu_height - 6`.
3. Clamp: `x` to `[8, window_width - menu_width - 8]`, `y` to `[8, window_height - menu_height - 8]`.

Menu size: 280 px wide, max 320 px tall (S:665); `slim` menus 220 px (S:678).

### 4.2 Selection bubble (S:2143-2145)

Centred over the selection rect: `x = rect.left + rect.width/2 - bubble_width/2`, clamped to
`[8, window_width - bubble_width - 8]`; `y = max(8, rect.top - bubble_height - 8)`. No flip below
the selection.

### 4.3 C menus

C's view menus sit at `top: calc(100% + 7px); right: 0` of their trigger's container, origin
`88% 0` (C:1003-1007), with no clamp and no flip. C's in-row picker sits at `right: 10px; top:
calc(50% + 16px)` inside the row (C:487). S's placement wins.

### 4.4 Desktop

A popup that leaves its surface is an `xdg_popup`: ds computes `PopoverRequest{anchor,
placement, size}` and shell-host maps it to `xdg_positioner` with flip and slide constraints
(plan, `PopoverRequest`). The rules above become the positioner's anchor (bottom-left of the
trigger, offset -8, +6), gravity (down-right), and constraint adjustment (flip_y, slide_x,
slide_y); the 8 px margin is the output's usable area.

## 5. Menu dismissal

| Event | S floating menu | C view menu | C in-row picker |
| --- | --- | --- | --- |
| click an item | close, then pick (S:2102-2103) | group: pick and close; properties: toggle and stay open (C:1762-1767) | snooze time: close and snooze; label: toggle and stay open (C:1987-2002) |
| click outside | close, unless the click is inside the menu, on a `[data-pick]` trigger, an object handle `.ograb`, or any `[data-op]` strip button (S:2104) | close unless inside `.menu` (C:1778) | close unless inside `.picker` or on a `[data-op]` (C:2004-2006) |
| click the trigger again | re-opens (a fresh menu replaces the old one) | toggles: closes if it was open (C:1770-1775) | re-opens |
| opening another menu, the command menu, or the composer | close (S:2069, S:1633, S:1942) | | any op closes it (C:1881) |
| Escape | close (only when `menuKey` runs; section 2.4) | close (global, C:2260) | not handled |
| mouse down inside | prevented, so focus stays where it was (S:2099) | | |

**Bug** (S): because clicks on any `[data-op]` are exempt, clicking Archive on a row while that
row's snooze menu is open archives the row and leaves the menu open, pointing at a row that is
leaving. Rule: only the menu's own trigger is exempt.

## 6. Drag and drop

### 6.1 Thread onto a place (C only; S has no thread drag)

State: `Idle`, `Armed{id, x0, y0}`, `Dragging{id, ghost, target}` (C:2032-2088).

| From | Event | Guard | To | Effect |
| --- | --- | --- | --- | --- |
| Idle | pointer down on a row | not on a button | Armed | record start point |
| Armed | pointer move | `abs(dx) + abs(dy) < 8` (Manhattan) | Armed | none |
| Armed | pointer move | distance >= 8 px | Dragging | create the ghost (subject, sender) at `(x - 40, y - 18)`; row gets `.is-dragging` (opacity .35) |
| Dragging | pointer move | | Dragging | move ghost to `(x - 40, y - 18)`; the `.view[data-drop="true"]` under the pointer becomes the target (`.is-drop-target`: accent-soft, scale 1.045), previous target cleared |
| Dragging | pointer up | target set | Idle | remove ghost; apply the drop effect (below) |
| Dragging | pointer up | no target | Idle | remove ghost; nothing happens |
| Armed | pointer up | | Idle | (the click opens the thread as usual) |

Drop targets: Inbox, Snoozed, Archive, Trash, and every label; not Starred (`drop:false`,
C:1587-1594).

| Target | Effect | Source |
| --- | --- | --- |
| a label | add the label if missing; the label gulps; the new chip plays `chip-land`; toast "Labelled x" | C:2066-2079 |
| Snoozed | snooze with the default time ("tomorrow") | C:2080 |
| Inbox | restore | C:2081 |
| Archive, Trash | that op | C:2082 |

**Contradiction**: dropping on Snoozed snoozes without asking "until when", which C's own notes
call wrong ("you cannot snooze without saying until when", C:1334-1335). Proposed: a drop on
Snoozed opens the snooze picker anchored to the Snoozed item.

Ghost: fixed, z 50, 250 px wide, `rotate(var(--tilt)) scale(1.02)`, `--shadow-drag` (C:579-586).

### 6.2 Objects inside the composer (S)

Only objects (image, table, quoted message, divider) have a handle, `.ograb` (S:745-751). The
prototype uses HTML5 drag (S:2269-2275): drag start on a handle marks the object `.dragging`
(opacity .35); while dragging over the body, a 3 px accent `.drop-line` is inserted before the
first child whose vertical midpoint is below the pointer (else at the end); drop replaces the
line with the object; drag end clears. A click on the handle opens the object menu: Move up,
Move down, Duplicate, Delete (S:2231-2236). The Rust editor replaces HTML5 drag with ds
`DragTracker` (plan, `motion/drag.rs`); the 8 px threshold of 6.1 applies.

## 7. Sidebar hide and edge peek

| State | Event | To | Effect | Source |
| --- | --- | --- | --- | --- |
| Shown | Mod+S, the sidebar's hide button, or entering focus mode | Hidden | window grid `0 1fr`, padding-left 8; sidebar `visibility:hidden`; a "Sidebar" mini button appears in the list bar; a 10 px `.edge` strip appears at the left | S:1584-1586, S:85, S:451-452 |
| Hidden | Mod+S, the "Sidebar" button, leaving focus mode, parking, sending | Shown | reverse | S:1584, S:1992, S:2330 |
| Hidden | pointer enters the edge strip | Peeking | sidebar floats at left 8, top 8, bottom 8, width 226, z 15, `--f-solid` ground, radius 14, `slide-r` | S:1842, S:453-454 |
| Peeking | pointer leaves the sidebar | Hidden | | S:1843 |

The width change animates (`.win` transition, `05-MOTION.md#6-transitions`).

Notes: parking and sending restore a hidden sidebar even if the person hid it before entering
focus mode; **bug** (S): discarding a draft from focus mode does not restore it (S:1995-1996
omits `setSide(false)`). Rule: focus mode remembers the sidebar state it found and restores
exactly that on every exit path. Peeking that starts on the edge and leaves leftwards without
touching the sidebar stays open until the pointer next leaves the sidebar (not specified; the
port should close it on edge leave unless the pointer entered the sidebar).

## 8. Space switching

`switchSpace(i)` (S:1477), from a Space dot, Mod+1..9 or the command menu:

1. No-op if `i` is the current Space or does not exist.
2. `dir = +1` if `i > current`, else `-1`. Current Space = i; the active editor dot resets to
   0; the place resets to Inbox (the selected thread per Space is kept).
3. The sidebar content re-renders and slides in (`slide-r` for +1, `slide-l` for -1; ±26 px,
   `--t-big`, `--e-spring`). Counts do **not** bump on a switch (S:1262 only bumps when there is
   no direction).
4. The list re-renders with `entering`: rows rise staggered.
5. The reader re-renders (a draft open in the composer is parked silently, S:1385).
6. The frame colour cross-fades: the back layer receives the new gradient and goes to opacity 1,
   the front goes to 0, and they swap roles (S:1181-1187); 380 ms `--e-out`. The `--f-*` frame
   tokens, grain opacity and the card's Post tokens switch instantly.

Undo across Spaces: the undo entry records its Space; undo is ignored in another Space
(S:1555), and the toast is not hidden on a switch. Proposed: hide the toast on switch.

On the desktop, the same machine runs per workspace (plan "UX decisions": each COSMIC
workspace has a SpaceLook; bar, dock and launcher chrome cross-fade the tint over 380 ms);
details in `21-SPACES.md`.

## 9. Undo toast and pull tab

### 9.1 Toast

Every op calls `pushUndo(text, snapshot)` (S:1548-1553): the undo entry becomes
`{space, snapshot}` (one level; a new op replaces it), the text is set, the toast shows, and the
ToastHold timer (5200 ms) restarts. Texts: "Archived", "Snoozed until tomorrow 08:00", "Marked
read", "Marked unread", "Starred", "Unstarred" (S); C adds "Moved to trash", "Back in the
inbox", "Woken", "Deleted for good", "Removed <label>", "Labelled <label>", "Snoozed · <when>"
(C:1844-1846, C:1873, C:1878, C:1999).

**Bug** (S): the snooze toast always says "tomorrow 08:00" whatever time was picked (S:1546;
the pick at S:1506 ignores the chosen item). The port names the chosen time.

Undo (S:1554-1561): ignored if there is no entry or it belongs to another Space; restores the
whole Space's thread snapshot; hides the toast; re-renders; rows that were not visible before
rise with `rise --t-big --e-spring backwards`. C restores one thread and plays `row-in`
(C:1922-1933).

### 9.2 Pull tab

State: `Idle`, `Dragging{x0, dx}` (S:1562-1572).

| From | Event | Guard | To | Effect |
| --- | --- | --- | --- | --- |
| Idle | pointer down on the tab | | Dragging{x0 = x, dx = 0} | capture the pointer; the tab's transition is disabled |
| Dragging | pointer move | | Dragging | `dx = clamp(x - x0, -6, 78)`; tab `translateX(dx)`; `armed = dx > 46` (accent fill) |
| Dragging | pointer up or cancel | `dx > 46` | Idle | restore the transition (the tab springs back, `--t-move --e-spring`); un-arm; **undo** |
| Dragging | pointer up or cancel | `dx <= 46` | Idle | spring back; un-arm |
| Idle | click | `abs(dx) < 3` | Idle | **undo**; `dx = 0` |

**Bug** (C): C resets `dx = 0` on pointer up (C:1955) and then treats `dx == 0` on the following
click as a tap (C:1959), so any drag that ends on the tab undoes even when it was not armed. S's
`abs(dx) < 3` test is the rule.

## 10. Undo send

### 10.1 Guards (`trySend(later)`, S:2316-2327)

Evaluated in order when Send is pressed or Mod+Enter:

1. If scheduling (Mod+Shift+Enter) and "Sends" is "Right away": set "Tomorrow · 08:00".
2. No recipient in To: the To row plays `shake-x` (420 ms), focus moves to the To input; stop.
3. The plain-text rendering matches `/\b(attach(ed|ment|ing)?|enclosed)\b|附件/i`, nothing is
   attached, and no warning is showing: show the warning line "You wrote about an attachment,
   and nothing is attached." with **Attach a file** (adds a file, removes the warning) and
   **Send anyway** (sends); stop.
4. Otherwise send.

### 10.2 Sending (`doSend`, S:2328-2345)

| Step | What |
| --- | --- |
| 1 | the composer closes: floats and bubble close, focus mode ends, a hidden sidebar is restored, the draft is deleted |
| 2 | page composer plays `compose-send` (620 ms); an inline reply gets `.parking`, which has no rule for `.inline-reply`, so it simply disappears (**bug**: no exit) |
| 3 | the reader re-renders at 560 ms (port: `settle(ComposeSend)`) |
| 4 | next frame: the send pill springs up; its ring runs 5 s linear |
| 5 | every 1 s: "Sending in 4 s", 3, 2, 1 (scheduled: "Scheduled · <time>" throughout) |
| 6 | at 0: Undo button hidden; text "Sent" or "Scheduled · waits in the outbox"; the pill hides after SentHold 1600 ms |
| Undo (before 0) | stop the countdown; hide the pill; re-open the composer page with the identical document (a new draft id) |

"Undo is the draft coming back exactly as it was: nothing has left the outbox yet" (S:2343).
C's outbox (C:2138-2186) is a separate demo of retry states, not undo send.

## 11. Command menu search

### 11.1 Operators (S:1594-1598)

Each operator is matched once (first occurrence, case-insensitive), removed from the query, and
shown as a token chip above the results:

| Operator | Filter | Token text |
| --- | --- | --- |
| `from:X` | sender name or address contains X | "from X" |
| `is:unread` | unread | "unread" |
| `is:starred` | starred | "starred" |
| `has:attachment` | has an attachment | "has an attachment" |
| `label:X` | has label X (lower-cased) | "label X" |

### 11.2 Fuzzy score (`fuzzy(q, text)`, S:2350-2363)

Both lower-cased. Word start = index 0 or previous character in `[\s\-_./@]`.

- Empty query: score 0, no marks.
- Substring at index `p`: `score = 100 + (40 if word start else 20) - 0.5 * p`; the substring
  is marked.
- Else, subsequence (every query character in order): for each matched character, `run += 1`
  and `score += 2 + 2 * run + (6 if word start else 0)`; an unmatched character resets `run` to
  0. If not all characters match: no match. Final `score -= 0.3 * (last_hit - first_hit)`. Each
  matched character is marked.

### 11.3 Ranking (`searchItems`, S:1600-1629)

Mail (threads in the current Space, not gone, passing every operator):

| Case | Score |
| --- | --- |
| subject matches | subject score + 10; snippet shown |
| sender matches and beats the subject score | sender score; sender marked in the snippet |
| body contains the query literally, and the best so far is below 60 | 60; snippet = up to 30 characters before, the match marked, 60 after |
| no query but operators present | 1 |
| then add | recency (prototype table: today 6/5, Yesterday 3, Mon 2, Sun 1, Fri 0) + 4 if unread |

People: contacts matched on "name address". Actions: Compose (C), Sync now, Hide/Show the
sidebar (Mod S), Go to <Space> (Mod n), matched on their title.

Result groups:

| Query | Groups, in order |
| --- | --- |
| empty, no operators | Actions (all), then Recent (top 5 mail) |
| otherwise | Top hit (the single best of mail, people, actions), Mail (up to 6), People (up to 3), Actions (up to 3; omitted when operators are present) |
| nothing matches | "Nothing in this Space matches. Search checks subjects, names, addresses and the text of every message." |

Selection resets to the first item on every keystroke; the selected item scrolls into view. The
recency table is prototype data; a real recency function is not specified.

Floating menus filter with the same `fuzzy` over "title key", sorted by score (S:2086-2088).
C's command menu is a plain substring filter over views, the selected thread's hover ops,
Compose, Sync, Simulate an arrival, Group by, and the peek modes (C:2197-2214).

## 12. Selection bubble

Shown only while a composer is open (S:2126-2146), re-evaluated on the next frame after every
`selectionchange` (S:2160).

| Condition | Result |
| --- | --- |
| no composer | hide |
| selection collapsed | hide, unless focus is inside the bubble (its link input) |
| selection outside the body or title | hide |
| selection rect has zero width | keep as is |
| otherwise | show and place (section 4.2) |

Contents: in the body, "<current block type> ▾" and a separator; then B, i, U, S, `</>`, a
separator, Link (S:2136-2141). Pressed state reflects the current formatting. Mouse down on the
bubble is prevented except on its input, so the selection survives. Clicking a format applies it
and re-places the bubble; Link turns the bubble into a URL input (section 2.7); "Turn ▾" hides
the bubble and opens the turn-into menu at the button (S:2163-2164).

## 13. Link pill

"the real destination of a link, like a browser's status bar … instant, and loud when the text
lies" (S:430, S:1811).

| Event | Effect |
| --- | --- |
| pointer over a link in the reader or a peek | remove any other pill; show one at the reader's bottom-left (left 12, bottom 12) immediately, no delay |
| pointer leaves that link | remove the pill |
| click the link | prevented in the prototype |

Content (S:1814-1828): `registered(host)` = the last two labels of the host name. If the link
text contains a domain whose registered part differs from the href's: danger pill "Goes to
**<real>**, not <shown>". Else: dim scheme and subdomain, bold registered domain, dim path. The
port must use the Public Suffix List for "registered domain" (the last-two-labels rule is wrong
for `co.uk` and similar); not specified which crate.

## 14. Destination preview

"actions preview their result: snooze names the time, archive lights its destination" (S:440).

| Event | Effect | Source |
| --- | --- | --- |
| pointer over a row's Archive or Snooze strip button | the sidebar item Archive or Snoozed gets `.dest` (inset accent ring, pulsing) | S:1832-1837 |
| pointer leaves that button | remove `.dest` | S:1837 |
| click any strip button (capture) | remove every `.dest` | S:1839 |
| pointer rests on any strip button | its label ("Archive → out of Inbox", "Snooze until…", "Label…", "Mark read") appears above after 350 ms, instantly while hover is warm | S:1287-1288, S:445-446 |

## 15. Peek modes

### 15.1 S

| Event | Effect | Source |
| --- | --- | --- |
| Shift+click a row | close any peek; add a scrim and a peek (the reader for that thread, with a close tool) inside the card | S:1514, S:1576-1582 |
| Space on an open thread hover card | same, for that thread | S:1804-1808 |
| click the scrim or the close tool, or Escape (no command menu open) | remove both, instantly (no exit animation) | S:1581-1583, S:1670 |
| unfold a quote or switch Reader/Original inside the peek | re-render the peek in place | S:1683-1692 |

A peek suppresses hover cards (S:1790). Closing has no animation; "opening gets a spring in
place" and closing is not specified.

### 15.2 C

`data-peek` on the shell is `side`, `center` or `full` (C:1612-1613, C:1619); the same reader element is
restyled, never moved ("restyling one container, never by moving the node", C:1368-1375). Centre
and full add a scrim; clicking it closes the reader (C:1812-1818). Switching mode removes the
scrim and re-opens the selected thread (C:1831-1840). Below 1000 px the side reader becomes a
slide-over (`translateX(102%)` -> 0, C:751-756).

## 16. Composer lifecycle and send animation

| Event | Effect | Source |
| --- | --- | --- |
| Compose (button, "C" action) | open a page composer in the reader; if one is already open, focus its body | S:1574, S:1941-1944 |
| Reply / Reply all | open an inline reply under the thread (one at a time), scroll it into view | S:1678-1679, S:1961-1964 |
| Forward | intended: open a page composer with "Fwd:" and the quoted message. **Bug** (S): the handler writes `ed.doc.blocks` and calls `nb()`/`renderBody()`, none of which exist in the document model S ends with (`newDoc` has `html`, no `blocks`), so Forward throws after opening an empty page | S:1680-1682, S:1903-1914 |
| open, page, no recipients | focus To after 60 ms | S:1967 |
| open, otherwise | caret at the start of the body | S:1968 |
| any edit | state pip turns warn, "Draft · saving…"; after 700 ms quiet: save, "Draft · saved just now" | S:1977-1980 |
| Keep for later (tool or Escape) | save if not empty; `park` (420 ms) then the reader returns; the draft appears in Today (`tab-in`) | S:1989-1994 |
| any reader re-render while composing (list click, op finishing, Space switch) | park silently (no animation) | S:1385 |
| Discard | delete the draft; the reader returns; no animation | S:1995-1996 |
| Focus mode (tool or Mod+Shift+F) | the page covers the card, text measure 70ch, sidebar hidden | S:1997, S:573-574 |
| Send | section 10 | |

C's composer is a floating card (compose-rise) that sends with `compose-send` and then shows the
outbox pill (C:2118-2186).

## 17. Focus management

| Situation | Rule | Source |
| --- | --- | --- |
| command menu opens | its input takes focus | S:1661 |
| command menu closes | S: focus not restored (**gap**); C: the shell takes focus (C:2257). Rule: restore focus to the element that had it | S:1663 |
| a menu opened from a field closes | the field takes the keyboard back (settled 2026-09-24): the caller passes the field `Focus::Controlled(request)` and calls `request.request()` from the menu's `onclose`; nothing is remounted (FINDINGS "Launcher gaps", sill Q44) | |
| any menu or the bubble is clicked | mouse down prevented; focus stays in the field that opened it | S:2099, S:1658, S:2135 |
| recipient added or removed | the same input keeps focus | S:2280, S:2266 |
| Cc shown | Cc input takes focus | S:2239 |
| editor handle moved by keys | the handle keeps focus | S:1457 |
| hover card, link pill, toast | never take focus | |
| focus ring | `:focus-visible` 2.5 px accent outline, offset 2, radius 4 | S:55 |
| desktop launcher | keyboard-first: never depends on hover for its initial state (cosmic-comp #2230) | plan, shell risks |

## 18. Escape by context

One Escape dismisses exactly one layer: the innermost. The table is the port's rule; the
"prototype" column records what S and C did.

| Priority | Context (innermost first) | Escape does | Prototype |
| --- | --- | --- | --- |
| 1 | a floating menu with keyboard focus in its field | close the menu | S:2095 (stops propagation) |
| 2 | the selection bubble's link input | hide the bubble | S:2155 |
| 3 | the command menu | close it | S:1656, S:1670; **bug**: also closes a peek behind it |
| 4 | the composer in focus mode | leave focus mode | S:2199 |
| 5 | the composer | park the draft | S:2199 (also triggers the global handler) |
| 6 | a list-opened menu (snooze, labels) | close it | **gap**: not handled in S |
| 7 | the peek | close it | S:1670 |
| 8 | C: an open view menu | close it | C:2260 |
| 9 | a hover card | not specified (proposed: close without warm) | |
| 10 | desktop launcher | close | plan sill |

## 19. Other list behaviours

| Event | Effect | Source |
| --- | --- | --- |
| click a row | select it, mark it read, push it to Today (the entry plays `tab-in` once) | S:1512-1515 |
| click a star | toggle; `star-pop`; sparks only when starring; Starred gulps; toast | S:1521-1528 |
| strip: Archive / Snooze(pick) / Read | section 8 of `05-MOTION.md` | S:1500-1510 |
| strip: Label | labels menu with checks; toggling updates chips and gulps that label | S:1507-1509 |
| Today entry × | `tab-out`, then removed | S:1481-1485 |
| Today "Clear" | empty Today, no animation | S:1486 |
| a Today entry | open that thread | S:1487-1488 |
| a draft entry | re-open that draft | S:1491-1492 |
| an account tile | filter the list to that account (address shown after the title); rows re-enter | S:1493 |
| a place or label | switch place | S:1495 |
| C: a view | switch view, close the reader | C:2023-2030 |
| C: hover ops are per view | Inbox: archive, snooze, label, trash, read; Starred: unstar, archive, read; Snoozed: wake, archive, trash; Archive: restore, trash, read; Trash: restore, purge; labels: unlabel, archive, trash | C:1587-1594 |
| C: Snooze and Label open a picker; the rest act | "Acting gets an exit. Opening gets a spring in place." | C:1962-1966, C:1339-1343 |

## 20. Desktop interactions (settled)

What the shell (sill, on shell-host) must do. Status: **settled (user)** = decided with the user
(plan, "UX decisions settled with the user"); **settled (plan)** = in the approved plan's
`<shell>` design; **proposed** = macOS behaviour from Appendix C adopted here, awaiting the
user; **not specified**. Appendix C confidence (H primary source, M reliable secondary, L
observed) is kept.

### 20.1 Dock

| Behaviour | Rule | Status | Source |
| --- | --- | --- | --- |
| Position, size | bottom; icons 48 px, magnified up to 96 px; magnification on by default; no auto-hide by default; all adjustable in Settings | settled (user) | plan |
| Click an icon | not running: launch (activation token; D-Bus activation when the app supports it); running: activate | settled (plan); H | plan sill; C-A |
| Click a running app with no open windows | re-open / un-minimize its most recent window | proposed (L for which window) | C-A |
| Repeated clicks on a running app | cycle its windows | settled (plan) ("click launch/activate/cycle"); differs from macOS, which does not cycle | plan sill |
| Right-click menu, order | the app's windows (and recent documents, not specified); Options > Keep in Dock, Open at Login, Show in Files; Show All Windows; Hide; Quit (Alt held: Force Quit) | proposed (Apple order, M); the plan's menu is "windows, desktop actions, Keep in Dock, Quit", so desktop actions (from the `.desktop` file) need a place: proposed after the windows | C-A; plan sill |
| Modifier clicks | macOS: Cmd-click reveals in Finder, Option-click switches and hides the current app, Option-Cmd-click hides the others. Mapping here (Cmd = Ctrl): Ctrl-click Show in Files, Alt-click switch and hide current, Ctrl+Alt-click hide others | proposed | C-A (H) |
| Drag an icon out | shows "Remove" once far enough out; release removes it from the dock; running apps cannot be removed | proposed (H); distance not specified | C-A |
| Drag a file onto an icon | opens the file with that app; the icon highlights while hovered | proposed (H) | C-A |
| Spring-loading | a file held over an icon for ~0.5 s activates the app | proposed (M) | C-A |
| Hover label | app name immediately above the icon in a rounded pill; delay and size not specified | proposed (L) | C-A |
| Badges, progress | red capsule badge top-right (action needed); progress ring; source: `com.canonical.Unity.LauncherEntry` | settled (plan) | plan sill; C-A |
| Running indicator | a dot ~4-5 pt below the icon | settled (plan) (indicator dots); geometry proposed (L) | plan sill; C-A |
| Bounce | informational: 1 s; critical: until the app is active; launch: until launched | proposed (H) | C-A; motion in `05-MOTION.md#10-shell-motion` |
| Auto-hide (when enabled) | shows when the pointer touches the edge and stays still for 0.2 s; ~0.5 s slide; a ~4 px strip stays | settled (user) for the numbers | plan; C-A |

### 20.2 Launcher

| Behaviour | Rule | Status | Source |
| --- | --- | --- | --- |
| Open / toggle | `sill launcher toggle` (COSMIC shortcut); `open --query` | settled (plan) | plan sill |
| Close | Escape, focus lost, click on the catcher (the transparent full-screen surface), or toggle | settled (plan) | plan sill |
| Navigate | Up / Down move the selection; Enter runs it | settled (plan) | plan sill |
| Providers | Tab cycles providers (apps, calculator, settings, windows, system) | settled (plan) | plan sill |
| Actions | Ctrl+K (Cmd+K via Toshy) and Alt+K open the selected result's actions | settled (plan) | plan sill |
| Latency | p95 < 100 ms open over 20 toggles | settled (plan) | plan sill |
| Search behaviour | the command menu's fuzzy score and grouping (section 11) | proposed | S |

### 20.3 Menu bar

| Behaviour | Rule | Status | Source |
| --- | --- | --- | --- |
| Click a menu title | toggles its menu | proposed (L) | C-D |
| Press, drag, release on an item | picks it | proposed (L) | C-D |
| Hover another title while a menu is open | switches menus instantly | proposed (L) | C-D |
| Open / close animation | open: none on macOS (conflicts with ds `menu-pop`; see `05-MOTION.md#12-open-decisions`); close: fade | proposed | C-D |
| Submenus | "safe triangle": moving toward an open submenu does not close it; submenu delay not specified | proposed (H for the triangle) | C-D |
| Geometry | bar 24 pt (macOS since Big Sur); status items up to 22 pt, icons 16 pt, disabled 35% opacity; item height ~22 pt, 13 pt font (L) | proposed | C-D |
| Every menu | an `xdg_popup` anchored to its button with ds `Menu`, keyboard grab on the seat | settled (plan) | plan sill |
| Workspace indicator | drag to reorder workspaces | settled (plan) | plan sill |

### 20.4 Focus and click-through

| Behaviour | Rule | Status | Source |
| --- | --- | --- | --- |
| First click on an inactive window | activates the window only; the click is not delivered, unless the control opted in (macOS `acceptsFirstMouse`) | proposed (H) | C-D |
| Moving a background window without activating it | macOS: Cmd-drag (M); mapping here not specified | not specified | C-D |
| Shell surfaces (dock, bar) | act on the first click (they never take keyboard focus except menus and the launcher) | proposed | |

### 20.5 Notifications

| Behaviour | Rule | Status | Source |
| --- | --- | --- | --- |
| Banner | top-right; dismisses itself after ~5 s; duration not user-changeable | proposed (M) | C-D |
| Alert | persists until acted on | proposed (M) | C-D |
| Swipe right on a banner | dismiss | proposed (M) | C-D |
| Control center numbers | none published | not specified | C-D |

### 20.6 Magic Mouse gestures

Apple's full set, mapped to COSMIC; every row is an entry in the gesture daemon's
gesture -> action table, and this mapping is the shipped default (plan "UX decisions").

| Gesture (Apple name) | Action here | Status | Source |
| --- | --- | --- | --- |
| Secondary click (click on the right side) | secondary click | settled (user) | plan; C-C |
| Scroll (one finger) | scroll, with momentum and rubber band owned by the host (`11-BEHAVIOUR-scroll.md`) | settled (user) | plan; C-C; Appendix B |
| Smart zoom (one-finger double-tap) | smart zoom where the app supports it | settled (user) | plan; C-C |
| Mission Control (two-finger double-tap) | workspace overview | settled (user) | plan; C-C |
| Swipe between pages (one finger left/right) | back / forward (Alt+Left / Alt+Right via uinput) | settled (user) | plan; C-C |
| Swipe between full-screen apps (two fingers left/right) | switch workspace: trigger on threshold now, live tracking later | settled (user) | plan; C-C |
| Axis lock | the existing daemon's axis lock (40 units, ratio 1.5, unlock after 0.25 s idle) is kept | settled (user) | plan; Appendix B |
| Thresholds for swipes | Apple's are unknown; candidates in Appendix C-C (WebKit GTK, libadwaita) | not specified | C-C |

### 20.7 Not covered here

Cmd+Tab (quick tap switches without UI; Tab/` step, Q quit, H hide, release switches, Escape
cancels; show delay unknown), Mission Control, minimize effects, UI sounds: see
`13-BEHAVIOUR-menus-windows.md` and `10-BEHAVIOUR-dock.md`. Not specified here.

## 21. Open decisions

1. **Keys typed in a field leak to global shortcuts** (2.1): adopt "a focused field owns the
   keys it handles"; decide whether Mod+K inside the composer means Link (Notion) or Command
   menu (Arc). Proposed: Link inside the composer body, command menu everywhere else.
2. **One Escape, one layer** (18): adopt the priority table.
3. **List menus need keyboard** (2.4): every ds `Menu` handles its own keys.
4. **Menu dismissal exemption** (5): only the menu's own trigger is exempt.
5. **Hover cards do not flip; `place()` does** (3): proposed `Flip::Never` for hover cards,
   flip for menus.
6. **Drop on Snoozed** (6.1): open the picker instead of snoozing to a default.
7. **Snooze toast text** (9.1): name the picked time.
8. **C pull-tab tap test** (9.2): use S's `abs(dx) < 3`.
9. **Inline reply send has no exit** (10.2): give it `park` or `compose-send`.
    Also: Forward is broken in S (16); the port builds the forwarded quote as an `o-rq` object like Reply does.
10. **Focus mode and the sidebar** (7): restore the state found on entry on every exit path.
11. **Peek close has no animation** (15.1): not specified; proposed none (closing is not acting).
12. **Toast on Space switch** (8): proposed hide.
13. **Registered-domain rule** (13): use the Public Suffix List.
14. **Dock right-click order vs the plan's list** (20.1): where desktop actions go.
15. **Dock repeated-click cycling** (20.1): the plan cycles windows, macOS does not. Confirm.
16. **Menu bar open animation** (20.3): none (macOS) vs `menu-pop` (ds).
17. **Background-window drag modifier** (20.4): not specified.
18. **Notification banner entry direction** (20.5): not specified.

## 22. Sources

- `~/mailo-design/mailo-spaces.html` (S): keys S:1665-1671, S:1804-1808, S:2194-2212,
  S:2256-2266, S:1452-1458; hover S:1697-1809; menus S:2051-2106; ops and undo S:1498-1572;
  peek and sidebar S:1576-1586, S:1842-1843; command menu S:1588-1671; composer S:1941-2345;
  bubble S:2124-2165; link pill S:1811-1829; destination S:1831-1839; fuzzy S:2350-2363.
- `~/mailo-design/mailo-charm.html` (C): data C:1587-1615; menus C:1739-1779; reader and peek
  C:1780-1840; ops, undo, picker C:1841-2031; drag C:2032-2088; compose and outbox C:2118-2186;
  command menu C:2193-2261; notes C:1331-1375.
- `~/.claude/plans/vast-toasting-peach.md`: "Design: `<ds>` design-system repo" (HoverIntent,
  place, PopoverRequest, Menu, ToastHost), "Design: `<shell>` repo" (bar, dock, launcher),
  "UX decisions settled with the user", Appendix A6, A8, Appendix B, Appendix C.
