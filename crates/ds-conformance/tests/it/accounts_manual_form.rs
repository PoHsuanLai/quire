//! The hand-typed server form on a real Blitz document: choosing POP3 in the Protocol pop-up
//! reports the slug "pop3" through the edit event, and no row squeezes its label (a label beside
//! a wide field keeps its words whole).

use dioxus::prelude::*;
use ds::prelude::*;
use ds_harness::harness::settle_until;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use ds_shell::accounts::model::{Choice, FieldRole, FieldText, FormField, FormPart, Requirement};
use ds_shell::prelude::*;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 560,
    height: 640,
    scale_percent: 100,
};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

fn fields(protocol: &str) -> Vec<FormField> {
    let plain = |role, text: &str| {
        FormField::new(
            role,
            Requirement::Required,
            FieldText::Plain(text.to_owned()),
        )
        .in_part(FormPart::Incoming)
    };
    vec![
        plain(FieldRole::Protocol, protocol).choosing(vec![
            Choice::new("imap", "IMAP"),
            Choice::new("pop3", "POP3"),
            Choice::new("jmap", "JMAP"),
        ]),
        plain(FieldRole::Server, "imap.example.org"),
        plain(FieldRole::OutgoingSecurity, "Description of a long label"),
    ]
}

#[allow(non_snake_case)]
fn Form() -> Element {
    let mut protocol = use_signal(|| "imap".to_owned());
    let mut heard = use_signal(Vec::<String>::new);
    rsx! {
        Ds { sheet: Some(ds_shell::stylesheet()), appearance: Appearance::default(), material: Material::Sheet, extent: RootExtent::Viewport,
            SignInForm {
                provider: "Example",
                mark: ds::components::content::provider_mark::MarkProvider::Imap,
                fields: fields(&protocol()),
                on_input: move |(role, text): (FieldRole, FieldText)| {
                    if let (FieldRole::Protocol, FieldText::Plain(slug)) = (role, text) {
                        heard.with_mut(|heard| heard.push(slug.clone()));
                        protocol.set(slug);
                    }
                },
                on_submit: |_| {},
                on_back: |_| {},
                on_cancel: |_| {},
            }
            span { id: "heard", "{heard().join(\",\")}" }
        }
    }
}

fn started() -> Harness {
    let mut harness = Harness::new(Form, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(ms(400));
    harness
}

#[test]
fn choosing_pop3_reports_the_slug() {
    let mut harness = started();
    let at = harness.centre(".ds-popup .ds-button").expect("the pop-up");
    harness.send(Input::click(at));
    harness.advance(ms(200));
    assert_eq!(harness.count(".ds-menu"), 1, "the menu is up");
    let second = harness
        .centre(".ds-menu-item:nth-child(2) .ds-menu-label")
        .expect("POP3");
    harness.send(Input::pointer_move(second));
    harness.send(Input::click(second));
    settle_until(&mut harness, |h| h.count(".ds-menu") == 0);
    assert_eq!(harness.text_of("#heard").as_deref(), Some("pop3"));
    assert!(
        harness
            .text_of(".ds-popup .ds-button")
            .unwrap_or_default()
            .contains("POP3"),
        "the pop-up shows the host's value"
    );
}

#[test]
fn a_label_beside_a_wide_field_keeps_its_words() {
    let harness = started();
    let label = harness
        .rect(".ds-field-row:nth-child(3) .ds-field-row-title")
        .expect("the label");
    assert!(
        label.size.width.0 > 90.0,
        "\"Outgoing security\" is not squeezed: {label:?}"
    );
}
