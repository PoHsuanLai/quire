//! Overlays and feedback (design/30 sections 2.5 and 2.9): every component of the step 4a.6
//! group in every state it can express. `Popover` under each dismiss policy and with its arrow,
//! `Sheet` hung from the window, centred and standing at the bottom, `Alert` (the alerts
//! section), `SidePanel`, `Tooltip` up and down, `HoverCard`, `Toast`,
//! `EmptyState` in its three forms, `Skeleton` in its three shapes and as a row, the `InlineBanner`
//! shown and hidden, and `Loadable` cycling its phases.

use crate::axes::{Axes, Showcase};
use crate::pages::{Section, Specimen};
use crate::wallpaper;
use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::content::text_runs::RunTone;
use ds::components::controls::button_model::Answers;
use ds::components::overlays::empty_state::EmptyForm;
use ds::components::overlays::inline_banner::InlineBanner;
use ds::components::overlays::popover::Arrow;
use ds::components::overlays::sheet_attach::Attach;
use ds::components::overlays::sheet_width::SheetWidth;
use ds::components::overlays::skeleton::{Skeleton, SkeletonShape};
use ds::host::measure::{Anchor, MountedRef};
use ds::prelude::*;
use ds::root::common::Common;
use ds::stack::toast_hub::{UndoToken, use_toast_hub};
use ds::style::tokens::control_size::ControlSize;
use ds_core::geometry::placement::{Align, Side};
use ds_core::vocab::Dismiss;

/// The page.
#[component]
pub fn OverlaysCataloguePage() -> Element {
    rsx! {
        Section { title: "Popover", note: "Popover {{ anchor, placement, gap, arrow, dismiss }}: a floating surface anchored to a control. Dismiss::Transient closes on Escape and an outside click, Semitransient on Escape only, Manual by its owner alone. It fades in and out over --t-quick. Arrow::Arrow points the 34 x 8 arrow at the anchor, as an app popover does; a shell popover hung from the bar has none.",
            div { class: "g-row g-row-top",
                PopoverCase { dismiss: Dismiss::Transient, arrow: Arrow::Arrow, label: "Transient, arrow" }
                PopoverCase { dismiss: Dismiss::Semitransient, arrow: Arrow::None, label: "Semitransient" }
                PopoverCase { dismiss: Dismiss::Manual, arrow: Arrow::None, label: "Manual" }
            }
        }
        Section { title: "Sheet", note: "Sheet {{ attach, width }} dims nothing. Attach::Window hangs a card 12 below the top edge of its root and fades in with a scale from .97 over --t-quick; Attach::Centre stands in the middle (an alert's narrow column); Attach::Bottom stands 8 above the bottom edge, wide, never taller than half its root (Edit Widgets). Hidden, it fades and scales out over --t-quick.",
            div { class: "g-row g-row-top",
                SheetCase { attach: Attach::Window, width: SheetWidth::Regular, name: "Attach::Window, Regular" }
                SheetCase { attach: Attach::Centre, width: SheetWidth::Narrow, name: "Attach::Centre, Narrow" }
            }
            SheetCase { attach: Attach::Bottom, width: SheetWidth::Wide, name: "Attach::Bottom, Wide" }
        }
        crate::pages::overlays::sheet_within::SheetWithinSection {}
        crate::pages::overlays::alert::Alerts {}
        Section { title: "SidePanel", note: "SidePanel {{ shown, width, header }} at the right edge of its root: it slides in over --t-move --e-out and out over --t-quick --e-exit, in the Popover material, with an optional header row above its body. The notification center is one (see Notifications).",
            div { class: "g-row g-row-top",
                SideCase {}
            }
        }
        Section { title: "Tooltip", note: "Tooltip {{ text, shown }}: one line below its target, opened by the Tip profile (1 s cold, at once while warm) and gone when the pointer leaves. Under a caller's shown it is up or down on that say alone.",
            div { class: "g-row g-row-top",
                Specimen { name: "Tooltip, hover it".to_string(),
                    Tooltip { text: "Archive → out of Inbox",
                        Button { size: ControlSize::Mini, label: "Archive", onclick: |_| {} }
                    }
                }
                Specimen { name: "Tooltip, shown".to_string(), code: Some("shown: Some(Shown::Visible)".to_string()),
                    div { class: "g-stage-pad",
                        Tooltip { text: "Snooze until tomorrow", shown: Some(Shown::Visible),
                            Button { size: ControlSize::Mini, label: "Snooze", onclick: |_| {} }
                        }
                    }
                }
                Specimen { name: "Tooltip, hidden".to_string(), code: Some("shown: Some(Shown::Hidden)".to_string()),
                    Tooltip { text: "Never shown", shown: Some(Shown::Hidden),
                        Button { size: ControlSize::Mini, label: "Kept down", onclick: |_| {} }
                    }
                }
            }
        }
        crate::pages::overlays::pills::Cards {}
        ToastCase {}
        Section { title: "EmptyState", note: "EmptyState {{ form, title, description, icon, action, onretry }}: what a list or pane says when there is nothing to show. Empty (nothing yet, with an action), NoResults (a search found nothing) and Failure (with Retry). Static: nothing moves.",
            div { class: "g-row g-row-top",
                Specimen { name: "EmptyForm::Empty, with an action".to_string(),
                    div { class: "g-stage g-stage-tall",
                        EmptyState {
                            title: "No messages",
                            description: "Mail you receive lands here.",
                            action: Some(rsx! { Button { answers: Answers::Return, label: "Compose", onclick: |_| {} } }),
                        }
                    }
                }
                Specimen { name: "EmptyForm::NoResults".to_string(),
                    div { class: "g-stage g-stage-tall",
                        EmptyState {
                            form: EmptyForm::NoResults,
                            title: "No results for “uidl”",
                            description: "Check the spelling or try another search.",
                        }
                    }
                }
                Specimen { name: "description with a code run".to_string(),
                    div { class: "g-stage g-stage-tall",
                        EmptyState {
                            title: "No accounts",
                            description: TextLine::Runs(vec![
                                TextRun::new("Run ", RunTone::Plain),
                                TextRun::new("mailo add-account", RunTone::Code),
                                TextRun::new(" to connect one.", RunTone::Plain),
                            ]),
                        }
                    }
                }
                Specimen { name: "EmptyForm::Failure, with Retry".to_string(),
                    div { class: "g-stage g-stage-tall",
                        EmptyState {
                            form: EmptyForm::Failure,
                            icon: Some(Icon::OctagonAlert),
                            title: "Couldn’t load your mail",
                            description: "The server didn’t answer.",
                            onretry: |_| {},
                        }
                    }
                }
            }
        }
        Section { title: "InlineBanner", note: "InlineBanner {{ severity, text, detail, icon, actions, onclose }}: a message in a pane's own flow, above the content it is about. Info is the quiet fill; Ok, Warn and Danger take their wash and mark; the words keep the ink. Danger is role=alert.",
            div { class: "g-row g-row-top",
                Specimen { name: "Info, with an action and close".to_string(),
                    div { style: "width:440px",
                        InlineBanner {
                            text: "Remote images are blocked.",
                            detail: "Loading them tells the sender you opened this.",
                            actions: rsx! { Button { label: "Load images", size: ControlSize::Small, onclick: |_| {} } },
                            onclose: |_| {},
                        }
                    }
                }
                Specimen { name: "Ok".to_string(),
                    div { style: "width:440px",
                        InlineBanner { severity: Severity::Ok, text: "You accepted this invitation." }
                    }
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "Warn, with an action".to_string(),
                    div { style: "width:440px",
                        InlineBanner {
                            severity: Severity::Warn,
                            text: "The sender asked for a read receipt.",
                            actions: rsx! { Button { label: "Send receipt", size: ControlSize::Small, onclick: |_| {} } },
                        }
                    }
                }
                Specimen { name: "Danger".to_string(),
                    div { style: "width:440px",
                        InlineBanner { severity: Severity::Danger, text: "This message may not be from who it says." }
                    }
                }
            }
        }
        SkeletonCase {}
        BannerShownCase {}
        LoadableCase {}
    }
}

/// A banner switched in and out with `shown`: it appears and disappears in place, as Mail's
/// remote-content bar does, and the text under it moves with it at once.
#[component]
fn BannerShownCase() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    rsx! {
        Section { title: "InlineBanner, shown and hidden", note: "InlineBanner {{ shown }}: static, as Mail's remote-content bar is. Visible (the default) draws it in the flow; Hidden draws nothing, and what is under it takes its place at once. There is no motion either way.",
            div { class: "g-row",
                Button {
                    label: "Show / hide",
                    onclick: move |_| shown.set(match shown() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }),
                }
            }
            Specimen { name: "shown".to_string(), code: Some("shown: Shown".to_string()),
                div { style: "width:440px",
                    InlineBanner {
                        severity: Severity::Info,
                        text: "Remote images are blocked.",
                        detail: "Loading them tells the sender you opened this.",
                        shown: shown(),
                    }
                    p { class: "g-note", style: "padding:8px 12px", "The message starts here and moves up when the banner goes." }
                }
            }
        }
    }
}

/// `Loadable` stepping Loading, Ready, Failed and back (Retry returns to Loading), with a
/// `SkeletonRow` placeholder beside the default spinner.
#[component]
fn LoadableCase() -> Element {
    let mut phase = use_signal(|| Phase::Loading(Operation::Running(PendingToken::start())));
    let next = move |_| {
        let next = match phase() {
            Phase::Loading(_) => Phase::Ready,
            Phase::Ready => Phase::Failed {
                title: "Could not load the folder".to_owned(),
                description: Some("The server did not answer.".into()),
            },
            Phase::Failed { .. } => Phase::Loading(Operation::Running(PendingToken::start())),
        };
        phase.set(next);
    };
    rsx! {
        Section { title: "Loadable and SkeletonRow", note: "Loadable {{ phase, placeholder, onretry }}: the placeholder while Loading (a centred spinner that turns only while the Operation runs, or your own), the children when Ready, a Failure EmptyState with Retry (and your own `action` beside it) when Failed. A change of phase cross-fades the arriving layer in over --t-quick (the section 1.3 primitive; it snaps under Reduced), and a skeleton placeholder fades out over it before it is dropped. SkeletonRow {{ lines }} is a static avatar and one or two bars at a settings row's size (--row-settings-h, --row-avatar).",
            div { class: "g-row",
                Button { label: "Next phase", onclick: next }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "Default placeholder".to_string(), code: Some("phase".to_string()),
                    div { style: "width:320px;height:200px;display:flex",
                        Loadable {
                            phase: phase(),
                            onretry: move |()| phase.set(Phase::Loading(Operation::Running(PendingToken::start()))),
                            action: Some(rsx! { Button { label: "Connection Doctor\u{2026}", onclick: |_| {} } }),
                            p { class: "g-note", style: "padding:12px", "The folder's contents." }
                        }
                    }
                }
                Specimen { name: "SkeletonRow placeholder".to_string(), code: Some("placeholder: Some(..)".to_string()),
                    div { style: "width:320px;height:200px;display:flex",
                        Loadable {
                            phase: phase(),
                            placeholder: Some(rsx! {
                                SkeletonRow {}
                                SkeletonRow { lines: SkeletonLines::One }
                                SkeletonRow {}
                            }),
                            onretry: move |()| phase.set(Phase::Loading(Operation::Running(PendingToken::start()))),
                            p { class: "g-note", style: "padding:12px", "The folder's contents." }
                        }
                    }
                }
            }
        }
    }
}

/// The appearance the page's axes ask for, in `theme`.
pub(super) fn appearance(theme: Theme) -> Appearance {
    let axes = use_context::<Signal<Axes>>();
    let axes = axes.read();
    Appearance {
        theme,
        accent: axes.accent,
        motion: axes.motion,
    }
}

/// A button and the popover it opens, open in a posed snapshot.
#[component]
fn PopoverCase(dismiss: Dismiss, arrow: Arrow, label: &'static str) -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let mut open = use_signal(|| showcase == Showcase::Posed);
    let mut anchor = use_signal(|| None::<MountedRef>);
    rsx! {
        Specimen { name: label.to_string(), code: Some(format!("dismiss: {dismiss:?}, arrow: {arrow:?}")),
            div { class: "g-stage-pad", style: "min-width:220px;min-height:140px",
                Button {
                    common: Common {
                        mounted: Some(EventHandler::new(move |event: MountedEvent| anchor.set(Some(MountedRef(event.data())))),),
                        ..Common::default()
                    },

                    label: "Open the popover",
                    onclick: move |_| open.set(true),
                }
            }
            if let (true, Some(mounted)) = (open(), anchor()) {
                Popover {
                    key: "{dismiss:?}",
                    anchor: Anchor::Mounted(mounted),
                    placement: Placement::new(Side::Bottom, Align::Center),
                    gap: Px(4.0),
                    arrow,
                    dismiss,
                    onclose: move |()| open.set(false),
                    div { class: "g-panel",
                        p { class: "g-note", "Anything at all." }
                        Button { size: ControlSize::Mini, label: "Close", onclick: move |_| open.set(false) }
                    }
                }
            }
        }
    }
}

/// A sheet in a root of its own, over the wallpaper.
#[component]
fn SheetCase(attach: Attach, width: SheetWidth, name: &'static str) -> Element {
    let mut open = use_signal(|| true);
    let appearance = appearance(Theme::Light);
    let tall = if attach == Attach::Bottom {
        "g-cat-wide"
    } else {
        ""
    };
    rsx! {
        Specimen { name: name.to_string(), code: Some(format!("attach: {attach:?}, width: {width:?}")),
            div { class: "g-wall g-modal {tall}", style: "background-image:url(\"{wallpaper::uri()}\")",
                Ds { appearance, material: Material::Sheet, stylesheet: Inject::Host,
                    div { class: "g-modal-stage",
                        div { class: "g-stage-pad",
                            Button {  label: "Show the sheet", onclick: move |_| open.set(true) }
                        }
                    }
                    Sheet {
                        label: "Sheet",
                        onclose: move |()| open.set(false),
                        shown: Some(if open() { Shown::Visible } else { Shown::Hidden }),
                        attach,
                        width,
                        div { class: "g-panel",
                            h3 { "A sheet" }
                            p { class: "g-note", "Dims nothing; Escape closes it." }
                            Button { answers: Answers::Return, label: "Done", onclick: move |_| open.set(false) }
                        }
                    }
                }
            }
        }
    }
}

/// A side panel with a header, and the button that shows and hides it.
#[component]
fn SideCase() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    let appearance = appearance(Theme::Light);
    rsx! {
        Specimen { name: "SidePanel with a header".to_string(), code: Some("shown, header".to_string()),
            div { class: "g-wall g-modal", style: "background-image:url(\"{wallpaper::uri()}\")",
                Ds { appearance, material: Material::Popover, stylesheet: Inject::Host,
                    div { class: "g-modal-stage",
                        div { class: "g-stage-pad",
                            Button {

                                label: "Show / hide",
                                onclick: move |_| shown.set(match shown() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }),
                            }
                        }
                    }
                    SidePanel {
                        label: "Notification Center",
                        shown: shown(),
                        width: Px(240.0),
                        header: Some(rsx! { strong { "Notification Center" } }),
                        p { class: "g-note", "Nothing new." }
                    }
                }
            }
        }
    }
}

/// The toast: pushed on demand, and once in a posed snapshot.
#[component]
fn ToastCase() -> Element {
    let showcase = use_context::<Signal<Axes>>().peek().showcase;
    let toasts = use_toast_hub();
    let mut next = use_signal(|| 100u64);
    use_hook(move || {
        if showcase == Showcase::Posed {
            toasts.push(
                "Archived “Lunch on Thursday?”".to_string(),
                Some(UndoToken(99)),
            );
        }
    });
    rsx! {
        Section { title: "Toast", note: "The toast slides in from the right edge over --t-move, holds for 5 s (the pointer over it pauses the hold), and slides out over --t-quick; swipe it to the right to dismiss it, or press Undo. One at a time.",
            div { class: "g-row",
                Button {

                    label: "Push a toast with undo",
                    onclick: move |_| {
                        toasts.push("Archived “Lunch on Thursday?”".to_string(), Some(UndoToken(next())));
                        next += 1;
                    },
                }
                Button {  label: "Push one without", onclick: move |_| toasts.push("Saved".to_string(), None) }
            }
        }
    }
}

/// The skeleton's three shapes, and a composed placeholder row, shown and cross-faded away.
#[component]
fn SkeletonCase() -> Element {
    let mut shown = use_signal(|| Shown::Visible);
    rsx! {
        Section { title: "Skeleton", note: "Skeleton {{ shape, width, height, shown }}: a static grey placeholder (no shimmer) that fades in over --t-quick and, when its content arrives, fades out. Line, Block and Circle; here composed into a row of an avatar and two lines.",
            div { class: "g-row",
                Button {

                    label: "Load / unload",
                    onclick: move |_| shown.set(match shown() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }),
                }
            }
            div { class: "g-row g-row-top",
                Specimen { name: "SkeletonShape::Line".to_string(),
                    div { class: "g-stage-pad", style: "width:200px",
                        Skeleton { shape: SkeletonShape::Line, shown: shown() }
                    }
                }
                Specimen { name: "SkeletonShape::Block".to_string(),
                    div { class: "g-stage-pad",
                        Skeleton { shape: SkeletonShape::Block, width: Some(Px(160.0)), height: Some(Px(80.0)), shown: shown() }
                    }
                }
                Specimen { name: "SkeletonShape::Circle".to_string(),
                    div { class: "g-stage-pad",
                        Skeleton { shape: SkeletonShape::Circle, width: Some(Px(40.0)), shown: shown() }
                    }
                }
                Specimen { name: "A row, composed".to_string(),
                    div { class: "g-stage-pad", style: "display:flex;gap:12px;align-items:center;width:280px",
                        Skeleton { shape: SkeletonShape::Circle, width: Some(Px(32.0)), shown: shown() }
                        div { style: "display:flex;flex-direction:column;gap:6px;flex:1",
                            Skeleton { shape: SkeletonShape::Line, shown: shown() }
                            Skeleton { shape: SkeletonShape::Line, width: Some(Px(120.0)), shown: shown() }
                        }
                    }
                }
            }
        }
    }
}
