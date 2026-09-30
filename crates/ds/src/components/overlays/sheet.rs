//! Sheet: the panel a window or the shell puts up to ask for something (design/30 section 2.5,
//! `NSWindow` sheet): it slides down from the top, dims nothing (macOS has no scrim; a host that
//! wants the click outside caught draws its own catcher) and takes Escape.
//!
//! [`Attach::Window`] hangs it from the top edge of its root; [`Attach::Centre`] centres it, as
//! the shell's app-modal dialog stands. A host that maps and unmaps its surface passes `shown` and
//! `on_hidden`, and the sheet plays its exit before it is gone.

use crate::components::overlays::popover::{Stacking, escape_closes, use_float};
use crate::components::overlays::sheet_attach::Attach;
use crate::components::overlays::sheet_width::SheetWidth;
use crate::root::common::Common;
use dioxus::core::queue_effect;
use dioxus::prelude::*;
use ds_core::vocab::{Dismiss, Shown};
use ds_core::word::Word;
use ds_motion::anim::Anim;
use ds_motion::presence::{
    Exit, Presence,
    spec::PresenceSpec,
    use_presence::{Presented, use_presence},
};
use ds_style::tokens::layer::ZLayer;

/// A sheet.
///
/// `shown` hands its showing to the host (`None`, the default: shown for as long as it is
/// mounted). Mounted shown, it slides in from the top over `--t-big`. Hidden, it slides back up
/// over `--t-move`, leaves the layer stack at once, and `on_hidden` runs once the exit has
/// settled, so the host can unmap its surface then. Shown again while leaving, it is present at
/// once and `on_hidden` does not run. Mounted hidden, it draws nothing until shown.
///
/// `attach` says where it hangs: [`Attach::Window`] (the default) from the top edge,
/// [`Attach::Centre`] centred in its root both ways, or [`Attach::Bottom`] standing above the
/// bottom edge (the widget gallery). A centred or bottom sheet needs a root that has a height: an
/// overlay root passes `Ds { extent: RootExtent::Viewport }`.
///
/// `width` is [`SheetWidth::Regular`] (560) by default, [`SheetWidth::Narrow`] (340) for an alert
/// or a password prompt, [`SheetWidth::Wide`] (1040) for a gallery.
///
/// `common` puts the consumer's `id` (for a host that resolves its blur or input region by id),
/// `data-*` and classes on the panel.
#[component]
pub fn Sheet(
    label: String,
    onclose: EventHandler<()>,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
    #[props(default)] attach: Attach,
    #[props(default)] width: SheetWidth,
    #[props(default)] common: Common,
    children: Element,
) -> Element {
    let float = use_float(ZLayer::Peek, Stacking::Layer(Dismiss::Semitransient));
    let Presented { presence, alias } = use_presence(
        shown.unwrap_or(Shown::Visible),
        PresenceSpec {
            enter: Anim::SheetIn,
            exit: Exit::SheetOut,
        },
        on_hidden,
    );
    // On the stack while up; off it from the moment the exit starts (and while mounted hidden),
    // so Escape reaches whatever is under it.
    let up = matches!(presence, Presence::Entering | Presence::Present);
    let mut joined = use_hook(|| CopyValue::new(true));
    if up != *joined.peek() {
        joined.set(up);
        queue_effect(move || {
            if up {
                float.rejoin();
            } else {
                float.withdraw();
            }
        });
    }
    let Some(drawn) = presence.drawn_slug() else {
        float.show(rsx! {}, onclose);
        return rsx! {};
    };
    let class = common.class("ds-sheet");
    let data = common.data_attributes();
    let panel = rsx! {
        div {
            id: common.id.clone(),
            class,
            "data-presence": drawn,
            "data-pulse": alias.slug(),
            "data-attach": attach.slug(),
            "data-width": width.attribute(),
            "data-overscroll": "band",
            role: "dialog",
            "aria-label": "{label}",
            onmounted: move |event| common.mounted(event),
            onkeydown: move |event| escape_closes(float, &event, onclose),
            ..data,
            div { class: "ds-sheet-body", {children} }
        }
    };
    float.show(attach.stage(panel), onclose);
    rsx! {}
}
