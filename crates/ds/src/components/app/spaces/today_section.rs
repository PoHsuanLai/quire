//! Today in the sidebar: the current Space's recent items as `TodayTabs`.

use super::controller::SpacesHandle;
use super::today_state::TodayHandle;
use crate::components::app::today_tabs::{TodayTab, TodayTabs};
use crate::components::lists::row::leading::RowLeading;
use crate::components::lists::section_header::{HeaderAction, SectionHeader};
use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::time::clock;
use ds_core::vocab::Shown;
use ds_style::space::list::Epoch;

/// How the app draws one of its Today items.
#[derive(Debug, Clone, PartialEq)]
pub struct TodayRow {
    /// The row's words.
    pub title: String,
    /// What leads it: an avatar, an icon.
    pub leading: RowLeading,
}

/// The "Today" group of the Space on screen: a header with Clear, then a tab for each item
/// opened in it within the last 12 hours, closing by its button or by running out. Closing and
/// expiring touch only Today, never the app's data.
///
/// `row` draws an item, or `None` to leave it out (the thing it pointed at is gone). `extra` is
/// the app's own rows under the header (parked drafts); `extra_shown: Visible` keeps the header
/// up for them when there are no tabs. With nothing to show, nothing is drawn: a source list
/// does not show an empty group.
#[component]
pub fn TodaySection<P, R, I, K>(
    spaces: SpacesHandle<P, R>,
    today: TodayHandle<I, K>,
    row: Callback<I, Option<TodayRow>>,
    on_pick: EventHandler<I>,
    #[props(default)] selected: Option<I>,
    #[props(default)] extra: Option<Element>,
    #[props(default)] extra_shown: Shown,
    #[props(default = "Today".to_owned())] label: String,
) -> Element
where
    P: Clone + PartialEq + 'static,
    R: Clone + Default + PartialEq + 'static,
    I: Clone + PartialEq + std::hash::Hash + 'static,
    K: Clone + PartialEq + 'static,
{
    let space = spaces.current();
    let tabs: Vec<TodayTab<I>> = today
        .today()
        .read()
        .live(space, Epoch::now())
        .into_iter()
        .filter_map(|(entry, left)| {
            let shown = row.call(entry.item.clone())?;
            Some(TodayTab {
                key: entry.item.clone(),
                title: shown.title,
                leading: shown.leading,
                expires: clock::now() + left,
                common: Common::default(),
                onpointerenter: None,
                onpointerleave: None,
            })
        })
        .collect();
    let live = !tabs.is_empty();
    let parked = !today.today().read().parked_in(space).is_empty();
    let any = live || parked || extra_shown == Shown::Visible;
    let actions = if live {
        vec![HeaderAction::new(
            "Clear",
            EventHandler::new(move |()| today.clear(space)),
        )]
    } else {
        Vec::new()
    };
    rsx! {
        if any {
            SectionHeader { title: label.clone(), actions }
        }
        if let Some(extra) = extra {
            {extra}
        }
        TodayTabs::<I> {
            label,
            tabs,
            selected,
            onpick: move |item: I| on_pick.call(item),
            onclose: move |item: I| today.close(space, &item),
        }
    }
}
