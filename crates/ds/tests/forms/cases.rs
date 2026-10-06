//! Form, FormSection and IconTile in every state, as data: the table the golden test walks.
//! Goldens live in `tests/snapshots/forms/<component>/<state>.html`.

use crate::scoped::Scoped;
use dioxus::prelude::*;
use ds::components::content::avatar::{AvatarFace, AvatarShape, AvatarSize, AvatarTone, PersonHue};
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds_style::tokens::hex::Hex;

/// One component in one state.
pub struct Case {
    pub component: &'static str,
    pub state: &'static str,
    pub make: fn() -> Element,
}

const BLUE: Hex = Hex([0x0a, 0x84, 0xff]);

pub const CASES: &[Case] = &[
    Case {
        component: "icon_tile",
        state: "glyph",
        make: || rsx! { IconTile { icon: Icon::Wifi, colour: BLUE } },
    },
    Case {
        component: "icon_tile",
        state: "avatar",
        make: || rsx! { Scoped { {ds::components::forms::icon_tile::tile(TileFace::Avatar(AvatarFace { initial: 'D', size: AvatarSize::Size34, tone: AvatarTone::Person(PersonHue(120)), shape: AvatarShape::Round }))} } },
    },
    Case {
        component: "form_section",
        state: "bare",
        make: || rsx! { FormSection { div { "rows" } } },
    },
    Case {
        component: "form_section",
        state: "titled-with-footer",
        make: || rsx! { FormSection { title: "Network", footer: "Known networks are joined automatically.", div { "rows" } } },
    },
    Case {
        component: "form_section",
        state: "grouped-list",
        make: || rsx! { Scoped { FormSection { title: "Network", List::<&'static str> { label: "Network", style: ListStyle::Grouped, items: vec![ListItem::row("Wi-Fi", "Wi-Fi", rsx! { Scoped { Row { title: "Wi-Fi", size: RowSize::Settings, leading: RowLeading::Tile(TileFace::Glyph(Icon::Wifi, BLUE)), accessory: Accessory::Chevron } } })] } } } },
    },
    Case {
        component: "form",
        state: "two-sections",
        make: || rsx! { Form { FormSection { title: "General", div { "a" } } FormSection { title: "Privacy", footer: "Shared with no one.", div { "b" } } } },
    },
];
