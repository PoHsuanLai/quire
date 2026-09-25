//! SettingsRow's specimens: each trailing mark, a row with no glyph or detail, a disabled row,
//! and a list of three networks in both schemes.

use dioxus::prelude::*;
use ds::{
    Appearance, Availability, Ds, Icon, Inject, Material, RowTrailing, Run, RunTone, SettingsRow,
    Switch, Text, Theme,
};

/// Which specimen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowCase {
    None,
    CheckOn,
    CheckOff,
    Toggle,
    Chevron,
    Value,
    Glyph,
    Bare,
    Disabled,
    Networks(Theme),
}

#[derive(Props, Clone, PartialEq)]
pub struct RowProps {
    pub case: RowCase,
}

pub const CASES: [(RowCase, &str); 11] = [
    (RowCase::None, "row-none"),
    (RowCase::CheckOn, "row-check-on"),
    (RowCase::CheckOff, "row-check-off"),
    (RowCase::Toggle, "row-toggle"),
    (RowCase::Chevron, "row-chevron"),
    (RowCase::Value, "row-value"),
    (RowCase::Glyph, "row-glyph"),
    (RowCase::Bare, "row-bare"),
    (RowCase::Disabled, "row-disabled"),
    (RowCase::Networks(Theme::Light), "rows-networks-light"),
    (RowCase::Networks(Theme::Dark), "rows-networks-dark"),
];

/// The trailing mark a single-row case ends in.
fn mark(case: RowCase) -> RowTrailing {
    match case {
        RowCase::CheckOn => RowTrailing::Check(Switch::On),
        RowCase::CheckOff => RowTrailing::Check(Switch::Off),
        RowCase::Toggle => RowTrailing::Toggle {
            value: Switch::On,
            on_toggle: EventHandler::new(|_| {}),
        },
        RowCase::Chevron => RowTrailing::Chevron,
        RowCase::Value => RowTrailing::Text(Text::from("84%")),
        RowCase::Glyph => RowTrailing::Glyph(Icon::Lock),
        RowCase::None | RowCase::Bare | RowCase::Disabled | RowCase::Networks(_) => {
            RowTrailing::None
        }
    }
}

/// The three networks a Wi-Fi detail lists: the one in use checked, a secured one, an open one.
fn networks() -> Element {
    rsx! {
        SettingsRow { glyph: Icon::Wifi, title: "Home", detail: "Connected", trailing: RowTrailing::Check(Switch::On), onclick: |_| {} }
        SettingsRow { glyph: Icon::WifiHigh, title: "Studio 5G", trailing: RowTrailing::Glyph(Icon::Lock), onclick: |_| {} }
        SettingsRow {
            glyph: Icon::WifiLow,
            title: Text::Runs(vec![Run::new("Café ", RunTone::Plain), Run::new("Guest", RunTone::Faint)]),
            trailing: RowTrailing::Check(Switch::Off),
            onclick: |_| {},
        }
    }
}

/// A specimen in a Popover root.
pub fn row(props: RowProps) -> Element {
    let theme = match props.case {
        RowCase::Networks(theme) => theme,
        _ => Theme::Light,
    };
    let body = match props.case {
        RowCase::Networks(_) => networks(),
        RowCase::Bare => rsx! {
            SettingsRow { title: "Show in menu bar", onclick: |_| {} }
        },
        RowCase::Disabled => rsx! {
            SettingsRow {
                glyph: Icon::Bluetooth,
                title: "Keyboard",
                detail: "Not connected",
                trailing: RowTrailing::Chevron,
                availability: Availability::Disabled,
                onclick: |_| {},
            }
        },
        case => rsx! {
            SettingsRow { glyph: Icon::Headphones, title: "Headphones", detail: "Battery 84%", trailing: mark(case), onclick: |_| {} }
        },
    };
    rsx! {
        Ds {
            appearance: Appearance { theme, ..Appearance::default() },
            material: Material::Popover,
            stylesheet: Inject::Host,
            {body}
        }
    }
}
