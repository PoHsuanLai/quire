//! The confirmation card as markup: one root with its own `ds-` class, the arm state and the
//! offer written on it, lint clean, and the class styled. Its look is the fill's.

use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::companion::mark::AppMark;
use ds::prelude::*;
use ds_core::vocab::EffectMark;
use ds_lint::{LintConfig, markup};
use ds_shell::confirm::model::{ArmState, ConfirmView, GestureMark, ScopeOffer, TaintLine};
use ds_shell::confirm::view::ConfirmCard;

fn mark(name: &str) -> AppMark {
    AppMark {
        icon: Icon::Search.into(),
        name: name.to_owned(),
    }
}

fn view(arm: ArmState) -> ConfirmView {
    ConfirmView {
        asker: mark("Companion"),
        target: mark("Mail"),
        title: "Send this email?".to_owned(),
        facts: Vec::new(),
        verb: "Send".to_owned(),
        effect: EffectMark::Outbound,
        taint: TaintLine::Clean,
        offer: ScopeOffer::OnceOnly,
        gesture: GestureMark::Press,
        arm,
    }
}

fn card(arm: ArmState) -> String {
    let mut dom = VirtualDom::new_with_props(
        |arm: ArmState| {
            rsx! {
                Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host,
                    ConfirmCard { view: view(arm), on_answer: |_| {} }
                }
            }
        },
        arm,
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn the_card_names_its_arm_state_and_offer() {
    let arming = card(ArmState::Arming);
    assert!(arming.contains("data-arm=\"arming\""), "{arming}");
    assert!(arming.contains("data-offer=\"once-only\""), "{arming}");
    let armed = card(ArmState::Armed);
    assert!(armed.contains("data-arm=\"armed\""), "{armed}");
}

#[test]
fn the_card_lints_clean_and_its_class_is_styled() {
    let sheet = ds_shell::stylesheet();
    let html = card(ArmState::Armed);
    let offences: Vec<String> = markup(&html, sheet, &LintConfig::new(&ds_shell::kits()))
        .into_iter()
        .map(|offence| format!("{:?} {}", offence.rule, offence.text))
        .collect();
    assert!(offences.is_empty(), "{}", offences.join("\n"));
    assert!(html.contains("class=\"ds-confirm-card\""), "{html}");
    assert!(sheet.contains(".ds-confirm-card"));
}
