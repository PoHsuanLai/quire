//! A `ThreadRow`'s `Common` lands on the row's own element, the `.ds-row` the list draws: its
//! `id`, the caller's `data-*` and class, and the element handed to `mounted`, so a hover card or a
//! menu keyed on the row finds it there and every part of the thread is a descendant of it.

use dioxus::prelude::*;
use ds::components::app::thread_row::ThreadRow;
use ds::prelude::*;
use ds::root::common::Common;
use ds::root::pass_through::{DataAttr, DataName, ExtraClass};
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 400,
    height: 120,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Thread() -> Element {
    let mut mounted = use_signal(|| 0u32);
    let data = DataName::parse("hc")
        .map(|name| vec![DataAttr::new(name, "thread:7")])
        .unwrap_or_default();
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:380px",
                ThreadRow {
                    name: "Dana Okafor",
                    via: None,
                    subject: "Re: UIDL",
                    snippet: None,
                    time: "09:41",
                    tags: rsx! {},
                    star: None,
                    strip: None,
                    onclick: |_| {},
                    common: Common {
                        id: Some("row-7".to_string()),
                        data,
                        extra_class: ExtraClass::parse("mine").ok(),
                        mounted: Some(EventHandler::new(move |_| mounted += 1)),
                        ..Common::default()
                    },
                }
            }
            p { class: "mounted", "{mounted()}" }
        }
    }
}

#[test]
fn the_common_props_reach_the_row_and_the_thread_sits_inside_it() {
    let mut harness = Harness::new(Thread, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    assert_eq!(harness.count("#row-7.ds-row.mine"), 1, "{}", harness.html());
    assert_eq!(
        harness.attr("#row-7", "data-hc").as_deref(),
        Some("thread:7")
    );
    assert_eq!(harness.count("[data-hc=\"thread:7\"] .ds-thread-name"), 1);
    assert_eq!(harness.text_of(".mounted").as_deref(), Some("1"));
}
