//! Path control: Finder's path bar, with a short path, a segment lit as a drop target, and a long
//! path whose middle collapses behind a menu.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::prelude::*;

/// The folders of a path, the root first.
fn folders(names: &[&str]) -> Vec<PathSegment> {
    names
        .iter()
        .map(|name| PathSegment::new(*name, Icon::Folder))
        .collect()
}

/// The Path control section.
#[component]
pub fn PathControlSection() -> Element {
    let mut said = use_signal(|| "nothing yet".to_owned());
    let lit = folders(&["Macintosh HD", "Users", "pohsuan", "Documents"]);
    let mut targeted = lit.clone();
    targeted[1] = targeted[1].clone().with_drop(DropState::Target);
    targeted[2] = targeted[2].clone().with_drop(DropState::Accepts);
    rsx! {
        Section { title: "Path control", note: "Finder's path bar: a glyph and a name per folder, a chevron between, the last in the full ink. A click reports the segment's index. A segment the owner marks Accepts or Target takes a drop and is lit as a row is (the second bar: Users is the target, pohsuan accepts). A path longer than the room keeps its root and last folders and collapses the middle behind a button that lists them.",
            div { class: "g-list", style: "width:520px",
                PathControl {
                    label: "Path",
                    segments: lit,
                    onpick: move |at: usize| said.set(format!("picked segment {at}")),
                }
            }
            div { class: "g-list", style: "width:520px",
                PathControl {
                    label: "Path with a drop",
                    segments: targeted,
                    onpick: move |at: usize| said.set(format!("picked segment {at}")),
                    ondrop: move |at: usize| said.set(format!("dropped on segment {at}")),
                }
            }
            div { class: "g-list", style: "width:520px",
                PathControl {
                    label: "Long path",
                    segments: folders(&["Macintosh HD", "Users", "pohsuan", "Documents", "Projects", "Quire", "Design", "Drafts"]),
                    onpick: move |at: usize| said.set(format!("picked segment {at}")),
                }
            }
            p { class: "g-code", "last picked: {said}" }
        }
    }
}
