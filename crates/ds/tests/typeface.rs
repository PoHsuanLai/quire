//! The typeface (design/02-TYPE.md section 2, design/22-SETTINGS.md `appearance.typeface`): the
//! root and every nested scope stamp `data-typeface`, System by default; the stylesheet's `.ds`
//! block names Inter and its `.ds[data-typeface=editorial]` block puts back exactly the
//! editorial faces and values the stylesheet carried before the typeface existed.

use dioxus::core::VirtualDom;
use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Family, Inject, Material, Surface, Typeface, VoiceToken, stylesheet,
    use_typeface,
};

#[derive(Props, Clone, PartialEq)]
struct Setup {
    typeface: Option<Typeface>,
}

#[allow(non_snake_case)]
fn Root(setup: Setup) -> Element {
    let body = rsx! {
        Surface { material: Material::Sheet,
            Probe {}
        }
    };
    match setup.typeface {
        Some(typeface) => rsx! {
            Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host, typeface: Some(typeface), {body} }
        },
        None => rsx! {
            Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host, {body} }
        },
    }
}

#[component]
fn Probe() -> Element {
    let typeface = use_typeface();
    rsx! { p { "{typeface.slug()}" } }
}

fn render(typeface: Option<Typeface>) -> String {
    let mut dom = VirtualDom::new_with_props(Root, Setup { typeface });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_root_and_its_scopes_stamp_the_typeface() {
    let cases = [
        (None, "system"),
        (Some(Typeface::System), "system"),
        (Some(Typeface::Editorial), "editorial"),
    ];
    for (typeface, word) in cases {
        let markup = render(typeface);
        let stamp = format!("data-typeface=\"{word}\"");
        assert_eq!(markup.matches("<div class=\"ds\"").count(), 2, "{markup}");
        assert_eq!(markup.matches(&stamp).count(), 2, "{typeface:?}: {markup}");
        assert!(markup.contains(&format!("<p>{word}</p>")), "{markup}");
    }
}

/// The body of the token rule whose selector is exactly `selector`: the one that declares the
/// family tokens.
fn block<'a>(css: &'a str, selector: &str) -> &'a str {
    css.lines()
        .filter_map(|line| line.strip_prefix(selector)?.strip_prefix('{'))
        .find(|body| body.contains("--font-ui:"))
        .unwrap_or_else(|| panic!("no {selector} token block"))
}

#[test]
fn the_ds_block_names_inter_and_the_editorial_block_the_prototype_faces() {
    let css = stylesheet();
    let system = block(css, ".ds");
    let editorial = block(css, ".ds[*|data-typeface=editorial]");
    for family in Family::ALL {
        let name = family.var().as_str();
        let system_value = family.stack_in(Typeface::System);
        assert!(
            system.contains(&format!("{name}:{system_value};")),
            "{name}"
        );
        let editorial_value = family.stack_in(Typeface::Editorial);
        let expect = format!("{name}:{editorial_value};");
        // The editorial block writes only what differs.
        assert_eq!(
            editorial.contains(&expect),
            system_value != editorial_value,
            "{name}"
        );
    }
    for token in VoiceToken::ALL {
        let name = token.var().as_str();
        assert!(system.contains(&format!("{name}:{};", token.css(Typeface::System))));
        assert!(editorial.contains(&format!("{name}:{};", token.css(Typeface::Editorial))));
    }
    assert!(system.contains("--font-ui:\"Inter\""));
    assert!(system.contains("--font-display:\"Inter Display\""));
    assert!(system.contains("--font-code:\"Space Mono\""));
    assert!(editorial.contains("--font-ui:\"Karla\""));
    assert!(editorial.contains("--font-display:\"Bricolage Grotesque\""));
    assert!(editorial.contains("--font-data:\"Space Mono\""));
}

#[test]
fn editorial_restores_the_values_the_rules_carried() {
    // The literal each voice token replaced, as the stylesheet carried it before the typeface.
    let before = [
        (VoiceToken::TrackingHeading, "-.015em"),
        (VoiceToken::TrackingLockClock, "-.035em"),
        (VoiceToken::TrackingLockDate, ".01em"),
        (VoiceToken::TrackingCaps, ".14em"),
        (VoiceToken::TrackingCapsNarrow, ".12em"),
        (VoiceToken::WeightCaps, "400"),
        (VoiceToken::FsCaps, "var(--fs-eyebrow)"),
        (VoiceToken::TrackingMono, "-.02em"),
        (VoiceToken::FsMono, ".78em"),
    ];
    assert_eq!(before.len(), VoiceToken::ALL.len());
    for (token, value) in before {
        assert_eq!(token.css(Typeface::Editorial), value, "{token:?}");
    }
}

/// Besides `code` and `kbd`, only copied text asks for the code face: a clipboard row's excerpt
/// (sill Q290) and a preview pane's monospace text (Q292), both the words as they were copied.
const COPIED_TEXT: &[&str] = &[".ds-menu-clip{", ".ds-preview-text[*|data-face=mono]{"];

#[test]
fn only_code_and_kbd_ask_for_the_code_face() {
    let css = stylesheet();
    let code_rules: Vec<&str> = css
        .lines()
        .filter(|line| line.contains("font-family:var(--font-code)"))
        .filter(|line| !COPIED_TEXT.iter().any(|rule| line.starts_with(rule)))
        .collect();
    assert_eq!(code_rules.len(), 1, "{code_rules:#?}");
    for rule in COPIED_TEXT {
        assert!(
            css.lines()
                .any(|line| line.starts_with(rule) && line.contains("var(--font-code)")),
            "{rule}"
        );
    }
    assert!(css.contains(".ds-kbd{ display:inline-block;\n  font-family:var(--font-code);"));
}

#[test]
fn every_data_face_rule_is_tabular() {
    let css = stylesheet();
    for (at, _) in css.match_indices("font-family:var(--font-data)") {
        let start = css[..at].rfind('{').unwrap_or(0);
        let end = at + css[at..].find('}').unwrap_or(css.len() - at);
        let rule = &css[start..end];
        assert!(rule.contains("tabular-nums"), "{rule}");
    }
}

#[allow(non_snake_case)]
fn Nested() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host, typeface: Some(Typeface::Editorial),
            Ds { appearance: Appearance::default(), material: Material::Popover, stylesheet: Inject::Host,
                Probe {}
            }
        }
    }
}

#[test]
fn a_nested_root_takes_the_enclosing_typeface() {
    let mut dom = VirtualDom::new(Nested);
    dom.rebuild_in_place();
    let markup = dioxus_ssr::render(&dom);
    assert_eq!(
        markup.matches("data-typeface=\"editorial\"").count(),
        2,
        "{markup}"
    );
    assert!(markup.contains("<p>editorial</p>"), "{markup}");
}

#[test]
fn the_cap_fitted_sizes_keep_the_measured_caps() {
    // Bricolage's cap height is .66 em, Inter Display's 1490 / 2048: each System size draws the
    // Editorial size's cap to within the .5 px rounding.
    const BRICOLAGE: f64 = 0.66;
    const INTER_DISPLAY: f64 = 1490.0 / 2048.0;
    let px = |css: &str| {
        css.trim_end_matches("px")
            .parse::<f64>()
            .unwrap_or(f64::NAN)
    };
    for size in ds::FontSize::CAP_FITTED {
        let editorial = px(size.css_in(Typeface::Editorial));
        let system = px(size.css_in(Typeface::System));
        let want = editorial * BRICOLAGE / INTER_DISPLAY;
        assert!(
            (system - want).abs() <= 0.25,
            "{size:?}: {system} for {want}"
        );
        assert_eq!(
            system * 2.0,
            (system * 2.0).round(),
            "{size:?} on the .5 px grid"
        );
    }
    for size in ds::FontSize::ALL {
        if !ds::FontSize::CAP_FITTED.contains(&size) {
            assert_eq!(
                size.css_in(Typeface::System),
                size.css_in(Typeface::Editorial)
            );
        }
    }
    let css = stylesheet();
    let editorial = block(css, ".ds[*|data-typeface=editorial]");
    assert!(editorial.contains("--fs-widget-hero:47px;"));
    assert!(editorial.contains("--fs-lock-clock:140px;"));
    assert!(block(css, ".ds").contains("--fs-widget-hero:42.5px;"));
}
