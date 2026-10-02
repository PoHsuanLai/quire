//! Every card's footer: which model served it, what it drew on, what it carried.

use crate::components::content::icon_source::IconSource;
use ds_core::word::Word;
use ds_intents::ContextChip;

/// Where a model ran, `data-place`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum ServedPlaceKind {
    /// On this computer.
    ThisComputer,
    /// On another machine on the local network.
    LocalNetwork,
    /// At a cloud provider.
    Cloud,
}

/// Where a model ran, and for a cloud model, whose it is.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServedPlace {
    /// Where it ran.
    pub kind: ServedPlaceKind,
    /// The provider's name, for a cloud model.
    pub provider: Option<String>,
}

/// Which model served an answer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ServedByView {
    /// The model's name.
    pub model: String,
    /// Where it ran.
    pub place: ServedPlace,
}

/// A source key: opaque, opened by the shell.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceKey(pub String);

/// One thing the answer drew on, as a chip that opens it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceChip {
    /// What it is called.
    pub label: String,
    /// Its icon.
    pub icon: IconSource,
    /// What the shell opens.
    pub key: SourceKey,
}

/// A card's footer: served-by, sources and scope. Every card draws one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CardFooter {
    /// Which model served it.
    pub served_by: ServedByView,
    /// What it drew on.
    pub sources: Vec<SourceChip>,
    /// The context the prompt carried.
    pub scope: Vec<ContextChip>,
}
