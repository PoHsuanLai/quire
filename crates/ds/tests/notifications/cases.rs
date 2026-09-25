//! The notification parts' specimens (sill Q120-Q125), each in a Toast root of its scheme.

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, Expanded, GroupHeader, Icon, IconSource, Inject, Material, Rich, RichRun,
    RichText, Run, RunTone, Theme,
};

/// Every specimen: its golden name and how it is made.
pub const SPECIMENS: &[(&str, fn() -> Element)] = &[
    ("rich-runs", rich_runs),
    ("group-header-closed", group_header_closed),
    ("group-header-open-dark", group_header_open_dark),
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
    toast(Theme::Light, rsx! { p { RichText { body, on_link: |_| {} } } })
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
