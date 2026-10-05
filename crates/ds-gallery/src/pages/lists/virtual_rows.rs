//! VirtualList: ten thousand rows of which only the ones near the viewport are mounted. The
//! cursor moves with the arrow keys and is kept in view; the buttons jump it far away, and the
//! counter shows how many rows the list has asked for so far.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::base::press::Press;
use ds::components::lists::virtual_list::{RowHeight, VirtualList};
use ds::prelude::*;
use ds_core::vocab::RowState;

/// How many rows the list has.
const ROWS: u32 = 10_000;

/// The virtual list and its controls.
#[component]
pub fn VirtualGallery() -> Element {
    let mut cursor = use_signal(|| Some(0u32));
    let mut asked = use_signal(|| 0u32);
    let at = cursor().unwrap_or(0);
    let row = Callback::new(move |key: u32| {
        let state = RowState {
            selection: match Some(key) == cursor() {
                true => Selection::Selected,
                false => Selection::Unselected,
            },
            ..RowState::default()
        };
        rsx! {
            Row { title: format!("Message {}", key + 1), state, onclick: move |_| cursor.set(Some(key)) }
        }
    });
    rsx! {
        Section {
            title: "VirtualList",
            note: "Ten thousand rows, thirty-two points each: only the rows in the viewport and four on each side are mounted, whatever the scroll position. The arrow keys, Home and End move the cursor and the list scrolls the least that shows it; a row's own press selects it. Near the end the list asks for more.",
            div { class: "g-row g-row-top",
                div { class: "g-list", style: "width:280px; height:256px; display:flex; flex-direction:column",
                    VirtualList::<u32> {
                        label: "Messages",
                        keys: (0..ROWS).collect::<Vec<u32>>(),
                        row,
                        height: RowHeight::Fixed(Px(32.0)),
                        cursor: cursor(),
                        onselect: move |key| cursor.set(Some(key)),
                        near_end: move |()| asked += 1,
                    }
                }
                div { class: "g-col",
                    Button { label: "Jump to the middle", onclick: move |_: Press| cursor.set(Some(ROWS / 2)) }
                    Button { label: "Jump to the end", onclick: move |_: Press| cursor.set(Some(ROWS - 1)) }
                    Button { label: "Back to the top", onclick: move |_: Press| cursor.set(Some(0)) }
                    p { class: "g-note", "Cursor on row {at + 1}. The end was near {asked} time(s)." }
                }
            }
        }
    }
}
