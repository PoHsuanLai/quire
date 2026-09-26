//! Why sill drew digits and spaces from Noto Color Emoji (sill Q342, FINDINGS "Colour emoji"):
//! the stack `"Inter","Noto Color Emoji",system-ui,sans-serif` is laid out here through the font
//! context shell-host builds for every surface (`shell_host::dom::fonts::SharedFonts::system`:
//! `FontContext::new()` and Blitz's bullet face, nothing else) and through quire's own
//! (`ds_native::font_context`, which registers `ds::FACES`).
//!
//! Under quire's context Inter is a family, maps every character of ordinary text, and the stack
//! lays out exactly as `"Inter"` alone. Under shell-host's there is no family called Inter (it is
//! not installed on the system and `SharedFonts::register` has no caller), so the stack's first
//! family that exists is Noto Color Emoji, which maps U+0020 and 0-9 (the keycap sequences' bases):
//! the space and the digits come from it, wide, and every other character from system-ui.
//! Skipped, with a note, where the COLRv1 emoji font is not installed or Inter is a system font.

use blitz_dom::{DocumentConfig, FontContext};
use dioxus::prelude::*;
use dioxus_native_dom::DioxusDocument;
use parley::fontique::{Blob, Collection, CollectionOptions};
use skrifa::MetadataProvider;
use std::sync::Arc;

const COLRV1: &str = "/usr/share/fonts/google-noto-color-emoji-fonts/Noto-COLRv1.ttf";
const STACK: &str = "\"Inter\",\"Noto Color Emoji\",system-ui,sans-serif";
/// sill's broken strings: a file name and the bar clock.
const TEXTS: &[&str] = &["m9-extra-1.pdf", "00:22", "a b c"];

/// shell-host's context, as `SharedFonts::system()` builds it.
fn shell_host_fonts() -> FontContext {
    let mut fonts = FontContext::new();
    fonts.source_cache.make_shared();
    fonts
        .collection
        .register_fonts(Blob::new(Arc::new(blitz_dom::BULLET_FONT) as _), None);
    fonts
}

/// A context with the system fonts only, for asking what the system has.
fn system_only() -> Collection {
    Collection::new(CollectionOptions {
        shared: false,
        system_fonts: true,
    })
}

#[component]
fn Line(stack: String, text: String) -> Element {
    rsx! {
        span { id: "line", style: "display:inline-block; white-space:pre; font-size:16px; font-family:{stack}", "{text}" }
    }
}

/// The width `text` lays out at in `stack` under `fonts`, through Blitz as a shell-host surface
/// lays it out.
fn width(fonts: FontContext, stack: &str, text: &str) -> f64 {
    let vdom = VirtualDom::new_with_props(
        Line,
        LineProps {
            stack: stack.to_string(),
            text: text.to_string(),
        },
    );
    let mut doc = DioxusDocument::new(
        vdom,
        DocumentConfig {
            font_ctx: Some(fonts),
            ..Default::default()
        },
    );
    doc.initial_build();
    let mut inner = doc.inner.borrow_mut();
    inner.resolve(0.0);
    let node = inner
        .query_selector("#line")
        .ok()
        .flatten()
        .expect("the line is in the document");
    inner
        .get_client_bounding_rect(node)
        .expect("the line is laid out")
        .width
}

/// Whether the COLRv1 emoji face maps `c` in its cmap.
fn emoji_maps(c: char) -> bool {
    let bytes = std::fs::read(COLRV1).expect("the emoji font reads");
    let font = skrifa::FontRef::new(&bytes).expect("the emoji font parses");
    font.charmap().map(c).is_some()
}

fn applies() -> bool {
    if !std::path::Path::new(COLRV1).exists() {
        eprintln!("skipped: {COLRV1} is not installed");
        return false;
    }
    if system_only().family_by_name("Inter").is_some() {
        eprintln!("skipped: Inter is installed as a system font here, so shell-host resolves it");
        return false;
    }
    true
}

#[test]
fn the_emoji_face_maps_the_space_and_the_digits() {
    if !applies() {
        return;
    }
    for c in [' ', '0', '1', '2', '9', '#', '*'] {
        assert!(emoji_maps(c), "{c:?} is in Noto Color Emoji's cmap");
    }
    for c in ['a', 'm', '.', ':', '-'] {
        assert!(!emoji_maps(c), "{c:?} is not");
    }
}

#[test]
fn inter_is_not_a_family_in_shell_hosts_font_context() {
    if !applies() {
        return;
    }
    let mut shell = shell_host_fonts();
    assert!(
        shell.collection.family_by_name("Inter").is_none(),
        "shell-host's context has no Inter"
    );
    let mut quire = ds_native::font_context();
    assert!(
        quire.collection.family_by_name("Inter").is_some(),
        "quire's context has Inter"
    );
}

/// Under quire's context the stack is Inter's to the pixel; under shell-host's it is wider than
/// system-ui alone by the emoji face's advances, which is sill's "m 9 -extra- 1 .pdf".
#[test]
fn the_stack_is_inter_under_quire_and_takes_the_emoji_faces_digits_under_shell_host() {
    if !applies() {
        return;
    }
    for text in TEXTS {
        let quire_stack = width(ds_native::font_context(), STACK, text);
        let quire_inter = width(ds_native::font_context(), "\"Inter\"", text);
        assert!(
            (quire_stack - quire_inter).abs() < 0.01,
            "{text}: quire lays the stack out as Inter ({quire_stack} vs {quire_inter})"
        );
        let shell_stack = width(shell_host_fonts(), STACK, text);
        let shell_system = width(shell_host_fonts(), "system-ui,sans-serif", text);
        let shell_named_inter = width(shell_host_fonts(), "\"Inter\",system-ui,sans-serif", text);
        assert!(
            (shell_named_inter - shell_system).abs() < 0.01,
            "{text}: under shell-host a stack naming Inter is system-ui ({shell_named_inter} vs {shell_system})"
        );
        assert!(
            shell_stack > shell_system + 10.0,
            "{text}: under shell-host the emoji face widens it ({shell_stack} vs {shell_system})"
        );
    }
}

/// Why the emoji face cannot simply go last: `system-ui` already maps emoji (fontconfig lists
/// Symbola under it here), so a face named after it is never reached. Under either context the
/// emoji of `"Inter",system-ui,sans-serif,"Noto Color Emoji"` lay out as system-ui's, not as the
/// colour face's.
#[test]
fn an_emoji_face_after_system_ui_is_never_reached() {
    if !applies() {
        return;
    }
    let emoji = "\u{1F600}\u{1F44D}\u{1F3FD}\u{1F1F9}\u{1F1FC}";
    let last = "\"Inter\",system-ui,sans-serif,\"Noto Color Emoji\"";
    for fonts in [
        shell_host_fonts as fn() -> FontContext,
        ds_native::font_context,
    ] {
        let colour = width(fonts(), "\"Noto Color Emoji\"", emoji);
        let system = width(fonts(), "system-ui,sans-serif", emoji);
        assert!(
            (colour - system).abs() > 1.0,
            "the faces differ here ({colour} vs {system})"
        );
        let got = width(fonts(), last, emoji);
        assert!(
            (got - system).abs() < 0.01,
            "emoji last is system-ui's ({got} vs {system})"
        );
    }
}
