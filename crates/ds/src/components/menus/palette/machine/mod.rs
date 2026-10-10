//! The palette's pure machine now lives in `ds_core::palette`, with no renderer; this keeps the
//! old paths. [`CommandPalette`](super::command_palette::CommandPalette) draws; the machine decides.

pub mod model {
    //! The machine's states, inputs, outputs and params, from `ds_core::palette::model`.
    pub use ds_core::palette::model::{
        PaletteIn, PaletteIndex, PaletteMove, PaletteOut, PaletteParams, PaletteState,
    };
}
