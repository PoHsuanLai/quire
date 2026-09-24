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
//! Then the root chrome (`crate::RootChrome`, `crate::FrameTint`): a root drawing the tinted
//! frame (`data-frame=tinted`) paints no tint of its own, and its `.ds-frame` group shows the
//! Space gradient at `--m-frame-alpha` over blur and at the solid floor without it; a
//! transparent root (`data-chrome=transparent`) paints nothing, and its `.ds-popover` and
//! `.ds-sheet` cards paint the material's tint, edge and drop instead (sill FINDINGS Q9, Q13).

use super::emit::{attr_selector, declaration, presence_selector, property, rule};
use crate::appearance::Scheme;
use crate::material::recipe::{DEFAULT_TINT_ALPHA, SOLID_ALPHA, tint};
use crate::material::{Material, recipe};
use crate::tokens::{Hex, VarName, ZLayer};

/// The variable a root writes inline with `appearance.material_tint_alpha` as a fraction
/// (`.8` at the default). Absent, every tint is section 17.2's own.
pub(crate) const TINT_ALPHA: VarName = VarName("--m-tint-alpha");

/// The seven variables every material block declares, in the order it writes them.
pub(crate) const MATERIAL_VARS: [VarName; 7] = [
    VarName("--m-tint"),
    VarName("--m-tint-solid"),
    VarName("--m-edge"),
    VarName("--m-shadow"),
    VarName("--m-radius"),
    VarName("--m-box"),
    VarName("--m-frame-alpha"),
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
    css
}

/// The tinted frame and the transparent root, after the paint rules they override.
fn chrome_css(material: &str) -> String {
    let tinted = format!("{material}{}", attr_selector("data-frame", "tinted"));
    let transparent = format!("{material}{}", attr_selector("data-chrome", "transparent"));
    let cards = |root: &str| format!("{root} .ds-popover,{root} .ds-sheet");
    let blurred = format!("{transparent}{}", attr_selector("data-blur", "on"));
    [
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
        // The edge the root's box paints is under the frame now; the frame draws it again
        // over the gradient.
        rule(
            &format!("{tinted} > .ds-frame::after"),
            &[
                property("content", "\"\""),
                property("position", "absolute"),
                property("inset", "0"),
                property("border-radius", "inherit"),
                property("box-shadow", "var(--m-edge)"),
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
    let (tint, frame_alpha) = match tint(material, scheme) {
        Some((Hex([r, g, b]), base)) => {
            (format!("rgba({r},{g},{b},{})", scaled(base)), scaled(base))
        }
        None => (recipe.tint.clone(), "1".to_owned()),
    };
    let [
        tint_var,
        solid_var,
        edge_var,
        shadow_var,
        radius_var,
        box_var,
        frame_alpha_var,
    ] = MATERIAL_VARS;
    // The layers that paint, by reference, so the colours stay in the `--m-*` declarations.
    let painted = [(edge_var, &recipe.edge), (shadow_var, &recipe.shadow)]
        .into_iter()
        .filter(|(_, value)| value.as_str() != "none")
        .map(|(var, _)| var.reference())
        .collect::<Vec<_>>();
    let painted = if painted.is_empty() {
        "none".to_owned()
    } else {
        painted.join(",")
    };
    vec![
        declaration(tint_var, &tint),
        declaration(solid_var, &recipe.tint_solid),
        declaration(edge_var, &recipe.edge),
        declaration(shadow_var, &recipe.shadow),
        declaration(radius_var, &recipe.radius),
        declaration(box_var, &painted),
        declaration(frame_alpha_var, &frame_alpha),
        property("border-radius", &radius_var.reference()),
        property("box-shadow", &box_var.reference()),
    ]
}

#[cfg(test)]
mod tests {
    use super::materials_css;

    #[test]
    fn every_tint_scales_its_own_alpha_by_the_key() {
        let css = materials_css();
        const WANT: &[&str] = &[
            ".ds[*|data-material=bar]{--m-tint:rgba(248,249,246,calc(.7*var(--m-tint-alpha,.8)/.8));--m-tint-solid:rgba(248,249,246,.94);",
            ".ds[*|data-theme=dark][*|data-material=widget]{--m-tint:rgba(21,24,20,calc(.65*var(--m-tint-alpha,.8)/.8));",
            ".ds[*|data-material=window]{--m-tint:var(--f-grad);--m-tint-solid:var(--f-grad);--m-edge:none;--m-shadow:none;--m-radius:0;--m-box:none;--m-frame-alpha:1;border-radius:var(--m-radius);box-shadow:var(--m-box);}",
            ".ds[*|data-material][*|data-blur=off]{background:var(--m-tint-solid);}",
            "--m-radius:0;--m-box:var(--m-edge);--m-frame-alpha:calc(.7*var(--m-tint-alpha,.8)/.8);",
            "--m-radius:22px;--m-box:var(--m-edge),var(--m-shadow);",
            ".ds[*|data-material][*|data-blur=on]{background:var(--m-tint);}",
            ".ds[*|data-material][*|data-frame=tinted]{background:transparent;position:relative;z-index:var(--z-raise);}",
            ".ds[*|data-material][*|data-frame=tinted][*|data-blur=off] > .ds-frame{opacity:.94;}",
            ".ds[*|data-material][*|data-chrome=transparent]{background:transparent;box-shadow:none;}",
            ".ds[*|data-material][*|data-chrome=transparent] .ds-popover,.ds[*|data-material][*|data-chrome=transparent] .ds-sheet{background:var(--m-tint-solid);border-color:transparent;box-shadow:var(--m-box);}",
        ];
        for want in WANT {
            assert!(css.contains(want), "missing {want}\n{css}");
        }
    }
}
