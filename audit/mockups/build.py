"""Sizing audit mockups (design/29-SIZING.md): the bar, the control center and the shell
controls drawn from each option's numbers, plus the dark icon-plate fixes.

    python3 audit/mockups/build.py            # writes audit/mockups/*.html
    audit/mockups/render.sh                   # headless Chrome -> PNG at 1x and 2x

Every number a mockup draws comes from the OPTIONS table below, so the pictures and the
doc's token tables cannot drift apart.
"""

from __future__ import annotations

import math
from pathlib import Path

HERE = Path(__file__).resolve().parent

# ---------------------------------------------------------------------------------------------
# The options. "before" is quire + sill as they stand (measured, see audit/current/measured.txt).

OPTIONS = {
    "before": dict(
        name="Before (today)",
        bar_h=32, title_h=24, title_r=4, title_pad=10, status_w=22, status_h=22, status_r=4,
        glyph=16, bar_gap=4, bar_pad=8, font=13,
        cc_w=320, cc_pad=12, cc_gap=8, cc_r=14, mod_r=12,
        tile_h=52, disc=28, tile_pad="8px 4px 8px 10px",
        panel_pad="10px 12px", panel_gap=8, level_h=26, level_knob=None,
        seg_h=30, seg_r=15, seg_in=2, seg_thumb_r=13,
        tog_w=38, tog_h=26, knob=18, tog_r=13,
        sld_track=4, sld_knob=22, sld_ring=3,
        btn_h=38, btn_r=9, mini_h=24,
    ),
    "a": dict(
        name="A: the reference ladder (S16 / M22 / L28)",
        bar_h=24, title_h=22, title_r=5, title_pad=8, status_w=30, status_h=22, status_r=5,
        glyph=16, bar_gap=0, bar_pad=10, font=13,
        cc_w=320, cc_pad=10, cc_gap=10, cc_r=18, mod_r=8,
        tile_h=56, disc=28, tile_pad="0 10px",
        panel_pad="10px", panel_gap=6, level_h=22, level_knob=20,
        seg_h=22, seg_r=6, seg_in=1, seg_thumb_r=5,
        tog_w=38, tog_h=22, knob=20, tog_r=11,
        sld_track=4, sld_knob=20, sld_ring=0,
        btn_h=22, btn_r=5, mini_h=16,
    ),
    "b": dict(
        name="B: one unit per density (u = 24, Regular)",
        bar_h=32, title_h=24, title_r=6, title_pad=12, status_w=24, status_h=24, status_r=6,
        glyph=16, bar_gap=4, bar_pad=8, font=13,
        cc_w=320, cc_pad=12, cc_gap=8, cc_r=24, mod_r=12,
        tile_h=56, disc=32, tile_pad="0 12px",
        panel_pad="12px", panel_gap=8, level_h=24, level_knob=20,
        seg_h=28, seg_r=14, seg_in=2, seg_thumb_r=12,
        tog_w=38, tog_h=24, knob=20, tog_r=12,
        sld_track=4, sld_knob=20, sld_ring=0,
        btn_h=24, btn_r=6, mini_h=20,
    ),
    "c": dict(
        name="C: repair in place (prototype sizes kept)",
        bar_h=32, title_h=24, title_r=4, title_pad=10, status_w=24, status_h=24, status_r=4,
        glyph=16, bar_gap=4, bar_pad=8, font=13,
        cc_w=320, cc_pad=12, cc_gap=8, cc_r=24, mod_r=12,
        tile_h=52, disc=28, tile_pad="0 8px 0 10px",
        panel_pad="12px", panel_gap=8, level_h=24, level_knob=None,
        seg_h=30, seg_r=15, seg_in=2, seg_thumb_r=13,
        tog_w=44, tog_h=26, knob=22, tog_r=13,
        sld_track=4, sld_knob=22, sld_ring=3,
        btn_h=36, btn_r=9, mini_h=24,
    ),
}

LIGHT = dict(paper="#E9ECE6", card="#F6F7F4", plate="#EEF0EB", line="rgba(20,24,20,.10)",
             ink="#1A1E1A", soft="#4A5048", faint="#8A9088", accent="#8FB8FF",
             accent_soft="#DCE8FF", pill="rgba(20,24,20,.10)", well="rgba(20,24,20,.14)",
             fill="#FFFFFF", thumb="#1A1E1A", thumb_ink="#F6F7F4", knob="#FFFFFF",
             wall="linear-gradient(90deg,#b8c4dc,#c9d6d8 40%,#dcd4dc)")
DARK = dict(paper="#151814", card="#23261F", plate="#2C3029", line="rgba(255,255,255,.08)",
            ink="#E8EAE4", soft="#B8BDB4", faint="#7F867C", accent="#8FB8FF",
            accent_soft="#34405A", pill="rgba(255,255,255,.14)", well="rgba(255,255,255,.12)",
            fill="#F2F3F0", thumb="#E8EAE4", thumb_ink="#151814", knob="#F2F3F0",
            wall="linear-gradient(90deg,#1c2436,#20302f 40%,#2d2530)")

# ---------------------------------------------------------------------------------------------
# Glyphs (Lucide paths, 24 grid, stroke 2), drawn at the option's glyph size.

PATHS = {
    "wifi": '<path d="M12 20h.01"/><path d="M2 8.82a15 15 0 0 1 20 0"/><path d="M5 12.86a10 10 0 0 1 14 0"/><path d="M8.5 16.43a5 5 0 0 1 7 0"/>',
    "bt": '<path d="m7 7 10 10-5 5V2l5 5L7 17"/>',
    "moon": '<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>',
    "sun": '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>',
    "vol": '<path d="M11 4.7a.7.7 0 0 0-1.2-.5L6.4 7.6A1.4 1.4 0 0 1 5.4 8H3a1 1 0 0 0-1 1v6a1 1 0 0 0 1 1h2.4a1.4 1.4 0 0 1 1 .4l3.4 3.4a.7.7 0 0 0 1.2-.5z"/><path d="M16 9a5 5 0 0 1 0 6"/>',
    "switch": '<circle cx="8" cy="8" r="2"/><path d="M10 8h4a4 4 0 0 1 0 8h-4"/><rect x="2" y="4" width="20" height="8" rx="4"/><rect x="2" y="12" width="20" height="8" rx="4"/><circle cx="16" cy="16" r="2"/>',
    "chev": '<path d="m9 18 6-6-6-6"/>',
    "battery": '<rect x="2" y="7" width="16" height="10" rx="2"/><path d="M22 11v2"/><path d="M6 11v2M10 11v2"/>',
    "window": '<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 9h18"/><path d="M7 6.5h.01M10 6.5h.01"/>',
}


def glyph(name: str, size: int, colour: str = "currentColor", stroke: float = 2) -> str:
    return (f'<svg width="{size}" height="{size}" viewBox="0 0 24 24" fill="none" stroke="{colour}" '
            f'stroke-width="{stroke}" stroke-linecap="round" stroke-linejoin="round">{PATHS[name]}</svg>')


# ---------------------------------------------------------------------------------------------
# Pieces


def bar(o: dict, c: dict) -> str:
    title_pad = o["title_pad"]
    item = (f'display:inline-flex;align-items:center;height:{o["title_h"]}px;padding:0 {title_pad}px;'
            f'border-radius:{o["title_r"]}px;font-size:{o["font"]}px;')
    status = (f'display:inline-grid;place-items:center;width:{o["status_w"]}px;height:{o["status_h"]}px;'
              f'border-radius:{o["status_r"]}px;color:{c["soft"]};')
    ws = min(o["title_h"] - 4, 18)
    return f'''
<div style="height:{o["bar_h"]}px;display:flex;align-items:center;gap:{o["bar_gap"]}px;padding:0 {o["bar_pad"]}px;
  background:{c["wall"]};color:{c["ink"]};font-weight:500;box-shadow:inset 0 -1px 0 {c["line"]};">
  <span style="{item}font-weight:700;background:{c["pill"]}">Files</span>
  <span style="{item}">Edit</span><span style="{item}">View</span>
  <span style="{item}"><span style="display:inline-grid;place-items:center;min-width:{ws}px;height:{ws}px;border-radius:{max(2, o["title_r"] - 2)}px;
     box-shadow:inset 0 0 0 1px {c["line"]};font-size:11px">1</span></span>
  <span style="flex:1"></span>
  <span style="{status}">{glyph("battery", o["glyph"])}</span>
  <span style="{status}">{glyph("wifi", o["glyph"])}</span>
  <span style="{status}background:{c["pill"]};color:{c["ink"]}">{glyph("switch", o["glyph"])}</span>
  <span style="{item}font-variant-numeric:tabular-nums">Sun 27 Sep 12:48</span>
</div>'''


def toggle(o: dict, c: dict, on: bool) -> str:
    inset = (o["tog_h"] - o["knob"]) / 2
    x = o["tog_w"] - o["knob"] - inset if on else inset
    bg = c["accent"] if on else c["well"]
    return (f'<span style="position:relative;display:inline-block;width:{o["tog_w"]}px;height:{o["tog_h"]}px;'
            f'border-radius:{o["tog_r"]}px;background:{bg};flex:none">'
            f'<span style="position:absolute;top:{inset}px;left:{x}px;width:{o["knob"]}px;height:{o["knob"]}px;'
            f'border-radius:50%;background:{c["knob"]};box-shadow:0 1px 2px rgba(0,0,0,.25),0 0 0 .5px rgba(0,0,0,.06)"></span></span>')


def slider(o: dict, c: dict, width: int = 140, f: float = .6) -> str:
    k = o["sld_knob"]
    ring = f"border:{o['sld_ring']}px solid #fff;box-sizing:border-box;background:{c['accent']};" if o["sld_ring"] else f"background:{c['knob']};"
    return (f'<span style="position:relative;display:inline-block;width:{width}px;height:{k}px;flex:none">'
            f'<span style="position:absolute;left:0;right:0;top:{(k - o["sld_track"]) / 2}px;height:{o["sld_track"]}px;border-radius:{o["sld_track"] / 2}px;background:{c["well"]}"></span>'
            f'<span style="position:absolute;left:0;width:{f * (width - k) + k / 2}px;top:{(k - o["sld_track"]) / 2}px;height:{o["sld_track"]}px;border-radius:{o["sld_track"] / 2}px;background:{c["accent"]}"></span>'
            f'<span style="position:absolute;top:0;left:{f * (width - k)}px;width:{k}px;height:{k}px;border-radius:50%;{ring}box-shadow:0 1px 3px rgba(0,0,0,.3)"></span></span>')


def segmented(o: dict, c: dict, labels=("System", "Light", "Dark"), width: int = 0, font: int = 12) -> str:
    h, i = o["seg_h"], o["seg_in"]
    n = len(labels)
    w = width or n * 64
    seg_w = (w - 2 * i) / n
    cells = "".join(
        f'<span style="position:relative;z-index:1;display:grid;place-items:center;width:{seg_w}px;height:{h - 2 * i}px;'
        f'font-size:{font}px;font-weight:600;color:{c["thumb_ink"] if k == 0 else c["soft"]}">{t}</span>' for k, t in enumerate(labels))
    return (f'<span style="position:relative;display:inline-flex;box-sizing:border-box;width:{w}px;height:{h}px;padding:{i}px;'
            f'border-radius:{o["seg_r"]}px;background:{c["well"]};flex:none">'
            f'<span style="position:absolute;top:{i}px;left:{i}px;width:{seg_w}px;height:{h - 2 * i}px;border-radius:{o["seg_thumb_r"]}px;background:{c["thumb"]}"></span>'
            f'{cells}</span>')


def button(o: dict, c: dict, label: str, h: int, primary: bool) -> str:
    r = o["btn_r"] if h == o["btn_h"] else max(3, round(o["btn_r"] * h / o["btn_h"]))
    bg = c["accent"] if primary else c["card"]
    return (f'<span style="display:inline-flex;align-items:center;height:{h}px;box-sizing:border-box;padding:0 {max(8, h // 2)}px;'
            f'border-radius:{r}px;background:{bg};box-shadow:inset 0 0 0 1px {c["line"]},0 1px 1px rgba(0,0,0,.08);'
            f'font-size:{13 if h >= 22 else 11}px;font-weight:600;color:{c["ink"] if not primary else "#10203a"};flex:none">{label}</span>')


def level(o: dict, c: dict, glyph_name: str, f: float) -> str:
    h = o["level_h"]
    knob = o["level_knob"]
    fill_w = f"calc({f} * (100% - {h}px) + {h}px)" if knob else f"{f * 100}%"
    knob_html = ""
    if knob:
        inset = (h - knob) / 2
        knob_html = (f'<span style="position:absolute;top:{inset}px;left:calc({f} * (100% - {h}px) + {inset}px);width:{knob}px;height:{knob}px;'
                     f'border-radius:50%;background:#fff;box-shadow:0 1px 3px rgba(0,0,0,.3)"></span>')
    return (f'<span style="position:relative;display:block;height:{h}px;border-radius:{h / 2}px;background:{c["well"]};overflow:hidden">'
            f'<span style="position:absolute;left:0;top:0;bottom:0;width:{fill_w};border-radius:{h / 2}px;background:{c["fill"]}"></span>'
            f'<span style="position:absolute;left:{(h - 16) / 2 + 2}px;top:{(h - 16) / 2}px;color:{c["soft"]}">{glyph(glyph_name, 16)}</span>'
            f'{knob_html}</span>')


def tile(o: dict, c: dict, name: str, status: str, g: str, on: bool) -> str:
    disc_bg = c["accent"] if on else c["card"]
    plate = c["accent_soft"] if on else c["plate"]
    return (f'<div style="display:flex;align-items:center;gap:8px;height:{o["tile_h"]}px;box-sizing:border-box;padding:{o["tile_pad"]};'
            f'border-radius:{o["mod_r"]}px;background:{plate};box-shadow:inset 0 0 0 1px {c["line"]};min-width:0">'
            f'<span style="display:grid;place-items:center;width:{o["disc"]}px;height:{o["disc"]}px;border-radius:50%;background:{disc_bg};color:{c["ink"]};flex:none;box-shadow:0 1px 2px rgba(0,0,0,.12)">{glyph(g, 16 if o["disc"] <= 28 else 18)}</span>'
            f'<span style="display:flex;flex-direction:column;flex:1;min-width:0;line-height:1.2"><b style="font-size:13px;font-weight:600">{name}</b>'
            f'<span style="font-size:11px;color:{c["faint"]}">{status}</span></span>'
            f'<span style="color:{c["faint"]}">{glyph("chev", 14)}</span></div>')


def panel(o: dict, c: dict, title: str, g: str, trailing: str, body: str) -> str:
    return (f'<div style="grid-column:1/-1;display:flex;flex-direction:column;gap:{o["panel_gap"]}px;padding:{o["panel_pad"]};'
            f'border-radius:{o["mod_r"]}px;background:{c["plate"]};box-shadow:inset 0 0 0 1px {c["line"]}">'
            f'<div style="display:flex;align-items:center;gap:6px;height:16px;font-size:13px;font-weight:600">'
            f'<span style="color:{c["soft"]};display:flex">{glyph(g, 16)}</span><span style="flex:1">{title}</span>'
            f'<span style="font-size:11px;color:{c["faint"]};font-weight:500">{trailing}</span></div>{body}</div>')


def control_center(o: dict, c: dict) -> str:
    inner = o["cc_w"] - 2 * o["cc_pad"]
    appearance = (f'<div style="display:flex;flex-direction:column;gap:{o["panel_gap"]}px">'
                  f'{segmented(o, c, width=inner - 2 * int(o["panel_pad"].split()[-1].rstrip("px")), font=12)}</div>')
    return f'''
<div style="width:{o["cc_w"]}px;box-sizing:border-box;padding:{o["cc_pad"]}px;border-radius:{o["cc_r"]}px;background:{c["card"]};
  box-shadow:0 12px 32px rgba(0,0,0,.18),inset 0 0 0 1px {c["line"]};color:{c["ink"]};
  display:grid;grid-template-columns:1fr 1fr;gap:{o["cc_gap"]}px">
  {tile(o, c, "Wi-Fi", "Home", "wifi", True)}
  {tile(o, c, "Bluetooth", "Off", "bt", False)}
  {tile(o, c, "Focus", "Off", "moon", False)}
  <div></div>
  {panel(o, c, "Display", "sun", "60%", level(o, c, "sun", .6))}
  {panel(o, c, "Sound", "vol", "35%", level(o, c, "vol", .35))}
  {panel(o, c, "Appearance", "moon", "Light", appearance)}
</div>'''


def controls_row(o: dict, c: dict) -> str:
    return (f'<div style="display:flex;align-items:center;gap:14px;flex-wrap:wrap;color:{c["ink"]}">'
            f'{toggle(o, c, True)}{toggle(o, c, False)}{slider(o, c)}{segmented(o, c)}'
            f'{button(o, c, "Cancel", o["btn_h"], False)}{button(o, c, "Restart", o["btn_h"], True)}'
            f'{button(o, c, "Mini", o["mini_h"], False)}</div>')


def ruler(o: dict) -> str:
    keys = ["bar_h", "title_h", "status_w", "status_h", "tile_h", "level_h", "seg_h", "tog_w", "tog_h", "knob", "btn_h", "mini_h", "cc_pad", "cc_gap", "cc_r", "mod_r"]
    return " · ".join(f"{k} {o[k]}" for k in keys)


PAGE = """<!doctype html><html><head><meta charset="utf-8"><title>{title}</title><style>
body{{margin:0;background:{bg};font-family:'Inter Variable',Inter,system-ui,sans-serif;color:{ink};-webkit-font-smoothing:antialiased}}
.sheet{{padding:20px;display:flex;flex-direction:column;gap:18px;width:{w}px}}
h2{{margin:0;font-size:13px;letter-spacing:.04em;text-transform:uppercase;color:{faint}}}
.cap{{font-size:12px;font-weight:600;margin:0 0 6px}} .nums{{font:10px ui-monospace,monospace;color:{faint};margin-top:4px}}
.row{{display:flex;gap:20px;align-items:flex-start;flex-wrap:wrap}}
</style></head><body><div class="sheet">{body}</div></body></html>"""


def page(title: str, body: str, c: dict, w: int) -> str:
    return PAGE.format(title=title, body=body, bg=c["paper"], ink=c["ink"], faint=c["faint"], w=w)


def write_bars() -> None:
    for scheme, c in (("light", LIGHT), ("dark", DARK)):
        body = "<h2>The bar: before and the three options (1 css px = 1 pt)</h2>"
        for key, o in OPTIONS.items():
            body += f'<div><p class="cap">{o["name"]}</p>{bar(o, c)}<div class="nums">{ruler(o)}</div></div>'
        (HERE / f"bar-{scheme}.html").write_text(page("Bar options", body, c, 900))


def write_cc() -> None:
    for scheme, c in (("light", LIGHT), ("dark", DARK)):
        cells = "".join(f'<div><p class="cap">{o["name"]}</p>{control_center(o, c)}</div>' for o in OPTIONS.values())
        body = f"<h2>Control center at 320 wide: before and the three options</h2><div class='row'>{cells}</div>"
        (HERE / f"cc-{scheme}.html").write_text(page("Control center options", body, c, 1400))


def write_controls() -> None:
    for scheme, c in (("light", LIGHT), ("dark", DARK)):
        body = "<h2>Shell controls: toggle on/off, slider, segmented, buttons (regular, primary, mini)</h2>"
        for o in OPTIONS.values():
            body += f'<div><p class="cap">{o["name"]}</p><div style="background:{c["card"]};padding:14px;border-radius:10px">{controls_row(o, c)}</div></div>'
        (HERE / f"controls-{scheme}.html").write_text(page("Control options", body, c, 900))


# ---------------------------------------------------------------------------------------------
# Dark icon plates: OKLCh maths, the same as ds::icon::retint (lightness kept, hue and chroma
# replaced), then the three fixes.


def _lin(v: float) -> float:
    return v / 12.92 if v <= 0.04045 else ((v + 0.055) / 1.055) ** 2.4


def _enc(v: float) -> float:
    v = min(1.0, max(0.0, v))
    return 12.92 * v if v <= 0.0031308 else 1.055 * v ** (1 / 2.4) - 0.055


def oklch(hexa: str) -> tuple[float, float, float]:
    r, g, b = (_lin(int(hexa[i:i + 2], 16) / 255) for i in (1, 3, 5))
    l_ = (0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b) ** (1 / 3)
    m_ = (0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b) ** (1 / 3)
    s_ = (0.0883024619 * r + 0.2817188376 * g + 0.6299787745 * b) ** (1 / 3)
    L = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720619 * s_
    a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_
    bb = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_
    return L, math.hypot(a, bb), math.degrees(math.atan2(bb, a))


def hexa(L: float, C: float, h: float) -> str:
    while True:
        a, b = C * math.cos(math.radians(h)), C * math.sin(math.radians(h))
        l_ = (L + 0.3963377774 * a + 0.2158037573 * b) ** 3
        m_ = (L - 0.1055613458 * a - 0.0638541728 * b) ** 3
        s_ = (L - 0.0894841775 * a - 1.2914855480 * b) ** 3
        rgb = (4.0767416621 * l_ - 3.3077115913 * m_ + 0.2309699292 * s_,
               -1.2684380046 * l_ + 2.6097574011 * m_ - 0.3413193965 * s_,
               -0.0041960863 * l_ - 0.7034186147 * m_ + 1.7076147010 * s_)
        if all(-1e-6 <= v <= 1 + 1e-6 for v in rgb) or C <= 0:
            return "#" + "".join(f"{round(_enc(v) * 255):02X}" for v in rgb)
        C = max(0.0, C - 0.004)


def mono(colour: str, tint=(265.0, 0.05)) -> tuple[float, float, float]:
    L, _, _ = oklch(colour)
    shape = max(0.0, 1 - (2 * L - 1) ** 2)
    return L, tint[1] * shape, tint[0]


def luminance(colour: str) -> float:
    r, g, b = (_lin(int(colour[i:i + 2], 16) / 255) for i in (1, 3, 5))
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(x: str, y: str) -> float:
    a, b = sorted((luminance(x), luminance(y)))
    return (b + 0.05) / (a + 0.05)


NEUTRAL_DARK = ("#2A2E28", "#1D211B", "#E8EAE4")
NEUTRAL_LIGHT = ("#FFFFFF", "#F1F3EE", "#1A1E1A")
RASTER_DARK = "#202124"  # a third-party raster's darkest ink (Chrome's ring, a black logo)
DOCK_DARK = "#2A2C30"

FIXES = {
    "before": ("Before: lightness kept", lambda L: L, lambda L: L, NEUTRAL_DARK),
    "f1": ("F1: plate floor L >= .50", lambda L: max(L, 0.50), lambda L: L, NEUTRAL_DARK),
    "f2": ("F2: the light plate in both schemes", lambda L: L, lambda L: L, NEUTRAL_LIGHT),
    "f3": ("F3: tone band (plate .46-.56, raster lifted into .62-.96)", lambda L: 0.46 + 0.10 * L / 0.30 if L < 0.30 else L,
           lambda L: 0.62 + 0.34 * L, NEUTRAL_DARK),
}


def plate_html(stops, plate_map, raster_map, glyph_only: bool, tint=(265.0, 0.05)) -> tuple[str, dict]:
    base, deep, ink = stops
    Lb, Cb, hb = mono(base, tint)
    Ld, Cd, hd = mono(deep, tint)
    b_hex = hexa(plate_map(Lb), Cb if plate_map(Lb) == Lb else tint[1] * .6, hb)
    d_hex = hexa(plate_map(Ld), Cd if plate_map(Ld) == Ld else tint[1] * .6, hd)
    Li, Ci, hi = mono(ink, tint)
    i_hex = hexa(Li, Ci, hi)
    Lr, Cr, hr = mono(RASTER_DARK, tint)
    r_hex = hexa(raster_map(Lr), tint[1] * .5, hr)
    inner = glyph("window", 26, i_hex) if glyph_only else (
        f'<span style="display:grid;place-items:center;width:34px;height:34px;border-radius:9px;background:#ECEDEF">'
        f'<span style="width:18px;height:18px;border-radius:50%;border:4px solid {r_hex};box-sizing:border-box"></span></span>')
    html = (f'<span style="display:grid;place-items:center;width:48px;height:48px;border-radius:12px;'
            f'background:linear-gradient(135deg,{b_hex},{d_hex});box-shadow:inset 0 1px 0 rgba(255,255,255,.12),0 1px 2px rgba(0,0,0,.4)">{inner}</span>')
    return html, dict(base=b_hex, ink=i_hex, raster=r_hex)


def write_plates() -> None:
    rows = ""
    notes = []
    for key, (name, plate_map, raster_map, stops) in FIXES.items():
        cells = ""
        facts = None
        for glyph_only in (True, False, True, False):
            html, facts = plate_html(stops, plate_map, raster_map, glyph_only)
            cells += html
        if key != "f2":
            dock_c = contrast(facts["base"], DOCK_DARK)
        else:
            dock_c = contrast(facts["base"], DOCK_DARK)
        notes.append(f'{name}: plate {facts["base"]} ({dock_c:.2f}:1 on the dark dock {DOCK_DARK}), '
                     f'glyph ink {facts["ink"]} ({contrast(facts["ink"], facts["base"]):.2f}:1 on the plate), raster dark ink {facts["raster"]}')
        rows += (f'<div><p class="cap">{name}</p><div style="display:inline-flex;gap:12px;padding:10px 14px;border-radius:22px;'
                 f'background:{DOCK_DARK};box-shadow:inset 0 0 0 1px rgba(255,255,255,.08)">{cells}</div>'
                 f'<div class="nums">{notes[-1]}</div></div>')
    body = "<h2>Monochrome in dark: the neutral plate (glyph, and a third-party raster) on the dark dock</h2>" + rows
    (HERE / "plates-dark.html").write_text(page("Dark plate fixes", body, DARK, 900))
    (HERE / "plates-dark.txt").write_text("\n".join(notes) + "\n")


if __name__ == "__main__":
    write_bars()
    write_cc()
    write_controls()
    write_plates()
    print("wrote", sorted(p.name for p in HERE.glob("*.html")))
