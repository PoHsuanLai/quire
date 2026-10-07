//! The app-facing component sheets (the confirm card, the helper sheet, the accounts), each placed
//! in the `components` section right after the sheet it followed when the components listed them
//! (`ds_style::kit::Sheet`), so the cascade is the one the stylesheet always had.

use ds_style::kit::Sheet;

/// The sheets, in cascade order.
pub(crate) const SHEETS: [Sheet; 3] = [
    Sheet {
        name: "confirm_card",
        css: include_str!("confirm/style.css"),
        after: "served_by_chip",
    },
    Sheet {
        name: "helper_sheet",
        css: include_str!("helpers/style.css"),
        after: "alert",
    },
    Sheet {
        name: "accounts",
        css: include_str!("accounts/style.css"),
        after: "alert",
    },
];
