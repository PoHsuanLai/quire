//! A standalone panel for a host that has no window of its own to put a step in: centred in its
//! overlay root, with the design system's window radius, material and shadow from tokens. It hangs
//! from nothing. A host that has a window (a titled one, with `StepTitle::Host`) puts the bare
//! steps and [`ConsentBody`](super::consent::ConsentBody) in it and needs no `AccountSheet`.

use dioxus::prelude::*;
use ds::components::overlays::sheet::Sheet;
use ds::components::overlays::sheet_attach::Attach;
use ds::components::overlays::sheet_width::SheetWidth;

/// A regular-width panel centred in its root around one step (`ProviderList`, `SignInForm`,
/// `BrowserWait`, `ShowCode`, `ReviewServices`, ...). `on_dismiss` hears Escape and the panel's own
/// close; the steps take Escape themselves, so a host passes the same handler to both.
#[component]
pub fn AccountSheet(label: String, on_dismiss: EventHandler<()>, children: Element) -> Element {
    rsx! {
        Sheet { label, onclose: on_dismiss, attach: Attach::Centre, width: SheetWidth::Regular, {children} }
    }
}
