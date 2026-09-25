//! The notification parts' specimens (sill Q120-Q125), each in a Toast root of its scheme.

use dioxus::prelude::*;
use ds::{
    AppMark, Appearance, CardAction, Ds, Expanded, GroupCount, GroupHeader, Icon, IconSource,
    Inject, Layers, Material, NotificationCard, Rich, RichRun, RichText, Run, RunTone, Theme,
};

/// Every specimen: its golden name and how it is made.
pub const SPECIMENS: &[(&str, fn() -> Element)] = &[
    ("rich-runs", rich_runs),
    ("group-header-closed", group_header_closed),
    ("group-header-open-dark", group_header_open_dark),
    ("card-light", card_light),
    ("card-dark", card_dark),
    ("card-group", card_group),
    ("card-popover-root", card_popover_root),
];

/// `body` in a Toast root in `theme`.
fn toast(theme: Theme, body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, material: Material::Toast, stylesheet: Inject::Host,
            {body}
        }
    }
}

/// A body with every tone and a link (Q124).
pub fn rich_runs() -> Element {
    let body = Rich(vec![
        RichRun::Run(Run::new("Build ", RunTone::Plain)),
        RichRun::Run(Run::new("#42", RunTone::Strong)),
        RichRun::Run(Run::new(" passed ", RunTone::Plain)),
        RichRun::Run(Run::new("again", RunTone::Italic)),
        RichRun::Run(Run::new(", ", RunTone::Plain)),
        RichRun::Run(Run::new("no flakes", RunTone::Underline)),
        RichRun::Run(Run::new(". See ", RunTone::Faint)),
        RichRun::link("the run", "https://ci.example/runs/42"),
        RichRun::Run(Run::new(".", RunTone::Plain)),
    ]);
    toast(
        Theme::Light,
        rsx! { p { RichText { body, on_link: |_| {} } } },
    )
}

/// A folded group of four from Mail (Q125).
pub fn group_header_closed() -> Element {
    toast(
        Theme::Light,
        rsx! {
            GroupHeader { icon: IconSource::Glyph(Icon::Mail), name: "Mail", count: 4, on_toggle: |_| {}, on_clear: |_| {} }
        },
    )
}

/// An open group, in the dark scheme (Q125).
pub fn group_header_open_dark() -> Element {
    toast(
        Theme::Dark,
        rsx! {
            GroupHeader { icon: IconSource::Glyph(Icon::Terminal), name: "Terminal", count: 2, expanded: Expanded::Open, on_toggle: |_| {}, on_clear: |_| {} }
        },
    )
}

/// The Mail app's mark.
fn mail() -> AppMark {
    AppMark {
        icon: IconSource::Glyph(Icon::Mail),
        name: "Mail".into(),
    }
}

/// A plain card with a body that runs past two lines (Q120).
fn card(theme: Theme) -> Element {
    toast(
        theme,
        rsx! {
            NotificationCard {
                app: mail(),
                age: "now",
                summary: "Ada Lovelace",
                body: "I have translated the memoir and added notes of my own, which run rather longer than the memoir itself.",
                on_close: |_| {},
                on_open: |_| {},
            }
        },
    )
}

/// Light (Q120).
pub fn card_light() -> Element {
    card(Theme::Light)
}

/// Dark (Q120).
pub fn card_dark() -> Element {
    card(Theme::Dark)
}

/// A group of three with two layers behind it, two actions and a link in the body (Q120, Q124).
pub fn card_group() -> Element {
    let body = Rich(vec![
        RichRun::Run(Run::new("Build ", RunTone::Plain)),
        RichRun::Run(Run::new("#42", RunTone::Strong)),
        RichRun::Run(Run::new(" passed. ", RunTone::Plain)),
        RichRun::link("Open the run", "https://ci.example/runs/42"),
    ]);
    toast(
        Theme::Light,
        rsx! {
            NotificationCard {
                app: AppMark { icon: IconSource::Glyph(Icon::Terminal), name: "CI".into() },
                age: "2m",
                summary: "main is green",
                body,
                count: GroupCount { count: 3, layers: Layers(2) },
                actions: vec![
                    CardAction { label: "Rerun".into(), on_press: EventHandler::new(|_| {}) },
                    CardAction { label: "Mute".into(), on_press: EventHandler::new(|_| {}) },
                ],
                on_close: |_| {},
                on_open: |_| {},
                on_link: |_| {},
                id: "toast-7",
            }
        },
    )
}

/// A Toast card inside a Popover root: its own transparent scope paints the Toast material.
pub fn card_popover_root() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover, stylesheet: Inject::Host,
            NotificationCard { app: mail(), age: "9:41", summary: "Grace Hopper", on_close: |_| {}, on_open: |_| {} }
        }
    }
}
