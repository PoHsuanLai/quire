//! The Settings window specimen: a System-Settings-like window built from the P2 components: a
//! titlebar, a `SplitView` of a `Sidebar` and the content, the content's `Toolbar`, and panes of
//! `FieldGroup`s of `FieldRow`s holding real controls, one `TabView` among them.

mod panes;

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::chrome::sidebar::Sidebar;
use ds::components::chrome::split_view::model::{PaneSpec, SplitPane};
use ds::components::chrome::split_view::view::SplitView;
use ds::components::chrome::toolbar::model::{ToolbarItem, ToolbarRoom};
use ds::components::chrome::toolbar::view::Toolbar;
use ds::components::chrome::window_frame::WindowTitlebar;
use ds::prelude::*;
use ds_core::vocab::RowState;
use ds_style::tokens::control_size::{ControlSize, SidebarSize};

/// What the sidebar lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub(crate) enum Category {
    /// The network pane.
    #[word(slug = "wi-fi", label = "Wi-Fi")]
    WiFi,
    /// Theme, accent and text size.
    Appearance,
    /// Brightness and the scale.
    Displays,
    /// The date, the time and the clock's look.
    #[word(slug = "date-time", label = "Date & Time")]
    DateTime,
}

impl Category {
    fn icon(self) -> Icon {
        match self {
            Category::WiFi => Icon::Wifi,
            Category::Appearance => Icon::SunMoon,
            Category::Displays => Icon::Monitor,
            Category::DateTime => Icon::Clock,
        }
    }
}

/// The sidebar's rows: headings and the categories whose names hold `query`.
fn sidebar_items(
    here: Category,
    query: &str,
    onselect: EventHandler<Category>,
) -> Vec<ListItem<&'static str>> {
    let wanted = query.to_lowercase();
    let row = |category: Category| {
        ListItem::row(
            category.slug(),
            category.label(),
            rsx! {
                Row {
                    title: TextLine::from(category.label()),
                    leading: RowLeading::Icon(category.icon()),
                    state: RowState {
                        selection: if here == category { Selection::Selected } else { Selection::Unselected },
                        ..RowState::default()
                    },
                    onclick: move |_| onselect.call(category),
                }
            },
        )
    };
    let shown = |category: &Category| category.label().to_lowercase().contains(&wanted);
    let groups: [(&'static str, &'static str, &[Category]); 2] = [
        ("h-network", "Network", &[Category::WiFi]),
        (
            "h-personal",
            "Personal",
            &[Category::Appearance, Category::Displays, Category::DateTime],
        ),
    ];
    groups
        .into_iter()
        .flat_map(|(key, title, categories)| {
            let rows: Vec<_> = categories
                .iter()
                .filter(|c| shown(c))
                .copied()
                .map(row)
                .collect();
            let heading = (!rows.is_empty())
                .then(|| ListItem::heading(key, rsx! { SectionHeader { title } }));
            heading.into_iter().chain(rows)
        })
        .collect()
}

/// The Settings window section.
#[component]
pub fn SettingsWindowPage() -> Element {
    let mut category = use_signal(|| Category::Appearance);
    let mut history = use_signal(|| vec![Category::Appearance]);
    let mut at = use_signal(|| 0usize);
    let mut sidebar = use_signal(|| Shown::Visible);
    let mut query = use_signal(String::new);
    let go = EventHandler::new(move |next: Category| {
        let kept = history.peek()[..=*at.peek()].to_vec();
        history.set([kept, vec![next]].concat());
        let next_at = *at.peek() + 1;
        at.set(next_at);
        category.set(next);
    });
    let steps = EventHandler::new(move |to: usize| {
        at.set(to);
        let target = history.peek().get(to).copied();
        if let Some(target) = target {
            category.set(target);
        }
    });
    let leading = vec![
        ToolbarItem::new("sidebar", "Sidebar", Icon::PanelLeft),
        ToolbarItem::new("back", "Back", Icon::ChevronLeft).with(if at() == 0 {
            Availability::Disabled
        } else {
            Availability::Enabled
        }),
        ToolbarItem::new("forward", "Forward", Icon::ChevronRight).with(
            if at() + 1 >= history().len() {
                Availability::Disabled
            } else {
                Availability::Enabled
            },
        ),
    ];
    let side = rsx! {
        Sidebar::<&'static str> {
            label: "Settings",
            size: SidebarSize::Medium,
            cursor: Some(category().slug()),
            items: sidebar_items(category(), &query(), go),
            onselect: move |key: &'static str| {
                if let Some(next) = Category::parse(key) {
                    go.call(next);
                }
            },
            header: rsx! {
                TextField { label: "Search", value: query(), kind: FieldKind::Search, placeholder: "Search", size: ControlSize::Small, oninput: move |next| query.set(next) }
            },
        }
    };
    rsx! {
        Section { title: "Settings window", note: "A System-Settings-like window from the catalogue's own parts: WindowTitlebar over a SplitView whose first pane is a Sidebar (search header, source list, folds on a spring from the toolbar's first button or a drag of the divider) and whose content is a Toolbar (back and forward walk the history) over FieldGroups of FieldRows with a Toggle, PopUpButton, SegmentedControl, RadioGroup, Slider, Stepper, Checkbox, TextField, DatePicker and a TabView.",
            div { class: "g-window",
                div { class: "g-window-bar", WindowTitlebar { title: "System Settings" } }
                div { class: "g-window-body",
                    SplitView {
                        label: "Settings",
                        panes: vec![SplitPane::new(PaneSpec::SIDEBAR, side).shown(sidebar())],
                        on_shown: move |(_, shown)| sidebar.set(shown),
                        div { class: "g-window-content",
                            Toolbar::<&'static str> {
                                leading,
                                title: Some(TextLine::from(category().label())),
                                room: ToolbarRoom::Fixed(Px(680.0)),
                                onpick: move |value: &'static str| match value {
                                    "sidebar" => sidebar.set(sidebar().flipped()),
                                    "back" => steps.call(at().saturating_sub(1)),
                                    _ => steps.call(at() + 1),
                                },
                            }
                            div { class: "g-window-scroll", {panes::pane(category())} }
                        }
                    }
                }
            }
        }
    }
}
