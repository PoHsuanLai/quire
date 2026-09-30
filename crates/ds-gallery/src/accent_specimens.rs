//! What the accent sheet draws (`accent_sheet.rs`): the paint the settled band lends, and the
//! surfaces.

use crate::pages::calendar::month_sample::{AUGUST, First, month as lay_out};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::Answers;
use ds::Word;
use ds::{
    Accent, Appearance, Button, Check, Chip, ChipVariant, CommandPalette, CommandPaletteHost,
    Corner, Ds, Icon, Inject, Material, PaletteRow, Radius, RootChrome, RowLeading, Scheme,
    SegmentedControl, Surface, Theme, Toggle, accent_of,
};
use ds::{Choice, Tracking};
use ds_shell::{
    ModuleGrid, ModuleState, ModuleTile, MonthGrid, WidgetFrame, WidgetMetrics, WidgetSize,
};

/// The colours an accent lends its surfaces, as CSS.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Paint {
    fill: String,
    ink: String,
    text: String,
    text_material: String,
    wash: String,
    ring: String,
}

impl Paint {
    /// `accent` in `scheme`, from the settled band.
    fn of(accent: Accent, scheme: Scheme) -> Paint {
        let roles = accent_of(accent, scheme);
        Paint {
            fill: roles.fill.css(),
            ink: roles.ink.css(),
            text: roles.text.css(),
            text_material: roles.text_material.css(),
            wash: roles.wash_colour().css(),
            ring: roles.ring_colour().css(),
        }
    }

    /// The custom properties that repaint a subtree on the card (each root already writes
    /// Blue's; a hue swatch writes its own).
    fn vars(&self) -> String {
        self.vars_with(&self.text)
    }

    /// The same inside a Popover or Sheet, whose root points `--accent-text` at the material's
    /// text (design/03-COLOR.md section 20.6); written inline, it has to say so itself.
    fn material_vars(&self) -> String {
        self.vars_with(&self.text_material)
    }

    fn vars_with(&self, text: &str) -> String {
        format!(
            "--accent:{0};--seal:{0};--accent-ink:{1};--accent-text:{2};--accent-text-material:{3};--accent-soft:{4};--accent-ring:{5};",
            self.fill, self.ink, text, self.text_material, self.wash, self.ring
        )
    }
}

/// The sheet's own rules: the static menu and rows, and the focus ring drawn as the Mac draws
/// it (3 px, 1 px gap).
const CSS: &str = "\
.g-acc{display:flex;flex-direction:column;gap:12px;padding:14px;width:352px;background:var(--paper)}\
.g-acc-head{display:flex;flex-direction:column;gap:2px}\
.g-acc-wall{display:flex;flex-direction:column;gap:14px;padding:14px;border-radius:14px;background-size:cover;background-position:center}\
.g-acc-panel{display:flex;flex-direction:column;gap:10px;padding:12px;color:var(--ink);font-size:13px}\
.g-acc-line{display:flex;flex-direction:row;align-items:center;gap:10px;flex-wrap:wrap}\
.g-acc-link{color:var(--accent-text)}\
.g-acc-link{font-weight:600}\
.g-acc-focus{box-shadow:0 0 0 1px var(--surface),0 0 0 4px var(--accent-ring);border-radius:6px}\
.g-acc-list{display:flex;flex-direction:column;gap:1px}\
.g-acc-item{display:flex;flex-direction:column;padding:4px 10px;border-radius:5px}\
.g-acc-item[*|data-sel=on]{background:var(--accent-soft)}\
.g-acc-sub{font-size:11px;color:var(--ink-faint)}\
.g-acc-launcher{display:grid;height:236px}.g-acc-fill{display:grid}\
.g-acc-hues{display:flex;flex-direction:row;gap:4px;flex-wrap:wrap}\
.g-acc-hue{display:flex;flex-direction:column;align-items:center;gap:4px;width:48px}\
.g-acc-disc{width:26px;height:26px;border-radius:13px;display:grid;place-items:center;background:var(--accent);color:var(--accent-ink);font-weight:700;font-size:12px}\
.g-acc-tint{padding:1px 8px;border-radius:8px;background:var(--accent-soft);color:var(--accent-text);font-weight:600;font-size:12px}\
";

fn theme_of(scheme: Scheme) -> Theme {
    match scheme {
        Scheme::Light => Theme::Light,
        Scheme::Dark => Theme::Dark,
    }
}

/// The surfaces in `scheme`.
#[component]
pub fn Specimens(scheme: Scheme) -> Element {
    let paint = Paint::of(Accent::Blue, scheme);
    let vars = paint.vars();
    let material_vars = paint.material_vars();
    let wall = format!("background-image:url(\"{}\")", wallpaper::calm_uri(scheme));
    let summary = format!(
        "fill {} · ink {} · text {} / {} on glass · wash {} · ring {}",
        paint.fill, paint.ink, paint.text, paint.text_material, paint.wash, paint.ring
    );
    rsx! {
        style { {CSS} }
        div { class: "g-acc",
            div { class: "g-acc-head",
                span { class: "g-name", "Accent band B (Airy), Blue — {scheme.slug()}" }
                span { class: "g-code", "{summary}" }
            }
            div { class: "g-acc-wall", style: "{wall}",
                Ds {
                    appearance: Appearance { theme: theme_of(scheme), ..Appearance::default() },
                    material: Material::Popover,
                    stylesheet: Inject::Host,
                    chrome: Some(RootChrome::Painted),
                    div { class: "g-acc-panel", style: "{material_vars}", Controls {} }
                }
                Ds {
                    appearance: Appearance { theme: theme_of(scheme), ..Appearance::default() },
                    material: Material::Widget,
                    stylesheet: Inject::Host,
                    chrome: Some(RootChrome::Transparent),
                    div { style: WidgetMetrics::default().style_attr(),
                        WidgetFrame { size: WidgetSize::Small,
                            div { style: "{vars}",
                                MonthGrid { data: lay_out(AUGUST, First::Monday, TODAY, &BUSY), onstep: |_| {} }
                            }
                        }
                    }
                }
                Ds {
                    appearance: Appearance { theme: theme_of(scheme), ..Appearance::default() },
                    material: Material::Sheet,
                    stylesheet: Inject::Host,
                    chrome: Some(RootChrome::Transparent),
                    div { class: "g-acc-launcher", Launcher { vars: material_vars } }
                }
            }
            div { class: "g-acc-hues",
                for accent in Accent::ALL.iter().copied() {
                    Hue { accent, scheme }
                }
            }
        }
    }
}

const TODAY: ds_shell::DayKey = ds_shell::DayKey {
    year: 2026,
    month: 8,
    day: 14,
};
const BUSY: [ds_shell::DayKey; 2] = [
    ds_shell::DayKey {
        year: 2026,
        month: 8,
        day: 5,
    },
    ds_shell::DayKey {
        year: 2026,
        month: 8,
        day: 27,
    },
];

/// The controls, the menu, the rows and the tiles, on the popover panel.
#[component]
fn Controls() -> Element {
    let views: Vec<(u8, String)> = ["Day", "Week", "Month"]
        .into_iter()
        .zip(0..)
        .map(|(name, value)| (value, name.to_string()))
        .collect();
    rsx! {
        div { class: "g-acc-line",
            Button { answers: Answers::Return, label: "Done", onclick: |_| {} }
            Button { label: "Cancel", onclick: |_| {} }
            Toggle { label: "Wi-Fi", value: Check::On, onchange: |_| {} }
        }
        div { class: "g-acc-line",
            SegmentedControl::<u8> { label: "View", choices: Choice::pairs(views), tracking: Tracking::SelectOne(1), onchange: |_| {} }
        }
        div { class: "g-acc-line",
            Chip { variant: ChipVariant::Accent, text: "spec" }
            span { class: "g-acc-link", "A link" }
            span { class: "g-acc-focus",
                Button { label: "Focused", onclick: |_| {} }
            }
        }
        div { class: "g-acc-list",
            div { class: "g-acc-item", "Open" }
            div { class: "g-acc-item", "data-sel": "on", "Open in New Window" }
            div { class: "g-acc-item", "Rename…" }
        }
        div { class: "g-acc-list",
            div { class: "g-acc-item", "data-sel": "on",
                span { "Dana Okafor" }
                span { class: "g-acc-sub", "Spec review moved to Thursday" }
            }
            div { class: "g-acc-item",
                span { "Priya Raman" }
                span { class: "g-acc-sub", "Re: the accent band" }
            }
        }
        ModuleGrid { padding: ds::Px(0.0),
            ModuleTile { glyph: Icon::Wifi, title: "Wi-Fi", status: "Home", state: ModuleState::On, onclick: |_| {} }
            ModuleTile { glyph: Icon::Bluetooth, title: "Bluetooth", status: "Off", state: ModuleState::Off, onclick: |_| {} }
        }
    }
}

fn entry(value: u8, title: &str, icon: Icon) -> PaletteRow<u8> {
    PaletteRow {
        leading: RowLeading::Icon(icon),
        ..PaletteRow::new(value, title)
    }
}

/// The launcher's panel with its second row selected.
#[component]
fn Launcher(vars: String) -> Element {
    let rows = vec![
        entry(1, "Files", Icon::Folder),
        entry(2, "Mail", Icon::Mail),
        entry(3, "Terminal", Icon::Terminal),
    ];
    rsx! {
        Surface { material: Material::Sheet, radius: Some(Corner::Token(Radius::Panel)),
            div { class: "g-acc-fill", style: "{vars}",
            CommandPalette::<u8> {
                label: "Launch",
                placeholder: "Search apps",
                query: "",
                tokens: Vec::new(),
                groups: vec![ds::PaletteGroup::list("Applications", rows)],
                empty: "Nothing matches.",
                oninput: |_| {},
                onpick: |_| {},
                onclose: |_| {},
                host: CommandPaletteHost::Surface,
                selected: Some(1),
            }
            }
        }
    }
}

/// One built-in accent: its fill with ink, and its text on its wash.
#[component]
fn Hue(accent: Accent, scheme: Scheme) -> Element {
    let vars = Paint::of(accent, scheme).vars();
    rsx! {
        div { class: "g-acc-hue", style: "{vars}",
            div { class: "g-acc-disc", "14" }
            span { class: "g-acc-tint", "Aa" }
            span { class: "g-acc-sub", "{accent.label()}" }
        }
    }
}
