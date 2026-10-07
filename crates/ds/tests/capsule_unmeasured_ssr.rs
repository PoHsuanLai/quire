//! A capsule that has not measured its stage yet is drawn hidden and takes no presses, so a
//! narrow stage never flashes the full capsule for a frame before the fit drops slots.

use dioxus::core::VirtualDom;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::chrome::capsule::model::CapsuleSlot;
use ds::components::chrome::capsule::view::Capsule;
use ds::prelude::*;

#[allow(non_snake_case)]
fn Unmeasured() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host,
            Capsule::<u8> {
                label: "Playback",
                slots: vec![CapsuleSlot::button(1, "Play", Icon::Play).essential()],
                shown: Shown::Visible,
                onpick: |_| {},
            }
        }
    }
}

#[test]
fn an_unmeasured_capsule_is_hidden_and_takes_no_presses() {
    let mut dom = VirtualDom::new(Unmeasured);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);
    assert!(
        html.contains("visibility:hidden;pointer-events:none"),
        "unmeasured, it is not seen: {html}"
    );
}
