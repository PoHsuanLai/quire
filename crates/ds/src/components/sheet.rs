//! Sheet: the general modal panel for settings and dialogs, derived from Peek
//! (design/04-COMPONENTS.md section 24): Peek Center's surface, over a scrim, sized by its
//! content up to Peek Center's inset (O-14's proposal).
//!
//! Sheet and modal parts (sill Q90, Q91): a host that maps and unmaps its surface passes
//! `shown` and `on_hidden`, and the sheet plays its exit before it is gone; `placement` centres
//! it in its root, as a shutdown dialog is.

use crate::components::popover::{Dismiss, Stacking, escape_closes, use_float};
use crate::components::scrim::{ScrimLook, scrim_button_as};
pub use crate::components::sheet_placement::SheetPlacement;
use crate::components::sheet_presence::{SheetShowing, Step, use_sheet_showing};
use crate::components::tooltip::Shown;
use crate::tokens::ZLayer;
use dioxus::prelude::*;

/// A modal panel.
///
/// `shown` hands its showing to the host (`None`, the default: shown for as long as it is
/// mounted, as before). Hidden, it plays `sheet-out` (`--t-move --e-exit`), its scrim fades,
/// it leaves the layer stack at once, and `on_hidden` runs at `settle(SheetOut)`, not before, so
/// the host can unmap its surface then. Shown again while leaving, it enters again and
/// `on_hidden` does not run. Mounted hidden, it draws nothing until shown.
///
/// `placement` says where it stands: [`SheetPlacement::Top`] (the default) 36 px from the top,
/// [`SheetPlacement::Centre`] centred in its root both ways. A centred sheet needs a root that
/// has a height: an overlay root passes `Ds { extent: RootExtent::Viewport }`.
#[component]
pub fn Sheet(
    label: String,
    onclose: EventHandler<()>,
    #[props(default)] shown: Option<Shown>,
    #[props(default)] on_hidden: Option<EventHandler<()>>,
    #[props(default)] placement: SheetPlacement,
    children: Element,
) -> Element {
    let float = use_float(ZLayer::Peek, Stacking::Layer(Dismiss::EscOnly));
    let showing = use_sheet_showing(shown, on_hidden);
    // Mounted hidden: off the stack (after `use_float`'s own push) until first shown.
    use_hook(move || {
        if shown == Some(Shown::Hidden) {
            dioxus::core::queue_effect(move || float.withdraw());
        }
    });
    follow_stack(showing, move || float.withdraw(), move || float.rejoin());
    if !showing.drawn() {
        float.show(rsx! {}, onclose);
        return rsx! {};
    }
    let close = format!("Close {label}");
    let look = ScrimLook {
        presence: showing.leaving().then_some("leaving"),
    };
    let panel = rsx! {
        div {
            class: "ds-sheet",
            "data-presence": showing.slug(),
            "data-placement": placement.attribute(),
            role: "dialog",
            "aria-label": "{label}",
            onkeydown: move |event| escape_closes(float, &event, onclose),
            {children}
        }
    };
    float.show(
        rsx! {
            {scrim_button_as(&close, look, move || float.is_top(), onclose)}
            {placement.stage(panel)}
        },
        onclose,
    );
    rsx! {}
}

/// Leave the layer stack as the sheet starts to leave, so Escape and its scrim reach whatever
/// is under it; join it again, on top, as it is shown again.
fn follow_stack(
    showing: SheetShowing,
    withdraw: impl FnOnce() + 'static,
    rejoin: impl FnOnce() + 'static,
) {
    match showing.step {
        Step::Stay => {}
        Step::Leave => dioxus::core::queue_effect(withdraw),
        Step::Enter => dioxus::core::queue_effect(rejoin),
    }
}
