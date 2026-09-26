//! The token blocks: `.ds` (light, Standard) and `.ds[data-theme=dark]`, plus one
//! `.ds[data-motion=…]` block per non-standard level.
//!
//! The dark block and each level block write only the tokens whose value differs from `.ds`'s,
//! so every name a dark or level block sets is also declared on `.ds` (tests/tokens.rs).

use super::emit::{attr_selector, declaration, rule};
use crate::appearance::{MotionLevel, Scheme};
use crate::icon::family::{PLATE_GLYPH, PLATE_INSET};
use crate::tokens::dock::DOCK_TOKENS;
use crate::tokens::notifications::NOTIFICATION_TOKENS;
use crate::tokens::osd::OSD_TOKENS;
use crate::tokens::shell::SHELL_TOKENS;
use crate::tokens::widgets::WIDGET_TOKENS;
use crate::tokens::{
    ColourToken, DelayToken, DurationToken, EasingToken, Family, FontSize, HueMember, LabelHue,
    OpacityToken, PersonSwatch, PixelToken, Radius, ScalarToken, Shadow, SpacingToken, VarName,
    WidgetPaint, ZLayer,
};

/// Colours, radii, spacing, shadows, type, z and Standard motion on `.ds`; the dark colours under
/// `.ds[data-theme=dark]`; the level overrides under `.ds[data-motion]`.
pub fn tokens_css() -> String {
    let mut light = scheme_tokens(Scheme::Light);
    light.extend(fixed_tokens());
    light.extend(motion_tokens(MotionLevel::Standard));
    light.push("color-scheme:light;".to_owned());
    let mut dark = changed(&scheme_tokens(Scheme::Light), scheme_tokens(Scheme::Dark));
    dark.push("color-scheme:dark;".to_owned());
    let mut css = rule(".ds", &light);
    css.push_str(&rule(
        &format!(".ds{}", attr_selector("data-theme", Scheme::Dark.slug())),
        &dark,
    ));
    for level in [MotionLevel::Calm, MotionLevel::Extra, MotionLevel::Reduced] {
        let overrides = changed(&motion_tokens(MotionLevel::Standard), motion_tokens(level));
        css.push_str(&rule(
            &format!(".ds{}", attr_selector("data-motion", level.slug())),
            &overrides,
        ));
    }
    css
}

/// The declarations of `next` that are not in `base` verbatim.
fn changed(base: &[String], next: Vec<String>) -> Vec<String> {
    next.into_iter()
        .filter(|declaration| !base.contains(declaration))
        .collect()
}

/// Everything that follows the scheme: the card colours, the label hues, the shadows, the widget
/// paints.
fn scheme_tokens(scheme: Scheme) -> Vec<String> {
    let colours = ColourToken::ALL
        .into_iter()
        .map(|token| declaration(token.var(), &colour_value(token, scheme)));
    let hues = LabelHue::ALL.into_iter().flat_map(|hue| {
        HueMember::ALL
            .into_iter()
            .map(move |member| format!("{}:{};", hue.var(member), hue.value(member, scheme).css()))
    });
    let shadows = Shadow::ALL
        .into_iter()
        .map(|shadow| declaration(shadow.var(), shadow.css(scheme)));
    let widget = WidgetPaint::ALL
        .into_iter()
        .map(|paint| declaration(paint.var(), paint.css(scheme)));
    colours.chain(hues).chain(shadows).chain(widget).collect()
}

/// A colour token as the stylesheet writes it.
///
/// `--accent-ring` is the one exception to writing the table's value: it is `--accent` at .35,
/// and `--accent` changes with `data-accent` and with a Space's hue, so the stylesheet mixes it
/// from whatever `--accent` the root resolves (spike S14: `color-mix()` with `var()` paints).
/// With Postmark it is exactly [`ColourToken::value`]'s `rgba(…,.35)`.
fn colour_value(token: ColourToken, scheme: Scheme) -> String {
    match token {
        ColourToken::AccentRing => "color-mix(in srgb,var(--accent) 35%,transparent)".to_owned(),
        _ => token.value(scheme).css(),
    }
}

/// Everything that is the same in both schemes and at every level: radii, spacing, type, z.
fn fixed_tokens() -> Vec<String> {
    let radii = Radius::ALL
        .into_iter()
        .map(|radius| declaration(radius.var(), radius.css()));
    let spacing = SpacingToken::ALL
        .into_iter()
        .map(|step| declaration(step.var(), &step.css()));
    let families = Family::ALL
        .into_iter()
        .map(|family| declaration(family.var(), family.stack()));
    let sizes = FontSize::ALL
        .into_iter()
        .map(|size| declaration(size.var(), size.css()));
    let layers = ZLayer::ALL
        .into_iter()
        .map(|layer| declaration(layer.var(), &layer.z().to_string()));
    let opacities = OpacityToken::ALL
        .into_iter()
        .map(|opacity| declaration(opacity.var(), &opacity.css()));
    // The tuned tokens: the shell type scale and the dock's geometry, each read from the input
    // a consumer writes (`ShellMetrics`, `DockMetrics`) with its settings key's default behind,
    // and the pixel tokens, from the scale the root writes with the 1x value behind.
    let tuned = SHELL_TOKENS
        .into_iter()
        .chain(DOCK_TOKENS)
        .chain(OSD_TOKENS)
        .chain(NOTIFICATION_TOKENS)
        .chain(WIDGET_TOKENS)
        .chain([PLATE_GLYPH, PLATE_INSET])
        .chain(PixelToken::ALL.map(PixelToken::tuned))
        .map(|token| token.declaration());
    // Identity is data, not theme: the person swatches are the same in both schemes.
    let people = PersonSwatch::ALL
        .into_iter()
        .map(|swatch| format!("{}:{};", swatch.var(), swatch.hex().css()));
    radii
        .chain(people)
        .chain(spacing)
        .chain(families)
        .chain(sizes)
        .chain(layers)
        .chain(opacities)
        .chain(tuned)
        .collect()
}

/// Durations, the CSS delays, easings and scalars at `level`.
fn motion_tokens(level: MotionLevel) -> Vec<String> {
    let durations = DurationToken::ALL
        .into_iter()
        .map(|token| declaration(token.var(), &millis(token.duration(level).as_millis())));
    let delays = DelayToken::ALL.into_iter().filter_map(|token| {
        let var: VarName = token.var()?;
        Some(declaration(var, &millis(token.delay(level).as_millis())))
    });
    let easings = EasingToken::ALL
        .into_iter()
        .map(|token| declaration(token.var(), &token.easing(level).css()));
    let scalars = ScalarToken::ALL
        .into_iter()
        .map(|token| declaration(token.var(), &token.value(level).css()));
    durations
        .chain(delays)
        .chain(easings)
        .chain(scalars)
        .collect()
}

fn millis(ms: u128) -> String {
    format!("{ms}ms")
}
