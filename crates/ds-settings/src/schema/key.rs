//! One settings key, as the derive emits it and the Settings app renders it
//! (design/22-SETTINGS.md section 9.1).

use std::collections::BTreeMap;

use ds_core::settings::KeyPath;
use serde::{Deserialize, Serialize};

use super::column::Column;
use super::control::ControlName;
use super::live_action::LiveAction;
use super::page::Page;
use super::place::ChoicePlace;

/// What a picker calls the key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Label(pub String);

/// The one-line explanation under the control.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Help(pub String);

/// A short line a picker shows inline under the key, separate from [`Help`] (which stays behind
/// the info button): the value of [`KeySpec::note`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Note(pub String);

/// The words at a slider's two ends ("Small" and "Large"), as a Mac slider labels them: the
/// value of [`KeySpec::ends`], for a bounded key.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SliderEnds {
    /// At the low end.
    pub low: Label,
    /// At the high end.
    pub high: Label,
}

/// What a picker calls each word of an enum key, by the stored word: `workspace_prev` is
/// "Previous workspace". Optional: a word without an entry is shown as its own words.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WordLabels(pub std::collections::BTreeMap<String, String>);

impl WordLabels {
    /// Whether no word has a label.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The label of `word`, if the schema gave one.
    pub fn of(&self, word: &str) -> Option<&str> {
        self.0.get(word).map(String::as_str)
    }
}

/// One word of an enum key, as stored (`"gpt-5"`): the key of [`KeySpec::unavailable`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChoiceWord(pub String);

/// Why a choice cannot be picked right now, in words a person reads under the greyed choice:
/// "Add an account to use".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnavailableReason(pub String);

/// A `Set` or validation named a choice the key lists as unavailable. Carries the reason so the
/// caller can show it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{} is not available for {}: {}", .choice.0, .key.0, .reason.0)]
pub struct ChoiceUnavailable {
    pub key: KeyPath,
    pub choice: ChoiceWord,
    pub reason: UnavailableReason,
}

/// The display name of a group of choices in a menu ("Anthropic", "On this computer"): the value
/// of [`KeySpec::groups`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChoiceGroup(pub String);

/// A choice group's choices in order, as [`KeySpec::grouped_choices`] yields them. `group` is
/// `None` for the ungrouped choices at the top.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupedChoices<'a> {
    pub group: Option<&'a ChoiceGroup>,
    pub choices: Vec<&'a str>,
}

/// A key's group within its page, e.g. dock's "Magnification" section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Section(pub String);

/// File-only in v1 (section 5: "Advanced" = no Settings UI control) versus rendered on its
/// page. Not a `bool`: `CONVENTIONS.md#4-types`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exposure {
    /// Rendered on its page.
    Basic,
    /// File only in v1; the Settings app renders it under a disclosure (section 9.3).
    Advanced,
}

/// Whether an agent (the companion, through detent's intents) may set a key. Two states, not a
/// `bool`: `CONVENTIONS.md#4-types`. A key is hands-off unless its program marks it, so a
/// schema written before the mark existed keeps every key hands-off.
///
/// Wire form: the `[[key]]` table of the schema file carries `agent = "settable"` for a
/// settable key and nothing for a hands-off one (detent reads that spelling).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AgentSetting {
    /// Only the person sets the key; an agent may read it.
    #[default]
    HandsOff,
    /// An agent may set the key, within its own budget and review
    /// (`design/22-SETTINGS.md` section 9.7 lists what is proposed).
    Settable,
}

impl AgentSetting {
    /// Whether this is [`AgentSetting::HandsOff`]: serde's `skip_serializing_if` hook, so the
    /// written schema names the mark only when a key is settable.
    pub fn is_hands_off(&self) -> bool {
        matches!(self, AgentSetting::HandsOff)
    }
}

/// A key's shape: what values it can hold, and so which control draws it.
///
/// Inferred from the field's own type by the derive (`crates/ds-settings-derive/src/shape.rs`):
/// a one-variant enum is [`KeyKind::Fixed`], a two-variant enum [`KeyKind::Toggle`], three to five [`KeyKind::Segmented`], more
/// [`KeyKind::Menu`]; a newtype with `range` is [`KeyKind::Bounded`]; `String`, `PathBuf`,
/// `Cow<str>` or a field marked `#[settings(text)]` is [`KeyKind::Text`]; `Hex` is [`KeyKind::Colour`]; `Vec<String>` and `Vec<Word enum>` are [`KeyKind::List`]; `Vec<T>` with
/// `T: SettingsRow` is [`KeyKind::Rows`].
/// [`KeyKind::Provided`] is a field of type `Provided` with `#[settings(provided = "<capability>")]`.
/// [`KeyKind::Shortcut`] has no field-type rule yet (no settings key is a key binding today); a
/// live D-Bus module (section 9.4) constructs it directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "v", rename_all = "snake_case")]
pub enum KeyKind {
    /// A one-variant enum: the key exists and has its one value, with room for a second
    /// variant later (`control_center.material`'s `Sheet` for v1).
    Fixed { variant: String },
    /// A two-variant enum: on/off, natural/traditional, ....
    Toggle { variants: [String; 2] },
    /// A three-to-five-variant enum.
    Segmented { variants: Vec<String> },
    /// A more-than-five-variant enum.
    Menu { variants: Vec<String> },
    /// A number with a floor and a ceiling, and the unit it is shown in.
    Bounded {
        min: i64,
        max: i64,
        unit: Option<String>,
    },
    /// Free text.
    Text,
    /// A colour swatch.
    Colour,
    /// A captured key chord.
    Shortcut,
    /// A rows editor over a list of the boxed kind.
    List(Box<KeyKind>),
    /// A rows editor over a list of tables, one cell per column. Derived from a `Vec<T>` field
    /// whose `T` derives `SettingsRow`.
    Rows { columns: Vec<Column> },
    /// A choice among things somebody installed (a look, a set of file icons): the stored value is
    /// `"standard"` or the id of the thing chosen. The schema does not list the choices; it names
    /// the `capability` they provide (`"look"`), and the caller that draws the key fills them in
    /// ([`KeySpec::offering`]), as a menu. Written `kind = "provided"` with `v.capability`. A
    /// reader that predates the kind refuses the schema file it is in, as it does for any kind it
    /// does not know (the file is skipped with a warning); that is why a build that reads a newer
    /// quire schema must be rebuilt with it.
    Provided { capability: String },
    /// An action of a live module (section 9.4): a button, not a value. Setting the key runs the
    /// action in the service. Only a live module's schema carries it; a settings file has no such
    /// key. A live module's value keys use the ordinary kinds above.
    Live { action: LiveAction },
}

/// One field of one settings domain struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeySpec {
    pub path: KeyPath,
    pub kind: KeyKind,
    pub default: toml::Value,
    pub label: Label,
    pub help: Help,
    pub page: Page,
    pub section: Section,
    pub exposure: Exposure,
    /// Human labels for an enum key's words; empty for every other key and for a schema written
    /// before they existed (the field is absent then).
    #[serde(default, skip_serializing_if = "WordLabels::is_empty")]
    pub labels: WordLabels,
    /// Whether an agent may set the key. Absent from a schema written before the mark existed
    /// and from every hands-off key, so both read as [`AgentSetting::HandsOff`].
    #[serde(default, skip_serializing_if = "AgentSetting::is_hands_off")]
    pub agent: AgentSetting,
    /// Choices of this key that cannot be picked right now, each with the reason to show under
    /// it. Wire form: `"unavailable": {"<word>": "<reason>"}` (a TOML table in a file schema),
    /// absent when empty, so a reader or writer that predates the field is unaffected.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unavailable: BTreeMap<ChoiceWord, UnavailableReason>,
    /// Display group of each grouped choice of a menu key: `"groups": {"<word>": "<group>"}`
    /// (a TOML table in a file schema), absent when empty. Group order is the order of each
    /// group's first appearance in the key's choices, so there is no order field.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub groups: BTreeMap<ChoiceWord, ChoiceGroup>,
    /// Where each choice's model runs: `"places": {"<word>": {"kind": "cloud", "company":
    /// "Anthropic"}}` (a TOML table in a file schema), absent when empty. A picker draws a mark
    /// per place; a choice without an entry has none.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub places: BTreeMap<ChoiceWord, ChoicePlace>,
    /// A short line a picker shows inline under the key, apart from `help`, which stays behind
    /// the info button. Absent when `None`, so older schemas parse and write unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<Note>,
    /// The words at a bounded key's two ends, when its slider labels them. Wire form: `"ends":
    /// {"low": "Small", "high": "Large"}`, absent when `None`, so older schemas parse and write
    /// unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ends: Option<SliderEnds>,
    /// The control that draws the key, when its kind's default is not the one wanted: `"picture"`
    /// for a path chosen from a grid of pictures. Absent when `None`, so older schemas parse and
    /// write unchanged; a name this build does not know, or one that does not fit the kind, is
    /// ignored (see [`KeySpec::control`]).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control: Option<ControlName>,
}

impl KeySpec {
    /// The key's choices as sections: the ungrouped ones first (`group: None`, omitted when
    /// empty), then each group in order of first appearance in the choices, each with its
    /// choices in their order. Empty for a key that is not a menu or segmented key.
    pub fn grouped_choices(&self) -> Vec<GroupedChoices<'_>> {
        let words: &[String] = match &self.kind {
            KeyKind::Menu { variants } | KeyKind::Segmented { variants } => variants,
            _ => return Vec::new(),
        };
        let group_of = |word: &String| self.groups.get(&ChoiceWord(word.clone()));
        let mut sections = vec![GroupedChoices {
            group: None,
            choices: Vec::new(),
        }];
        for word in words {
            let group = group_of(word);
            let at = if let Some(at) = sections.iter().position(|section| section.group == group) {
                at
            } else {
                sections.push(GroupedChoices {
                    group,
                    choices: Vec::new(),
                });
                sections.len() - 1
            };
            sections[at].choices.push(word);
        }
        sections.retain(|section| !section.choices.is_empty());
        sections
    }

    /// This key with its choices filled in, for a key of kind [`KeyKind::Provided`]: the kind
    /// becomes a [`KeyKind::Menu`] of `words` (the first is the one a person sees first, the
    /// built-in `standard`) and `labels` names them. The stored value is unchanged. Any other
    /// kind comes back as it is.
    pub fn offering(&self, words: Vec<String>, labels: WordLabels) -> KeySpec {
        match &self.kind {
            KeyKind::Provided { .. } => KeySpec {
                kind: KeyKind::Menu { variants: words },
                labels,
                ..self.clone()
            },
            _ => self.clone(),
        }
    }

    /// Where the model behind `word` runs, if the schema says.
    pub fn place_of(&self, word: &str) -> Option<&ChoicePlace> {
        self.places.get(&ChoiceWord(word.to_owned()))
    }

    /// The reason `value` cannot be picked, if it is a string naming an unavailable choice.
    pub fn unavailable_reason(&self, value: &toml::Value) -> Option<&UnavailableReason> {
        let word = value.as_str()?;
        self.unavailable.get(&ChoiceWord(word.to_owned()))
    }

    /// Refuses `value` when it names an unavailable choice, with the reason.
    pub fn check_available(&self, value: &toml::Value) -> Result<(), ChoiceUnavailable> {
        match (self.unavailable_reason(value), value.as_str()) {
            (Some(reason), Some(word)) => Err(ChoiceUnavailable {
                key: self.path.clone(),
                choice: ChoiceWord(word.to_owned()),
                reason: reason.clone(),
            }),
            _ => Ok(()),
        }
    }
}

/// Every variant count from 1 up, mapped to the [`KeyKind`] it picks (section 9.1: "a two-
/// variant enum -> Toggle; 3..=5 variants -> Segmented; more -> Menu"; one variant is a
/// read-only [`KeyKind::Fixed`]).
///
/// Pure: takes the words, returns the kind. The derive's `kind_of::<T>()` (`crate::schema`)
/// is the only caller that reaches for a `Word`'s variants to get them.
pub fn kind_from_variants(mut words: Vec<String>) -> KeyKind {
    match words.len() {
        0 => panic!("a settings enum needs at least one variant, got none"),
        1 => KeyKind::Fixed {
            variant: words.remove(0),
        },
        2 => {
            let [a, b]: [String; 2] = words
                .try_into()
                .unwrap_or_else(|words: Vec<String>| panic!("checked len() == 2: {words:?}"));
            KeyKind::Toggle { variants: [a, b] }
        }
        3..=5 => KeyKind::Segmented { variants: words },
        _ => KeyKind::Menu { variants: words },
    }
}

#[cfg(test)]
mod tests {
    use super::{KeyKind, kind_from_variants};

    #[test]
    fn two_variants_are_a_toggle() {
        let kind = kind_from_variants(vec!["on".to_owned(), "off".to_owned()]);
        assert_eq!(
            kind,
            KeyKind::Toggle {
                variants: ["on".to_owned(), "off".to_owned()]
            }
        );
    }

    #[test]
    fn three_to_five_variants_are_segmented() {
        for count in 3..=5 {
            let words: Vec<String> = (0..count).map(|i| format!("v{i}")).collect();
            assert!(matches!(
                kind_from_variants(words),
                KeyKind::Segmented { .. }
            ));
        }
    }

    #[test]
    fn more_than_five_variants_are_a_menu() {
        let words: Vec<String> = (0..6).map(|i| format!("v{i}")).collect();
        assert!(matches!(kind_from_variants(words), KeyKind::Menu { .. }));
    }

    #[test]
    fn one_variant_is_fixed() {
        assert_eq!(
            kind_from_variants(vec!["only".to_owned()]),
            KeyKind::Fixed {
                variant: "only".to_owned()
            }
        );
    }

    #[test]
    #[should_panic(expected = "at least one variant")]
    fn no_variant_panics() {
        kind_from_variants(Vec::new());
    }
}

#[cfg(test)]
mod label_tests {
    use super::{KeySpec, WordLabels};

    const OLD: &str = "path = \"a.b\"\ndefault = \"x\"\nlabel = \"B\"\nhelp = \"\"\nsection = \"\"\nexposure = \"basic\"\n[kind]\nkind = \"text\"\n[page]\nkind = \"appearance\"\n";

    #[test]
    fn a_key_written_before_labels_existed_still_reads_and_writes_without_them() {
        let spec: KeySpec = toml::from_str(OLD).expect("old schema text parses");
        assert!(spec.labels.is_empty());
        assert!(!toml::to_string(&spec).unwrap().contains("labels"));
    }

    #[test]
    fn labels_round_trip_by_word() {
        let text = format!("{OLD}[labels]\nback = \"Back\"\n");
        let spec: KeySpec = toml::from_str(&text).expect("parses");
        assert_eq!(spec.labels.of("back"), Some("Back"));
        assert_eq!(spec.labels.of("other"), None);
        let again: KeySpec = toml::from_str(&toml::to_string(&spec).unwrap()).unwrap();
        assert_eq!(again.labels, WordLabels(spec.labels.0.clone()));
    }
}

#[cfg(test)]
mod agent_tests {
    use super::{AgentSetting, KeySpec};

    const OLD: &str = "path = \"a.b\"\ndefault = \"x\"\nlabel = \"B\"\nhelp = \"\"\nsection = \"\"\nexposure = \"basic\"\n[kind]\nkind = \"text\"\n[page]\nkind = \"appearance\"\n";

    #[test]
    fn a_key_written_before_the_mark_existed_is_hands_off_and_stays_unmarked() {
        let spec: KeySpec = toml::from_str(OLD).expect("old schema text parses");
        assert_eq!(spec.agent, AgentSetting::HandsOff);
        assert!(!toml::to_string(&spec).unwrap().contains("agent"));
    }

    #[test]
    fn a_settable_key_is_written_and_read_as_agent_settable() {
        let spec: KeySpec = toml::from_str(&format!("agent = \"settable\"\n{OLD}")).unwrap();
        assert_eq!(spec.agent, AgentSetting::Settable);
        let text = toml::to_string(&spec).unwrap();
        assert!(text.contains("agent = \"settable\""), "{text}");
        assert_eq!(toml::from_str::<KeySpec>(&text).unwrap(), spec);
    }

    #[test]
    fn an_unknown_word_is_an_error_not_a_silent_hands_off() {
        let text = format!("agent = \"always\"\n{OLD}");
        assert!(toml::from_str::<KeySpec>(&text).is_err());
    }
}

#[cfg(test)]
mod provided_tests {
    use super::{KeyKind, KeySpec, WordLabels};
    use crate::schema::Control;

    const PROVIDED: &str = "path = \"appearance.look\"\ndefault = \"standard\"\nlabel = \"Look\"\nhelp = \"\"\nsection = \"\"\nexposure = \"basic\"\n[kind]\nkind = \"provided\"\n[kind.v]\ncapability = \"look\"\n[page]\nkind = \"appearance\"\n";

    #[test]
    fn a_provided_key_is_written_with_its_capability_and_read_back() {
        let spec: KeySpec = toml::from_str(PROVIDED).expect("parses");
        assert_eq!(
            spec.kind,
            KeyKind::Provided {
                capability: "look".to_owned()
            }
        );
        let text = toml::to_string(&spec).unwrap();
        assert!(text.contains("kind = \"provided\""), "{text}");
        assert!(text.contains("capability = \"look\""), "{text}");
        assert_eq!(toml::from_str::<KeySpec>(&text).unwrap(), spec);
    }

    #[test]
    fn a_provided_key_is_drawn_as_a_menu() {
        let spec: KeySpec = toml::from_str(PROVIDED).expect("parses");
        assert_eq!(spec.kind.default_control(), Control::Menu);
        assert_eq!(spec.control(), Control::Menu);
    }

    #[test]
    fn offering_fills_the_choices_and_leaves_other_kinds_alone() {
        let spec: KeySpec = toml::from_str(PROVIDED).expect("parses");
        let labels = WordLabels(
            [("standard", "Standard"), ("org.example.Paper", "Paper")]
                .into_iter()
                .map(|(word, label)| (word.to_owned(), label.to_owned()))
                .collect(),
        );
        let words = vec!["standard".to_owned(), "org.example.Paper".to_owned()];
        let filled = spec.offering(words.clone(), labels.clone());
        assert_eq!(filled.kind, KeyKind::Menu { variants: words });
        assert_eq!(filled.labels, labels);
        assert_eq!(filled.default, spec.default);
        assert_eq!(filled.path, spec.path);
        let text: KeySpec = toml::from_str(&PROVIDED.replace(
            "[kind]\nkind = \"provided\"\n[kind.v]\ncapability = \"look\"\n",
            "[kind]\nkind = \"text\"\n",
        ))
        .expect("parses");
        assert_eq!(text.offering(vec!["x".to_owned()], labels), text);
    }

    #[test]
    fn a_kind_this_build_does_not_know_refuses_the_key() {
        // Unchanged behaviour: an unknown kind makes the schema file unreadable to an old reader,
        // which skips the whole file with a warning (`discover`).
        let text = PROVIDED.replace("provided", "hologram");
        assert!(toml::from_str::<KeySpec>(&text).is_err());
    }
}

#[cfg(test)]
mod control_tests {
    use super::KeySpec;
    use crate::schema::{Control, ControlName};

    const OLD: &str = "path = \"a.b\"\ndefault = \"x\"\nlabel = \"B\"\nhelp = \"\"\nsection = \"\"\nexposure = \"basic\"\n[kind]\nkind = \"text\"\n[page]\nkind = \"appearance\"\n";

    #[test]
    fn a_key_written_before_controls_existed_has_none_and_stays_without() {
        let spec: KeySpec = toml::from_str(OLD).expect("old schema text parses");
        assert_eq!(spec.control, None);
        assert_eq!(spec.control(), Control::Text);
        assert!(!toml::to_string(&spec).unwrap().contains("control"));
    }

    #[test]
    fn a_named_control_is_written_and_read_back() {
        let spec: KeySpec = toml::from_str(&format!("control = \"picture\"\n{OLD}")).unwrap();
        assert_eq!(spec.control, Some(ControlName("picture".to_owned())));
        assert_eq!(spec.control(), Control::Picture);
        let text = toml::to_string(&spec).unwrap();
        assert!(text.contains("control = \"picture\""), "{text}");
        assert_eq!(toml::from_str::<KeySpec>(&text).unwrap(), spec);
    }
}
