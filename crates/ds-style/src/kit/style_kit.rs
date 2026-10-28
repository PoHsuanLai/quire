//! The style layer's kit: the token families, the reset, the accent, material, shape and ground
//! sections, and the custom properties those sections declare for themselves.

use super::model::{Kit, KitRank, Section, Vocabulary};
use crate::appearance::{accent::Accent, theme::Scheme};
use crate::css::{
    RESET,
    accents_css::{accents_css, swatch_var},
    ground_css::ground_css,
    materials_css::{MATERIAL_VARS, TINT_ALPHA, materials_css},
    shape_css::{SHAPE_VARS, SQUIRCLE_VARS, shape_css},
};
use crate::icon::{family::PlateShare, file_kind::FileKind};
use crate::kit::Kits;
use crate::material::{level::LEVEL_VARS, stack::STACK_INPUTS};
use crate::space::{frame_vars::FrameVars, look::SpaceLook};
use crate::tokens::{
    chrome::{ChromeToken, LightPaint},
    colour::ColourToken,
    dock_floor::DockFloorToken,
    easing::EasingToken,
    elevation::Shadow,
    emoji_face::EmojiFace,
    file_icon::FileIconColour,
    label_hue::HueColour,
    layer::ZLayer,
    name::VarName,
    opacity::OpacityToken,
    orb::ORB_VARS,
    person::PersonSwatch,
    pixel::PixelToken,
    row_scale::{RowSize, SidebarToken},
    selection::SelectionToken,
    set::{Place, TokenSet},
    shape::Radius,
    sheet_scale::SheetSize,
    shell_scale::ShellSize,
    shell_type::ShellType,
    signal_colour::SignalColour,
    size_vars::SizeToken,
    spacing::SpacingToken,
    status,
    system_hue::SystemHue,
    terminal::{TerminalColour, TerminalMetric},
    timing::DurationToken,
    type_scale::{Family, FontSize},
    type_voice::VoiceToken,
    widget_paint::WidgetPaint,
};
use ds_core::word::Word;
use std::borrow::Cow;

/// The style layer's contribution to the stylesheet and the linter.
pub static KIT: Kit = Kit {
    rank: KitRank::Style,
    tokens: &[
        TokenSet::of::<ColourToken>().at(Place::Scheme),
        TokenSet::of::<HueColour>().at(Place::Scheme),
        TokenSet::of::<SelectionToken>().at(Place::Scheme),
        TokenSet::of::<TerminalColour>().at(Place::Scheme),
        TokenSet::of::<SignalColour>().at(Place::Scheme),
        TokenSet::of::<SystemHue>().at(Place::Scheme),
        TokenSet::of::<FileIconColour>().at(Place::Scheme),
        TokenSet::of::<FileKind>().at(Place::Scheme),
        TokenSet::of::<Shadow>().at(Place::Scheme),
        TokenSet::of::<WidgetPaint>().at(Place::Scheme),
        TokenSet::of::<Radius>().at(Place::Shape),
        TokenSet::of::<SizeToken>().at(Place::Ladder),
        TokenSet::of::<ShellSize>().at(Place::Ladder),
        TokenSet::of::<RowSize>().at(Place::Ladder),
        TokenSet::of::<SidebarToken>().at(Place::Ladder),
        TokenSet::of::<SheetSize>().at(Place::Ladder),
        TokenSet::of::<ShellType>().at(Place::Metrics),
        TokenSet::of::<DockFloorToken>().at(Place::Metrics),
        TokenSet::of::<PersonSwatch>().at(Place::Scale),
        TokenSet::of::<SpacingToken>().at(Place::Scale),
        TokenSet::of::<ChromeToken>().at(Place::Scale),
        TokenSet::of::<LightPaint>().at(Place::Scheme),
        TokenSet::of::<FontSize>().at(Place::Scale),
        TokenSet::of::<ZLayer>().at(Place::Scale),
        TokenSet::of::<OpacityToken>().at(Place::Scale),
        TokenSet::of::<PlateShare>().at(Place::Pixel),
        TokenSet::of::<PixelToken>().at(Place::Pixel),
        TokenSet::of::<EmojiFace>().at(Place::Face),
        TokenSet::of::<Family>().at(Place::Type),
        TokenSet::of::<VoiceToken>().at(Place::Type),
        TokenSet::of::<TerminalMetric>().at(Place::Scale),
        TokenSet::of::<DurationToken>().at(Place::Motion),
        TokenSet::of::<EasingToken>().at(Place::Motion),
    ],
    sections: &[
        Section {
            name: "reset",
            css: reset,
        },
        Section {
            name: "tokens",
            css: tokens,
        },
        Section {
            name: "accents",
            css: accents,
        },
        Section {
            name: "materials",
            css: materials,
        },
        Section {
            name: "shapes",
            css: shapes,
        },
        Section {
            name: "ground",
            css: ground,
        },
    ],
    sheets: &[],
    vocabulary: Vocabulary {
        inline_vars,
        ..Vocabulary::NONE
    },
};

fn reset(_: &Kits) -> Cow<'static, str> {
    Cow::Borrowed(RESET)
}

fn tokens(kits: &Kits) -> Cow<'static, str> {
    Cow::Owned(kits.token_blocks())
}

fn accents(_: &Kits) -> Cow<'static, str> {
    Cow::Owned(accents_css())
}

fn materials(_: &Kits) -> Cow<'static, str> {
    Cow::Owned(materials_css())
}

fn shapes(_: &Kits) -> Cow<'static, str> {
    Cow::Owned(shape_css())
}

fn ground(_: &Kits) -> Cow<'static, str> {
    Cow::Owned(ground_css())
}

/// The custom properties a style section declares for itself or a root writes inline, which no
/// token family lists: the materials', the material stack's, the level control's, the orb's,
/// the shapes', the accent swatches, the frame's and the bar status item's.
fn inline_vars() -> Vec<String> {
    let named = MATERIAL_VARS
        .into_iter()
        .chain([TINT_ALPHA, status::BOX_VAR, status::GLYPH_VAR])
        .chain(STACK_INPUTS)
        .chain(LEVEL_VARS)
        .chain(ORB_VARS)
        .chain(SQUIRCLE_VARS)
        .chain(SHAPE_VARS)
        .map(VarName::as_str)
        .map(str::to_owned);
    let swatches = Accent::ALL.iter().copied().map(swatch_var);
    // Ask the frame for its names rather than restating them.
    let frame = FrameVars::of(&SpaceLook::default(), Scheme::Light)
        .names()
        .into_iter()
        .map(str::to_owned);
    named.chain(swatches).chain(frame).collect()
}
