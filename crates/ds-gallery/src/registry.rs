//! Every page: its title, what it shows, how tall a snapshot of it is, and the component that
//! draws it. One entry per [`Page`] variant (tested).

use crate::page::Page;
use crate::pages;
use dioxus::prelude::*;

/// One page of the gallery.
#[derive(Debug, Clone, Copy)]
pub struct Entry {
    /// Which page.
    pub page: Page,
    /// Its tab label.
    pub title: &'static str,
    /// One sentence under the title: what a reviewer checks here.
    pub lede: &'static str,
    /// How tall its snapshot is, in logical pixels.
    pub height: u32,
    /// The page's body.
    pub body: fn() -> Element,
}

/// The pages, in the gallery's order.
pub const REGISTRY: [Entry; 24] = [
    Entry {
        page: Page::Tokens,
        title: "Tokens",
        lede: "Every colour token with its hex in both schemes, the accents, the label hues, the spacing scale, radii, shadows and z layers.",
        height: 2300,
        body: crate::pages::foundations::tokens::TokensPage,
    },
    Entry {
        page: Page::Type,
        title: "Type",
        lede: "The five face jobs in the toolbar's typeface and the size ramp: every --fs step drawn at its size, with its role.",
        height: 2500,
        body: crate::pages::foundations::type_ramp::TypePage,
    },
    Entry {
        page: Page::Controls,
        title: "Controls",
        lede: "Every control in every state it can express: variants, pressed, expanded, disabled, empty and filled. Press Tab to see the keyboard focus ring.",
        height: 3450,
        body: crate::pages::controls::overview::ControlsPage,
    },
    Entry {
        page: Page::Catalogue,
        title: "Catalogue",
        lede: "Every control and field of the catalogue in every state it can express, at each size: label, button, toggle, checkbox, radio group, segmented control, slider, text field, progress and level indicators, badge, key equivalent. Press Tab to see the focus ring, Return or Space to press.",
        height: 9000,
        body: pages::catalogue::CataloguePage,
    },
    Entry {
        page: Page::Lists,
        title: "Lists",
        lede: "A live List: add rows, remove them with each exit and watch the rows below heal, then undo. Row with every accessory, leading element, height and state; section headers and disclosures; a source list at each sidebar size with an outline and drop places; search hits with a keyboard-shown strip, tiles, the hover strip and the appearance picker.",
        height: 4800,
        body: crate::pages::lists::overview::ListsPage,
    },
    Entry {
        page: Page::Forms,
        title: "Forms",
        lede: "A Form of FormSections as a System Settings pane draws it: a grouped list of panes led by icon tiles and a person, a chevron on every row that opens something, field rows with a toggle, the section footer, and the icon tile in five colours.",
        height: 1560,
        body: crate::pages::forms::FormsPage,
    },
    Entry {
        page: Page::Menus,
        title: "Menus",
        lede: "Open each menu placement, the submenus and the status lines; every item state with its highlight posed; the pop-up button at each size. A pick blinks its item twice.",
        height: 4100,
        body: crate::pages::menus::overview::MenusPage,
    },
    Entry {
        page: Page::Structure,
        title: "Structure",
        lede: "Stepper with its field and bare, a sortable resizable Table, Toolbar with its overflow chevron, SplitView dragged and folded, Sidebar at each size, TabView, FieldRow and FieldGroup, the MenuBar model's menus, the drag image with its count badge and the window titlebar with its subtitle, proxy icon and edited dot.",
        height: 5900,
        body: pages::structure::StructurePage,
    },
    Entry {
        page: Page::SettingsWindow,
        title: "Settings window",
        lede: "A System-Settings-like window from the catalogue's own parts: a titlebar over a SplitView of a Sidebar and the content, the content's Toolbar over FieldGroups of FieldRows holding a Toggle, PopUpButton, SegmentedControl, RadioGroup, Slider, Stepper, Checkbox, TextField and a TabView.",
        height: 1100,
        body: pages::settings_window::SettingsWindowPage,
    },
    Entry {
        page: Page::Overlays,
        title: "Overlays",
        lede: "Open the palette, popovers, peek and sheet, the toast with its pull tab; hover the targets for cards and tooltips.",
        height: 10800,
        body: crate::pages::overlays::overview::OverlaysPage,
    },
    Entry {
        page: Page::Feedback,
        title: "Overlays and feedback",
        lede: "Popover under each dismiss policy and with its arrow, Sheet hung from the window, centred and at the bottom, the alerts, SidePanel, Tooltip up and down, hover cards, the toast, EmptyState in its three forms, Skeleton in its three shapes and as a row, the InlineBanner shown and hidden, and Loadable cycling Loading, Ready and Failed, each with the states it can express.",
        height: 6600,
        body: crate::pages::overlays::catalogue::OverlaysCataloguePage,
    },
    Entry {
        page: Page::App,
        title: "App features",
        lede: "Pinned tiles (drag one onto another to reorder), Today tabs that expire, the edge-peek sidebar (rest the pointer on the left edge), the link pill, the launcher's commands grouped in a Space's order, and Control and a digit to switch Space.",
        height: 2600,
        body: crate::pages::shell::app_features::AppFeaturesPage,
    },
    Entry {
        page: Page::Materials,
        title: "Materials",
        lede: "The eight materials as chrome over a wallpaper, blur on and off, with the ink's four legibility floors measured live at the tint alpha below.",
        height: 1700,
        body: crate::pages::foundations::materials::MaterialsPage,
    },
    Entry {
        page: Page::Motion,
        title: "Motion",
        lede: "Every duration, delay, easing and scalar token at each motion level, and every animation's recipe with its settle time.",
        height: 2600,
        body: crate::pages::foundations::motion::MotionPage,
    },
    Entry {
        page: Page::Space,
        title: "Space",
        lede: "The Space editor bound to the gallery's own Space: the frame tokens it derives, printed, and the eight presets as Space dots.",
        height: 1900,
        body: crate::pages::foundations::space::SpacePage,
    },
    Entry {
        page: Page::BlitzLimits,
        title: "Blitz limits",
        lede: "What Blitz cannot do and what quire draws instead (spike S1-S16), and what the design names that quire does not draw yet.",
        height: 2250,
        body: crate::pages::foundations::blitz_limits::BlitzLimitsPage,
    },
    Entry {
        page: Page::Matrix,
        title: "Matrix",
        lede: "One component in a Surface cell for each scheme and accent: pick the component.",
        height: 900,
        body: crate::pages::foundations::matrix::MatrixPage,
    },
    Entry {
        page: Page::MotionLab,
        title: "Motion lab",
        lede: "Fire each animation on a sample. Beside it, the CSS duration token at the current level and the Rust settle() that times the state after it.",
        height: 2050,
        body: crate::pages::foundations::motion_lab::MotionLabPage,
    },
    Entry {
        page: Page::Edit,
        title: "Edit",
        lede: "An EditSurface over an app's own paragraphs and a chip: the surface hands the app its input and reports geometry; the caret here is the page's own, drawn from the host's caret rect.",
        height: 600,
        body: crate::pages::editor::edit_surface::EditPage,
    },
    Entry {
        page: Page::Details,
        title: "Details",
        lede: "The grammar of small state details (design/26-DETAILS.md): Sweep with CountUp in step, Reveal, the bounded pending loop on a layered Wi-Fi glyph and on the Spinner, Settle's fill, check and seal, Shake, Nudge, every MorphGlyph style and RollDigits, each with a button that plays its moment again, and the bar's layered status glyphs (Wi-Fi, battery, Bluetooth, volume) in every state they draw. Nothing here loops: each settles to 0 frames.",
        height: 3760,
        body: crate::pages::details::overview::DetailsPage,
    },
    Entry {
        page: Page::VoiceOrb,
        title: "Voice orb",
        lede: "The voice orb: the default at 192 px, a small one at 96 px and one with its own colours at 128 px on a 15 s period, all turning while active and at rest when not, and a ladder of sizes that crosses every threshold of its size-derived look.",
        height: 900,
        body: crate::pages::content::voice_orb::VoiceOrbPage,
    },
    Entry {
        page: Page::Symbols,
        title: "Symbols",
        lede: "Symbol effects (design/35-SYMBOL-EFFECTS.md), modelled on SF Symbols': Bounce, Pulse, Wiggle, Breathe, Rotate, Variable Color and the icon's own moving part fire once from a button; the same effects, Scale up and Scale down run while their toggle is on; Appear, Disappear, Draw On and Replace play when their state changes; and the part-aware icons (trash, bell, mail, folder, refresh, Wi-Fi, volume, battery, lock, star, heart) move the piece Apple moves. Nothing runs by itself: each settles to 0 frames except a toggle left on.",
        height: 3000,
        body: crate::pages::content::symbols::SymbolsPage,
    },
    Entry {
        page: Page::Accounts,
        title: "Accounts",
        lede: "The account sheets: the consent alert with one account, several and none; each step of the add-account sheet (providers with a search, the sign-in form with a refused password, the browser wait, the device code, the review with limits and the add-and-allow button) in light and dark; and the parts an app shows: the account picker, the no-account states, the badge and the limited note.",
        height: 4200,
        body: crate::pages::shell::accounts::AccountsPage,
    },
    Entry {
        page: Page::Helpers,
        title: "Helpers",
        lede: "The missing-helper sheet: an app that needs a distro tool this machine lacks asks, at the moment of use, to install it (the question, installing, failed, and the not-found and unsupported wordings), in light and dark. Install... goes to PackageKit, which asks for the password itself.",
        height: 2400,
        body: crate::pages::shell::helpers::HelpersPage,
    },
];

/// The entry for `page`.
pub fn entry(page: Page) -> &'static Entry {
    REGISTRY
        .iter()
        .find(|entry| entry.page == page)
        .unwrap_or(&REGISTRY[0])
}

#[cfg(test)]
mod tests {
    use super::REGISTRY;
    use crate::page::Page;
    use ds::prelude::*;

    #[test]
    fn every_page_has_exactly_one_entry_in_the_page_order() {
        assert_eq!(REGISTRY.len(), Page::ALL.len());
        for (entry, page) in REGISTRY.iter().zip(Page::ALL.iter().copied()) {
            assert_eq!(entry.page, page, "{} is out of order", entry.title);
        }
        for page in Page::ALL.iter().copied() {
            let count = REGISTRY.iter().filter(|entry| entry.page == page).count();
            assert_eq!(count, 1, "{page:?}");
        }
    }

    #[test]
    fn every_entry_says_what_it_shows() {
        for entry in REGISTRY {
            assert!(
                !entry.title.is_empty() && !entry.lede.is_empty(),
                "{:?}",
                entry.page
            );
            assert!(
                entry.height >= 600,
                "{:?} is too short to snapshot",
                entry.page
            );
        }
    }
}
