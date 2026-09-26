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
pub const REGISTRY: [Entry; 18] = [
    Entry {
        page: Page::Tokens,
        title: "Tokens",
        lede: "Every colour token with its hex in both schemes, the accents, the label hues, the spacing scale, radii, shadows and z layers.",
        height: 2300,
        body: pages::tokens::TokensPage,
    },
    Entry {
        page: Page::Type,
        title: "Type",
        lede: "The four faces and the size ramp: every --fs step drawn at its size, with its role.",
        height: 1500,
        body: pages::type_ramp::TypePage,
    },
    Entry {
        page: Page::Controls,
        title: "Controls",
        lede: "Every control in every state it can express: variants, pressed, expanded, disabled, empty and filled. Press Tab to see the keyboard focus ring.",
        height: 3450,
        body: pages::controls::ControlsPage,
    },
    Entry {
        page: Page::Lists,
        title: "Lists",
        lede: "A live AnimatedList: add rows, remove them with each exit and watch the rows below heal, then undo. Search hits with a keyboard-shown strip, sidebar items, tiles, the hover strip and the appearance picker.",
        height: 3150,
        body: pages::lists::ListsPage,
    },
    Entry {
        page: Page::Overlays,
        title: "Overlays",
        lede: "Open each menu kind, the palette, popovers, peek and sheet, the toast with its pull tab; hover the targets for cards and tooltips.",
        height: 9950,
        body: pages::overlays::OverlaysPage,
    },
    Entry {
        page: Page::Materials,
        title: "Materials",
        lede: "The eight materials as chrome over a wallpaper, blur on and off, with the ink's four legibility floors measured live at the tint alpha below.",
        height: 1700,
        body: pages::materials::MaterialsPage,
    },
    Entry {
        page: Page::Motion,
        title: "Motion",
        lede: "Every duration, delay, easing and scalar token at each motion level, and every animation's recipe with its settle time.",
        height: 2600,
        body: pages::motion::MotionPage,
    },
    Entry {
        page: Page::Space,
        title: "Space",
        lede: "The Space editor bound to the gallery's own Space: the frame tokens it derives, printed, and the eight presets as Space dots.",
        height: 1900,
        body: pages::space::SpacePage,
    },
    Entry {
        page: Page::Gaps,
        title: "Gaps",
        lede: "What Blitz cannot do and what quire draws instead (spike S1-S16), and what the design names that quire does not draw yet.",
        height: 2250,
        body: pages::gaps::GapsPage,
    },
    Entry {
        page: Page::Matrix,
        title: "Matrix",
        lede: "One component in a Surface cell for each scheme and accent: pick the component.",
        height: 900,
        body: pages::matrix::MatrixPage,
    },
    Entry {
        page: Page::MotionLab,
        title: "Motion lab",
        lede: "Fire each animation on a sample. Beside it, the CSS duration token at the current level and the Rust settle() that times the state after it.",
        height: 2050,
        body: pages::motion_lab::MotionLabPage,
    },
    Entry {
        page: Page::Polish,
        title: "Polish",
        lede: "The shell chrome beside the macOS numbers it targets: the material stack, a text menu and the menu-bar items, squircle corners, the dock pill and its plates, the launcher and the window shadow, each with the target printed under it; then the window frame with its traffic lights and the tiling menu open.",
        height: 4250,
        body: pages::polish::PolishPage,
    },
    Entry {
        page: Page::Edit,
        title: "Edit",
        lede: "An EditSurface over an app's own paragraphs and a chip: the surface hands the app its input and reports geometry; the caret here is the page's own, drawn from the host's caret rect.",
        height: 600,
        body: pages::edit_surface::EditPage,
    },
    Entry {
        page: Page::Level,
        title: "Level",
        lede: "The level control in its three looks for the user to choose from (capsule with the glyph inside, capsule and knob, sixteen segments), live and as a grid of states, and the OSD card at the top right that carries it.",
        height: 2300,
        body: pages::level::LevelPage,
    },
    Entry {
        page: Page::WidgetLooks,
        title: "Widget looks",
        lede: "The widgets flat, bright and measured (design/23-WIDGETS.md section 2): the battery as bright rings with the device glyph inside and the percentage under each, and the world clock as white day dials and dark night dials with an orange seconds hand; each in a Small and a Medium card over a calm wallpaper, the second medium card tinted by the Space.",
        height: 1000,
        body: pages::widget_looks::WidgetLooksPage,
    },
    Entry {
        page: Page::WidgetReference,
        title: "Widget reference",
        lede: "The widgets posed as the reference screenshots design/23 section 1.1 measures, at the same size, for the side-by-side comparison: the battery alone, four small rings, the medium row with a low and a charging device, the small analog clock, and the medium world clock by day and by night.",
        height: 1150,
        body: pages::widget_reference::WidgetReferencePage,
    },
    Entry {
        page: Page::LockSwitcher,
        title: "Lock and switcher",
        lede: "The shell's own lock screen over the calm wallpaper (at rest, a wrong password mid-shake, checking in the Space's colour), the polkit prompt's sheet, and the app switcher with five and fourteen apps, shrunk and scrolled.",
        height: 3150,
        body: pages::lock_switcher::LockSwitcherPage,
    },
    Entry {
        page: Page::Persona,
        title: "Persona",
        lede: "The user's own character (design/24-PERSONA.md): 48 seeds, two finishes for the user's pick, every top on every creature, every mood at Large, the three sizes, and a user picture of each kind.",
        height: 3200,
        body: pages::persona::PersonaPage,
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

    #[test]
    fn every_page_has_exactly_one_entry_in_the_page_order() {
        assert_eq!(REGISTRY.len(), Page::ALL.len());
        for (entry, page) in REGISTRY.iter().zip(Page::ALL) {
            assert_eq!(entry.page, page, "{} is out of order", entry.title);
        }
        for page in Page::ALL {
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
