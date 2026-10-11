//! Structure of the two temor additions: a `RowLeading::Element` draws decorative, in the
//! leading box, with no handlers of its own; a `RunTone::Mono` run is a span with its tone word,
//! not the chip's `code`.

use dioxus::prelude::*;
use ds::prelude::*;

fn render(make: fn() -> Element) -> String {
    #[derive(Props, Clone)]
    struct Host {
        make: fn() -> Element,
    }
    impl PartialEq for Host {
        fn eq(&self, _: &Self) -> bool {
            false
        }
    }
    fn host(props: Host) -> Element {
        (props.make)()
    }
    let mut dom = VirtualDom::new_with_props(host, Host { make });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn an_element_leads_a_row_decoratively() {
    let html = render(|| {
        rsx! {
            Row {
                title: "main",
                leading: RowLeading::Element(rsx! { span { class: "orb" } }),
            }
        }
    });
    assert!(html.contains(r#"data-leading="element""#), "{html}");
    assert!(html.contains(r#"aria-hidden="true""#), "{html}");
    assert!(html.contains(r#"<span class="orb">"#), "{html}");
    assert!(!html.contains(r#"data-slot="trailing""#), "{html}");
}

#[test]
fn other_leading_parts_are_not_hidden() {
    let html = render(|| {
        rsx! { Row { title: "main", leading: RowLeading::Text("A".to_string()) } }
    });
    assert!(!html.contains("aria-hidden"), "{html}");
}

#[test]
fn a_mono_run_is_a_plain_span_with_its_tone() {
    let html = render(|| {
        rsx! {
            Row {
                title: TextLine::Runs(vec![TextRun::new("src/lib", RunTone::Mono)]),
            }
        }
    });
    assert!(
        html.contains(r#"<span class="ds-run" data-tone="mono">src/lib</span>"#),
        "{html}"
    );
    assert!(!html.contains("<code"), "{html}");
}
