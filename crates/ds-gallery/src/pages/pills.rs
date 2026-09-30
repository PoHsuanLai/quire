//! The overlays page's second half: hover cards and tooltips, the toast,
//! the link pill and the send pill.

use super::outbox::Outbox;
use super::{Section, Specimen};
use crate::axes::Showcase;
use dioxus::prelude::*;
use ds::{
    Avatar, AvatarSize, AvatarTone, Button, Fraction, Glyph, HoverCard, HoverKey, HoverKind,
    HoverTarget, Icon, IconSize, LinkPill, LinkTarget, SendPhase, SendPill, Shortcut, ShortcutKey,
    TargetElement, Tooltip, UndoToken, sleep, use_hover_hub, use_toast_hub,
};
use ds::{Bezel, ControlSize};
use ds::{KeyEquivalent, KeyStyle};

/// The hover targets, one per card kind.
const TARGETS: [(HoverKind, &str, &str); 4] = [
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
];

/// The kind of card `key`'s target opens: the target table's, or a pinned person's side card.
/// A tooltip's key is not in either, and draws its own surface.
fn card_kind(key: &HoverKey) -> Option<HoverKind> {
    let id = key.0.as_str();
    TARGETS
        .iter()
        .find(|(_, target, _)| *target == id)
        .map(|(kind, _, _)| *kind)
        .or_else(|| {
            PINNED
                .iter()
                .any(|(pinned, _)| *pinned == id)
                .then_some(HoverKind::Side)
        })
}

/// Pinned people drawn as list items that are hover targets themselves.
const PINNED: [(&str, &str); 2] = [("side:4", "Sam Lindqvist"), ("side:5", "Priya Raman")];

/// Hover targets for every card kind, and the tooltip.
#[component]
pub fn Cards() -> Element {
    let hub = use_hover_hub();
    // A hook-keyed card is drawn by its own section (`overlays_mailo4`).
    let open = hub
        .open()
        .or(hub.leaving())
        .filter(|(key, _)| !key.0.starts_with(super::overlays_mailo4::HOOK_KEYED));
    rsx! {
        Section {
            title: "Hover cards and tooltips",
            note: "Rest the pointer on a target: 500 ms to open a card, 150 ms to close, then warm for 400 ms so the next opens at once; a tooltip waits 1 s cold. The pinned people below are li targets (TargetElement::Li).",
            div { class: "g-row",
                for (kind , key , text) in TARGETS {
                    HoverTarget { hover_key: HoverKey(key.to_string()), kind,
                        Button { bezel: Bezel::Inline, label: text, onclick: |_| {} }
                    }
                }
            }
            ul { class: "g-list g-stage-pad",
                for (key , name) in PINNED {
                    HoverTarget { key: "{key}", hover_key: HoverKey(key.to_string()), kind: HoverKind::Side, as_: TargetElement::Li,
                        Button { bezel: Bezel::Inline, label: format!("Pinned: {name} (li target)"), onclick: |_| {} }
                    }
                }
            }
            div { class: "g-row",
                Tooltip { text: "Archive → out of Inbox",
                    Button { size: ControlSize::Mini, label: "Tooltip", onclick: |_| {} }
                }
                Tooltip { text: "Until tomorrow 08:00",
                    Button { size: ControlSize::Mini, label: "Another tooltip", onclick: |_| {} }
                }
            }
            if let Some((key, kind)) = open.and_then(|(key, _)| card_kind(&key).map(|kind| (key, kind))) {
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
                            KeyEquivalent { shortcut: Shortcut(vec![ShortcutKey::Space]) , style: KeyStyle::Cap}
                            " peek"
                        }
                    }
                }
            }
        }
    }
}

/// The toast, the link pill both ways, and the send pill with a live countdown.
#[component]
pub fn Pills(showcase: Showcase) -> Element {
    let toasts = use_toast_hub();
    let mut next = use_signal(|| 10u64);
    rsx! {
        Section { title: "Toast", note: "The toast slides in from the right edge, holds for 5 s (the pointer over it pauses the hold) and slides out; swipe it to the right to dismiss it, or press Undo.",
            div { class: "g-row",
                Button {
                    label: "Push a toast with undo",
                    onclick: move |_| {
                        toasts.push("Archived “Lunch on Thursday?”".to_string(), Some(UndoToken(next())));
                        next += 1;
                    },
                }
                Button { label: "Push one without", onclick: move |_| toasts.push("Saved".to_string(), None) }
                span { class: "g-code", "last undo: {toasts.last_undo().map_or(\"none\".to_string(), |token| token.0.to_string())}" }
            }
        }
        Section { title: "LinkPill and SendPill", note: "Each sits at its surface's edge; here each surface is a stage. The outbox states: Cancel for a held send, the spinning ring while it waits, a failed mood (nudge, shake, fatal) played once on demand, and a refusal on a second line.",
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
                Outbox {}
            }
        }
    }
}

/// Undo send: five one-second ticks, then Sent.
#[component]
fn Countdown(showcase: Showcase) -> Element {
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
            let ticks = ds::SEND_COUNTDOWN.as_millis() / ds::SEND_TICK.as_millis().max(1);
            let ticks = u16::try_from(ticks).unwrap_or(5).max(1);
            for tick in 1..=ticks {
                sleep(ds::SEND_TICK).await;
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
            Button { size: ControlSize::Mini, label: "Send", onclick: start }
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
