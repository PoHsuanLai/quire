//! Spellchecking for `ds::EditSurface` (the `spellcheck` feature; design/04-COMPONENTS.md
//! section 50): `ds::HostSpell` on Blitz. The system's Hunspell dictionaries
//! (`/usr/share/hunspell`; nothing is bundled) are read and checked by `spellbook`, unmodified
//! (MPL-2.0), on a worker thread; learned words go to `~/.local/share/quire/spelling/<lang>.dic`.
//!
//! An app provides it at the top of its root with [`provide`] (the system's dictionaries, in
//! the locale's language); a test gives it dictionaries of its own with [`provide_with`].

mod books;
mod choose;
mod config;
mod paragraphs;
mod service;
mod worker;

pub use choose::{locale_lang, pick};
pub use config::{SYSTEM_DICTIONARIES, SpellConfig};
pub use service::NativeSpell;

use dioxus::prelude::*;
use ds::{HostSpell, Lang};
use std::rc::Rc;

/// Provide the system's spellchecker to the calling component's subtree.
pub fn provide() -> HostSpell {
    use_context_provider(|| HostSpell(Rc::new(NativeSpell::system())))
}

/// Provide a spellchecker over `config`'s dictionaries, in `languages` by default.
pub fn provide_with(config: SpellConfig, languages: Vec<Lang>) -> HostSpell {
    use_context_provider(|| HostSpell(Rc::new(NativeSpell::with_config(config, languages))))
}
