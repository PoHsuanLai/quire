//! Pictures of a mail-like window built from the layout parts, light and dark: a `SplitView` whose
//! sidebar pane is an `EdgePeek` holding a `Sidebar` of several sections and a foot clear over the Space's tint, and a card with a `Toolbar` whose picked item hangs a `Menu` or a `Sheet` from the card.
//! Posed on the virtual clock: pinned; folded away and peeking; a menu hung from a toolbar button;
//! a sheet hanging from the card. Each render must paint; the pictures are written only when
//! `QUIRE_GALLERY_SHOTS` names a directory, as `window-layout-<pose>-<scheme>.png`.

use dioxus::prelude::*;
use ds::base::geometry::units::{Point, Px};
use ds::base::vocab::RowState;
use ds::components::app::edge_peek::EdgePeek;
use ds::components::app::space_editor::dot::SpaceDot;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::sidebar_model::{SidebarFill, SidebarSection};
use ds::components::chrome::split_view::model::{PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::components::chrome::toolbar::model::{Picked, ToolbarItem, ToolbarRoom};
use ds::components::chrome::toolbar::view::Toolbar;
use ds::components::controls::button_model::Answers;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::components::overlays::sheet_attach::Attach;
use ds::host::measure::{Anchor, use_rect};
use ds::prelude::*;
use ds::style::space::frame_vars::FrameVars;
use ds::style::space::presets::PRESETS;
use ds::style::tokens::control_size::ControlSize;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::cell::Cell;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 1000,
    height: 640,
    scale_percent: 100,
};

thread_local! {
    static DARK: Cell<bool> = const { Cell::new(false) };
}

const PLACES: &[(&str, Icon)] = &[
    ("Inbox", Icon::Inbox),
    ("Sent", Icon::Send),
    ("Trash", Icon::Trash),
];
const LABELS: &[(&str, Icon)] = &[("Work", Icon::Tag), ("Travel", Icon::Tag)];

fn heading(title: &'static str) -> ListItem<&'static str> {
    ListItem::heading(title, rsx! { SectionHeader { title } })
}

fn rows(
    title: &'static str,
    rows: &'static [(&'static str, Icon)],
    here: &'static str,
) -> SidebarSection<&'static str> {
    let mut items = vec![heading(title)];
    items.extend(rows.iter().map(|&(name, icon)| {
        ListItem::row(
            name,
            name,
            rsx! {
                Row {
                    title: TextLine::from(name),
                    leading: RowLeading::Icon(icon),
                    state: RowState { selection: Selection::of(&here, &name), ..RowState::default() },
                }
            },
        )
    }));
    SidebarSection::List(items)
}

fn look() -> SpaceLook {
    SpaceLook {
        dots: PRESETS[3].dots.to_vec(),
        grain: Grain(0),
        ..SpaceLook::default()
    }
}

#[allow(non_snake_case)]
fn Page() -> Element {
    let theme = if DARK.with(Cell::get) {
        Theme::Dark
    } else {
        Theme::Light
    };
    let scheme = if theme == Theme::Dark {
        Scheme::Dark
    } else {
        Scheme::Light
    };
    let mut pinned = use_signal(|| Shown::Visible);
    let mut menu = use_signal(|| None::<Anchor>);
    let mut sheet = use_signal(|| Shown::Hidden);
    let card = use_rect();
    let foot = rsx! {
        Button { bezel: Bezel::Inline, label: "Home", onclick: |_| {} }
        div { style: "display:flex;gap:var(--s-5);margin-left:auto",
            for index in 0..3usize {
                SpaceDot {
                    key: "{index}",
                    name: "Space",
                    frame: FrameVars::of(&SpaceLook { dots: PRESETS[index].dots.to_vec(), ..look() }, scheme),
                    selection: Selection::of(&index, &0),
                    shortcut: Shortcut(vec![]),
                    onclick: |()| {},
                }
            }
        }
        Button {
            bezel: Bezel::Toolbar, image: ImagePosition::Only, size: ControlSize::Small,
            icon: Icon::PanelLeft, label: "Hide sidebar",
            onclick: move |_| pinned.set(Shown::Hidden),
        }
    };
    let side = rsx! {
        EdgePeek { label: "Sidebar", pinned: pinned(), onpin: move |()| pinned.set(Shown::Visible),
            Sidebar::<&'static str> {
                label: "Mail",
                fill: SidebarFill::Clear,
                cursor: Some("Inbox"),
                onselect: |_| {},
                sections: vec![rows("Places", PLACES, "Inbox"), rows("Labels", LABELS, "Inbox")],
                foot,
            }
        }
    };
    let tools = vec![
        ToolbarItem::new("group", "Group", Icon::Group),
        ToolbarItem::new("rules", "Rules", Icon::Settings),
    ];
    rsx! {
        Ds { appearance: Appearance { theme, ..Appearance::default() }, look: look(), material: Material::Window, extent: RootExtent::Viewport,
            div { style: "box-sizing:border-box;height:100vh;padding:8px 8px 8px 0",
                SplitView {
                    label: "Mail",
                    panes: vec![SplitPane::new(PaneSpec::SIDEBAR, side).shown(pinned()).peeking()],
                    on_shown: move |(_, shown)| pinned.set(shown),
                    div {
                        class: "card",
                        style: "box-sizing:border-box;height:100%;background:var(--surface);color:var(--ink);border-radius:var(--r-card);overflow:hidden",
                        onmounted: move |event| card.on_mounted(event),
                        Toolbar::<&'static str> {
                            trailing: tools,
                            title: Some(TextLine::from("Inbox")),
                            room: ToolbarRoom::Fixed(Px(700.0)),
                            onpick: move |pick: Picked<&'static str>| match pick.value {
                                "group" => menu.set(pick.anchor),
                                _ => sheet.set(Shown::Visible),
                            },
                        }
                    }
                }
            }
            if let Some(anchor) = menu() {
                Menu::<u8> {
                    placement: MenuPlacement::Popup,
                    anchor,
                    items: vec![
                        MenuItem::new(1u8, "None").with_check(Check::Off),
                        MenuItem::new(2u8, "By sender").with_check(Check::On),
                        MenuItem::new(3u8, "By date").with_check(Check::Off),
                    ],
                    onpick: move |_| menu.set(None),
                    onclose: move |()| menu.set(None),
                }
            }
            if let Some(anchor) = card.anchor() {
                Sheet {
                    label: "Rules",
                    onclose: move |()| sheet.set(Shown::Hidden),
                    shown: Some(sheet()),
                    attach: Attach::Within(anchor),
                    div { style: "padding:var(--s-16);display:flex;flex-direction:column;gap:var(--s-8)",
                        h3 { "Rules" }
                        p { "A sheet hangs from the card, not the window." }
                        Button { answers: Answers::Return, label: "Done", onclick: move |_| sheet.set(Shown::Hidden) }
                    }
                }
            }
        }
    }
}

fn save(harness: &mut Harness, pose: &str, dark: bool) {
    let shot = harness.render().expect("renders");
    assert!(shot.width() > 0, "{pose}");
    if let Ok(dir) = std::env::var("QUIRE_GALLERY_SHOTS") {
        let scheme = if dark { "dark" } else { "light" };
        shot.save(format!("{dir}/window-layout-{pose}-{scheme}.png"))
            .expect("writes the shot");
    }
}

fn at(x: f32, y: f32) -> Point {
    Point { x: Px(x), y: Px(y) }
}

fn pose(dark: bool) {
    DARK.with(|cell| cell.set(dark));
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(600));
    save(&mut harness, "pinned", dark);

    let hide = harness
        .centre(".ds-sidebar-foot > .ds-button:last-child")
        .expect("the hide button");
    harness.send(Input::click(hide));
    harness.advance(Duration::from_millis(700));
    harness.send(Input::pointer_move(at(600.0, 300.0)));
    harness.advance(Duration::from_millis(600));
    harness.send(Input::pointer_move(at(3.0, 300.0)));
    harness.advance(Duration::from_millis(900));
    assert_eq!(
        harness.attr(".ds-side", "data-side").as_deref(),
        Some("peek")
    );
    save(&mut harness, "peeking", dark);

    harness.send(Input::click(at(3.0, 300.0)));
    harness.advance(Duration::from_millis(900));
    let group = harness
        .centre(".ds-toolbar-trailing > .ds-button:first-child")
        .expect("the Group button");
    harness.send(Input::click(group));
    harness.advance(Duration::from_millis(500));
    assert!(
        harness.count(".ds-menu-item") >= 3,
        "the menu hangs from Group"
    );
    save(&mut harness, "menu", dark);

    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(Duration::from_millis(400));
    let rules = harness
        .centre(".ds-toolbar-trailing > .ds-button:last-child")
        .expect("the Rules button");
    harness.send(Input::click(rules));
    harness.advance(Duration::from_millis(700));
    save(&mut harness, "sheet", dark);
}

#[test]
fn the_window_layout_paints_each_pose_in_light() {
    pose(false);
}

#[test]
fn the_window_layout_paints_each_pose_in_dark() {
    pose(true);
}
