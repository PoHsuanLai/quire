//! Fade: a wrapper that lets its content go away gently. While `shown` is `Visible` the content is
//! drawn as it is; when `shown` turns `Hidden` it fades out over `--t-quick` (the popover's exit)
//! and is then unmounted, and `on_hidden` runs once it has settled. Finder's download badge is the
//! use: the ring or pie stays mounted through the fade when a download completes, where an `if`
//! would drop it at once. Under Reduced motion it leaves on the reduced settle, as every presence does.
//!
//! Built on `use_presence`, like every surface that is shown and hidden; the content appears at
//! once (no entrance), and a show while it fades takes the hide back. While it fades it takes no
//! press and is hidden from assistive technology.
//!
//! Markup: `span.ds-fade[data-presence]` around the content.

use crate::root::common::Common;
use dioxus::prelude::*;
use ds_core::vocab::Shown;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit, Presence,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};

/// `children`, drawn while `shown` (`Visible` by default) and for one fade after it turns
/// `Hidden`. `on_hidden` runs once the fade has settled and the content is gone.
#[component]
pub fn Fade(
    #[props(default = Shown::Visible)] shown: Shown,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
    #[props(default)] children: Element,
    #[props(default)] common: Common,
) -> Element {
    let Presented { presence, .. } = use_presence(
        shown,
        PresenceSpec {
            enter: Anim::PaletteFade,
            exit: Exit::Fade,
        },
        on_hidden,
    );
    let Some(drawn) = presence.drawn_slug() else {
        return rsx! {};
    };
    let leaving = matches!(presence, Presence::Leaving(_)).then_some("true");
    let class = common.class("ds-fade");
    let data = common.data_attributes();
    rsx! {
        span {
            id: common.id.clone(),
            class,
            "data-presence": drawn,
            "aria-hidden": leaving,
            onmounted: move |event| common.mounted(event),
            ..data,
            {children}
        }
    }
}
