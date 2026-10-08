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
use crate::icon::family::PlateShare;
use crate::kit::Kits;
use crate::material::{level::LEVEL_VARS, stack::STACK_INPUTS};
use crate::space::{
    frame_vars::FrameVars,
    look::{CardAccent, SpaceLook},
};
use crate::tokens::{
    chrome::ChromeToken,
    colour::ColourToken,
    dock_floor::DockFloorToken,
    easing::EasingToken,
    elevation::Shadow,
    emoji_face::EmojiFace,
    label_hue::HueColour,
    layer::ZLayer,
    name::VarName,
    opacity::OpacityToken,
    orb::ORB_VARS,
    person::PersonSwatch,
    pixel::PixelToken,
    row_scale::{RowSize, SidebarRowSize, SidebarTileSize},
    selection::SelectionToken,
    set::{Only, Place, TokenSet},
    shape::Radius,
    shell_scale::ShellSize,
    shell_type::ShellType,
    size_vars::SizeToken,
    spacing::SpacingToken,
    status::StatusMetrics,
    terminal::TerminalColour,
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
        TokenSet::of::<Shadow>().at(Place::Scheme),
        TokenSet::of::<WidgetPaint>().at(Place::Scheme),
        TokenSet::of::<Radius>().at(Place::Shape),
        TokenSet::of::<SizeToken>().at(Place::Ladder),
        TokenSet::of::<ShellSize>().at(Place::Ladder),
        TokenSet::of::<RowSize>().at(Place::Ladder),
        TokenSet::of::<SidebarRowSize>().at(Place::Ladder),
        TokenSet::of::<SidebarTileSize>().at(Place::Ladder),
        TokenSet::of::<ShellType>().at(Place::Metrics),
        TokenSet::of::<DockFloorToken>().at(Place::Metrics),
        TokenSet::of::<PersonSwatch>().at(Place::Scale),
        TokenSet::of::<SpacingToken>().at(Place::Scale),
        TokenSet::of::<ChromeToken>().at(Place::Scale),
        TokenSet::of::<FontSize>()
            .at(Place::Scale)
            .only(Only::TypefaceFixed),
        TokenSet::of::<ZLayer>().at(Place::Scale),
        TokenSet::of::<OpacityToken>().at(Place::Scale),
        TokenSet::of::<PlateShare>().at(Place::Pixel),
        TokenSet::of::<PixelToken>().at(Place::Pixel),
        TokenSet::of::<EmojiFace>().at(Place::Face),
        TokenSet::of::<Family>().at(Place::Typeface),
        TokenSet::of::<VoiceToken>().at(Place::Typeface),
        TokenSet::of::<FontSize>()
            .at(Place::Typeface)
            .only(Only::TypefaceVarying),
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
        .chain([TINT_ALPHA, StatusMetrics::BOX_VAR, StatusMetrics::GLYPH_VAR])
        .chain(STACK_INPUTS)
        .chain(LEVEL_VARS)
        .chain(ORB_VARS)
        .chain(SQUIRCLE_VARS)
        .chain(SHAPE_VARS)
        .map(VarName::as_str)
        .map(str::to_owned);
    let swatches = Accent::ALL.iter().copied().map(swatch_var);
    // Ask the frame for its names rather than restating them; a Space that lends its hue
    // writes the most.
    let look = SpaceLook {
        card_accent: CardAccent::SpaceHue,
        ..SpaceLook::default()
    };
    let frame = FrameVars::of(&look, Scheme::Light)
        .names()
        .into_iter()
        .map(str::to_owned);
    named.chain(swatches).chain(frame).collect()
}
