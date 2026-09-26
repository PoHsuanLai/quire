# 09. Arc heuristics: the workflow brief

Source: Gautham, "Arc Browser: Rethinking the Web Through a Designer's Lens", Medium (Design
Bootcamp), 2025-04-20. The article is an appreciation, not a spec: it names principles and
heuristics and gives no sizes, timings or colours. What it does give is a usable brief for how a
surface or app should *behave* and *feel*, which our docs so far cover only through the mail
prototypes (00 §6 lists which visual parts are Arc). This doc turns the article into rules an
agent brief can cite, each with a check. Where a rule already has a home, it points there
rather than restating it.

Confidence: the rules are the article's own claims (a designer's reading of Arc), marked A;
the mappings to our surfaces are ours, marked P (proposed).

## 1. The stance the article adds

- "Good design isn't just about solving surface-level pain points — it's about going deeper,
  redefining problems, and changing behavior." Arc "dared to question the familiar, challenged
  Jakob's Law, and created something meaningfully better." (A)
- "Let's stop designing for familiarity, and start designing for clarity, flow, and joy." (A)
- The flaws the author names: "the fun, playful design might feel out of place in strictly
  professional environments", and "for less tech-savvy users, the learning curve can be steep
  despite the strong onboarding." (A)

Our reading (P): we keep the Mac's conventions where they carry no cost (menus, windows, the
dock: 00 §8.5 "Mac behaviour, Arc surface") and break convention only where the break pays for
itself in flow, and then we onboard the break. Playfulness lives in motion and copy, never in
the chrome's legibility (00 §3 "Colour is a claim"); an app must still read as serious in a
meeting.

## 2. The seven heuristics

| # | Heuristic (A) | Rule for us (P) | Check |
| --- | --- | --- | --- |
| H1 | **Design for workflows, not just features.** "Spaces are like browser-based workspaces — tailored environments for specific mindsets." The author runs Work, Research, Career, Fun, Learning, and switches "mental modes" with them. | A workflow is a first-class object: a desktop Space (21) or an app Space (mailo) has its own name, look (SpaceLook), contents and shortcuts; switching one switches everything the user sees, with the content slide and frame cross-fade (21, 05). No feature lives outside a Space. | Every surface names which Space it belongs to; `Ctrl 1..9` switches; the frame tint changes with it; nothing else has to be re-chosen after a switch. |
| H2 | **The sidebar is the paradigm.** Shortcuts on top, folders that make bookmarks usable, open tabs vertical, Spaces switching with a two-finger swipe. "Everything has a place, so your brain doesn't have to juggle." | Apps use one left sidebar drawn on the Space colour (00 §6): pinned 4-up tiles, places, folders, a living "Today" list that drops off by itself (mailo), Space dots in the foot. No horizontal tab strips. Two-finger horizontal swipe switches Spaces (12). | The sidebar order is pins, places, folders, today, foot; a vertical list replaces any tab strip; the two-finger swipe is mapped and configurable (22 gestures). |
| H3 | **Split view and vertical tabs: use space for focus and comparison.** "Think more like Figma." | A shell surface or app pane can be split to compare two things side by side without leaving the Space; the window's Zoom/tile menu offers halves (13 §13.3.11). | Split is reachable from the frame's tile menu and a shortcut; both halves keep the Space's chrome. |
| H4 | **Integrate related tools, don't silo them.** Easels and Notes live inside the browser. | A quick capture surface (Quick Note, 20 §1.15; the composer's objects in mailo) is one shortcut away from any surface and lands in the current Space. | A note or capture can be made without switching app or Space; it shows where it went (the "gulp" on the destination, 05). |
| H5 | **Fewer icons, more shortcuts.** "Teach users to be efficient by encouraging habits." | Chrome shows few glyphs; every action has a key, the command menu lists keys next to actions and is "the same menu, just bigger and centred" (06); tooltips show the key. Onboarding teaches the three keys that matter first. | Every menu item shows its shortcut (`Kbd`); the command palette ranks actions with keys; no toolbar exceeds the count 20 sets for its surface. |
| H6 | **Boosts: power to the people.** Themes, shortcuts and Boosts "let users mold the browser around their unique needs". | Every default is a setting (22 §0 rule); looks, accents, motion levels and Space palettes are user-editable in one picker (AppearancePicker); apps may hide any distraction (a pane, a badge, a sound) per Space. | Each surface's row in 22 exists and every proposed value is a key; the picker reaches every look. |
| H7 | **Onboarding that delights.** "A smooth import process, fun language, and a sense of joy — that's how you retain users." The break with familiarity is paid for by onboarding. | First run imports what exists (accounts, calendars, wallpapers, pinned apps), explains each convention break once, in the design's voice (short, warm, no exclamation marks), and never blocks. | A first-run flow exists per app and for the shell (M12+); each unconventional element (sidebar, Spaces, command pill) has one explanatory moment that can be skipped and re-found in Help. |

## 3. Visual principles the article names (already ours)

- "Clean, with soft gradients, purposeful typography, and a layout that respects space. The
  design creates mental calm." → the Space frame gradient with grain (03 §Palette), three
  faces with every pair fixed (02), the spacing scale and 8 px clamp (01). No new rule.
- "Minimal and frictionless ... clear, structured, and responsive." → the four rules (00 §3) and
  the motion principle "springs only on contact" (00 §4).
- "Vertical tabs make managing dozens of pages feel natural", "fullscreen browsing" → H2, H3.

## 4. What the article does not give

No sizes, radii, durations or colours. For Arc's actual values we rely on the prototype's
calibration to Arc's `--arc-palette-*` exports (00 §6, 03) and on measurement, not on this
source. Do not cite this doc for a number.

## 5. How an agent brief uses this

Cite `design/09-ARC-HEURISTICS.md#2-the-seven-heuristics` and name the heuristic (H1..H7) the
work serves; run the `Workflow` block of `CHECKLIST.md` at review. A surface that serves none
of the seven is chrome and should be as small and conventional as 13 allows.

## Sources

- Gautham, "Arc Browser: Rethinking the Web Through a Designer's Lens", Medium, 2025-04-20
  (read 2026-09-26 through a text mirror; Medium refuses direct fetches).
- 00-PRINCIPLES.md §6 (which visual parts are Arc), §8.5; 21-SPACES.md; 06-INTERACTIONS.md
  (command menu); 12-BEHAVIOUR-gestures.md (two-finger swipe); 22-SETTINGS.md §0.
