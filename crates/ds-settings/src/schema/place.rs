//! Where a choice's model runs, as a typed tag a picker draws a mark for
//! (design/22-SETTINGS.md section 9.4, "Choice places").

use serde::{Deserialize, Serialize};

/// The name of the engine that runs a model on this computer ("llama.cpp"): the value of
/// [`ChoicePlace::ThisComputer::engine`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EngineName(pub String);

/// The name of one of the person's other computers ("frieren"): the value of
/// [`ChoicePlace::YourComputer::host`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HostName(pub String);

/// The company that runs a model in the cloud ("Anthropic"): the value of
/// [`ChoicePlace::Cloud::company`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CompanyName(pub String);

/// Where the model behind a choice runs: the value of `KeySpec::places`.
///
/// Wire form: an internally tagged table, `{"kind":"this_computer","engine":"llama.cpp"}`,
/// `{"kind":"your_computer","host":"frieren"}`, `{"kind":"cloud","company":"Anthropic"}`,
/// `{"kind":"assistant"}`; a TOML table in a file schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ChoicePlace {
    /// On this computer, optionally through a named engine.
    ThisComputer {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        engine: Option<EngineName>,
    },
    /// On another computer of the person's.
    YourComputer { host: HostName },
    /// On a company's servers.
    Cloud { company: CompanyName },
    /// Wherever the assistant decides; the choice is the assistant's own pick.
    Assistant,
}
