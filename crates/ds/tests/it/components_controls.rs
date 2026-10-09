//! The control components (design/04-COMPONENTS.md sections 1-7, 9-15): every component in
//! every state rendered through dioxus-ssr and compared with a golden in
//! `tests/snapshots/controls/<component>/<state>.html`, plus scans that every `ds-` class a
//! golden uses is styled by that component's CSS and that the CSS uses tokens only.
//!
//! `DS_BLESS=1 cargo test -p ds --test it components_controls` rewrites the goldens.

#[path = "controls/cases.rs"]
mod cases;
#[path = "controls/cases_catalogue.rs"]
mod cases_catalogue;
use crate::css_scan;
use crate::support::golden;
use crate::support::scoped;

use cases::{CASES, Case, MOTION_CASES};
use css_scan::{STYLES, classes, styles_class, token_violations};
use dioxus::prelude::*;
use ds::prelude::*;
use ds_style::icon::render::Glyph;
use ds_style::icon::shape::Shape;
use scoped::Scoped;

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

/// Renders a case inside a scope, so its handlers have a runtime to attach to and a component
/// that reads the scope (a text field) draws instead of panicking.
fn host(props: HostProps) -> Element {
    let case = (props.make)();
    rsx! { Scoped { {case} } }
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
fn every_catalogue_control_matches_its_golden() {
    check_all(cases_catalogue::CASES);
}

#[test]
fn motion_driven_controls_match_their_goldens() {
    check_all(MOTION_CASES);
}

#[test]
fn every_class_in_a_golden_is_styled_by_its_component() {
    let goldens = golden::all_in("controls");
    assert!(
        goldens.len() >= CASES.len() + cases_catalogue::CASES.len(),
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
            rsx! { Glyph { icon: Icon::Wifi, size: props.size, style: GlyphStyle::Outline } }
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
        Some(&Shape::Path("M12 20h.01"))
    );
    assert_eq!(
        Icon::Brightness.shapes().first(),
        Some(&Shape::Path("M12 9a3 3 0 0 0 0 6v-6"))
    );
}

#[test]
fn a_shortcut_is_glyphs_without_separators() {
    let shortcut = Shortcut(vec![
        ShortcutKey::Ctrl,
        ShortcutKey::Shift,
        ShortcutKey::Char('t'),
    ]);
    assert_eq!(shortcut.glyphs(), "⌃⇧T");
    assert_eq!(
        Shortcut(vec![
            ShortcutKey::Super,
            ShortcutKey::Alt,
            ShortcutKey::Enter
        ])
        .glyphs(),
        "⌥⌘↵",
        "the Mac's order, whatever order the keys were given in"
    );
    assert_eq!(Shortcut::default().glyphs(), "");
}

/// The slider consumes wheel input itself (11-BEHAVIOUR-scroll.md section 11.3.1 item 2:
/// "sliders, zoomable canvases"): shell-host's scroll engine hands `data-wheel="capture"`
/// elements the raw `BlitzWheelEvent` instead of scrolling the page under them. Every slider
/// case carries the marker, not just the default one, so a state that later drops it (a variant,
/// a disabled slider) is caught here rather than only in a golden diff.
#[test]
fn every_slider_carries_the_wheel_capture_marker() {
    let sliders: Vec<&Case> = CASES
        .iter()
        .chain(MOTION_CASES.iter())
        .filter(|case| case.component == "slider")
        .collect();
    assert!(!sliders.is_empty(), "no slider cases to check");
    for case in sliders {
        let html = render(case.make);
        assert!(
            html.contains("data-wheel=\"capture\""),
            "{}/{} is missing data-wheel=\"capture\": {html}",
            case.component,
            case.state
        );
    }
}
