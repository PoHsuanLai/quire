# 01 Layout: the frame model (archived 2026-10-02)

> **History, read-only.** Moved from `design/01-LAYOUT.md` sections 3 (the window) and 5 (the card).
> The window is now flush: sidebar and content side by side, no frame, no inset card, no card shadow
> (`../30-CATALOGUE.md` section 3.4). Source keys `S`, `C` are the Arc-era prototypes.

## 3. The window

The window is a coloured frame with the sidebar on the colour and a card inset 8 px on three
sides. Verbatim:

```css
.win{
  --f-ink:#1b1b22; --f-ink-soft:#44444f; --f-ink-faint:#5d5d6a;
  --f-pill:rgba(255,255,255,.58); --f-pill-hover:rgba(255,255,255,.34); --f-line:rgba(0,0,0,.08);
  position:relative; min-width:980px; height:680px; border-radius:18px; overflow:hidden;
  display:grid; grid-template-columns:232px minmax(0,1fr); padding:8px 8px 8px 0;
  color:var(--f-ink); isolation:isolate; box-shadow:0 18px 40px -22px rgba(0,0,0,.45);
  transition:grid-template-columns var(--t-move) var(--e-out), padding var(--t-move) var(--e-out);
}
.win.no-side{ grid-template-columns:0 minmax(0,1fr); padding-left:8px; }
.layer{ position:absolute; inset:0; z-index:-2; transition:opacity 380ms var(--e-out); }
.grain{ position:absolute; inset:0; z-index:-1; pointer-events:none; mix-blend-mode:overlay; background-size:128px 128px; }
```

`S:77-87`

| Property | Value | Source |
| --- | --- | --- |
| Minimum width | 980 | `S:80` |
| Height (prototype) | 680 | `S:80` |
| Corner radius | 18 | `S:80` |
| Columns | sidebar 232, card `minmax(0,1fr)` | `S:81` |
| Padding | top 8, right 8, bottom 8, left 0 (the sidebar supplies its own left padding) | `S:81` |
| Sidebar hidden | columns `0 minmax(0,1fr)`, padding-left 8 | `S:85` |
| Frame layers | two `.layer` elements (a and b) for the Space cross-fade, then `.grain`, then content | `S:835-838` |

`C` has no frame. Its shell is one bordered panel with three columns
`206px minmax(0,1fr) minmax(0,1.05fr)`, height `min(72vh,700px)`, min-height 560, border 1 px
`--line`, radius `--r-panel`, background `--surface`, shadow `--shadow-2` (`C:262-268`). `S` wins.

## 5. The card

The card is Post, inset 8 px, with a flap corner and a list and reader side by side.

```css
.card{
  position:relative; display:grid; grid-template-columns:minmax(0,1fr) minmax(0,1.08fr); min-width:0;
  background:var(--surface); border-radius:12px 12px 12px 4px; overflow:hidden;
  box-shadow:0 0 0 1px rgba(0,0,0,.06), 0 10px 30px -14px rgba(0,0,0,.45);
}
.list-col{ display:flex; flex-direction:column; min-width:0; border-right:1px solid var(--line); min-height:0; }
.list-bar{ display:flex; align-items:center; gap:8px; padding:11px 14px; border-bottom:1px solid var(--line-soft); }
.list-bar .spacer{ margin-left:auto; }
.list{ list-style:none; margin:0; padding:8px; overflow-y:auto; flex:1; min-height:0; }
```

`S:156-164`, `S:174`

| Part | Rule | Source |
| --- | --- | --- |
| Columns | list `minmax(0,1fr)`, reader `minmax(0,1.08fr)` | `S:157` |
| Radius | `12px 12px 12px 4px` (the flap corner is bottom-left) | `S:158` |
| List bar | padding 11/14, gap 8, bottom border `--line-soft`; title, spacer, "Sidebar" mini (only when hidden), Compose mini | `S:162-164`, `S:852-857` |
| List | padding 8, scrolls | `S:174` |
| Divider | list column right border 1 px `--line` | `S:161` |

