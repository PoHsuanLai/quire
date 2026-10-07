//! PaneStack on a real Blitz document: a chevron row pushes its detail page into the same pane
//! under a header whose back button names the parent; the back button, Escape, the command chord
//! `[` and Alt+Left pop it; the focus lands on the new page after a push and returns to the row
//! that opened it after a pop; both pages are drawn while the spring moves and only the shown
//! one once it rests, and nothing asks for a frame after that.

use dioxus::prelude::*;
use ds::components::lists::list::model::ListStyle;
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;
use ds::root::common::Common;
use ds_harness::harness::{assert_settles_to_zero_frames, settle_until};
use ds_harness::{Clock, Driver, FocusState, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 480,
    scale_percent: 100,
};

/// The pages: the list of accounts and one account's detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    Accounts,
    Account(&'static str),
}

fn title(page: Page) -> String {
    match page {
        Page::Accounts => "Accounts".to_owned(),
        Page::Account(name) => name.to_owned(),
    }
}

fn opener(page: Page) -> Option<String> {
    match page {
        Page::Accounts => None,
        Page::Account(name) => Some(format!("open-{}", name.to_lowercase())),
    }
}

#[allow(non_snake_case)]
fn StackApp() -> Element {
    stack_app(Motion::default())
}

#[allow(non_snake_case)]
fn ReducedApp() -> Element {
    stack_app(Motion::Reduced)
}

/// The stack over a path the app owns, under `motion`.
fn stack_app(motion: Motion) -> Element {
    let mut path = use_signal(|| PanePath::new(Page::Accounts));
    let open = EventHandler::new(move |name: &'static str| {
        let next = path.peek().pushed(Page::Account(name));
        path.set(next);
    });
    let page = Callback::new(move |page: Page| match page {
        Page::Accounts => rsx! {
            Form {
                FormSection { title: "Accounts",
                    List::<&'static str> {
                        label: "Accounts",
                        style: ListStyle::Grouped,
                        onpick: move |name| open.call(name),
                        items: ["Dana", "Lee"]
                            .into_iter()
                            .map(|name| {
                                ListItem::row(
                                    name,
                                    name,
                                    rsx! {
                                        Row {
                                            title: name,
                                            size: RowSize::Settings,
                                            accessory: Accessory::Chevron,
                                            common: Common { id: Some(format!("open-{}", name.to_lowercase())), ..Common::default() },
                                            onclick: move |_| open.call(name),
                                        }
                                    },
                                )
                            })
                            .collect(),
                    }
                }
            }
        },
        Page::Account(name) => rsx! {
            Form {
                FormSection { title: "Details", footer: "The account's settings.",
                    Row { title: "{name} settings", size: RowSize::Settings }
                }
            }
        },
    });
    rsx! {
        Ds { appearance: Appearance { motion, ..Appearance::default() }, material: Material::Popover,
            div { style: "width:360px",
                PaneStack::<Page> {
                    path: path(),
                    title: Callback::new(title),
                    page,
                    opener: Callback::new(opener),
                    on_back: move |()| {
                        let next = path.peek().popped();
                        path.set(next);
                    },
                    common: Common { id: Some("pane".to_owned()), ..Common::default() },
                }
            }
        }
    }
}

fn harness() -> Harness {
    harness_of(StackApp)
}

fn harness_of(app: fn() -> Element) -> Harness {
    let mut harness = Harness::new(app, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(50));
    harness
}

fn click(harness: &mut Harness, selector: &str) {
    let at = harness
        .centre(selector)
        .unwrap_or_else(|| panic!("{selector} is missing:\n{}", harness.html()));
    harness.send(Input::click(at));
}

/// One page drawn and nothing moving.
fn at_rest(harness: &Harness) -> bool {
    harness.count(".ds-pane-stack-page") == 1
        && harness.attr(".ds-pane-stack", "data-moving").is_none()
}

fn title_shown(harness: &Harness) -> String {
    harness.text_of(".ds-page-title").unwrap_or_default()
}

/// Open Dana and let the push settle.
fn pushed() -> Harness {
    let mut harness = harness();
    click(&mut harness, "#open-dana");
    settle_until(&mut harness, at_rest);
    harness
}

#[test]
fn a_chevron_row_pushes_its_detail_under_a_header_that_names_the_parent() {
    let mut harness = harness();
    assert_eq!(title_shown(&harness), "Accounts");
    assert_eq!(
        harness.count(".ds-page-header button"),
        0,
        "the root has no back button"
    );

    click(&mut harness, "#open-dana");
    harness.advance(Duration::from_millis(30));
    assert_eq!(
        harness.count(".ds-pane-stack-page"),
        2,
        "both pages are drawn while moving"
    );
    assert_eq!(
        harness.attr(".ds-pane-stack", "data-way").as_deref(),
        Some("push")
    );
    assert_eq!(
        harness
            .attr(".ds-pane-stack-page[*|data-pane=detail]", "data-presence")
            .as_deref(),
        Some("entering")
    );
    assert_eq!(
        harness
            .attr(".ds-pane-stack-page[*|data-pane=root]", "data-presence")
            .as_deref(),
        Some("leaving")
    );

    settle_until(&mut harness, at_rest);
    assert_eq!(
        harness.count(".ds-pane-stack-page"),
        1,
        "only the new page once settled"
    );
    assert_eq!(title_shown(&harness), "Dana");
    assert_eq!(
        harness.text_of(".ds-page-header button").as_deref(),
        Some("Accounts")
    );
    assert_eq!(
        harness
            .attr(".ds-page-header button", "aria-label")
            .as_deref(),
        Some("Back to Accounts")
    );
    assert_eq!(harness.focus_of("#pane-back-detail"), FocusState::Focused);
    assert_settles_to_zero_frames(&mut harness);
}

#[test]
fn every_way_back_pops_and_returns_the_focus_to_the_row() {
    type Way = fn(&mut Harness);
    let ways: [(&str, Way); 4] = [
        ("back button", |h| click(h, "#pane-back-detail")),
        ("escape", |h| h.send(Input::key(ShortcutKey::Escape))),
        ("command bracket", |h| {
            h.send(Input::chord(&[ShortcutKey::Super], ShortcutKey::Char('[')))
        }),
        ("alt left", |h| {
            h.send(Input::chord(&[ShortcutKey::Alt], ShortcutKey::Left))
        }),
    ];
    for (name, way) in ways {
        let mut harness = pushed();
        assert_eq!(title_shown(&harness), "Dana", "{name}: precondition");
        way(&mut harness);
        harness.advance(Duration::from_millis(30));
        assert_eq!(
            harness.count(".ds-pane-stack-page"),
            2,
            "{name}: both pages while popping"
        );
        assert_eq!(
            harness.attr(".ds-pane-stack", "data-way").as_deref(),
            Some("pop"),
            "{name}"
        );
        settle_until(&mut harness, at_rest);
        assert_eq!(title_shown(&harness), "Accounts", "{name}");
        assert_eq!(
            harness.focus_of("#open-dana"),
            FocusState::Focused,
            "{name}: focus returns to the opener"
        );
        assert_settles_to_zero_frames(&mut harness);
    }
}

#[test]
fn the_keys_do_nothing_at_the_root() {
    let mut harness = harness();
    click(&mut harness, "#open-lee");
    settle_until(&mut harness, at_rest);
    harness.send(Input::key(ShortcutKey::Escape));
    settle_until(&mut harness, at_rest);
    assert_eq!(title_shown(&harness), "Accounts");
    harness.send(Input::key(ShortcutKey::Escape));
    harness.advance(Duration::from_millis(50));
    assert_eq!(
        title_shown(&harness),
        "Accounts",
        "escape at the root pops nothing"
    );
    assert_eq!(harness.count(".ds-pane-stack-page"), 1);
}

/// `--pane-q`, how far the arrival has come, 30 ms into a push under `app`.
fn arrival_30ms(app: fn() -> Element) -> (f32, Option<String>) {
    let mut harness = harness_of(app);
    click(&mut harness, "#open-dana");
    harness.advance(Duration::from_millis(30));
    let style = harness.attr(".ds-pane-stack", "style").unwrap_or_default();
    let q = style
        .split(';')
        .find_map(|part| part.strip_prefix("--pane-q:"))
        .and_then(|q| q.trim().parse().ok())
        .unwrap_or_else(|| panic!("no --pane-q in {style:?}"));
    (q, harness.attr(".ds", "data-motion"))
}

#[test]
fn reduced_motion_runs_the_same_arrival_under_the_reduced_look() {
    let (standard, level) = arrival_30ms(StackApp);
    assert_ne!(level.as_deref(), Some("reduced"));
    let (reduced, level) = arrival_30ms(ReducedApp);
    assert_eq!(level.as_deref(), Some("reduced"));
    for (name, q) in [("standard", standard), ("reduced", reduced)] {
        assert!(
            0.0 < q && q < 1.0,
            "{name}: the arrival is under way at {q}"
        );
    }
}
