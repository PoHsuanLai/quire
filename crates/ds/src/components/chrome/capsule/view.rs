//! Capsule: the pill of controls that floats over content (design/30 section 2.7a). It is a
//! card in the Osd material (the transparent-scope rule of `ds_style`'s materials paints it, as
//! it does a popover), bottom centre of its parent, whose buttons are `Button { Toolbar }` and
//! whose readouts are text. The owner decides when it shows (`shown`): it fades in over
//! `--t-quick`, fades out over `--t-quick` and calls `on_hidden` once it has gone, and takes
//! the pointer only while it is on screen.
//!
//! Markup: `div.ds.ds-capsule-scope` (a transparent Osd scope that fills its parent) holding
//! `div.ds-capsule[role=toolbar][data-shown][data-presence]` of `button.ds-button`,
//! `span.ds-capsule-readout` and `span.ds-capsule-divider`.
//!
//! Its parent is a box with a size and `position:relative`, since Blitz places an absolutely
//! positioned scope against its parent, not its nearest positioned ancestor.

use crate::components::chrome::capsule::model::CapsuleSlot;
use crate::components::chrome::toolbar::model::ToolbarItem;
use crate::components::content::icon_source::IconSource;
use crate::components::controls::button::Button;
use crate::components::controls::button_model::{Bezel, ImagePosition};
use crate::root::common::Common;
use crate::root::surface::ClassedScope;
use dioxus::prelude::*;
use ds_core::press::Press;
use ds_core::vocab::Shown;
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_style::appearance::material::Material;
use ds_style::tokens::control_size::ControlSize;

/// One button of the capsule.
fn button<T: Clone + PartialEq + 'static>(
    item: &ToolbarItem<T>,
    onpick: EventHandler<T>,
) -> Element {
    let value = item.value.clone();
    rsx! {
        Button {
            label: item.label.clone(),
            title: Some(item.label.clone()),
            bezel: Bezel::Toolbar,
            size: ControlSize::Large,
            image: ImagePosition::Only,
            icon: Some(IconSource::from(item.icon)),
            value: item.check,
            availability: item.availability,
            onclick: move |_: Press| onpick.call(value.clone()),
        }
    }
}

/// A capsule of `slots`. `label` names the toolbar to a screen reader. `onpick` hears a button's
/// value; `onpointerenter` and `onpointerleave` tell the owner when the pointer is over the
/// capsule, which is when it must not hide. `on_hidden` runs once after a hide has settled; a
/// show while it leaves takes the hide back.
#[component]
pub fn Capsule<T: Clone + PartialEq + 'static>(
    label: String,
    slots: Vec<CapsuleSlot<T>>,
    shown: Shown,
    #[props(default)] on_hidden: EventHandler<()>,
    onpick: EventHandler<T>,
    #[props(default)] onpointerenter: EventHandler<()>,
    #[props(default)] onpointerleave: EventHandler<()>,
    #[props(default)] common: Common,
) -> Element {
    let Presented { presence, alias } = use_presence(
        shown,
        PresenceSpec {
            enter: Anim::PaletteFade,
            exit: Exit::Fade,
        },
        Some(on_hidden),
    );
    let class = common.class("ds-capsule");
    let data = common.data_attributes();
    let name = common.aria_label.clone().unwrap_or(label);
    rsx! {
        ClassedScope { material: Material::Osd, class: "ds-capsule-scope",
            div {
                id: common.id.clone(),
                class,
                role: "toolbar",
                "aria-label": "{name}",
                "data-shown": presence.shown().slug(),
                "data-presence": presence.drawn_slug(),
                "data-pulse": alias.slug(),
                onmounted: move |event| common.mounted(event),
                onpointerenter: move |_| onpointerenter.call(()),
                onpointerleave: move |_| onpointerleave.call(()),
                ..data,
                for slot in slots.iter() {
                    match slot {
                        CapsuleSlot::Item(item) => rsx! { {button(item, onpick)} },
                        CapsuleSlot::Readout(text) => rsx! {
                            span { class: "ds-capsule-readout", "{text}" }
                        },
                        CapsuleSlot::Divider => rsx! {
                            span { class: "ds-capsule-divider", role: "separator" }
                        },
                    }
                }
            }
        }
    }
}
