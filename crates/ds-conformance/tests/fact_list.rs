//! `FactList` on a real Blitz document: the labels share one column and the values line up
//! beside them, however long a label is.

use dioxus::prelude::*;
use ds::components::fields::fact_list::{Fact, FactList};
use ds::prelude::*;
use ds_harness::{Driver, Harness, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 240,
    scale_percent: 100,
};

#[allow(non_snake_case)]
fn Facts() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "width:400px",
                FactList { facts: vec![
                    Fact::new("When", "Tue 4 Nov, 10:00 to 11:00"),
                    Fact::new("Organiser", "Dana Okafor"),
                    Fact::new("Where", "Room 4"),
                ] }
            }
        }
    }
}

#[test]
fn the_values_line_up_and_each_fact_is_a_row_under_the_last() {
    let mut harness = Harness::new(Facts, VIEW);
    harness.advance(Duration::from_millis(100));
    let values: Vec<_> = (1..=3)
        .map(|n| {
            harness
                .rect(&format!(
                    ".ds-fact-list-item:nth-child({n}) .ds-fact-list-value"
                ))
                .expect("a value")
        })
        .collect();
    assert_eq!(harness.count(".ds-fact-list-item"), 3);
    assert!(
        values
            .iter()
            .all(|v| (v.origin.x.0 - values[0].origin.x.0).abs() < 0.5),
        "the values share a left edge: {values:?}"
    );
    assert!(
        values[1].origin.y.0 > values[0].origin.y.0 && values[2].origin.y.0 > values[1].origin.y.0,
        "each fact is under the last: {values:?}"
    );
    let label = harness.rect(".ds-fact-list-label").expect("a label");
    assert!(
        (label.size.width.0 - 120.0).abs() < 1.0,
        "the label column is 120 px: {label:?}"
    );
}
