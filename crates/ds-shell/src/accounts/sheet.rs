//! The panel the add-account steps stand in.

use dioxus::prelude::*;
use ds::components::overlays::sheet::Sheet;
use ds::components::overlays::sheet_attach::Attach;
use ds::components::overlays::sheet_width::SheetWidth;

/// The sheet around one step (`ProviderList`, `SignInForm`, `BrowserWait`, `ShowCode`,
/// `ReviewServices`): a regular-width `Sheet` hanging from the top of its root. `on_dismiss` hears
/// Escape and the sheet's own close; the steps take Escape themselves, so a host passes the same
/// handler to both.
#[component]
pub fn AccountSheet(
    label: String,
    on_dismiss: EventHandler<()>,
    #[props(default)] attach: Attach,
    children: Element,
) -> Element {
    rsx! {
        Sheet { label, onclose: on_dismiss, attach, width: SheetWidth::Regular, {children} }
    }
}
