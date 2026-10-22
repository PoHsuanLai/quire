//! DocumentTabs: the Finder and Safari tab strip, live: select, close, new and drag to reorder.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::chrome::document_tabs::{DocumentTab, DocumentTabs};
use ds::components::chrome::document_tabs_model::strip_shown;

/// The DocumentTabs section.
#[component]
pub fn DocumentTabsSection() -> Element {
    let mut titles = use_signal(|| {
        vec![
            "Documents".to_owned(),
            "Downloads".to_owned(),
            "Projects and the very long name of a folder that does not fit".to_owned(),
        ]
    });
    let mut selected = use_signal(|| 0usize);
    let mut opened = use_signal(|| 0u32);
    let tabs: Vec<DocumentTab> = titles().into_iter().map(DocumentTab::new).collect();
    rsx! {
        Section { title: "DocumentTabs", note: "A document window's tab strip: the tabs share the width, the close button shows under the pointer and on the selected tab, a tab's full title is its tooltip, and a tab can be dragged onto another's place. The app draws the strip only from two tabs.",
            div { style: "width:520px",
                if strip_shown(tabs.len()) {
                    DocumentTabs {
                        label: "Open folders",
                        tabs,
                        selected: selected(),
                        onselect: move |at| selected.set(at),
                        onclose: move |at: usize| {
                            titles.write().remove(at);
                            let last = titles.read().len().saturating_sub(1);
                            selected.set(selected().min(last));
                        },
                        onnew: move |_| {
                            opened += 1;
                            titles.write().push(format!("Untitled {opened}"));
                            selected.set(titles.read().len() - 1);
                        },
                        onreorder: move |(from, to): (usize, usize)| {
                            let mut list = titles.write();
                            let moved = list.remove(from);
                            list.insert(to, moved);
                            selected.set(to);
                        },
                    }
                } else {
                    p { "One tab left: the strip is gone." }
                }
            }
        }
    }
}
