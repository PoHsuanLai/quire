//! The Polish page (the macOS polish pass, 2026-09-24): quire's shell chrome beside the macOS
//! numbers it targets (design/13-BEHAVIOUR-menus-windows.md section 13.2, design/10-BEHAVIOUR-dock.md
//! section 10.2, plan Appendix C), each with the target printed under it: the material stack,
//! a text menu and the menu-bar items, the squircle corner, the dock pill and its tiles, the
//! launcher and the window shadow.

use super::app_icons::{APPS, app_icon};
use super::{Section, Specimen};
use crate::axes::{Axes, Showcase};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::tokens::Alpha;
use ds::{
    Anchor, Appearance, Availability, Check, CommandPalette, CommandPaletteHost, Corner, DockFloor,
    DockFloorSetting, DockMetrics, Ds, Emphasis, Here, Icon, IconButton, IconButtonVariant,
    IconSize, IconSource, IconView, Inject, Material, MaterialStack, MenuBarItem, MenuEntrance,
    MenuEntry, MenuKind, PlateFamily, Point, Px, RootChrome, RunningDot, Scheme, Shortcut, Shown,
    SpaceLook, Surface, Switch, Theme, Tile, Tooltip, TooltipKind, Trail, WorkspacePill,
    WorkspacePills, components::vocab::Key, use_env,
};

/// A nested root in `material` with the page's look and blur state, as a shell surface's root: its chrome is
/// the material's own (a Popover root is transparent and its cards paint), unlike `Scope`'s.
#[component]
fn Root(
    material: Material,
    #[props(default)] stack: Option<MaterialStack>,
    #[props(default)] radius: Option<Corner>,
    #[props(default)] chrome: Option<RootChrome>,
    #[props(default)] style: String,
    children: Element,
) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (motion, look, accent, blur) = {
        let axes = axes.read();
        (axes.motion, axes.look.clone(), axes.accent, axes.blur)
    };
    let scheme = use_env().scheme;
    let theme = match scheme {
        Scheme::Light => Theme::Light,
        Scheme::Dark => Theme::Dark,
    };
    let tint = use_context::<Signal<Alpha>>();
    rsx! {
        div { style,
            Ds {
                appearance: Appearance { theme, accent, motion },
                look: SpaceLook { theme, ..look },
                material,
                blur,
                stylesheet: Inject::Host,
                tint_alpha: Some(tint()),
                chrome,
                radius,
                stack,
                {children}
            }
        }
    }
}

/// The stack off: no highlight, no hairline, no shadows, no vibrancy (wave 1's look, nearly).
const FLAT: MaterialStack = MaterialStack {
    highlight_light: Alpha(0),
    highlight_dark: Alpha(0),
    hairline_light: Alpha(0),
    hairline_dark: Alpha(0),
    shadow_strength: Alpha(0),
    vibrancy: Alpha(0),
};

/// The Polish page.
#[component]
pub fn PolishPage() -> Element {
    rsx! {
        MenuBarSection {}
        MaterialSection {}
        OsdSection {}
        CornerSection {}
        DockSection {}
        LauncherSection {}
        WindowSection {}
    }
}

/// A caption line: what quire draws, then the macOS number with its confidence.
fn target(ours: &str, macos: &str) -> String {
    format!("ours: {ours}  |  macOS: {macos}")
}

fn menu_entries() -> Vec<MenuEntry<u8>> {
    let item = |value: u8, title: &str, check: Option<Check>, trail: Trail, availability| {
        MenuEntry::Item {
            value,
            title: title.to_owned(),
            detail: None,
            tile: None,
            trail,
            check,
            availability,
        }
    };
    let keys = |key: char| Trail::Shortcut(Shortcut(vec![Key::Super, Key::Char(key)]));
    vec![
        item(1, "New Window", None, keys('N'), Availability::Enabled),
        item(2, "New Tab", None, keys('T'), Availability::Enabled),
        item(3, "Open Recent", None, Trail::None, Availability::Disabled),
        MenuEntry::Separator,
        item(
            4,
            "Show Sidebar",
            Some(Check::Checked),
            Trail::None,
            Availability::Enabled,
        ),
        item(
            5,
            "Show Path Bar",
            Some(Check::Unchecked),
            Trail::None,
            Availability::Enabled,
        ),
        MenuEntry::Separator,
        item(6, "Close Window", None, keys('W'), Availability::Enabled),
    ]
}

/// Where the posed menu hangs: under the open "File" item of the bar specimen.
const MENU_AT: Point = Point {
    x: Px(118.0),
    y: Px(236.0),
};

#[component]
fn MenuBarSection() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    rsx! {
        Section { title: "Menu bar and a text menu", note: "MenuBarItem and IconButton {{ Status }} on a Bar root over the wallpaper: 13/500 text, the 4 px pill on hover and while open (File is open), the workspaces as one segmented group on the frame (WorkspacePills). Under it, a Slim menu on a Popover root at the shell scale: 22 px rows, the check column, 13 px text, a 6 px inset highlight, hairline separators with 5 px margins, a disabled row at .35.",
            div { class: "g-wall g-polish-wall", style: "background-image:url(\"{wallpaper::uri()}\")",
                Root { material: Material::Bar, style: "width:100%",
                    div { class: "g-polish-bar",
                        WorkspacePills { label: "Workspaces",
                            WorkspacePill { label: "1", current: Here::Current, onclick: |_| {} }
                            WorkspacePill { label: "2", onclick: |_| {} }
                            WorkspacePill { label: "3", onclick: |_| {} }
                        }
                        MenuBarItem { emphasis: Emphasis::Strong, span { "Files" } }
                        MenuBarItem { open: Switch::On, span { "File" } }
                        MenuBarItem { span { "Edit" } }
                        MenuBarItem { span { "View" } }
                        span { class: "g-spacer" }
                        IconButton { variant: IconButtonVariant::Status, icon: Icon::Wifi, label: "Wi-Fi", onclick: |_| {} }
                        IconButton { variant: IconButtonVariant::Status, icon: Icon::Volume2, label: "Volume", expanded: Some(Switch::On), onclick: |_| {} }
                        IconButton { variant: IconButtonVariant::Status, icon: Icon::BatteryFull, label: "Battery", onclick: |_| {} }
                        MenuBarItem { span { class: "ds-tabular", "Thu 24 Sep 09:41" } }
                    }
                }
                Root { material: Material::Popover, style: "height:250px",
                    if showcase == Showcase::Posed {
                        ds::Menu::<u8> {
                            kind: MenuKind::Slim,
                            anchor: Anchor::Point(MENU_AT),
                            entries: menu_entries(),
                            entrance: MenuEntrance::Instant,
                            onpick: move |_| {},
                            onclose: move |_| {},
                        }
                    }
                }
            }
            div { class: "g-grid2",
                Specimen { name: "Menu bar item",
                    code: target("pill 24 high, radius 4, --f-pill-hover on hover, --f-pill while open, text 13/500, status box 22 with a 16 px glyph", "menu bar 24 pt (R1, H); status items ≤ 22 pt, icons 16 pt (R2, H); a rounded highlight behind the open title (L)"),
                }
                Specimen { name: "Text menu",
                    code: target("rows 22, text 13/400, check column 22, highlight radius 6 inset by the 5 px panel padding, separator 1 px with 5 px margins, disabled .35, fade out over --t-quick", "items ~22 pt, 13 pt text (R5, L); disabled 35 % (R2, H); fade on close (R3, L)"),
                }
            }
        }
    }
}

#[component]
fn MaterialSection() -> Element {
    let cards = [
        (Material::Popover, "Popover (menus)"),
        (Material::Sheet, "Sheet (launcher, control center)"),
        (Material::Toast, "Toast (banner)"),
        (Material::Osd, "OSD"),
        (Material::Widget, "Widget"),
    ];
    rsx! {
        Section { title: "Material stack v2", note: "Each card twice over the same wallpaper: left with the stack off (MaterialStack all zero: wave 1's flat tint), right with the defaults: the vibrancy boost baked into the tint (Blitz cannot saturate what is behind), the 1 px inner top highlight, the 0.5 px dark outer hairline, a tight contact shadow and a wide ambient one.",
            div { class: "g-wall g-polish-cards", style: "background-image:url(\"{wallpaper::uri()}\")",
                for (material , name) in cards {
                    div { class: "g-polish-pair",
                        for (label , stack) in [("off", FLAT), ("v2", MaterialStack::default())] {
                            Root { material, stack: Some(stack), chrome: Some(RootChrome::Painted), style: "width:220px",
                                Surface { material,
                                    div { class: "g-polish-card",
                                        span { class: "g-name", "{name}" }
                                        span { class: "g-code", "stack {label}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Specimen { name: "Targets",
                code: target("highlight white .30 light / .12 dark; hairline 0.5 px black .14 / .60; contact 0 1px 2px black .10 / .30; ambient 0 12px 40px -12px black .28 / .55 (menus); vibrancy: tint chroma x1.4, light L +.012", "vibrancy saturates what is behind (not reproducible on Blitz, S15/S16); a 1 px inner highlight, a thin dark outer border and a soft two-part shadow on every menu and popover (L)"),
            }
        }
    }
}

/// The OSD card at the top right (design/20 section 1.7): a control-center module's shape on the
/// Osd material over the Work Space, in both schemes; the Level page has its looks and motion.
#[component]
fn OsdSection() -> Element {
    use super::level_tile::{Ground, LevelTile, STATES};
    rsx! {
        Section { title: "OSD", note: "The on-screen display at the top right under the bar, as current macOS shows volume and brightness: a title line over the level capsule on the Osd material with the Space's tint, radius --r-tile and the control-center grid's padding. The Level page has the three looks, the motion and the show and hide.",
            div { class: "g-row g-row-top",
                for scheme in [Scheme::Light, Scheme::Dark] {
                    for state in [STATES[1], STATES[4]] {
                        LevelTile { look: ds::LevelLook::Capsule, scheme, ground: Ground::Work, state }
                    }
                }
            }
            Specimen { name: "Targets",
                code: target("card 296 wide, radius 12 (--r-tile), padding 12, title 13/600, capsule 26 high with the glyph inside; drops in over --t-quick --e-out, lifts away over --t-move --e-exit", "a small panel at the top right under the menu bar, a title and a capsule slider with the glyph inside (L)"),
            }
        }
    }
}

#[component]
fn CornerSection() -> Element {
    rsx! {
        Section { title: "Continuous-curvature corners", note: "Corner::Squircle(r): one quadrant of design/08's n = 5 superellipse reaching 2 r along each edge, drawn as a mask; its shadow follows the circle that touches it at 45 degrees. Beside each, a CSS radius of the same r. The plate is the whole superellipse beside a 22.37 % CSS radius.",
            div { class: "g-polish-corners",
                for r in [22.0_f32, 40.0] {
                    Specimen { name: format!("Squircle({r}) and Px({r})"),
                        code: target(&format!("45° point {:.1} px in from the corner (circle {:.1})", r * 0.366, r * 0.414), "continuous corners on every window, menu and icon (L)"),
                        div { class: "g-row",
                            for corner in [Corner::Squircle(Px(r)), Corner::Px(Px(r))] {
                                div { class: "g-polish-fill",
                                    Surface { material: Material::Dock, theme: Some(Scheme::Dark), radius: Some(corner),
                                        div { class: "g-polish-fill" }
                                    }
                                }
                            }
                        }
                    }
                }
                Specimen { name: "Plate and a 22.37 % radius",
                    code: target("Lamé n = 5, area 95.0 % of the square", "the app-icon grid's rounded square, 22.37 % radius (design/08 section 2.1)"),
                    div { class: "g-row",
                        IconView { source: IconSource::Glyph(Icon::Folder), size: IconSize::Tile96, plate: Some(PlateFamily::Blue) }
                        div { class: "g-polish-plate-box",
                            Surface { material: Material::Dock, theme: Some(Scheme::Dark), radius: Some(Corner::Px(Px(21.5))),
                                div { class: "g-polish-plate-box" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DockSection() -> Element {
    let families = [
        (PlateFamily::Blue, Icon::Folder, "Files", Switch::On),
        (PlateFamily::Red, Icon::Camera, "Camera", Switch::Off),
        (PlateFamily::Amber, Icon::StickyNote, "Notes", Switch::On),
        (PlateFamily::Green, Icon::Terminal, "Terminal", Switch::On),
        (PlateFamily::Violet, Icon::Sparkles, "Studio", Switch::Off),
        (
            PlateFamily::Neutral,
            Icon::Settings,
            "Settings",
            Switch::Off,
        ),
    ];
    let pills = [
        ("Floor off (default)", DockMetrics::default()),
        (
            "Floor on (dock.floor = On)",
            DockMetrics {
                floor: DockFloorSetting::On,
                ..DockMetrics::default()
            },
        ),
    ];
    rsx! {
        Section { title: "Dock pill and tiles", note: "A Dock root with Corner::Squircle(18) and DockMetrics' defaults: 48 px tiles 8 apart, 6 px pill padding, 4 px running dots 3 px under the tiles. Every tile is an IconView plate in one of design/08's families (a placeholder until the generated icons arrive); the second label is shown by the caller at the shell tooltip size.",
            div { class: "g-wall g-row g-row-top", style: "background-image:url(\"{wallpaper::uri()}\")",
                for (name , metrics) in pills {
                    Specimen { name,
                        div { class: "g-polish-dock", style: metrics.style_attr(),
                            Root { material: Material::Dock, radius: Some(Corner::Squircle(Px(18.0))),
                                DockFloor {}
                                div { class: "g-polish-tiles",
                                    for (index , (family , icon , label , running)) in families.into_iter().enumerate() {
                                        div { class: "g-polish-tile",
                                            Tooltip {
                                                kind: TooltipKind::Fly,
                                                text: label,
                                                shown: Some(if index == 1 { Shown::Visible } else { Shown::Hidden }),
                                                IconView { source: IconSource::Glyph(icon), size: IconSize::Tile48, plate: Some(family) }
                                            }
                                            if running == Switch::On {
                                                RunningDot {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Specimen { name: "Targets",
                code: target("tile 48, gap 8, pad 6 (pill 60 high), dot 4 px, 3 px under the tile, squircle 18, label 12 px, floor off", "tilesize 48 (R1, H); running dot ~4-5 pt below the icon (R26, L); pill and gaps not published; the 3D floor is pre-Yosemite"),
            }
        }
    }
}

#[component]
fn LauncherSection() -> Element {
    let apps = APPS
        .iter()
        .zip(1u8..)
        .map(|(&(name, hue), value)| MenuEntry::Item {
            value,
            title: name.to_owned(),
            detail: Some("Application".to_owned()),
            tile: Some(
                app_icon(hue, IconSize::Tile48).map_or(Tile::Icon(Icon::Window), Tile::Source),
            ),
            trail: Trail::None,
            check: None,
            availability: Availability::Enabled,
        })
        .collect::<Vec<_>>();
    rsx! {
        Section { title: "Launcher", note: "CommandPalette in a Sheet surface 600 x 380: the card is as tall as its three rows (no empty box), with a squircle 14 corner and the stack's double shadow; the query at 22/500 beside a 20 px glyph, rows at 14 with 12 px details.",
            div { class: "g-wall", style: "background-image:url(\"{wallpaper::uri()}\")",
                Root { material: Material::Sheet, style: "width:600px; height:380px",
                    CommandPalette::<u8> {
                        label: "Launch",
                        placeholder: "Search",
                        query: "",
                        tokens: Vec::new(),
                        groups: vec![("Applications".to_owned(), apps)],
                        empty: "Nothing matches.",
                        oninput: |_| {},
                        onpick: |_| {},
                        onclose: |_| {},
                        host: CommandPaletteHost::Surface,
                        corner: Some(Corner::Squircle(Px(14.0))),
                    }
                }
            }
            Specimen { name: "Targets",
                code: target("field 22/500, glyph 20, rows 14 / 12, card sized to content, squircle 14, contact + ambient shadow", "Spotlight's position and row height are not published (R12, UNKNOWN); a large single-line field over compact rows, sized to its results (L)"),
            }
        }
    }
}

#[component]
fn WindowSection() -> Element {
    rsx! {
        Section { title: "Window shadow", note: "--shadow-window, documented for app windows: a contact shadow under a wide soft ambient one. Beside it, a single mid drop like S's .win, which it replaces.",
            div { class: "g-row g-polish-windows",
                Specimen { name: "--shadow-window", code: target("0 1px 3px .12 + 0 24px 64px -16px .40 (dark .40 / .66)", "a large soft window shadow with a tight contact edge (L)"),
                    div { class: "g-polish-window", style: "box-shadow:var(--shadow-window)" }
                }
                Specimen { name: "One drop (before)", code: "S's .win was 0 18px 40px -22px .45; drawn here with --shadow-pop, 0 18px 40px -16px .45".to_owned(),
                    div { class: "g-polish-window g-polish-window-old" }
                }
            }
        }
    }
}
