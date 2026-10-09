//! Tooltip: one line that names what a control does or what a value means (design/30 section
//! 2.5, `NSToolTip`).
//!
//! The tip waits by the Tip profile through the hover hub (at once by default, 1 s cold under `TipDelay::Standard`, at once while the hub is
//! warm, gone the moment the pointer leaves), stands below its target and fades in and out over
//! `--t-quick`. A caller that runs its own machine passes `shown`: the tip then shows or hides
//! on that say alone, at once, with no hover and no delay of its own.
//!
//! A caller that already has the pointer (a thread row's time, where the row's own hooks run)
//! keys the tip on them instead: `hover_key` names the tip, the caller feeds
//! [`HoverDriver::over`](crate::components::overlays::hover_card::intent::HoverDriver::over)
//! with `HoverProfile::Tip` and the same key from its own pointer events (and `out` when it
//! leaves), and the tip wraps nothing: `Tooltip { hover_key, text }` draws it while the hub has
//! that key open, below the anchor the caller filed.
//!
//! [`Hint`] is the one implementation: a `Tooltip` is a `Hint` below its target on the Tip
//! profile, and the shell's `DockLabel` is a `Hint` above its target on the Label profile.

use crate::components::content::tip_text::TipText;
use crate::components::content::title_tip::{Hinted, own_key};
use crate::components::overlays::hover_card::intent::{HoverAnchor, use_hover_intent};
use crate::components::overlays::hover_card::target::HoverTarget;
use crate::components::overlays::hover_card::{Standing, use_card};
use crate::host::measure::{MountedRef, client_rect};
use crate::root::common::Common;
use crate::stack::hover_hub::{HoverKey, use_hover_hub};
use dioxus::prelude::*;
use ds_core::time::{FRAME_SLACK, clock::sleep};
use ds_core::vocab::{Shortcut, Shown};
use ds_core::word::Word;
use ds_motion::hover_intent::HoverProfile;

/// Which side of its target a hint stands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum HintSide {
    /// Below, as a tooltip does.
    #[default]
    Below,
    /// Above, as a dock label does.
    Above,
}

impl HintSide {
    fn standing(self) -> Standing {
        match self {
            HintSide::Below => Standing::Tip,
            HintSide::Above => Standing::Label,
        }
    }
}

/// A tooltip on `children`. `shown` hands it to the caller: `None` follows the pointer (the Tip
/// profile through the hover hub), `Some` shows or hides it at once.
///
/// `hover_key` hands the pointer to the caller: the tip is keyed by it, wraps no children, and
/// stands while the caller's own hooks hold that key open (see above). `shown` wins when both
/// are given.
///
/// `text` is the tip's name and `shortcut` its key, drawn through [`TipText`] as `Name  ⌘K`
/// (the house style, design/30 section 2.5).
///
/// `common` goes on the tip's surface (`role="tooltip"`); its `aria_label` names it in place of
/// its text.
#[component]
pub fn Tooltip(
    text: String,
    #[props(default)] shortcut: Option<Shortcut>,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] hover_key: Option<HoverKey>,
    #[props(default)] common: Common,
    #[props(default)] children: Element,
) -> Element {
    let text = TipText::new(text).with_shortcut_opt(shortcut).render();
    rsx! {
        Hint {
            text,
            shown,
            hover_key,
            profile: HoverProfile::Tip,
            side: HintSide::Below,
            root: "ds-tooltip",
            common,
            {children}
        }
    }
}

/// A hint on `children`: the shared implementation of a tooltip and a dock label. `profile` is
/// the wait it opens by, `side` where it stands, `root` the class of its surface.
#[component]
pub fn Hint(
    text: String,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] hover_key: Option<HoverKey>,
    profile: HoverProfile,
    #[props(default)] side: HintSide,
    root: &'static str,
    #[props(default)] common: Common,
    #[props(default)] children: Element,
) -> Element {
    let own = use_hook(own_key);
    use_context_provider(|| Hinted);
    let hooked = hover_key.is_some();
    let key = hover_key.unwrap_or(own);
    let hub = use_hover_hub();
    let element = use_signal(|| None::<MountedRef>);
    let mine = match shown {
        Some(Shown::Visible) => true,
        Some(Shown::Hidden) => false,
        None => hub
            .open()
            .or(hub.leaving())
            .is_some_and(|(open, _)| open == key),
    };
    rsx! {
        {
            match (shown, hooked) {
                (None, false) => rsx! {
                    HoverTarget { hover_key: key.clone(), profile, {children} }
                },
                (None, true) => rsx! {
                    {children}
                },
                (Some(_), _) => rsx! {
                    Anchored { hover_key: key.clone(), element, {children} }
                },
            }
        }
        if mine {
            HintSurface { hover_key: key, text, root, side, own: shown.is_some(), element, common }
        }
    }
}

/// The wrapper of a caller-driven hint: it files its own rect as the anchor once mounted, since
/// no pointer will come over it to do that.
#[component]
fn Anchored(
    hover_key: HoverKey,
    element: Signal<Option<MountedRef>>,
    children: Element,
) -> Element {
    let driver = use_hover_intent();
    rsx! {
        span {
            class: "ds-hover-target",
            "data-hover-key": "{hover_key.0}",
            "data-hint": "true",
            onmounted: move |event| {
                let mut element = element;
                element.set(Some(MountedRef(event.data())));
                driver.anchor(hover_key.clone(), HoverAnchor::Element(MountedRef(event.data())));
            },
            {children}
        }
    }
}

/// The hint's surface, mounted while it is up.
#[component]
fn HintSurface(
    hover_key: HoverKey,
    text: String,
    root: &'static str,
    side: HintSide,
    own: bool,
    element: Signal<Option<MountedRef>>,
    common: Common,
) -> Element {
    // A caller-driven hint follows its target for as long as it is up: layout can move the target
    // after the hint was placed, and the hint is placed again against the latest rect. The loop
    // belongs to this surface and ends with it.
    let driver = use_hover_intent();
    let followed = hover_key.clone();
    use_future(move || {
        let followed = followed.clone();
        async move {
            if !own {
                return;
            }
            loop {
                sleep(FRAME_SLACK).await;
                let Some(target) = element.peek().clone() else {
                    continue;
                };
                if let Some(rect) = client_rect(&target.0).await
                    && driver.anchor_of(&followed) != Some(rect)
                {
                    driver.anchor(followed.clone(), HoverAnchor::Rect(rect));
                }
            }
        }
    });
    let (float, style, presence) = use_card(side.standing(), own.then_some(&hover_key));
    let probe = float.surface();
    let class = common.class(&format!("ds-popover {root}"));
    let data = common.data_attributes();
    float.show(
        rsx! {
            div {
                id: common.id.clone(),
                class,
                "aria-label": common.aria_label.clone(),
                "data-elevation": "pop",
                "data-layer": "card",
                "data-presence": presence,
                role: "tooltip",
                style,
                onmounted: move |event| {
                    common.mounted(event.clone());
                    probe.on_mounted(event);
                },
                ..data,
                "{text}"
            }
        },
        EventHandler::new(|()| {}),
    );
    rsx! {}
}
