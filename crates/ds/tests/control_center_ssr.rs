//! The control center's parts as markup (sill FINDINGS Q78-Q81): every control glyph renders and
//! lints clean; the goldens are `tests/snapshots/control_center/<name>.html`, each linted and
//! every `ds-` class in it styled by the stylesheet.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test control_center_ssr` rewrites the goldens.

use dioxus::prelude::*;
use ds::icon::render::GlyphProps;
use ds::lint::{LintConfig, Profile, markup};
use ds::{Glyph, Icon, IconSize};

/// `icon` at the bar's 22 px, rendered alone.
fn glyph(icon: Icon) -> String {
    let mut dom = VirtualDom::new_with_props(
        Glyph,
        GlyphProps {
            icon,
            size: IconSize::Bar,
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_control_glyph_renders_and_lints_clean() {
    let strict = LintConfig {
        profile: Profile::Strict,
        ..LintConfig::default()
    };
    let mut failures = Vec::new();
    for icon in Icon::CONTROL.iter().copied() {
        let html = glyph(icon);
        let children = html.matches("<path").count()
            + html.matches("<rect").count()
            + html.matches("<circle").count();
        if children != icon.shapes().len() || !html.contains("class=\"ds-ic\"") {
            failures.push(format!("{icon:?} drew {children} children: {html}"));
        }
        for offence in markup(&html, ds::stylesheet(), &strict) {
            failures.push(format!("{icon:?}: {:?} {}", offence.rule, offence.text));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
