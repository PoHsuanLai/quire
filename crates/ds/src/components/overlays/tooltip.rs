//! Tooltip: one line that names what a control does or what a value means (design/30 section
//! 2.5, `NSToolTip`).
//!
//! The tip waits by the Tip profile through the hover hub (1 s cold, at once while the hub is
//! warm, gone the moment the pointer leaves), stands below its target and fades in and out over
//! `--t-quick`. A caller that runs its own machine passes `shown`: the tip then shows or hides
//! on that say alone, at once, with no hover and no delay of its own.
//!
//! [`Hint`] is the one implementation: a `Tooltip` is a `Hint` below its target on the Tip
//! profile, and the shell's `DockLabel` is a `Hint` above its target on the Label profile.

use crate::components::overlays::hover_card::{
    Standing,
    intent::{HoverAnchor, use_hover_intent},
    target::HoverTarget,
    use_card,
};
use crate::host::measure::MountedRef;
use crate::root::common::Common;
use crate::stack::hover_hub::{HoverKey, use_hover_hub};
use dioxus::core::current_scope_id;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
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
/// `common` goes on the tip's surface (`role="tooltip"`); its `aria_label` names it in place of
/// its text.
#[component]
pub fn Tooltip(
    text: String,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    rsx! {
        Hint {
            text,
            shown,
            profile: HoverProfile::Tip,
            side: HintSide::Below,
            root: "ds-tooltip",
            common,
            {children}
        }
    }
}

/// The hover key a hint files its target under: its own, so two hints never share a card.
fn own_key() -> HoverKey {
    HoverKey(format!("hint:{}", current_scope_id().0))
}

/// A hint on `children`: the shared implementation of a tooltip and a dock label. `profile` is
/// the wait it opens by, `side` where it stands, `root` the class of its surface.
#[component]
pub fn Hint(
    text: String,
    #[props(default)] shown: Option<Shown>,
    profile: HoverProfile,
    #[props(default)] side: HintSide,
    root: &'static str,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let key = use_hook(own_key);
    let hub = use_hover_hub();
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
            match shown {
                None => rsx! {
                    HoverTarget { hover_key: key.clone(), profile, {children} }
                },
                Some(_) => rsx! {
                    Anchored { hover_key: key.clone(), {children} }
                },
            }
        }
        if mine {
            HintSurface { hover_key: key, text, root, side, own: shown.is_some(), common }
        }
    }
}

/// The wrapper of a caller-driven hint: it files its own rect as the anchor once mounted, since
/// no pointer will come over it to do that.
#[component]
fn Anchored(hover_key: HoverKey, children: Element) -> Element {
    let driver = use_hover_intent();
    rsx! {
        span {
            class: "ds-hover-target",
            "data-hover-key": "{hover_key.0}",
            "data-hint": "true",
            onmounted: move |event| {
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
    common: Common,
) -> Element {
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
