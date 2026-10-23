//! A `Row` without a `common.aria_label` is spoken by its title alone, not by its run-together
//! text (leading letter, title, detail); an explicit label wins. A menu item is spoken by its
//! title, not by its key glyph and hint.

use dioxus::prelude::*;
use ds::prelude::RowLeading;
use ds::prelude::*;
use ds::root::common::Common;
use ds_harness::{Clock, Harness, HarnessConfig, Viewport};

const VIEW: Viewport = Viewport {
    width: 400,
    height: 200,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:360px",
                Row {
                    leading: RowLeading::Text("M".to_owned()),
                    title: "Mada@rig.test",
                    detail: Some("Microsoft".into()),
                    common: Common { id: Some("plain".to_owned()), ..Common::default() },
                }
                Row {
                    title: "Mada@rig.test",
                    common: Common {
                        id: Some("named".to_owned()),
                        aria_label: Some("Work account".to_owned()),
                        ..Common::default()
                    },
                }
            }
        }
    }
}

#[test]
fn a_row_is_spoken_by_its_title_unless_the_caller_named_it() {
    let harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    assert_eq!(
        harness.attr("#plain", "aria-label").as_deref(),
        Some("Mada@rig.test")
    );
    assert_eq!(
        harness.attr("#named", "aria-label").as_deref(),
        Some("Work account")
    );
}
