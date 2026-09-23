//! One `.ds[data-material=…]` block per material and scheme, and the `data-blur` switch
//! between `--m-tint` and `--m-tint-solid`.
//!
//! The paint rules come last: a root or [`crate::Surface`] with a material paints the solid tint
//! unless it says `data-blur=on`, which is the safe default of [`crate::BlurState`]. The
//! material's edge and drop are painted as one `box-shadow` naming only the layers that are not
//! `none` (a `var(--m-edge),var(--m-shadow)` list would be invalid whenever either is), so the
//! colours stay in the `--m-*` declarations.

use super::emit::{attr_selector, declaration, presence_selector, property, rule};
use crate::appearance::Scheme;
use crate::material::recipe::{DEFAULT_TINT_ALPHA, tint};
use crate::material::{Material, recipe};
use crate::tokens::{Hex, VarName};

/// The variable a root writes inline with `appearance.material_tint_alpha` as a fraction
/// (`.8` at the default). Absent, every tint is section 17.2's own.
pub(crate) const TINT_ALPHA: VarName = VarName("--m-tint-alpha");

/// The five variables every material block declares, in the order it writes them.
pub(crate) const MATERIAL_VARS: [VarName; 5] = [
    VarName("--m-tint"),
    VarName("--m-tint-solid"),
    VarName("--m-edge"),
    VarName("--m-shadow"),
    VarName("--m-radius"),
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
    css
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
    let tint = match tint(material, scheme) {
        // `base x key / default`: the key's default reproduces the recipe's own value.
        Some((Hex([r, g, b]), base)) => format!(
            "rgba({r},{g},{b},calc({}*var({},{default})/{default}))",
            base.css(),
            TINT_ALPHA.as_str(),
            default = DEFAULT_TINT_ALPHA.css(),
        ),
        None => recipe.tint.clone(),
    };
    let [tint_var, solid_var, edge_var, shadow_var, radius_var] = MATERIAL_VARS;
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
        property("border-radius", &radius_var.reference()),
        property("box-shadow", &painted),
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
            ".ds[*|data-material=window]{--m-tint:var(--f-grad);--m-tint-solid:var(--f-grad);--m-edge:none;--m-shadow:none;--m-radius:0;border-radius:var(--m-radius);box-shadow:none;}",
            ".ds[*|data-material][*|data-blur=off]{background:var(--m-tint-solid);}",
            "--m-radius:0;border-radius:var(--m-radius);box-shadow:var(--m-edge);}",
            "--m-radius:22px;border-radius:var(--m-radius);box-shadow:var(--m-edge),var(--m-shadow);}",
            ".ds[*|data-material][*|data-blur=on]{background:var(--m-tint);}",
        ];
        for want in WANT {
            assert!(css.contains(want), "missing {want}\n{css}");
        }
    }
}
