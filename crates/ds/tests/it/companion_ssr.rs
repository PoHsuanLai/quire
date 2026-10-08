//! The companion's placeholder components as markup: each draws one root with its own `ds-` class,
//! lints clean under the design system's stylesheet, and that class is styled. Their looks are
//! the fill's; this pins that the lint passes and the sheets are registered, and that the
//! unseen-outcome mark comes and goes with what the caller passes.

use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::companion::activity::model::{ActivityEntry, ActivityState, UndoOffer};
use ds::components::companion::activity::view::ActivityStrip;
use ds::components::companion::answer::footer::{
    CardFooter, ServedByView, ServedPlace, ServedPlaceKind,
};
use ds::components::companion::answer::model::{AnswerView, Streaming, TextAnswer};
use ds::components::companion::answer::view::AnswerCard;
use ds::components::companion::chips::view::ContextChips;
use ds::components::companion::memory::model::{ConsolidationDiff, MemoryDay};
use ds::components::companion::memory::view::{ConsolidationView, MemoryTimeline};
use ds::components::companion::orb::model::OrbSize;
use ds::components::companion::orb::view::CompanionOrb;
use ds::components::companion::outcome::model::Outcome;
use ds::components::companion::plan::model::{PlanPhase, PlanView};
use ds::components::companion::plan::view::PlanList;
use ds::components::companion::replace::model::{ReplacePhase, ReplaceProposal};
use ds::components::companion::replace::view::ReplaceBar;
use ds::components::companion::run_row::model::{RunPlace, RunRowView, RunState};
use ds::components::companion::run_row::view::RunRow;
use ds::components::companion::served_by::view::ServedByChip;
use ds::components::companion::{ChipKind, ContextChip, Removal};
use ds::prelude::*;
use ds_core::vocab::{ActorMark, CompanionPresence, Tally};
use ds_lint::{LintConfig, markup};

fn footer() -> CardFooter {
    CardFooter {
        served_by: served_by(),
        sources: Vec::new(),
        scope: Vec::new(),
    }
}

fn served_by() -> ServedByView {
    ServedByView {
        model: "local".to_owned(),
        place: ServedPlace {
            kind: ServedPlaceKind::ThisComputer,
            provider: None,
        },
    }
}

fn app() -> ds::components::companion::mark::AppMark {
    ds::components::companion::mark::AppMark {
        icon: Icon::Search.into(),
        name: "Mail".to_owned(),
    }
}

fn root(body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, stylesheet: Inject::Host,
            {body}
        }
    }
}

/// A specimen: the root class it draws and how it is made.
type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("ds-companion-orb", || {
        root(rsx! { CompanionOrb { presence: CompanionPresence::Working, size: OrbSize::Bar } })
    }),
    ("ds-context-chips", || {
        let chips = vec![ContextChip {
            kind: ChipKind::Results,
            label: "Results".to_owned(),
            count: Some(Tally(12)),
            removal: Removal::Removable,
        }];
        root(rsx! { ContextChips { chips } })
    }),
    ("ds-answer-card", || {
        let view = AnswerView::Text(TextAnswer {
            body: vec!["Done.".into()],
            streaming: Streaming::Complete,
            actions: Vec::new(),
            footer: footer(),
        });
        root(rsx! { AnswerCard { view, on_action: |_| {} } })
    }),
    ("ds-plan-list", || {
        let view = PlanView {
            title: "Forward".to_owned(),
            groups: Vec::new(),
            phase: PlanPhase::Draft,
            footer: footer(),
        };
        root(rsx! { PlanList { view, on_out: |_| {} } })
    }),
    ("ds-replace-bar", || {
        let proposal = ReplaceProposal {
            original: "teh".to_owned(),
            proposed: "the".to_owned(),
            phase: ReplacePhase::Proposed,
            footer: footer(),
        };
        root(rsx! { ReplaceBar { proposal, on_in: |_| {} } })
    }),
    ("ds-run-row", || {
        let view = RunRowView {
            goal: "Book a table".to_owned(),
            app: app(),
            step: Tally(3),
            budget: Some(Tally(40)),
            thought: None,
            place: RunPlace::AgentWorkspace,
            state: RunState::Running,
            served_by: served_by(),
            actions: Vec::new(),
        };
        root(rsx! { RunRow { view, on_action: |_| {} } })
    }),
    ("ds-activity-strip", || {
        let entries = vec![ActivityEntry {
            key: ds::stack::toast_hub::UndoToken(1),
            title: "Archived 3 threads".to_owned(),
            actor: ActorMark::Companion,
            app: Some(app()),
            when: "now".to_owned(),
            state: ActivityState::Done,
            undo: UndoOffer::Undo,
        }];
        root(rsx! { ActivityStrip { entries, on_undo: |_| {}, on_open: |_| {} } })
    }),
    ("ds-memory-timeline", || {
        let days: Vec<MemoryDay> = Vec::new();
        root(rsx! { MemoryTimeline { days, on_verb: |_| {} } })
    }),
    ("ds-consolidation-view", || {
        let diff = ConsolidationDiff {
            night: "Last night".to_owned(),
            lines: Vec::new(),
        };
        root(rsx! { ConsolidationView { diff } })
    }),
    ("ds-served-by-chip", || {
        root(rsx! { ServedByChip { current: served_by(), options: Vec::new(), on_choose: |_| {} } })
    }),
];

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_placeholder_draws_its_root_class_and_it_is_styled() {
    let sheet = ds::stylesheet();
    for (class, make) in SPECIMENS {
        let html = render(*make);
        assert!(
            html.contains(&format!("class=\"{class}\"")),
            "{class}: {html}"
        );
        let needle = format!(".{class}");
        let styled = sheet.match_indices(&needle).any(|(at, _)| {
            !sheet[at + needle.len()..]
                .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        });
        assert!(styled, "{class} is not styled");
    }
}

#[test]
fn every_placeholder_lints_clean() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (class, make) in SPECIMENS {
        for offence in markup(&render(*make), sheet, &LintConfig::new(&ds::kits())) {
            failures.push(format!("{class}: {:?} {}", offence.rule, offence.text));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_orb_names_its_presence_and_size() {
    let html = render(SPECIMENS[0].1);
    assert!(html.contains("data-presence=\"working\""), "{html}");
    assert!(html.contains("data-size=\"bar\""), "{html}");
}

/// What carries the mark under test.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Carrier {
    Orb,
    Row,
}

#[derive(Clone, PartialEq)]
struct Marked {
    carrier: Carrier,
    outcome: Option<Outcome>,
}

#[allow(non_snake_case)]
fn MarkedStage(props: Marked) -> Element {
    let Marked { carrier, outcome } = props;
    match carrier {
        Carrier::Orb => root(rsx! { CompanionOrb { presence: CompanionPresence::Idle, outcome } }),
        Carrier::Row => {
            let view = RunRowView {
                goal: "Book a table".to_owned(),
                app: app(),
                step: Tally(3),
                budget: None,
                thought: None,
                place: RunPlace::InPlace,
                state: RunState::Done,
                served_by: served_by(),
                actions: Vec::new(),
            };
            root(rsx! { RunRow { view, outcome, on_action: |_| {} } })
        }
    }
}

fn render_marked(carrier: Carrier, outcome: Option<Outcome>) -> String {
    let mut dom = VirtualDom::new_with_props(MarkedStage, Marked { carrier, outcome });
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// An unseen outcome draws one still mark on the orb and on a run row, in its own tone word; the
/// caller passing `None` clears it, and the orb's presence is not touched either way.
#[test]
fn an_unseen_outcome_marks_the_orb_and_a_run_row_until_the_caller_clears_it() {
    for carrier in [Carrier::Orb, Carrier::Row] {
        for outcome in [Outcome::Done, Outcome::Failed] {
            let html = render_marked(carrier, Some(outcome));
            let word = format!("data-outcome=\"{}\"", outcome.slug());
            assert_eq!(
                html.matches("ds-outcome-mark").count(),
                1,
                "{carrier:?}: {html}"
            );
            assert!(html.contains(&word), "{carrier:?} {outcome:?}: {html}");
            assert!(
                !html.contains("animation"),
                "{carrier:?}: the mark is still"
            );
        }
        let cleared = render_marked(carrier, None);
        assert!(
            !cleared.contains("ds-outcome-mark"),
            "{carrier:?}: {cleared}"
        );
    }
    let marked = render_marked(Carrier::Orb, Some(Outcome::Failed));
    assert!(marked.contains("data-presence=\"idle\""), "{marked}");
}
