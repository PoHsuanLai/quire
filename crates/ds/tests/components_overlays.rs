//! The overlay components (design/04-COMPONENTS.md sections 18, 20-25, 29-31): every component
//! in every state rendered inside a `Ds` through dioxus-ssr and compared with a golden in
//! `tests/snapshots/overlays/<component>/<state>.html`; the root's toast host and tint alpha;
//! the layer stack a floating component joins; and the goldens' markup against the stylesheet.
//!
//! `DS_BLESS=1 cargo test -p ds --test components_overlays` rewrites the goldens.

#[path = "overlays/cases.rs"]
mod cases;
#[path = "controls/css_scan.rs"]
#[allow(dead_code)] // Only the token scan is used here.
mod css_scan;
#[path = "support/golden.rs"]
mod golden;
#[path = "overlays/mailo.rs"]
mod mailo;
#[path = "overlays/mailo4.rs"]
mod mailo4;
#[path = "overlays/mailo5.rs"]
mod mailo5;

use cases::{CASES, Case};
use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use ds::{
    Alpha, Anchor, Appearance, Availability, Ds, Inject, LayerStack, Material, Menu, MenuEntry,
    MenuKind, Peek, PeekMode, Point, Px, Trail,
};
use std::cell::Cell;
use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

#[derive(Props, Clone)]
struct HostProps {
    make: fn() -> Element,
    tint: Option<Alpha>,
}

/// Never equal: the host renders once, and function addresses are not comparable anyway.
impl PartialEq for HostProps {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

/// A case inside a `Ds` that inlines no stylesheet, with a host tint alpha when given.
fn host(props: HostProps) -> Element {
    rsx! {
        Ds {
            appearance: Appearance::default(),
            material: Material::Popover,
            stylesheet: Inject::Host,
            tint_alpha: props.tint,
            {(props.make)()}
        }
    }
}

struct Unpark(Thread);

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Let the dom's timers run for `span`, rendering whatever they dirty.
fn run_for(dom: &mut VirtualDom, span: Duration) {
    let end = Instant::now() + span;
    let waker = Waker::from(Arc::new(Unpark(thread::current())));
    let mut cx = Context::from_waker(&waker);
    while Instant::now() < end {
        let ready = {
            let mut work = pin!(dom.wait_for_work());
            loop {
                if let Poll::Ready(()) = work.as_mut().poll(&mut cx) {
                    break true;
                }
                let now = Instant::now();
                if now >= end {
                    break false;
                }
                thread::park_timeout(end - now);
            }
        };
        if ready {
            dom.render_immediate(&mut NoOpMutations);
        }
    }
}

/// Build the dom, flush the effects that register overlays, and let `wait` pass.
fn built(make: fn() -> Element, tint: Option<Alpha>, wait: Duration) -> VirtualDom {
    let mut dom = VirtualDom::new_with_props(host, HostProps { make, tint });
    dom.rebuild_in_place();
    for _ in 0..3 {
        dom.render_immediate(&mut NoOpMutations);
    }
    if !wait.is_zero() {
        run_for(&mut dom, wait);
        dom.render_immediate(&mut NoOpMutations);
    }
    dom
}

/// What a case renders inside the root: the markup between `.ds`'s own tags, so a change to
/// the frame variables does not rewrite every golden.
fn inside_root(dom: &VirtualDom) -> String {
    let html = dioxus_ssr::render(dom);
    let open = html.find('>').map_or(0, |at| at + 1);
    let close = html.rfind("</div>").unwrap_or(html.len());
    html[open..close].to_string()
}

fn golden_name(case: &Case) -> String {
    format!("overlays/{}/{}.html", case.component, case.state)
}

#[test]
fn every_overlay_matches_its_golden() {
    let failures: Vec<String> = CASES
        .iter()
        .chain(mailo::MAILO_CASES)
        .chain(mailo4::MAILO4_CASES)
        .chain(mailo5::MAILO5_CASES)
        .filter_map(|case| {
            let dom = built(case.make, None, case.wait);
            golden::check(&golden_name(case), &inside_root(&dom)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn floating_surfaces_render_through_the_overlay_host() {
    // A menu drawn inside a paragraph lands after it, at the end of `.ds`, in the menu layer.
    fn inside() -> Element {
        rsx! {
            p { class: "here",
                Menu {
                    kind: MenuKind::Slim,
                    anchor: Anchor::Point(Point { x: Px(10.0), y: Px(10.0) }),
                    entries: vec![MenuEntry::Item { availability: Availability::Enabled, value: 1u8, title: "One".to_string(), detail: None, tile: None, trail: Trail::None, check: None }],
                    onpick: |_| {},
                    onclose: |_| {},
                }
            }
        }
    }
    let html = inside_root(&built(inside, None, Duration::ZERO));
    let paragraph = html
        .find("<p class=\"here\"></p>")
        .expect("the paragraph is empty");
    let layer = html
        .find("<div class=\"ds-overlay\" data-layer=\"menu\">")
        .expect("the menu layer");
    assert!(paragraph < layer, "{html}");
    assert!(html[layer..].contains("ds-menu"), "{html}");
}

thread_local! {
    static STACK: Cell<Option<Signal<LayerStack>>> = const { Cell::new(None) };
}

#[test]
fn the_newest_layer_takes_the_escape() {
    // A peek, then a menu over it: one Escape names the menu, and with the menu gone the next
    // names the peek (design/06-INTERACTIONS.md section 18).
    #[allow(non_snake_case)]
    fn Layered() -> Element {
        STACK.set(Some(use_context::<Signal<LayerStack>>()));
        rsx! {
            Peek { mode: PeekMode::Center, label: "Thread", onclose: |_| {},
                p { "reader" }
            }
            Menu {
                kind: MenuKind::Slim,
                anchor: Anchor::Point(Point::default()),
                entries: Vec::<MenuEntry<u8>>::new(),
                onpick: |_| {},
                onclose: |_| {},
            }
        }
    }
    fn layered() -> Element {
        rsx! { Layered {} }
    }
    let dom = built(layered, None, Duration::ZERO);
    let stack = STACK.get().expect("the stack was read");
    let (first, second) = dom.in_runtime(|| {
        let now = stack.peek().clone();
        let top = now.top().expect("two layers are open");
        let first = now.escape();
        let second = now.clone().remove(top).escape();
        (first, second)
    });
    assert_eq!(
        first,
        ds::Dismissal::Close(ds::LayerId(stack_top(&dom, stack, 0)))
    );
    assert_eq!(
        second,
        ds::Dismissal::Close(ds::LayerId(stack_top(&dom, stack, 1)))
    );
    assert_ne!(first, second);
}

/// The id of the layer `down` places below the top.
fn stack_top(dom: &VirtualDom, stack: Signal<LayerStack>, down: usize) -> u32 {
    dom.in_runtime(|| {
        let mut now = stack.peek().clone();
        for _ in 0..down {
            let top = now.top().expect("a layer");
            now = now.remove(top);
        }
        now.top().expect("a layer").0
    })
}

#[test]
fn an_empty_hub_lays_out_no_toast() {
    // Gallery fix A: a hidden toast was laid out in every root, and its translateY(160%) did not
    // clear a small root, so an empty pill showed at the bottom of each.
    fn empty() -> Element {
        rsx! { p { "inside" } }
    }
    let html = inside_root(&built(empty, None, Duration::from_millis(80)));
    assert!(html.contains("<p>inside</p>"), "{html}");
    assert!(!html.contains("ds-toast"), "{html}");
}

#[test]
fn the_root_renders_the_toast_host_after_the_overlay_host() {
    #[component]
    fn Pushed() -> Element {
        let toasts = ds::use_toasts();
        use_hook(move || toasts.push("Archived".to_string(), None));
        rsx! {}
    }
    fn pushed() -> Element {
        rsx! { p { "inside" } Pushed {} }
    }
    let html = inside_root(&built(pushed, None, Duration::from_millis(80)));
    let child = html.find("<p>inside</p>").expect("the children");
    let toast = html.find("class=\"ds-toast\"").expect("the toast host");
    assert!(child < toast, "{html}");
    assert!(html.contains("data-shown=\"shown\""), "{html}");
}

#[test]
fn a_toast_mounts_below_the_edge_and_is_dropped_after_it_sinks() {
    #[component]
    fn Pushed() -> Element {
        let toasts = ds::use_toasts();
        use_hook(move || toasts.push("Archived".to_string(), None));
        rsx! {}
    }
    fn pushed() -> Element {
        rsx! { Pushed {} }
    }
    // Its first frame is below the edge, so the spring rises from there.
    let mut dom = built(pushed, None, Duration::ZERO);
    run_for(&mut dom, Duration::from_millis(5));
    dom.render_immediate(&mut NoOpMutations);
    let first = inside_root(&dom);
    assert!(first.contains("data-shown=\"hidden\""), "{first}");
    run_for(&mut dom, Duration::from_millis(80));
    let up = inside_root(&dom);
    assert!(up.contains("data-shown=\"shown\""), "{up}");
    // The hub hides it after its 5200 ms hold (ToastHold): it sinks, still drawn below the
    // edge, then is gone once `--t-big` and a frame have passed (420 + 34 ms at Standard).
    // (The hold, not `hide()`: `ToastHub::stop_hold` writes the hold signal while its own
    // `if let` still borrows it, which panics; reported in FINDINGS "Gallery fixes A".)
    run_for(&mut dom, Duration::from_millis(5200));
    let sinking = inside_root(&dom);
    assert!(sinking.contains("data-shown=\"hidden\""), "{sinking}");
    run_for(&mut dom, Duration::from_millis(520));
    let gone = inside_root(&dom);
    assert!(!gone.contains("ds-toast"), "{gone}");
}

#[test]
fn the_root_writes_the_tint_alpha() {
    fn empty() -> Element {
        rsx! {}
    }
    // The key's default, then a host's own value (thousandths: 640 is .64).
    const CASES: &[(Option<Alpha>, &str)] = &[
        (None, "--m-tint-alpha:.8;"),
        (Some(Alpha(640)), "--m-tint-alpha:.64;"),
        (Some(Alpha(1000)), "--m-tint-alpha:1;"),
    ];
    for (tint, want) in CASES {
        let html = dioxus_ssr::render(&built(empty, *tint, Duration::ZERO));
        let root = &html[..html.find('>').unwrap_or(html.len())];
        assert!(root.contains(want), "{tint:?}: {root}");
        assert_eq!(root.matches("--m-tint-alpha").count(), 1, "{root}");
    }
}

#[test]
fn overlay_stylesheets_use_tokens_only() {
    const SHEETS: &[(&str, &str)] = &[
        (
            "command_palette",
            include_str!("../src/components/command_palette.css"),
        ),
        (
            "hover_card",
            include_str!("../src/components/hover_card.css"),
        ),
        ("link_pill", include_str!("../src/components/link_pill.css")),
        ("menu", include_str!("../src/components/menu.css")),
        (
            "menu_entry",
            include_str!("../src/components/menu_entry.css"),
        ),
        ("peek", include_str!("../src/components/peek.css")),
        ("popover", include_str!("../src/components/popover.css")),
        ("scrim", include_str!("../src/components/scrim.css")),
        (
            "selection_bubble",
            include_str!("../src/components/selection_bubble.css"),
        ),
        ("send_pill", include_str!("../src/components/send_pill.css")),
        ("sheet", include_str!("../src/components/sheet.css")),
        ("toast", include_str!("../src/components/toast.css")),
        ("tooltip", include_str!("../src/components/tooltip.css")),
    ];
    let failures: Vec<String> = SHEETS
        .iter()
        .flat_map(|(name, css)| {
            css_scan::token_violations(css)
                .into_iter()
                .map(move |problem| format!("{name}.css: {problem}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Coherence rule 2 on the overlay goldens: every class is one the stylesheet styles and no
/// element is hand-written markup, except what quire's own components must write inline.
#[cfg(feature = "lint")]
#[test]
fn every_overlay_golden_lints_clean() {
    use ds::lint::{Exception, LintConfig, Rule, markup};
    const EXCEPTIONS: &[Exception] = &[
        Exception {
            rule: Rule::HexColour,
            selector: "span.ds-avatar",
            reason: "the person hue and account colour are computed per face (O-7); the letter is #fff (O-3)",
        },
        Exception {
            rule: Rule::RawMarkup,
            selector: "svg.ds-send-ring",
            reason: "the send ring is two circles Rust redraws per tick, not a glyph (O-20, spike S6)",
        },
    ];
    let config = LintConfig {
        exceptions: EXCEPTIONS,
        ..LintConfig::default()
    };
    let goldens = golden::all_in("overlays");
    assert!(
        goldens.len() >= CASES.len(),
        "only {} goldens",
        goldens.len()
    );
    let failures: Vec<String> = goldens
        .iter()
        .flat_map(|(name, html)| {
            markup(html, ds::stylesheet(), &config)
                .into_iter()
                .map(move |offence| format!("{name}: {:?} {}", offence.rule, offence.text))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// A sender card composed from `HoverCardPart`s draws exactly the markup section 22 writes by
/// hand, so a consumer loses nothing by using the parts.
#[test]
fn the_parts_draw_the_hand_written_card() {
    let read = |state: &str| {
        std::fs::read_to_string(golden::path(&format!("overlays/hover_card/{state}.html")))
            .expect("the golden exists")
    };
    assert_eq!(read("sender-parts"), read("sender-open"));
}
