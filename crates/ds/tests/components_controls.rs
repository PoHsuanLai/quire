//! The control components (design/04-COMPONENTS.md sections 1-7, 9-15): every component in
//! every state rendered through dioxus-ssr and compared with a golden in
//! `tests/snapshots/controls/<component>/<state>.html`, plus scans that every `ds-` class a
//! golden uses is styled by that component's CSS and that the CSS uses tokens only.
//!
//! `DS_BLESS=1 cargo test -p ds --test components_controls` rewrites the goldens.

#[path = "controls/cases.rs"]
mod cases;
#[path = "controls/css_scan.rs"]
mod css_scan;
#[path = "support/golden.rs"]
mod golden;

use cases::{CASES, Case, MOTION_CASES};
use css_scan::{STYLES, classes, styles_class, token_violations};
use dioxus::prelude::*;
use ds::components::vocab::{Key, PulseKey, Shortcut, StaggerIndex};
use ds::{Anim, Glyph, Icon, IconSize};

#[derive(Props, Clone)]
struct HostProps {
    make: fn() -> Element,
}

/// Never equal: the host renders once, and function addresses are not comparable anyway.
impl PartialEq for HostProps {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

/// Renders a case inside a scope, so its handlers have a runtime to attach to.
fn host(props: HostProps) -> Element {
    (props.make)()
}

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

fn check_all(cases: &[Case]) {
    let failures: Vec<String> = cases
        .iter()
        .filter_map(|case| {
            let name = format!("controls/{}/{}.html", case.component, case.state);
            golden::check(&name, &render(case.make)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_control_matches_its_golden() {
    check_all(CASES);
}

#[test]
#[ignore = "needs use_drag and Anim::class from the w1-motion branch"]
fn motion_driven_controls_match_their_goldens() {
    check_all(MOTION_CASES);
}

#[test]
fn every_class_in_a_golden_is_styled_by_its_component() {
    let goldens = golden::all_in("controls");
    assert!(
        goldens.len() >= CASES.len(),
        "only {} goldens",
        goldens.len()
    );
    let mut failures = Vec::new();
    for (name, html) in &goldens {
        let component = name.split('/').nth(1).unwrap_or_default();
        let Some((_, sheets)) = STYLES.iter().find(|(c, _)| *c == component) else {
            failures.push(format!("{name}: no stylesheet listed for {component}"));
            continue;
        };
        for class in classes(html) {
            // `ds-ic` is `Glyph`'s: sized by its own attributes, never by component CSS (S6).
            // `a-*` pulse classes come from the motion stylesheet.
            if class == "ds-ic" || !class.starts_with("ds-") {
                continue;
            }
            if !sheets.iter().any(|css| styles_class(css, class)) {
                failures.push(format!(
                    "{name}: .{class} is in no stylesheet of {component}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn control_stylesheets_use_tokens_only() {
    let mut failures = Vec::new();
    for (component, sheets) in STYLES {
        for problem in token_violations(sheets[0]) {
            failures.push(format!("{component}.css: {problem}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_token_scan_catches_what_it_bans() {
    const BAD: &[&str] = &[
        ".x{ color:#fff; }",
        ".x{ color:rgb(0,0,0); }",
        ".x[data-variant=a]{}",
        ".x{ transition:color 170ms linear; }",
        ".x{ animation:spin 1.1s linear; }",
        ".x{ font-family:Karla; }",
        ".x:focus-visible{}",
    ];
    for css in BAD {
        assert!(!token_violations(css).is_empty(), "missed: {css}");
    }
    assert!(
        token_violations(".x[*|data-variant=a]{ font-size:var(--fs-note); padding:1.5px 7px; }")
            .is_empty()
    );
}

#[test]
fn a_glyph_is_stroked_and_sized_by_its_attributes() {
    const SIZES: &[(IconSize, &str)] = &[
        (IconSize::Micro, "11"),
        (IconSize::Compact, "14"),
        (IconSize::Base, "16"),
        (IconSize::Bar, "22"),
    ];
    for (size, px) in SIZES {
        #[derive(Props, Clone, PartialEq)]
        struct At {
            size: IconSize,
        }
        fn glyph(props: At) -> Element {
            rsx! { Glyph { icon: Icon::Wifi, size: props.size } }
        }
        let mut dom = VirtualDom::new_with_props(glyph, At { size: *size });
        dom.rebuild_in_place();
        let html = dioxus_ssr::render(&dom);
        for want in [
            "class=\"ds-ic\"".to_string(),
            format!("width=\"{px}\""),
            format!("height=\"{px}\""),
            format!("data-size=\"{px}\""),
            "stroke=\"currentColor\"".to_string(),
            "stroke-width=\"2\"".to_string(),
            "stroke-linecap=\"round\"".to_string(),
            "stroke-linejoin=\"round\"".to_string(),
            "fill=\"none\"".to_string(),
        ] {
            assert!(html.contains(&want), "{size:?}: no {want} in {html}");
        }
    }
}

#[test]
fn every_shell_glyph_has_geometry() {
    let empty: Vec<String> = Icon::SHELL
        .iter()
        .filter(|icon| icon.shapes().is_empty())
        .map(|icon| format!("{icon:?}"))
        .collect();
    assert!(empty.is_empty(), "no shapes: {}", empty.join(", "));
    // Lucide 1.47.0 `wifi`, first path; Tabler `brightness-half` without its bounding path.
    assert_eq!(
        Icon::Wifi.shapes().first(),
        Some(&ds::Shape::Path("M12 20h.01"))
    );
    assert_eq!(
        Icon::Brightness.shapes().first(),
        Some(&ds::Shape::Path("M12 9a3 3 0 0 0 0 6v-6"))
    );
}

#[test]
fn a_shortcut_is_glyphs_without_separators() {
    let shortcut = Shortcut(vec![Key::Ctrl, Key::Shift, Key::Char('t')]);
    assert_eq!(shortcut.glyphs(), "⌃⇧T");
    assert_eq!(
        Shortcut(vec![Key::Super, Key::Alt, Key::Enter]).glyphs(),
        "⌘⌥↵"
    );
    assert_eq!(Shortcut::default().glyphs(), "");
}

#[test]
fn a_stagger_index_saturates_at_the_cap() {
    const CASES: &[(usize, u8)] = &[(0, 0), (5, 5), (12, 12), (13, 12), (usize::MAX, 12)];
    for (n, want) in CASES {
        assert_eq!(StaggerIndex::new(*n).get(), *want, "n {n}");
    }
}

#[test]
fn a_pulse_alternates_its_alias() {
    use ds::components::vocab::PulsePhase;
    let rest = PulseKey::rest(Anim::Bump);
    assert_eq!(rest.attrs(), None, "at rest nothing is rendered");
    const STEPS: &[PulsePhase] = &[PulsePhase::A, PulsePhase::B, PulsePhase::A];
    let mut key = rest;
    for want in STEPS {
        let before = key.phase();
        key = key.fired();
        assert_eq!(key.phase(), *want, "fired from {before:?}");
        assert_eq!(key.anim(), Anim::Bump);
    }
}
