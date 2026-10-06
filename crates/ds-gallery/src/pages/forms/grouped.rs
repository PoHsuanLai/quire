//! A grouped `List` of panes (the Forms page, and the Lists page's grouped section): tiles, a person, a chevron on every row that opens something, a
//! value on one that only shows it, and the selection wash.

use dioxus::prelude::*;
use ds::components::content::avatar::{AvatarFace, AvatarShape, AvatarSize, AvatarTone, PersonHue};
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds::style::tokens::hex::Hex;

/// One settings row: `key` led by a tile of `icon` in `colour`.
fn pane(
    key: &'static str,
    icon: Icon,
    colour: Hex,
    accessory: Accessory,
) -> ListItem<&'static str> {
    ListItem::row(
        key,
        key,
        rsx! { Row { title: key, size: RowSize::Settings, leading: RowLeading::Tile(TileFace::Glyph(icon, colour)), accessory } },
    )
}

/// The panes, with the cursor resting on the second.
#[component]
pub fn Panes() -> Element {
    let mut cursor = use_signal(|| Some("Bluetooth"));
    let person = TileFace::Avatar(AvatarFace {
        initial: 'D',
        size: AvatarSize::Size34,
        tone: AvatarTone::Person(PersonHue(120)),
        shape: AvatarShape::Round,
    });
    rsx! {
        List::<&'static str> {
            label: "Panes",
            style: ListStyle::Grouped,
            cursor: cursor(),
            onselect: move |key| cursor.set(Some(key)),
            items: vec![
                pane("Wi-Fi", Icon::Wifi, Hex([0x0a, 0x84, 0xff]), Accessory::Chevron),
                pane("Bluetooth", Icon::Bluetooth, Hex([0x0a, 0x84, 0xff]), Accessory::Text("On".to_owned())),
                pane("Notifications", Icon::Bell, Hex([0xff, 0x3b, 0x30]), Accessory::Chevron),
                pane("Sound", Icon::Speaker, Hex([0xff, 0x9f, 0x0a]), Accessory::Chevron),
                ListItem::row("Dana", "Dana", rsx! { Row { title: "Dana Okafor", detail: TextLine::from("Apple ID, iCloud and more"), size: RowSize::Settings, leading: RowLeading::Tile(person), accessory: Accessory::Chevron } }),
            ],
        }
    }
}
