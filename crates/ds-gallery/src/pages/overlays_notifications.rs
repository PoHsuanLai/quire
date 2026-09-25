//! The Overlays page's notifications (sill Q120-Q125): in each scheme, over the Work Space's
//! tint, a `BannerStack` of three banners in a Toast root (one grouped, with its count chip and
//! two layers behind it; one with a link and actions), a card posed as if dragged 56 px to the
//! right, and the notification center, a `Panel` at the right edge whose first group has a
//! `GroupHeader`. Live, the banners dismiss on close or swipe, the button posts another, and the
//! center slides out and in.

use super::Section;
use super::level_tile::work;
use crate::axes::{Axes, Showcase};
use dioxus::prelude::*;
use ds::{
    AppMark, Appearance, Banner, BannerKey, BannerStack, Button, ButtonVariant, CardAction, Ds,
    Expanded, GroupCount, GroupHeader, Icon, IconSource, Inject, Layers, Material,
    NotificationCard, NotificationMetrics, Panel, Px, Rich, RichRun, Run, RunTone, Shown, Swipe,
    Theme,
};

/// The notifications section.
#[component]
pub fn Notifications() -> Element {
    rsx! {
        Section { title: "Notifications", note: "Over the Work Space's tint, light and dark. Left: a BannerStack (newest first; banner-in from the right at --t-move --e-spring, banner-out to the right at --t-move --e-exit, the rows below heal by the height the leaving one measured) of NotificationCards in the Toast material: a group of three with its count chip and two layers 4 px apart behind it, a body with a link (a.ds-run-link keeps its press) and Mini actions that open on hover with the body (two lines to six) and the 18 px close button at the top left. Under it, a card posed as mid-drag, 56 px right; a drag follows 1:1 right and a quarter left, springs back under 80 px and flies out past it or at 600 px/s, and a horizontal scroll does the same once its deltas stop. Right: the notification center, a Panel of the Popover material 8 in from the right edge (panel-in, --t-move --e-out), with a GroupHeader (the icon at 16, the name in the group type, Show less and Clear).",
            div { class: "g-notif-row",
                for theme in [Theme::Light, Theme::Dark] {
                    Scene { theme }
                }
            }
        }
    }
}

/// One scheme's scene.
#[component]
fn Scene(theme: Theme) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion, blur, showcase) = {
        let axes = axes.read();
        (axes.accent, axes.motion, axes.blur, axes.showcase)
    };
    let appearance = Appearance {
        theme,
        accent,
        motion,
    };
    let mut keys = use_signal(|| vec![3u32, 2, 1]);
    let mut next = use_signal(|| 4u32);
    let mut center = use_signal(|| Shown::Visible);
    let banners = keys()
        .into_iter()
        .map(|key| banner(key, keys))
        .collect::<Vec<_>>();
    rsx! {
        div { class: "g-notif",
            Ds { appearance, look: work(theme), material: Material::Window, stylesheet: Inject::Host,
                div { class: "g-notif-ground", style: NotificationMetrics::default().style_attr(),
                    div { class: "g-notif-left",
                        Ds { appearance, look: work(theme), material: Material::Toast, blur, stylesheet: Inject::Host,
                            BannerStack { banners }
                            div { class: "g-notif-swiped",
                                NotificationCard {
                                    app: AppMark { icon: IconSource::Glyph(Icon::Terminal), name: "Terminal".into() },
                                    age: "1m",
                                    summary: "cargo test finished",
                                    body: "All 1,204 tests passed.",
                                    on_close: |_| {},
                                    on_open: |_| {},
                                    swipe: Swipe::Dismiss(EventHandler::new(|()| {})),
                                }
                            }
                        }
                        if showcase == Showcase::Live {
                            div { class: "g-row",
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    label: "Post a notification",
                                    onclick: move |_| {
                                        let key = next();
                                        next.set(key + 1);
                                        keys.with_mut(|keys| keys.insert(0, key));
                                    },
                                }
                                Button {
                                    variant: ButtonVariant::Secondary,
                                    label: "Toggle the center",
                                    onclick: move |_| center.set(match center() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }),
                                }
                            }
                        }
                    }
                    div { class: "g-notif-center",
                        Ds { appearance, look: work(theme), material: Material::Popover, blur, stylesheet: Inject::Host,
                            div { class: "g-notif-stage" }
                            Panel { label: "Notification Center", shown: center(), width: Px(360.0), onclose: move |_| center.set(Shown::Hidden),
                                CenterRows {}
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The center's rows: one app's group under its header, and another app's card.
#[component]
fn CenterRows() -> Element {
    let mut open = use_signal(|| Expanded::Open);
    rsx! {
        GroupHeader {
            icon: IconSource::Glyph(Icon::Mail),
            name: "Mail",
            count: 2,
            expanded: open(),
            on_toggle: move |_| open.set(match open() { Expanded::Open => Expanded::Closed, Expanded::Closed => Expanded::Open }),
            on_clear: |_| {},
        }
        NotificationCard { app: mail(), age: "9:41", summary: "Grace Hopper", body: "Are we still on for Thursday? I booked the room by the harbour.", on_close: |_| {}, on_open: |_| {} }
        if open() == Expanded::Open {
            NotificationCard { app: mail(), age: "9:12", summary: "Ada Lovelace", body: "The notes are longer than the memoir.", on_close: |_| {}, on_open: |_| {} }
        }
        NotificationCard {
            app: AppMark { icon: IconSource::Glyph(Icon::Clock), name: "Calendar".into() },
            age: "8:00",
            summary: "Standup in 10 minutes",
            on_close: |_| {},
            on_open: |_| {},
        }
    }
}

/// The Mail app's mark.
fn mail() -> AppMark {
    AppMark {
        icon: IconSource::Glyph(Icon::Mail),
        name: "Mail".into(),
    }
}

/// Banner `key` of the stack: 3 is a group of three with two layers, 2 carries a link and
/// actions, the rest are plain. Closing or swiping one takes it off `keys`.
fn banner(key: u32, keys: Signal<Vec<u32>>) -> Banner {
    let dismiss = move || {
        let mut keys = keys;
        keys.with_mut(|keys| keys.retain(|held| *held != key));
    };
    let card = match key {
        3 => rsx! {
            NotificationCard {
                app: mail(),
                age: "now",
                summary: "Grace Hopper",
                body: "Three new messages about Thursday's review.",
                count: GroupCount { count: 3, layers: Layers(2) },
                on_close: move |_| dismiss(),
                on_open: |_| {},
                swipe: Swipe::Dismiss(EventHandler::new(move |()| dismiss())),
            }
        },
        2 => rsx! {
            NotificationCard {
                app: AppMark { icon: IconSource::Glyph(Icon::Terminal), name: "CI".into() },
                age: "2m",
                summary: "main is green",
                body: Rich(vec![
                    RichRun::Run(Run::new("Build ", RunTone::Plain)),
                    RichRun::Run(Run::new("#42", RunTone::Strong)),
                    RichRun::Run(Run::new(" passed in ", RunTone::Plain)),
                    RichRun::Run(Run::new("4 min", RunTone::Italic)),
                    RichRun::Run(Run::new(". ", RunTone::Plain)),
                    RichRun::link("Open the run", "https://ci.example/runs/42"),
                ]),
                actions: vec![
                    CardAction { label: "Rerun".into(), on_press: EventHandler::new(|_| {}) },
                    CardAction { label: "Mute".into(), on_press: EventHandler::new(|_| {}) },
                ],
                on_close: move |_| dismiss(),
                on_open: |_| {},
                on_link: |_| {},
                swipe: Swipe::Dismiss(EventHandler::new(move |()| dismiss())),
            }
        },
        n => rsx! {
            NotificationCard {
                app: AppMark { icon: IconSource::Glyph(Icon::Bell), name: "Reminders".into() },
                age: "5m",
                summary: "Reminder {n}",
                body: "Water the plants on the balcony.",
                on_close: move |_| dismiss(),
                on_open: |_| {},
                swipe: Swipe::Dismiss(EventHandler::new(move |()| dismiss())),
            }
        },
    };
    Banner {
        key: BannerKey(key),
        card,
    }
}
