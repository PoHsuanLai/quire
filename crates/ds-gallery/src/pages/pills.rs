//! The overlays page's second half: hover cards and tooltips, the selection bubble, the toast,
//! the link pill and the send pill.

use super::{Section, Specimen};
use crate::axes::Showcase;
use dioxus::prelude::*;
use ds::{
    Avatar, AvatarSize, AvatarTone, BubbleAction, BubbleButton, BubbleMode, Button, ButtonVariant,
    DelayToken, Fraction, Glyph, HoverCard, HoverKey, HoverKind, HoverTarget, Icon, IconSize, Kbd,
    Key, LinkPill, LinkTarget, MountedRef, Rect, SelectionBubble, SendPhase, SendPill, Shortcut,
    Switch, TargetElement, Tooltip, TooltipKind, UndoToken, sleep, use_env, use_hover_hub,
    use_toast_hub,
};

/// The hover targets, one per card kind.
const TARGETS: [(HoverKind, &str, &str); 5] = [
    (
        HoverKind::Thread,
        "thread:88",
        "Re: UIDL stability (thread card)",
    ),
    (HoverKind::Sender, "sender:3", "Dana Okafor (sender card)"),
    (
        HoverKind::Account,
        "account:1",
        "Work account (account card)",
    ),
    (HoverKind::Side, "side:2", "Pinned: Mei Chen (side card)"),
    (HoverKind::Tip, "time:88", "09:41 (time tip)"),
];

/// Pinned people drawn as list items that are hover targets themselves (mailo gaps 2).
const PINNED: [(&str, &str); 2] = [("side:4", "Sam Lindqvist"), ("side:5", "Priya Raman")];

/// Hover targets for every card kind, and both tooltip kinds.
#[component]
pub fn Cards() -> Element {
    let hub = use_hover_hub();
    let open = hub.open().or(hub.leaving());
    rsx! {
        Section {
            title: "Hover cards and tooltips",
            note: "Rest the pointer on a target: 450 ms to open, 150 ms to close, then warm for 400 ms so the next opens at once. The time tip is HoverKind::Tip; the pinned people below are li targets (TargetElement::Li).",
            div { class: "g-row",
                for (kind , key , text) in TARGETS {
                    HoverTarget { hover_key: HoverKey(key.to_string()), kind,
                        Button { variant: ButtonVariant::Quiet, label: text, onclick: |_| {} }
                    }
                }
            }
            ul { class: "g-list g-stage-pad",
                for (key , name) in PINNED {
                    HoverTarget { key: "{key}", hover_key: HoverKey(key.to_string()), kind: HoverKind::Side, as_: TargetElement::Li,
                        Button { variant: ButtonVariant::Quiet, label: format!("Pinned: {name} (li target)"), onclick: |_| {} }
                    }
                }
            }
            div { class: "g-row",
                Tooltip { kind: TooltipKind::Fly, text: "Archive → out of Inbox",
                    Button { variant: ButtonVariant::Mini, label: "Fly tooltip", onclick: |_| {} }
                }
                Tooltip { kind: TooltipKind::Card, text: "Snooze", sub: Some("Until tomorrow 08:00".to_string()),
                    Button { variant: ButtonVariant::Mini, label: "Card tooltip", onclick: |_| {} }
                }
            }
            if let Some((key, HoverKind::Tip)) = open.clone() {
                HoverCard { key: "{key.0}", kind: HoverKind::Tip, "Wed 23 Sep 2026, 09:41" }
            } else if let Some((key, kind)) = open {
                HoverCard { key: "{key.0}", kind,
                    div { class: "ds-hovercard-person",
                        Avatar { initial: 'D', size: AvatarSize::Size34, tone: AvatarTone::Ink }
                        div {
                            h5 { class: "ds-hovercard-title", "Dana Okafor" }
                            div { class: "ds-hovercard-sub", "{key.0}" }
                        }
                    }
                    div { class: "ds-hovercard-stats",
                        span { b { "14" } "threads" }
                        span { b { "Mon" } "last wrote" }
                    }
                    div { class: "ds-hovercard-flag", "data-tone": "danger",
                        Glyph { icon: Icon::OctagonAlert, size: IconSize::Compact }
                        span { "Not the address Dana usually writes from." }
                    }
                    div { class: "ds-hovercard-foot",
                        "stays unread while you look"
                        span { class: "ds-hovercard-keys",
                            Kbd { shortcut: Shortcut(vec![Key::Space]) }
                            " peek"
                        }
                    }
                }
            }
        }
    }
}

/// A sentence to select, and the bubble over it in either mode.
#[component]
pub fn Bubble() -> Element {
    let mut text = use_signal(|| None::<MountedRef>);
    let mut shown = use_signal(|| None::<(Rect, Switch)>);
    let mut bold = use_signal(|| Switch::Off);
    let open = move |link: Switch| {
        if let Some(MountedRef(element)) = text() {
            spawn(async move {
                if let Ok(rect) = element.get_client_rect().await {
                    shown.set(Some((to_rect(rect), link)));
                }
            });
        }
    };
    let actions = vec![
        BubbleAction::Button(BubbleButton {
            label: rsx! { b { "B" } },
            title: "Bold".to_string(),
            pressed: Some(bold()),
            onclick: EventHandler::new(move |()| {
                bold.set(match bold() {
                    Switch::On => Switch::Off,
                    Switch::Off => Switch::On,
                })
            }),
        }),
        BubbleAction::Separator,
        BubbleAction::Button(BubbleButton {
            label: rsx! { Glyph { icon: Icon::Link, size: IconSize::Compact } },
            title: "Link".to_string(),
            pressed: None,
            onclick: EventHandler::new(move |()| open(Switch::On)),
        }),
    ];
    rsx! {
        Section { title: "SelectionBubble", note: "The composer supplies the selection's rect; the bubble sits 8 px above it and never flips.",
            div { class: "g-row",
                span {
                    class: "g-name",
                    onmounted: move |event| text.set(Some(MountedRef(event.data()))),
                    "Pretend this sentence is selected."
                }
                Button { variant: ButtonVariant::Mini, label: "Actions", onclick: move |_| open(Switch::Off) }
                Button { variant: ButtonVariant::Mini, label: "Link field", onclick: move |_| open(Switch::On) }
            }
            if let Some((anchor, link)) = shown() {
                SelectionBubble {
                    key: "{link:?}",
                    anchor,
                    mode: match link { Switch::On => BubbleMode::Link, Switch::Off => BubbleMode::Actions(actions) },
                    onlink: |_| {},
                    onclose: move |_| shown.set(None),
                }
            }
        }
    }
}

fn to_rect(rect: dioxus::html::geometry::PixelsRect) -> Rect {
    Rect {
        origin: ds::Point {
            x: ds::Px(rect.origin.x as f32),
            y: ds::Px(rect.origin.y as f32),
        },
        size: ds::Size {
            width: ds::Px(rect.size.width as f32),
            height: ds::Px(rect.size.height as f32),
        },
    }
}

/// The toast, the link pill both ways, and the send pill with a live countdown.
#[component]
pub fn Pills(showcase: Showcase) -> Element {
    let toasts = use_toast_hub();
    let mut next = use_signal(|| 10u64);
    rsx! {
        Section { title: "Toast", note: "The undo toast springs up from the card's bottom edge; drag its tab right past 46 px, or tap it, to undo.",
            div { class: "g-row",
                Button {
                    variant: ButtonVariant::Secondary,
                    label: "Push a toast with undo",
                    onclick: move |_| {
                        toasts.push("Archived “Lunch on Thursday?”".to_string(), Some(UndoToken(next())));
                        next += 1;
                    },
                }
                Button { variant: ButtonVariant::Secondary, label: "Push one without", onclick: move |_| toasts.push("Saved".to_string(), None) }
                span { class: "g-code", "last undo: {toasts.last_undo().map_or(\"none\".to_string(), |token| token.0.to_string())}" }
            }
        }
        Section { title: "LinkPill and SendPill", note: "Each sits at its surface's edge; here each surface is a stage.",
            div { class: "g-grid3",
                Specimen { name: "honest link",
                    div { class: "g-stage",
                        LinkPill { target: LinkTarget::Honest { scheme_sub: "https://docs.".to_string(), registered: "example.org".to_string(), path: "/guides/imap/uidl".to_string() } }
                    }
                }
                Specimen { name: "lying link",
                    div { class: "g-stage",
                        LinkPill { target: LinkTarget::Lying { registered: "examp1e-login.net".to_string(), shown: "example.org".to_string() } }
                    }
                }
                Specimen { name: "send, live",
                    div { class: "g-stage", Countdown { showcase } }
                }
            }
        }
    }
}

/// Undo send: five one-second ticks, then Sent.
#[component]
fn Countdown(showcase: Showcase) -> Element {
    let level = use_env().resolved.motion;
    let mut elapsed = use_signal(|| match showcase {
        Showcase::Posed => Fraction(400),
        Showcase::Live => Fraction(0),
    });
    let mut phase = use_signal(|| SendPhase::Counting);
    let mut run = use_signal(|| 0u32);
    let start = move |_| {
        elapsed.set(Fraction(0));
        phase.set(SendPhase::Counting);
        run += 1;
        let this = run();
        spawn(async move {
            let ticks = DelayToken::SendCountdown.delay(level).as_millis()
                / DelayToken::SendTick.delay(level).as_millis().max(1);
            let ticks = u16::try_from(ticks).unwrap_or(5).max(1);
            for tick in 1..=ticks {
                sleep(DelayToken::SendTick.delay(level)).await;
                if run() != this || phase() == SendPhase::Done {
                    return;
                }
                elapsed.set(Fraction(1000 * tick / ticks));
            }
            phase.set(SendPhase::Done);
        });
    };
    rsx! {
        div { class: "g-stage-pad",
            Button { variant: ButtonVariant::Mini, label: "Send", onclick: start }
        }
        if run() > 0 || showcase == Showcase::Posed {
            SendPill {
                key: "{run}",
                text: match phase() { SendPhase::Counting => "Sending…", SendPhase::Done => "Sent" },
                progress: elapsed(),
                phase: phase(),
                onundo: move |_| {
                    run += 1;
                    phase.set(SendPhase::Done);
                },
            }
        }
    }
}
