"""Build the progress page: a dated log of milestones with embedded screenshots.

Run with:  uv run --with pillow python tools/progress/build.py
Reads tools/progress/manifest.json, writes tools/progress/out/progress.html.
Images are downscaled to at most MAX_W px wide and embedded as data URIs.
"""

from __future__ import annotations

import base64
import html
import io
import json
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
OUT = HERE / "out" / "progress.html"
MAX_W = 1100


def embed(path: str, upscale: int = 1) -> tuple[str, int, int]:
    img = Image.open(Path(path).expanduser())
    if img.mode not in ("RGB", "RGBA"):
        img = img.convert("RGBA")
    if upscale > 1:
        img = img.resize((img.width * upscale, img.height * upscale), Image.NEAREST)
    if img.width > MAX_W:
        ratio = MAX_W / img.width
        img = img.resize((MAX_W, round(img.height * ratio)), Image.LANCZOS)
    buf = io.BytesIO()
    if img.mode == "RGBA" and any(a < 255 for a in img.getchannel("A").getdata()):
        img.save(buf, format="PNG", optimize=True)
        mime = "image/png"
    else:
        img.convert("RGB").save(buf, format="JPEG", quality=74, optimize=True)
        mime = "image/jpeg"
    data = base64.b64encode(buf.getvalue()).decode("ascii")
    return f"data:{mime};base64,{data}", img.width, img.height


def figure(im: dict) -> str:
    src, w, h = embed(im["path"], im.get("upscale", 1))
    cap = html.escape(im.get("caption", ""))
    pix = " pixel" if im.get("upscale", 1) > 1 else ""
    return (
        f'<figure class="shot{pix}"><img src="{src}" width="{w}" height="{h}" alt="{cap}" loading="lazy">'
        f"<figcaption>{cap}</figcaption></figure>"
    )


def entry(e: dict) -> str:
    figs = "".join(figure(im) for im in e.get("images", []))
    status = e.get("status", "done")
    notes = "".join(f"<li>{html.escape(n)}</li>" for n in e.get("notes", []))
    return f"""
<article class="entry" id="{html.escape(e['id'])}">
  <header>
    <div class="when">{html.escape(e['date'])}</div>
    <h2>{html.escape(e['title'])}</h2>
    <span class="chip" data-status="{status}">{html.escape(status)}</span>
  </header>
  <p class="lede">{html.escape(e['lede'])}</p>
  {f'<ul class="notes">{notes}</ul>' if notes else ''}
  <div class="shots">{figs}</div>
</article>"""


def build() -> None:
    manifest = json.loads((HERE / "manifest.json").read_text())
    entries = "".join(entry(e) for e in manifest["entries"])
    page = TEMPLATE.replace("{{ENTRIES}}", entries).replace(
        "{{UPDATED}}", html.escape(manifest["updated"])
    )
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(page)
    print(f"wrote {OUT} ({OUT.stat().st_size // 1024} KB, {len(manifest['entries'])} entries)")


TEMPLATE = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<title>Desktop Shell Progress</title>
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Bricolage+Grotesque:opsz,wght@12..96,500..800&family=Karla:ital,wght@0,400..700;1,400&family=Space+Mono:wght@400;700&display=swap">
<style>
:root{
  color-scheme:light;
  --paper:#E9ECE6; --surface:#F8F9F6; --surface-2:#F1F3EE; --raise:#FFFFFF;
  --ink:#1A1E1A; --ink-soft:#586057; --ink-faint:#676E65;
  --line:#D6DBD0; --line-soft:#E3E7DE;
  --accent:#23508F; --accent-ink:#F4F8FF; --accent-soft:#DCE5F3;
  --ok:#2C7A57; --warn:#A5761A; --danger:#B03A2A;
  --shadow-1:0 1px 0 rgba(255,255,255,.7) inset, 0 1px 2px rgba(26,30,26,.10);
  --shadow-2:0 1px 0 rgba(255,255,255,.7) inset, 0 6px 16px -6px rgba(26,30,26,.30);
  --r-card:12px 12px 12px 4px; --r-chip:6px 6px 6px 2px; --r-panel:14px;
  --font-display:"Bricolage Grotesque","Trebuchet MS",system-ui,sans-serif;
  --font-ui:"Karla","Segoe UI",system-ui,sans-serif;
  --font-data:"Space Mono",ui-monospace,Menlo,monospace;
  --t-quick:170ms; --e-out:cubic-bezier(.22,.9,.30,1);
}
@media (prefers-color-scheme: dark){
  :root:not([data-theme="light"]){
    color-scheme:dark;
    --paper:#151814; --surface:#1D211B; --surface-2:#232722; --raise:#2A2F28;
    --ink:#E7EBE3; --ink-soft:#A0A79B; --ink-faint:#8A9284;
    --line:#333A30; --line-soft:#282E26;
    --accent:#7FA6E6; --accent-ink:#0B142A; --accent-soft:#1E2A44;
    --ok:#5EB489; --warn:#D2A249; --danger:#E0705A;
    --shadow-1:0 1px 0 rgba(255,255,255,.05) inset, 0 2px 4px rgba(0,0,0,.45);
    --shadow-2:0 1px 0 rgba(255,255,255,.05) inset, 0 10px 22px -8px rgba(0,0,0,.7);
  }
}
:root[data-theme="dark"]{
  color-scheme:dark;
  --paper:#151814; --surface:#1D211B; --surface-2:#232722; --raise:#2A2F28;
  --ink:#E7EBE3; --ink-soft:#A0A79B; --ink-faint:#8A9284;
  --line:#333A30; --line-soft:#282E26;
  --accent:#7FA6E6; --accent-ink:#0B142A; --accent-soft:#1E2A44;
  --ok:#5EB489; --warn:#D2A249; --danger:#E0705A;
  --shadow-1:0 1px 0 rgba(255,255,255,.05) inset, 0 2px 4px rgba(0,0,0,.45);
  --shadow-2:0 1px 0 rgba(255,255,255,.05) inset, 0 10px 22px -8px rgba(0,0,0,.7);
}
*{box-sizing:border-box}
html,body{margin:0}
body{background:var(--paper);color:var(--ink);font:15px/1.55 var(--font-ui);padding-inline:16px;padding-block:28px 64px;-webkit-font-smoothing:antialiased}
.wrap{max-width:960px;margin:0 auto}
.eyebrow{font:11px var(--font-data);letter-spacing:.14em;text-transform:uppercase;color:var(--ink-faint)}
h1{font:700 clamp(26px,4vw,38px)/1.05 var(--font-display);letter-spacing:-.015em;margin:8px 0 10px;text-wrap:balance}
.intro{color:var(--ink-soft);max-width:62ch;margin:0 0 6px}
.updated{font:10.5px var(--font-data);color:var(--ink-faint);margin:0 0 28px}
.entry{background:var(--surface);border-radius:var(--r-card);box-shadow:0 0 0 1px rgba(0,0,0,.06),0 10px 30px -14px rgba(0,0,0,.45);padding:18px 20px 20px;margin:0 0 18px}
.entry header{display:flex;flex-wrap:wrap;align-items:baseline;gap:6px 12px;margin-bottom:6px}
.when{font:10.5px var(--font-data);color:var(--ink-faint);font-variant-numeric:tabular-nums;order:1;width:100%}
h2{font:700 20px/1.15 var(--font-display);letter-spacing:-.015em;margin:0;order:2}
.chip{order:3;font:9.5px var(--font-data);padding:2px 8px;border-radius:var(--r-chip);background:var(--accent-soft);color:var(--ink);white-space:nowrap}
.chip[data-status="proven"]{background:var(--accent);color:var(--accent-ink)}
.chip[data-status="running"]{background:var(--surface-2);color:var(--ink-soft);box-shadow:0 0 0 1px var(--line) inset}
.lede{margin:0 0 8px;max-width:66ch}
.notes{margin:0 0 12px;padding-left:18px;color:var(--ink-soft);max-width:66ch}
.notes li{margin:2px 0}
.shots{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:12px}
.shot{margin:0;background:var(--surface-2);border-radius:10px;padding:8px;box-shadow:var(--shadow-1)}
.shot img{display:block;width:100%;height:auto;max-width:100%;border-radius:6px;background:var(--raise)}
.shot.pixel img{image-rendering:pixelated}
.shot figcaption{font:10.5px/1.4 var(--font-data);color:var(--ink-faint);padding:7px 2px 0}
.shot img:hover{box-shadow:var(--shadow-2);transition:box-shadow var(--t-quick) var(--e-out)}
@media (prefers-reduced-motion: reduce){*{transition:none !important}}
</style>
</head>
<body>
<div class="wrap">
  <div class="eyebrow">quire · shell-host · sill</div>
  <h1>Desktop shell progress</h1>
  <p class="intro">Screenshots and results as they land, newest first. Each entry is one milestone gate or spike result; the design docs and code are in the repos.</p>
  <p class="updated">Updated {{UPDATED}}</p>
  {{ENTRIES}}
</div>
</body>
</html>
"""


if __name__ == "__main__":
    build()
