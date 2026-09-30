//! `spaces.json` as a settings document: `by_id` is read key by key, so a stale value costs only
//! its own key and is reported in `Loaded::invalid`; `by_index` is salvaged entry by entry in
//! `ds_style::space::store`.

use crate::doc::{FileName, Format, SettingsDoc};
use ds_style::space::store::SpaceStore;

impl SettingsDoc for SpaceStore {
    const FILE: FileName = FileName("spaces.json");
    const FORMAT: Format = Format::Json;
}
