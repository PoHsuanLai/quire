//! Shapes the stylesheet draws itself (the macOS polish pass, 2026-09-24): the squircle corner
//! (`data-corner="squircle"`, [`crate::Corner::Squircle`]), the app-icon plate
//! ([`crate::PlateFamily`] on an `IconView`) and the dock's reflective floor.
//!
//! A squircle is a `mask-image` of six layers (four corner quadrants and the cross between
//! them, `icon/plate.rs`), and a mask clips the element's own `box-shadow` too, so the mask never
//! goes on the element that casts the shadow: a material root or surface paints its tint on a
//! masked `::before` under its content and keeps its hairline and shadows on its own box, drawn
//! at the circle the squircle touches at 45 degrees (`--r-squircle`); a tinted root masks its
//! `.ds-frame`; the palette's card does as a surface does. Colours live in custom properties
//! declared here, as the materials' do.

use super::emit::{attr_selector, presence_selector, property, rule};
use crate::appearance::Scheme;
use crate::icon::family::{NEUTRAL_DARK, PlateFamily};
use crate::icon::plate::{Quadrant, fill_mask, plate_mask, quadrant_mask};
use crate::tokens::{Hex, VarName, ZLayer};

/// What a squircle element writes inline (`Corner::squircle_style`): its extent and the circle
/// its shadows follow.
#[cfg_attr(not(feature = "lint"), allow(dead_code))] // Read by the lint's registry.
pub(crate) const SQUIRCLE_VARS: [VarName; 2] = [VarName("--sq-k"), VarName("--r-squircle")];

/// What the plate and floor rules declare for their own use.
#[cfg_attr(not(feature = "lint"), allow(dead_code))] // Read by the lint's registry.
pub(crate) const SHAPE_VARS: [VarName; 7] = [
    VarName("--r-plate"),
    VarName("--plate-inner"),
    VarName("--plate-drop"),
    VarName("--plate-base"),
    VarName("--plate-deep"),
    VarName("--plate-ink"),
    VarName("--dock-floor-fill"),
];

/// The corner extent a squircle reads when its element wrote none: 2 x the panel radius.
const DEFAULT_EXTENT: &str = "28px";

/// The squircle, the plates and the floor, after the materials they override.
pub fn shape_css() -> String {
    [squircle_css(), plate_css(), floor_css()].concat()
}

/// The six mask layers for a squircle whose extent is `--sq-k`, capped at half the box.
fn squircle_mask() -> Vec<String> {
    let extent = format!("min(var(--sq-k,{DEFAULT_EXTENT}),50%)");
    let corners =
        Quadrant::ALL.map(|quadrant| format!("url(\"{}\")", quadrant_mask(quadrant).as_str()));
    let fill = format!("url(\"{}\")", fill_mask().as_str());
    let images = [corners.join(","), fill.clone(), fill].join(",");
    let positions = Quadrant::ALL
        .map(Quadrant::position)
        .into_iter()
        .chain(["center", "center"])
        .collect::<Vec<_>>()
        .join(",");
    let corner_size = format!("{extent} {extent}");
    let sizes = [
        corner_size.clone(),
        corner_size.clone(),
        corner_size.clone(),
        corner_size,
        format!("100% calc(100% - 2*{extent})"),
        format!("calc(100% - 2*{extent}) 100%"),
    ]
    .join(",");
    vec![
        property("mask-image", &images),
        property("mask-position", &positions),
        property("mask-size", &sizes),
        property("mask-repeat", "no-repeat"),
    ]
}

/// The element that paints under its own content: position and a stacking context, so the
/// masked `::before` at the scene layer stays above whatever is behind the element.
fn painter(selector: &str, radius_fallback: &str) -> String {
    let under = [
        vec![
            property("content", "\"\""),
            property("position", "absolute"),
            property("inset", "0"),
            property("z-index", &ZLayer::Scene.var().reference()),
            property("border-radius", "0"),
            property("pointer-events", "none"),
        ],
        squircle_mask(),
    ]
    .concat();
    [
        rule(
            selector,
            &[
                property("background", "transparent"),
                property("position", "relative"),
                property("z-index", &ZLayer::Raise.var().reference()),
                property(
                    "border-radius",
                    &format!("var(--r-squircle,{radius_fallback})"),
                ),
            ],
        ),
        rule(&format!("{selector}::before"), &under),
        // The inner highlight and edge, which the element's own box would paint under the
        // tint, drawn over it at the shadow circle.
        rule(
            &format!("{selector}::after"),
            &[
                property("content", "\"\""),
                property("position", "absolute"),
                property("inset", "0"),
                property("border-radius", "inherit"),
                property("box-shadow", "var(--m-inner)"),
                property("pointer-events", "none"),
            ],
        ),
    ]
    .concat()
}

fn squircle_css() -> String {
    let squircle = attr_selector("data-corner", "squircle");
    let material = format!(".ds{}{squircle}", presence_selector("data-material"));
    let blurred = format!(
        ".ds{}{}{squircle}",
        presence_selector("data-material"),
        attr_selector("data-blur", "on")
    );
    let tinted = format!(".ds{}{squircle}", attr_selector("data-frame", "tinted"));
    let card = format!(".ds-palette{squircle}");
    let blurred_card = format!(
        ".ds{} .ds-palette{squircle}",
        attr_selector("data-blur", "on")
    );
    [
        painter(&material, "var(--m-radius)"),
        rule(
            &format!("{material}::before"),
            &[property("background", "var(--m-tint-solid)")],
        ),
        rule(
            &format!("{blurred}::before"),
            &[property("background", "var(--m-tint)")],
        ),
        // A tinted root's frame is its paint: the frame takes the mask, the root keeps none.
        rule(
            &format!("{tinted}::before,{tinted}::after"),
            &[property("content", "none")],
        ),
        rule(
            &format!("{tinted} > .ds-frame"),
            &[squircle_mask(), vec![property("overflow", "visible")]].concat(),
        ),
        painter(&card, "var(--r-panel)"),
        rule(
            &format!("{card}::before"),
            &[property("background", "var(--m-tint-solid)")],
        ),
        rule(
            &format!("{blurred_card}::before"),
            &[property("background", "var(--m-tint)")],
        ),
    ]
    .concat()
}

/// Section 2.5 at a 48 px tile: the inner top highlight (white .35), the inner rim (black .08)
/// and a drop from `--shadow-2`'s outer part under a 1 px contact shadow.
fn plate_css() -> String {
    let plate = ".ds-plate";
    let face = ".ds-plate-face";
    let mut css = rule(
        plate,
        &[
            property("--r-plate", "22.37%"),
            property(
                "--plate-inner",
                "inset 0 var(--hair) 0 rgba(255,255,255,.35),inset 0 0 0 var(--hair) rgba(0,0,0,.08)",
            ),
            property(
                "--plate-drop",
                "0 1px 1.5px rgba(0,0,0,.16),0 6px 16px -6px rgba(26,30,26,.3)",
            ),
            property("display", "inline-grid"),
            property("place-items", "center"),
            property("position", "relative"),
            property("flex", "none"),
            property("vertical-align", "middle"),
            property("width", "var(--ic-size)"),
            property("height", "var(--ic-size)"),
            property("border-radius", "var(--r-plate)"),
            property("box-shadow", "var(--plate-drop)"),
            property("color", "var(--plate-ink)"),
        ],
    );
    css.push_str(&rule(
        face,
        &[
            property("position", "absolute"),
            property("inset", "0"),
            property("border-radius", "var(--r-plate)"),
            property(
                "background",
                "linear-gradient(135deg,var(--plate-base),var(--plate-deep))",
            ),
            property("box-shadow", "var(--plate-inner)"),
            property("mask-image", &format!("url(\"{}\")", plate_mask().as_str())),
            property("mask-size", "100% 100%"),
            property("mask-repeat", "no-repeat"),
        ],
    ));
    css.push_str(&rule(
        &format!("{plate} > .ds-ic,{plate} > .ds-ext-icon[*|data-kind=symbolic]"),
        &[
            property("position", "relative"),
            property("width", "var(--plate-glyph)"),
            property("height", "var(--plate-glyph)"),
        ],
    ));
    css.push_str(&rule(
        &format!("{plate} > .ds-ext-icon[*|data-kind=image]"),
        &[
            property("position", "relative"),
            property("width", "var(--plate-inset)"),
            property("height", "var(--plate-inset)"),
        ],
    ));
    for family in PlateFamily::ALL {
        let (base, deep) = family.stops();
        css.push_str(&rule(
            &format!("{plate}{}", attr_selector("data-family", family.slug())),
            &stops(base, deep, family.glyph()),
        ));
    }
    let (base, deep, ink) = NEUTRAL_DARK;
    css.push_str(&rule(
        &format!(
            ".ds{} {plate}{}",
            attr_selector("data-theme", Scheme::Dark.slug()),
            attr_selector("data-family", PlateFamily::Neutral.slug())
        ),
        &stops(base, deep, ink),
    ));
    css
}

fn stops(base: Hex, deep: Hex, ink: Hex) -> Vec<String> {
    vec![
        property("--plate-base", &base.css()),
        property("--plate-deep", &deep.css()),
        property("--plate-ink", &ink.css()),
    ]
}

/// The reflective floor (`dock.floor`, off by default): a soft light rising from the pill's
/// floor under the tiles, shown at `--dock-floor` (0 or 1) and masked as the pill is.
fn floor_css() -> String {
    let floor = ".ds-dock-floor";
    let squircle = attr_selector("data-corner", "squircle");
    [
        rule(
            floor,
            &[
                property(
                    "--dock-floor-fill",
                    "linear-gradient(to top,rgba(255,255,255,.26),rgba(255,255,255,.08) 38%,rgba(255,255,255,0) 40%)",
                ),
                property("position", "absolute"),
                property("inset", "0"),
                property("border-radius", "inherit"),
                property("background", "var(--dock-floor-fill)"),
                property("opacity", "var(--dock-floor)"),
                property("pointer-events", "none"),
            ],
        ),
        rule(&format!(".ds{squircle} > {floor}"), &squircle_mask()),
    ]
    .concat()
}

#[cfg(test)]
mod tests {
    use super::shape_css;

    #[test]
    fn the_squircle_masks_the_paint_and_never_the_shadow() {
        let css = shape_css();
        const WANT: &[&str] = &[
            ".ds[*|data-material][*|data-corner=squircle]{background:transparent;position:relative;z-index:var(--z-raise);border-radius:var(--r-squircle,var(--m-radius));}",
            ".ds[*|data-material][*|data-corner=squircle]::before{content:\"\";position:absolute;inset:0;z-index:var(--z-scene);border-radius:0;pointer-events:none;mask-image:url(\"data:image/svg+xml,",
            "mask-position:left top,right top,left bottom,right bottom,center,center;",
            "mask-size:min(var(--sq-k,28px),50%) min(var(--sq-k,28px),50%),",
            "100% calc(100% - 2*min(var(--sq-k,28px),50%)),calc(100% - 2*min(var(--sq-k,28px),50%)) 100%;mask-repeat:no-repeat;}",
            ".ds[*|data-frame=tinted][*|data-corner=squircle]::before,.ds[*|data-frame=tinted][*|data-corner=squircle]::after{content:none;}",
            ".ds-plate[*|data-family=amber]{--plate-base:#f0a81e;--plate-deep:#8e5a05;--plate-ink:#16171a;}",
            ".ds[*|data-theme=dark] .ds-plate[*|data-family=neutral]{--plate-base:#2a2e28;",
            ".ds-dock-floor{--dock-floor-fill:",
        ];
        for want in WANT {
            assert!(css.contains(want), "missing {want}\n{css}");
        }
    }
}
