//! One `.ds[data-material=…]` block per material and scheme, and the `data-blur` switch
//! between `--m-tint` and `--m-tint-solid`.
//!
//! The paint rules come last: a root or [`crate::Surface`] with a material paints the solid tint
//! unless it says `data-blur=on`, which is the safe default of [`crate::BlurState`]. The
//! material's edge and drop are painted as one `box-shadow` naming only the layers that are not
//! `none` (a `var(--m-edge),var(--m-shadow)` list would be invalid whenever either is), so the
//! colours stay in the `--m-*` declarations. That list is `--m-box`, so a card inside a
//! transparent root paints the same edge and drop.
//!
//! Then the root chrome (`crate::RootChrome`, `crate::FrameTint`): a window's root
//! (`data-frame=opaque`) is its own stacking context, so its frame layers and grain paint over
//! its background; a root drawing the tinted
//! frame (`data-frame=tinted`) paints no tint of its own, and its `.ds-frame` group shows the
//! Space gradient at `--m-frame-alpha` over blur and at the solid floor without it; a
//! transparent root (`data-chrome=transparent`) paints nothing, and its `.ds-popover` and
//! `.ds-sheet` cards paint the material's tint, edge and drop instead (sill FINDINGS Q9, Q13).

use super::emit::{attr_selector, declaration, presence_selector, property, rule};
use crate::appearance::Scheme;
use crate::material::layer::{Layer, joined};
use crate::material::level::level_css;
use crate::material::recipe::{DEFAULT_TINT_ALPHA, SOLID_ALPHA, flat_tint, layers, tint};
use crate::material::stack::VIBRANCY;
use crate::material::{Material, recipe};
use crate::tokens::{Hex, VarName, ZLayer};

/// The variable a root writes inline with `appearance.material_tint_alpha` as a fraction
/// (`.8` at the default). Absent, every tint is section 17.2's own.
pub(crate) const TINT_ALPHA: VarName = VarName("--m-tint-alpha");

/// The variables every material block declares, in the order it writes them. The last four
/// are stack v2's layers (design/03-COLOR.md section 17.4); `--m-shadow` is the ambient drop.
pub(crate) const MATERIAL_VARS: [VarName; 12] = [
    VarName("--m-tint"),
    VarName("--m-tint-solid"),
    VarName("--m-edge"),
    VarName("--m-shadow"),
    VarName("--m-radius"),
    VarName("--m-box"),
    VarName("--m-frame-alpha"),
    VarName("--m-highlight"),
    VarName("--m-hairline"),
    VarName("--m-shadow-contact"),
    VarName("--m-shadow-ambient"),
    VarName("--m-inner"),
];

/// The material recipes. The tint alpha over blur is a settings key
/// (`appearance.material_tint_alpha`), so the stylesheet reads it from a variable the root
/// writes inline rather than baking in `.80`.
pub fn materials_css() -> String {
    let mut css = String::new();
    for scheme in Scheme::ALL {
        for material in Material::ALL {
            css.push_str(&rule(
                &selector(material, scheme),
                &declarations(material, scheme),
            ));
        }
    }
    let material = format!(".ds{}", presence_selector("data-material"));
    let solid = [property("background", "var(--m-tint-solid)")];
    css.push_str(&rule(&material, &solid));
    css.push_str(&rule(
        &format!("{material}{}", attr_selector("data-blur", "off")),
        &solid,
    ));
    css.push_str(&rule(
        &format!("{material}{}", attr_selector("data-blur", "on")),
        &[property("background", "var(--m-tint)")],
    ));
    css.push_str(&chrome_css(&material));
    css.push_str(&osd_card_css());
    css.push_str(&level_css());
    css
}

/// The OSD card (`crate::Osd`, sill FINDINGS Q76): a card inside a transparent Osd root that
/// paints what a tinted root paints on its own box (its `.ds-frame` group at the frame alpha, the
/// solid floor without blur, the inner pair redrawn over it), so one root serves both the fade and
/// the card's tokens.
fn osd_card_css() -> String {
    let card = ".ds-osd > .ds-frame";
    [
        rule(
            &format!("{card}::after"),
            &[
                property("content", "\"\""),
                property("position", "absolute"),
                property("inset", "0"),
                property("border-radius", "inherit"),
                property("box-shadow", "var(--m-inner)"),
            ],
        ),
        rule(
            &format!(".ds{} {card}", attr_selector("data-blur", "off")),
            &[property("opacity", &SOLID_ALPHA.css())],
        ),
    ]
    .concat()
}

/// The cards a transparent root's material is painted on: a popover, a sheet, a notification's
/// plate and the layers of its group behind it, an edge panel (notification parts), a
/// desktop widget's card (sill FINDINGS Q182; a widget tile has no material of its own), and
/// the screenshot thumbnail's plate (sill Q181).
const CARDS: [&str; 7] = [
    ".ds-popover",
    ".ds-sheet",
    ".ds-notification-plate",
    ".ds-notification-layer",
    ".ds-panel",
    ".ds-widget[*|data-host=desktop]",
    ".ds-shot-plate",
];

/// The tinted frame and the transparent root, after the paint rules they override.
fn chrome_css(material: &str) -> String {
    let opaque = format!("{material}{}", attr_selector("data-frame", "opaque"));
    let tinted = format!("{material}{}", attr_selector("data-frame", "tinted"));
    let transparent = format!("{material}{}", attr_selector("data-chrome", "transparent"));
    let cards = |root: &str| {
        CARDS
            .iter()
            .map(|card| format!("{root} {card}"))
            .collect::<Vec<_>>()
            .join(",")
    };
    let blurred = format!("{transparent}{}", attr_selector("data-blur", "on"));
    [
        // A window's root keeps its gradient as its own background, under both layers, so the
        // window stays opaque through a cross-fade; as its own stacking context the layers
        // (z -2) and the grain (z -1) paint over that background instead of beneath it, where
        // the switch was an instant swap and the grain never showed (FINDINGS "mailo gaps").
        rule(
            &opaque,
            &[
                property("position", "relative"),
                property("z-index", &ZLayer::Raise.var().reference()),
            ],
        ),
        rule(
            &tinted,
            &[
                property("background", "transparent"),
                property("position", "relative"),
                // Its own stacking context: the frame's negative z-index paints above whatever
                // is behind the root (a gallery card, a page), not beneath it.
                property("z-index", &ZLayer::Raise.var().reference()),
            ],
        ),
        // The inner edge and highlight the root's box paints are under the frame now; the frame
        // draws them again over the gradient.
        rule(
            &format!("{tinted} > .ds-frame::after"),
            &[
                property("content", "\"\""),
                property("position", "absolute"),
                property("inset", "0"),
                property("border-radius", "inherit"),
                property("box-shadow", "var(--m-inner)"),
            ],
        ),
        rule(
            &format!("{tinted}{} > .ds-frame", attr_selector("data-blur", "off")),
            &[property("opacity", &SOLID_ALPHA.css())],
        ),
        rule(
            &transparent,
            &[
                property("background", "transparent"),
                property("box-shadow", "none"),
            ],
        ),
        rule(
            &cards(&transparent),
            &[
                property("background", "var(--m-tint-solid)"),
                property("border-color", "transparent"),
                property("box-shadow", "var(--m-box)"),
            ],
        ),
        rule(&cards(&blurred), &[property("background", "var(--m-tint)")]),
    ]
    .concat()
}

fn selector(material: Material, scheme: Scheme) -> String {
    let theme = match scheme {
        Scheme::Light => String::new(),
        Scheme::Dark => attr_selector("data-theme", scheme.slug()),
    };
    format!(
        ".ds{theme}{}",
        attr_selector("data-material", material.slug())
    )
}

fn declarations(material: Material, scheme: Scheme) -> Vec<String> {
    let recipe = recipe(material, scheme, DEFAULT_TINT_ALPHA);
    // `base x key / default`: the key's default reproduces the recipe's own value.
    let scaled = |base: crate::tokens::Alpha| {
        format!(
            "calc({}*var({},{default})/{default})",
            base.css(),
            TINT_ALPHA.as_str(),
            default = DEFAULT_TINT_ALPHA.css(),
        )
    };
    let (tint, solid, frame_alpha) = match (tint(material, scheme), flat_tint(material, scheme)) {
        (Some((boosted, base)), Some((flat, _))) => (
            vibrant(boosted, flat, &scaled(base)),
            vibrant(boosted, flat, &SOLID_ALPHA.css()),
            scaled(base),
        ),
        _ => (
            recipe.tint.clone(),
            recipe.tint_solid.clone(),
            "1".to_owned(),
        ),
    };
    let stack = layers(material, scheme);
    let one = |layer: Option<Layer>| joined(&Vec::from_iter(layer), Layer::tuned_css);
    let [
        tint_var,
        solid_var,
        edge_var,
        shadow_var,
        radius_var,
        box_var,
        frame_alpha_var,
        highlight_var,
        hairline_var,
        contact_var,
        ambient_var,
        inner_var,
    ] = MATERIAL_VARS;
    let values = [
        (hairline_var, one(stack.hairline)),
        (highlight_var, one(stack.highlight)),
        (edge_var, joined(&stack.edge, Layer::tuned_css)),
        (contact_var, one(stack.contact)),
        (ambient_var, one(stack.ambient)),
    ];
    // The layers that paint, outside in, by reference, so the colours stay in the `--m-*`
    // declarations.
    let painted = references(&values);
    // The inner layers alone: a tinted root's frame covers its own box, so it draws them again.
    let inner = references(&values[1..3]);
    let [hairline, highlight, edge, contact, ambient] = values.map(|(_, value)| value);
    vec![
        declaration(tint_var, &tint),
        declaration(solid_var, &solid),
        declaration(edge_var, &edge),
        declaration(shadow_var, &ambient),
        declaration(radius_var, &recipe.radius),
        declaration(box_var, &painted),
        declaration(frame_alpha_var, &frame_alpha),
        declaration(highlight_var, &highlight),
        declaration(hairline_var, &hairline),
        declaration(contact_var, &contact),
        declaration(ambient_var, &ambient),
        declaration(inner_var, &inner),
        property("border-radius", &radius_var.reference()),
        property("box-shadow", &box_var.reference()),
    ]
}

/// The variables among `values` that paint, as one `box-shadow` list of references, or `none`.
fn references(values: &[(VarName, String)]) -> String {
    let painted = values
        .iter()
        .filter(|(_, value)| value.as_str() != "none")
        .map(|(var, _)| var.reference())
        .collect::<Vec<_>>();
    if painted.is_empty() {
        "none".to_owned()
    } else {
        painted.join(",")
    }
}

/// The tint at `alpha`, its vibrancy boost weighted by the `--m-vibrancy` input (1 by default:
/// all boost; 0: section 17.2's flat colour). `color-mix()` with `var()` paints (spike S14).
fn vibrant(boosted: Hex, flat: Hex, alpha: &str) -> String {
    let rgba = |Hex([r, g, b]): Hex| format!("rgba({r},{g},{b},{alpha})");
    if boosted == flat {
        return rgba(flat);
    }
    format!(
        "color-mix(in srgb,{} calc(var({},1)*100%),{})",
        rgba(boosted),
        VIBRANCY.as_str(),
        rgba(flat)
    )
}

#[cfg(test)]
mod tests {
    use super::materials_css;

    #[test]
    fn every_tint_scales_its_own_alpha_by_the_key() {
        let css = materials_css();
        const WANT: &[&str] = &[
            ".ds[*|data-material=bar]{--m-tint:color-mix(in srgb,rgba(252,253,249,calc(.7*var(--m-tint-alpha,.8)/.8)) calc(var(--m-vibrancy,1)*100%),rgba(248,249,246,calc(.7*var(--m-tint-alpha,.8)/.8)));--m-tint-solid:color-mix(in srgb,rgba(252,253,249,.94) calc(var(--m-vibrancy,1)*100%),rgba(248,249,246,.94));",
            ".ds[*|data-theme=dark][*|data-material=widget]{--m-tint:color-mix(in srgb,rgba(20,24,19,calc(.67*var(--m-tint-alpha,.8)/.8))",
            ".ds[*|data-material=popover]{--m-tint:rgba(255,255,255,calc(.78*var(--m-tint-alpha,.8)/.8));",
            ".ds[*|data-material=window]{--m-tint:var(--f-grad);--m-tint-solid:var(--f-grad);--m-edge:none;--m-shadow:none;--m-radius:0;--m-box:none;--m-frame-alpha:1;--m-highlight:none;--m-hairline:none;--m-shadow-contact:none;--m-shadow-ambient:none;--m-inner:none;border-radius:var(--m-radius);box-shadow:var(--m-box);}",
            ".ds[*|data-material][*|data-blur=off]{background:var(--m-tint-solid);}",
            "--m-radius:0;--m-box:var(--m-hairline),var(--m-edge);--m-frame-alpha:calc(.7*var(--m-tint-alpha,.8)/.8);",
            "--m-radius:22px;--m-box:var(--m-hairline),var(--m-highlight),var(--m-edge),var(--m-shadow-contact),var(--m-shadow-ambient);",
            "--m-highlight:inset 0 var(--hair) 0 rgba(255,255,255,var(--m-highlight-light,.3));--m-hairline:0 0 0 var(--hairline) rgba(0,0,0,var(--m-hairline-light,.14));--m-shadow-contact:0 1px 2px rgba(0,0,0,calc(.1*var(--m-shadow-strength,1)));",
            "--m-highlight:inset 0 var(--hair) 0 rgba(255,255,255,var(--m-highlight-dark,.12));--m-hairline:0 0 0 var(--hairline) rgba(0,0,0,var(--m-hairline-dark,.6));",
            ".ds[*|data-material][*|data-blur=on]{background:var(--m-tint);}",
            ".ds[*|data-material][*|data-frame=tinted]{background:transparent;position:relative;z-index:var(--z-raise);}",
            ".ds[*|data-material][*|data-frame=tinted][*|data-blur=off] > .ds-frame{opacity:.94;}",
            ".ds[*|data-material][*|data-chrome=transparent]{background:transparent;box-shadow:none;}",
            ".ds[*|data-material][*|data-chrome=transparent] .ds-popover,.ds[*|data-material][*|data-chrome=transparent] .ds-sheet,.ds[*|data-material][*|data-chrome=transparent] .ds-notification-plate,.ds[*|data-material][*|data-chrome=transparent] .ds-notification-layer,.ds[*|data-material][*|data-chrome=transparent] .ds-panel,.ds[*|data-material][*|data-chrome=transparent] .ds-widget[*|data-host=desktop],.ds[*|data-material][*|data-chrome=transparent] .ds-shot-plate{background:var(--m-tint-solid);border-color:transparent;box-shadow:var(--m-box);}",
        ];
        for want in WANT {
            assert!(css.contains(want), "missing {want}\n{css}");
        }
    }
}
