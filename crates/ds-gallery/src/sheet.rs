//! The contact sheet: one self-contained HTML page, every picture as a `data:` URI, no script.

use crate::data_uri;
use crate::registry;
use crate::snapshot::Shot;
use std::path::Path;

/// The sheet for `shots`, read back from `dir`.
pub fn html(dir: &Path, shots: &[Shot]) -> std::io::Result<String> {
    let mut body = String::new();
    for entry in registry::REGISTRY {
        let figures = shots
            .iter()
            .filter(|shot| shot.page == entry.page)
            .map(|shot| figure(dir, shot))
            .collect::<std::io::Result<String>>()?;
        body.push_str(&format!(
            "<section id=\"{slug}\"><h2>{title}</h2><p>{lede}</p><div class=\"row\">{figures}</div></section>\n",
            slug = entry.page.slug(),
            title = escape(entry.title),
            lede = escape(entry.lede),
        ));
    }
    let nav = registry::REGISTRY
        .iter()
        .map(|entry| {
            format!(
                "<a href=\"#{}\">{}</a>",
                entry.page.slug(),
                escape(entry.title)
            )
        })
        .collect::<Vec<_>>()
        .join(" · ");
    Ok(format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>quire gallery</title>\n<style>{STYLE}</style></head>\n<body><h1>quire gallery</h1><p>Every page in light and dark, Postmark and green, standard motion; rendered headless on the CPU by ds-gallery --snapshot.</p><nav>{nav}</nav>\n{body}</body></html>\n"
    ))
}

/// The sheet's own look: a plain review page, not a quire surface.
const STYLE: &str = "body{margin:24px;font:14px system-ui,sans-serif;background:#f4f4f1;color:#1b1d1a}h2{margin:28px 0 4px}p{margin:0 0 10px;max-width:900px}.row{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:12px}figure{margin:0}img{width:100%;height:auto;border:1px solid #ccc;display:block}figcaption{font:12px ui-monospace,monospace;margin-top:4px}nav a{color:inherit}";

/// One picture with its caption.
fn figure(dir: &Path, shot: &Shot) -> std::io::Result<String> {
    let bytes = std::fs::read(dir.join(shot.file()))?;
    Ok(format!(
        "<figure><a href=\"{file}\"><img alt=\"{file}\" src=\"{uri}\"></a><figcaption>{file}</figcaption></figure>",
        file = shot.file(),
        uri = data_uri::png(&bytes),
    ))
}

/// `text` safe inside HTML.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
