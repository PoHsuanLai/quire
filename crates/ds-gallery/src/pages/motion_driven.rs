//! Driven motion (design/05 section 14): each part that runs on a spring, in its own
//! cell with the buttons that move it. Each settles to 0 frames; a second press mid-flight
//! redirects from where it is.

use super::Section;
use super::details::{Cell, mini};
use crate::axes::Axes;
use dioxus::prelude::*;
use ds::Choice;
use ds::Tracking;
use ds::detail::{Contact, Touch};
use ds::motion::{DragReturn, Release, VelocityMeter, use_drag_return};
use ds::{
    Appearance, Attach, Check, DragReturnFrame, Ds, Fraction, Icon, IconSource, Inject, Material,
    Pane, PaneSwitcher, PlateFamily, Point, Px, RootChrome, SegmentedControl, Sheet, Shown,
    SidePanel, Slider, Toggle,
};
use ds_shell::{AppKey, AppMark, AppSwitcher, NotificationCard, NotificationSwipe, SwitcherApp};

/// The section on the Motion page.
#[component]
pub fn DrivenSection() -> Element {
    rsx! {
        Section { title: "Driven motion", note: "Springs in Rust on the frame clock (design/05 section 14): Spring {{ damping, response }}, 1.0 for a tap, a key or a remote change, 0.8 for a throw toward the target, --spring-quick 300 ms and --spring-move 450 ms. Press again mid-flight: it turns from where it is at the speed it has. Under Reduced every spring is critically damped and the sheet, panel and panes only cross-fade.",
            div { class: "g-detail-grid",
                ToggleCell {}
                SegmentCell {}
                SliderCell {}
                SwitcherCell {}
                PaneCell {}
                SwipeCell {}
                SheetCell {}
                PanelCell {}
                DockCell {}
            }
        }
    }
}

#[component]
fn ToggleCell() -> Element {
    let mut on = use_signal(|| Check::On);
    rsx! {
        Cell { name: "Toggle knob", code: "use_spring(--knob-x), Quick",
            controls: rsx! { {mini("Flip (remote)", move |_| on.set(on().flipped()))} },
            div { class: "g-detail",
                Toggle { label: "Wi-Fi", value: on(), onchange: move |to| on.set(to) }
            }
        }
    }
}

#[component]
fn SegmentCell() -> Element {
    let mut pick = use_signal(|| 0u8);
    let options = vec![
        (0u8, "Day".to_owned()),
        (1, "Week".to_owned()),
        (2, "Month".to_owned()),
    ];
    rsx! {
        Cell { name: "Segmented thumb", code: "use_spring(--seg-dx), Quick",
            controls: rsx! {
                {mini("Day", move |_| pick.set(0))}
                {mini("Month", move |_| pick.set(2))}
            },
            div { class: "g-detail",
                SegmentedControl { label: "View", choices: Choice::pairs(options), tracking: Tracking::SelectOne(pick()), onchange: move |to| pick.set(to) }
            }
        }
    }
}

#[component]
fn SliderCell() -> Element {
    let mut level = use_signal(|| Fraction(300));
    rsx! {
        Cell { name: "Slider release", code: "Throw::landing, use_spring_motion(--f)",
            controls: rsx! {
                {mini("30 %", move |_| level.set(Fraction(300)))}
                {mini("90 %", move |_| level.set(Fraction(900)))}
            },
            div { class: "g-detail",
                Slider { label: "Volume", value: level(), onchange: move |to| level.set(to) }
            }
        }
    }
}

/// Three apps for the switcher cell.
const APPS: [(&str, &str, Icon, PlateFamily); 3] = [
    ("mail", "Mail", Icon::Mail, PlateFamily::Blue),
    ("clock", "Clock", Icon::Clock, PlateFamily::Neutral),
    ("keyboard", "Keyboard", Icon::Keyboard, PlateFamily::Violet),
];

#[component]
fn SwitcherCell() -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion) = {
        let axes = axes.read();
        (axes.theme, axes.accent, axes.motion)
    };
    let mut at = use_signal(|| 0usize);
    let apps: Vec<SwitcherApp> = APPS
        .iter()
        .map(|(key, name, icon, family)| SwitcherApp {
            plate: Some(*family),
            ..SwitcherApp::new(*key, *name, IconSource::Glyph(*icon))
        })
        .collect();
    let chosen = AppKey(APPS[at() % APPS.len()].0.to_owned());
    rsx! {
        Cell { name: "Switcher ring", code: "use_spring(--switcher-at), Quick",
            controls: rsx! { {mini("Next", move |_| at.set(at() + 1))} },
            div { class: "g-detail g-driven-switcher",
                Ds {
                    appearance: Appearance { theme, accent, motion },
                    material: Material::Osd,
                    stylesheet: Inject::Host,
                    chrome: Some(RootChrome::Transparent),
                    AppSwitcher { apps, selected: chosen, output: Some(Px(360.0)), onhover: |_| {}, onactivate: |_| {} }
                }
            }
        }
    }
}

#[component]
fn PaneCell() -> Element {
    let mut shown = use_signal(|| Pane::Root);
    rsx! {
        Cell { name: "Pane switch", code: "use_spring(--pane-p), Move",
            controls: rsx! {
                {mini("Detail", move |_| shown.set(Pane::Detail))}
                {mini("Root", move |_| shown.set(Pane::Root))}
            },
            div { class: "g-detail g-detail-list",
                PaneSwitcher {
                    shown: shown(),
                    root: rsx! { p { class: "g-note", "Wi-Fi · Bluetooth · Focus" } },
                    detail: rsx! { p { class: "g-note", "Home · Office · Café" } },
                }
            }
        }
    }
}

#[component]
fn SwipeCell() -> Element {
    let mut round = use_signal(|| 0u32);
    rsx! {
        Cell { name: "Swipe return", code: "use_spring_motion(--swipe-dx), Move",
            controls: rsx! { {mini("Replay", move |_| *round.write() += 1)} },
            div { class: "g-detail g-detail-list",
                for n in [round()] {
                    NotificationCard {
                        key: "{n}",
                        app: AppMark { icon: IconSource::Glyph(Icon::Mail), name: "Mail".into() },
                        age: "now",
                        summary: "Drag me right, let go short",
                        body: "Under 80 px it springs home at your release speed.",
                        swipe: NotificationSwipe::Dismiss(EventHandler::new(|()| {})),
                        on_close: |_| {},
                        on_open: |_| {},
                    }
                }
            }
        }
    }
}

/// A stage for an overlay specimen: its own root with a height, in the page's axes.
#[component]
fn Stage(material: Material, children: Element) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (theme, accent, motion) = {
        let axes = axes.read();
        (axes.theme, axes.accent, axes.motion)
    };
    rsx! {
        div { class: "g-driven-stage",
            Ds { appearance: Appearance { theme, accent, motion }, material, stylesheet: Inject::Host,
                div { class: "g-driven-stage-fill" }
                {children}
            }
        }
    }
}

#[component]
fn SheetCell() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    let flip = move |_| {
        shown.set(match shown() {
            Shown::Visible => Shown::Hidden,
            Shown::Hidden => Shown::Visible,
        })
    };
    rsx! {
        Cell { name: "Sheet", code: "sheet-in / sheet-out",
            controls: rsx! { {mini("Show / hide", flip)} },
            Stage { material: Material::Sheet,
                Sheet {
                    label: "Rename",
                    onclose: move |_| shown.set(Shown::Hidden),
                    shown: Some(shown()),
                    attach: Attach::Centre,
                    div { class: "g-panel", p { class: "g-note", "Hide it and show it again mid-way." } }
                }
            }
        }
    }
}

#[component]
fn PanelCell() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    let flip = move |_| {
        shown.set(match shown() {
            Shown::Visible => Shown::Hidden,
            Shown::Hidden => Shown::Visible,
        })
    };
    rsx! {
        Cell { name: "Side panel", code: "panel-in / panel-out",
            controls: rsx! { {mini("Show / hide", flip)} },
            Stage { material: Material::Window,
                SidePanel { label: "Notification Center", shown: shown(), width: Px(160.0),
                    p { class: "g-note", "Nothing new." }
                }
            }
        }
    }
}

#[component]
fn DockCell() -> Element {
    let drag = use_drag_return();
    let mut from = use_signal(|| None::<Point>);
    let mut meters = use_signal(|| (VelocityMeter::default(), VelocityMeter::default()));
    rsx! {
        Cell { name: "Dock drag return", code: "DragReturn, DragReturnFrame",
            controls: rsx! { {mini("Nudge", move |_| nudge(drag))} },
            div {
                class: "g-detail",
                onpointerdown: move |event| {
                    let at = event.client_coordinates();
                    let at = Point { x: Px(at.x as f32), y: Px(at.y as f32) };
                    from.set(Some(at));
                    let now = ds::time::now();
                    meters.set((VelocityMeter::default().moved(at.x, now), VelocityMeter::default().moved(at.y, now)));
                },
                onpointermove: move |event| {
                    let Some(start) = from() else { return };
                    let at = event.client_coordinates();
                    let at = Point { x: Px(at.x as f32), y: Px(at.y as f32) };
                    let now = ds::time::now();
                    let (mx, my) = meters();
                    meters.set((mx.moved(at.x, now), my.moved(at.y, now)));
                    drag.follow(Point { x: Px(at.x.0 - start.x.0), y: Px(at.y.0 - start.y.0) });
                },
                onpointerup: move |event| {
                    if from.take().is_some() {
                        let now = ds::time::now();
                        let (mx, my) = meters();
                        let release = Release { x: mx.released(now), y: my.released(now) };
                        drag.home(Touch::Contact(Contact::from_event(&event)), release);
                    }
                },
                DragReturnFrame { drag,
                    div { class: "g-driven-tile", "Drag" }
                }
            }
        }
    }
}

/// Knock the tile off its place and let it spring home, as a drop that missed does.
fn nudge(drag: DragReturn) {
    drag.follow(Point {
        x: Px(-40.0),
        y: Px(-24.0),
    });
    spawn(async move {
        ds::sleep(std::time::Duration::from_millis(120)).await;
        drag.home(Touch::Remote, Release::default());
    });
}
