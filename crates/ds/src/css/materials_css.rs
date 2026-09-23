//! One `.ds[data-material=…]` block per material and scheme, and the `data-blur` switch
//! between `--m-tint` and `--m-tint-solid`.
//!
//! The paint rules come last: a root or [`crate::Surface`] with a material paints the solid tint
//! unless it says `data-blur=on`, which is the safe default of [`crate::BlurState`]. The
//! material's edge and drop are painted as one `box-shadow` built here from the recipe, since a
//! `var(--m-edge),var(--m-shadow)` list would be invalid whenever either is `none`.

use super::emit::{attr_selector, declaration, presence_selector, property, rule};
use crate::appearance::Scheme;
use crate::material::recipe::{DEFAULT_TINT_ALPHA, tint};
use crate::material::{Material, recipe};
use crate::tokens::{Hex, VarName};

/// The variable a root writes inline with `appearance.material_tint_alpha` as a fraction
/// (`.8` at the default). Absent, every tint is section 17.2's own.
pub(crate) const TINT_ALPHA: VarName = VarName("--m-tint-alpha");

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
    let painted = [recipe.edge.as_str(), recipe.shadow.as_str()]
        .into_iter()
        .filter(|layer| *layer != "none")
        .collect::<Vec<_>>();
    let painted = if painted.is_empty() {
        "none".to_owned()
    } else {
        painted.join(",")
    };
    vec![
        declaration(VarName("--m-tint"), &tint),
        declaration(VarName("--m-tint-solid"), &recipe.tint_solid),
        declaration(VarName("--m-edge"), &recipe.edge),
        declaration(VarName("--m-shadow"), &recipe.shadow),
        declaration(VarName("--m-radius"), &recipe.radius),
        property("border-radius", "var(--m-radius)"),
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
            ".ds[*|data-theme=dark][*|data-material=widget]{--m-tint:rgba(21,24,20,calc(.45*var(--m-tint-alpha,.8)/.8));",
            ".ds[*|data-material=window]{--m-tint:var(--f-grad);--m-tint-solid:var(--f-grad);--m-edge:none;--m-shadow:none;--m-radius:0;border-radius:var(--m-radius);box-shadow:none;}",
            ".ds[*|data-material][*|data-blur=off]{background:var(--m-tint-solid);}",
            ".ds[*|data-material][*|data-blur=on]{background:var(--m-tint);}",
        ];
        for want in WANT {
            assert!(css.contains(want), "missing {want}\n{css}");
        }
    }
}
