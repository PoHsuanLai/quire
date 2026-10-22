//! The controls a key can be drawn with, and the one place that says which a key gets
//! (design/22-SETTINGS.md section 9.1). Every program that draws settings reads this table; none
//! keeps its own.

use ds_core::word::Word;
use serde::{Deserialize, Serialize};

use super::key::{KeyKind, KeySpec};

/// The id of a control, as a schema writes it: `control = "picture"` on a key. Kept as text so a
/// schema that names a control this build does not have still reads; [`Control::named`] says
/// whether it is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ControlName(pub String);

/// One control of the Settings vocabulary. The set is closed and owned by Settings: an owner
/// picks one by id, it never ships its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum Control {
    /// A read-only label: the one value a one-variant enum can hold.
    Readout,
    /// A switch, for two words that are on and off.
    Toggle,
    /// Segments side by side, for a choice between two to five words.
    Segmented,
    /// A pop-up button, for any choice.
    Menu,
    /// A slider, for a bounded number.
    Slider,
    /// A text field.
    Text,
    /// A colour swatch row.
    Colour,
    /// A captured key chord.
    Shortcut,
    /// A rows editor, one cell per column.
    Rows,
    /// A button that runs a live module's action.
    Button,
    /// A grid of pictures with "Add Picture…"; the value is the path of the chosen one. A text
    /// key only.
    Picture,
    /// The four hot corners as one sheet: a screen picture with a menu at each corner. A text
    /// key only; the value is the corner's list of bindings.
    Corners,
}

impl Control {
    /// The control called `name`, `None` for one this build does not have. A reader then draws
    /// the key's default control.
    pub fn named(name: &ControlName) -> Option<Control> {
        Control::parse(&name.0)
    }

    /// This control's id as a schema writes it.
    pub fn name(self) -> ControlName {
        ControlName(self.slug().to_owned())
    }

    /// Whether this control can draw a key of `kind`. A control that cannot is ignored, so a
    /// schema never makes a slider of a word.
    pub fn fits(self, kind: &KeyKind) -> bool {
        match self {
            Control::Picture | Control::Corners => matches!(kind, KeyKind::Text),
            Control::Menu => matches!(
                kind,
                KeyKind::Toggle { .. } | KeyKind::Segmented { .. } | KeyKind::Menu { .. }
            ),
            other => other == kind.default_control(),
        }
    }
}

impl KeyKind {
    /// The control a key of this kind gets when its schema names none. Total: every kind is
    /// written, so a new [`KeyKind`] is a compile error here until it picks one.
    pub fn default_control(&self) -> Control {
        match self {
            KeyKind::Fixed { .. } => Control::Readout,
            KeyKind::Toggle { variants } => match TogglePair::of(variants) {
                Some(_) => Control::Toggle,
                None => Control::Segmented,
            },
            KeyKind::Segmented { .. } => Control::Segmented,
            KeyKind::Menu { .. } => Control::Menu,
            KeyKind::Bounded { .. } => Control::Slider,
            KeyKind::Text => Control::Text,
            KeyKind::Colour => Control::Colour,
            KeyKind::Shortcut => Control::Shortcut,
            KeyKind::List(_) | KeyKind::Rows { .. } => Control::Rows,
            KeyKind::Live { .. } => Control::Button,
        }
    }
}

impl KeySpec {
    /// The control that draws this key: the one its schema names when this build has it and it
    /// fits the key's kind, else the kind's default.
    pub fn control(&self) -> Control {
        self.control
            .as_ref()
            .and_then(Control::named)
            .filter(|control| control.fits(&self.kind))
            .unwrap_or_else(|| self.kind.default_control())
    }
}

/// The two words of an on/off enum, which one is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TogglePair {
    /// The word a switched-on toggle stores.
    pub on: String,
    /// The word a switched-off toggle stores.
    pub off: String,
}

/// Words that mean "not doing it": when a two-word enum has exactly one of them, the other is on.
const OFF_WORDS: &[&str] = &["off", "hide", "never", "none", "nothing"];

/// Pairs that are a setting and its absence without an off word, written `(on, off)`.
/// `reduced`/`standard` is macOS's "Reduce motion": on means reduced; `reduced`/`system` is
/// "Reduce transparency" (`appearance.transparency`).
const ON_OFF_PAIRS: &[(&str, &str)] = &[
    ("natural", "traditional"),
    ("reduced", "standard"),
    ("reduced", "system"),
    ("enabled", "disabled"),
    ("yes", "no"),
];

impl TogglePair {
    /// The pair `variants` make when they are on and off, else `None`: a choice between two
    /// things (`ring`/`bar`, `bottom`/`left`) is not a switch. Two words are on and off when
    /// exactly one is an off word (the other is on), or they are a row of the pair table, in
    /// either order.
    pub fn of(variants: &[String; 2]) -> Option<TogglePair> {
        let [first, second] = variants;
        let is_off = |word: &String| OFF_WORDS.contains(&word.as_str());
        let pair = |on: &String, off: &String| TogglePair {
            on: on.clone(),
            off: off.clone(),
        };
        match (is_off(first), is_off(second)) {
            (false, true) => return Some(pair(first, second)),
            (true, false) => return Some(pair(second, first)),
            (true, true) => return None,
            (false, false) => {}
        }
        ON_OFF_PAIRS
            .iter()
            .find_map(|(on, off)| match (first.as_str(), second.as_str()) {
                (a, b) if (a, b) == (*on, *off) => Some(pair(first, second)),
                (a, b) if (a, b) == (*off, *on) => Some(pair(second, first)),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{Control, ControlName, TogglePair};
    use crate::schema::{
        ActionLabel, ActionWeight, Exposure, Help, KeyKind, KeyPath, KeySpec, Label, LiveAction,
        Page, Section,
    };

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|word| (*word).to_owned()).collect()
    }

    fn toggle(a: &str, b: &str) -> KeyKind {
        KeyKind::Toggle {
            variants: [a.to_owned(), b.to_owned()],
        }
    }

    fn key(kind: KeyKind, control: Option<&str>) -> KeySpec {
        KeySpec {
            path: KeyPath("a.b".to_owned()),
            kind,
            default: toml::Value::String(String::new()),
            label: Label("B".to_owned()),
            help: Help(String::new()),
            page: Page::Appearance,
            section: Section(String::new()),
            exposure: Exposure::Basic,
            labels: Default::default(),
            agent: Default::default(),
            unavailable: Default::default(),
            groups: Default::default(),
            places: Default::default(),
            note: None,
            control: control.map(|name| ControlName(name.to_owned())),
        }
    }

    #[test]
    fn each_kind_gets_its_default_control() {
        let bounded = KeyKind::Bounded {
            min: 0,
            max: 9,
            unit: None,
        };
        let live = KeyKind::Live {
            action: LiveAction {
                label: ActionLabel("Revoke".to_owned()),
                weight: ActionWeight::Plain,
            },
        };
        let cases = [
            (
                KeyKind::Fixed {
                    variant: "a".to_owned(),
                },
                Control::Readout,
            ),
            (toggle("on", "off"), Control::Toggle),
            (toggle("off", "on"), Control::Toggle),
            (toggle("show", "hide"), Control::Toggle),
            (toggle("traditional", "natural"), Control::Toggle),
            (toggle("standard", "reduced"), Control::Toggle),
            (toggle("system", "reduced"), Control::Toggle),
            (toggle("enabled", "disabled"), Control::Toggle),
            (toggle("no", "yes"), Control::Toggle),
            (toggle("icons", "letters"), Control::Segmented),
            (toggle("off", "none"), Control::Segmented),
            (
                KeyKind::Segmented {
                    variants: words(&["a", "b", "c"]),
                },
                Control::Segmented,
            ),
            (
                KeyKind::Menu {
                    variants: words(&["a", "b"]),
                },
                Control::Menu,
            ),
            (bounded, Control::Slider),
            (KeyKind::Text, Control::Text),
            (KeyKind::Colour, Control::Colour),
            (KeyKind::Shortcut, Control::Shortcut),
            (KeyKind::List(Box::new(KeyKind::Text)), Control::Rows),
            (KeyKind::Rows { columns: vec![] }, Control::Rows),
            (live, Control::Button),
        ];
        for (kind, want) in cases {
            assert_eq!(kind.default_control(), want, "{kind:?}");
        }
    }

    #[test]
    fn a_named_control_is_used_when_it_exists_and_fits_else_the_default_stands() {
        let menu = KeyKind::Segmented {
            variants: words(&["a", "b", "c"]),
        };
        let cases = [
            (
                "picture on text",
                KeyKind::Text,
                Some("picture"),
                Control::Picture,
            ),
            (
                "picture on a slider",
                KeyKind::Bounded {
                    min: 0,
                    max: 9,
                    unit: None,
                },
                Some("picture"),
                Control::Slider,
            ),
            (
                "corners on text",
                KeyKind::Text,
                Some("corners"),
                Control::Corners,
            ),
            (
                "a menu for segments",
                menu.clone(),
                Some("menu"),
                Control::Menu,
            ),
            (
                "a slider on text",
                KeyKind::Text,
                Some("slider"),
                Control::Text,
            ),
            (
                "a control nobody has",
                KeyKind::Text,
                Some("hologram"),
                Control::Text,
            ),
            ("none named", menu, None, Control::Segmented),
        ];
        for (name, kind, control, want) in cases {
            assert_eq!(key(kind, control).control(), want, "{name}");
        }
    }

    #[test]
    fn a_control_name_round_trips_through_its_id() {
        for control in <Control as ds_core::word::Word>::ALL {
            assert_eq!(Control::named(&control.name()), Some(*control));
        }
    }

    #[test]
    fn two_words_are_a_switch_only_when_one_is_off_or_the_pair_is_listed() {
        let pair = |a: &str, b: &str| TogglePair::of(&[a.to_owned(), b.to_owned()]);
        let on_off = |on: &str, off: &str| {
            Some(TogglePair {
                on: on.to_owned(),
                off: off.to_owned(),
            })
        };
        assert_eq!(pair("on", "off"), on_off("on", "off"));
        assert_eq!(pair("hide", "show"), on_off("show", "hide"));
        assert_eq!(pair("disabled", "enabled"), on_off("enabled", "disabled"));
        assert_eq!(pair("ring", "bar"), None);
        assert_eq!(pair("never", "none"), None);
    }
}
