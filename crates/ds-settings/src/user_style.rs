//! `style.css` as a settings document: the person's own stylesheet, raw text under the program's
//! config directory. A missing file is an empty style, and no text is ever refused at load
//! (ARCHITECTURE.md section 11).

use crate::doc::{FileName, Format, SettingsDoc};
use ds_style::kit::UserStyle;

impl SettingsDoc for UserStyle {
    const FILE: FileName = FileName("style.css");
    const FORMAT: Format = Format::Css;
}
