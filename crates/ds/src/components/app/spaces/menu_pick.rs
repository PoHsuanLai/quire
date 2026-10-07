//! The Space's menu as data: what it lists, and what a pick does. Pure.

use super::open_menu::Showing;
use crate::components::menus::item::item::MenuItem;
use ds_core::vocab::{Availability, Check};
use ds_core::word::Word;
use ds_style::appearance::theme::Theme;
use ds_style::space::look::{CardAccent, SpaceLook};

/// A row of the Space's menu, or a row the app added.
#[derive(Debug, Clone, PartialEq)]
pub enum SpacePick<A> {
    /// Rename...: a field at the pointer.
    Rename,
    /// Colour...: the colour field, stops, grain and presets at the pointer.
    Colour,
    /// Appearance, one of the themes.
    Theme(Theme),
    /// Accent Inside the Card: the Space's colour, or the person's accent.
    Accent(CardAccent),
    /// New Space; it opens its name at once.
    New,
    /// Delete Space...: a question at the pointer.
    Delete,
    /// A pick from the app's own submenu.
    App(A),
}

/// What a pick asks of the window.
#[derive(Debug, Clone, PartialEq)]
pub enum Then<A> {
    /// Open one of the Space's parts at the pointer.
    Open(Showing),
    /// The look changed: keep it.
    Kept(SpaceLook),
    /// Make a new Space, switch to it and open its name.
    NewSpace,
    /// The app's own row.
    App(A),
}

fn accent_name(accent: CardAccent) -> &'static str {
    match accent {
        CardAccent::SpaceHue => "Space Colour",
        CardAccent::Chosen => "Your Accent",
    }
}

fn submenu<A>(title: &str, children: Vec<MenuItem<SpacePick<A>>>) -> MenuItem<SpacePick<A>> {
    MenuItem::Submenu {
        title: title.to_owned(),
        image: None,
        availability: Availability::Enabled,
        children,
    }
}

fn checked(on: bool) -> Check {
    if on { Check::On } else { Check::Off }
}

/// The menu for a Space looking like `look`, one of `count`: Rename, Colour, Appearance, Accent
/// Inside the Card, the app's `extra` rows (usually one submenu whose picks keep the menu open),
/// a rule, New Space, and Delete only while there is another Space to show.
pub fn rows<A: Clone>(
    look: &SpaceLook,
    count: usize,
    extra: Vec<MenuItem<A>>,
) -> Vec<MenuItem<SpacePick<A>>> {
    let themes = Theme::ALL
        .iter()
        .map(|theme| {
            MenuItem::new(SpacePick::Theme(*theme), theme.label())
                .with_check(checked(look.theme == *theme))
        })
        .collect();
    let accents = [CardAccent::SpaceHue, CardAccent::Chosen]
        .into_iter()
        .map(|accent| {
            MenuItem::new(SpacePick::Accent(accent), accent_name(accent))
                .with_check(checked(look.card_accent == accent))
        })
        .collect();
    let mut out = vec![
        MenuItem::new(SpacePick::Rename, "Rename\u{2026}"),
        MenuItem::new(SpacePick::Colour, "Colour\u{2026}"),
        submenu("Appearance", themes),
        submenu("Accent Inside the Card", accents),
    ];
    out.extend(extra.into_iter().map(|item| item.map(&SpacePick::App)));
    out.push(MenuItem::Separator);
    out.push(MenuItem::new(SpacePick::New, "New Space"));
    if count > 1 {
        out.push(MenuItem::new(SpacePick::Delete, "Delete Space\u{2026}"));
    }
    out
}

/// Do `pick` to a Space looking like `look`.
pub fn apply<A>(pick: SpacePick<A>, look: &SpaceLook) -> Then<A> {
    match pick {
        SpacePick::Rename => Then::Open(Showing::Rename),
        SpacePick::Colour => Then::Open(Showing::Colour),
        SpacePick::Delete => Then::Open(Showing::Delete),
        SpacePick::New => Then::NewSpace,
        SpacePick::Theme(theme) => Then::Kept(SpaceLook {
            theme,
            ..look.clone()
        }),
        SpacePick::Accent(card_accent) => Then::Kept(SpaceLook {
            card_accent,
            ..look.clone()
        }),
        SpacePick::App(app) => Then::App(app),
    }
}

/// What the delete question says, for the Space called `name`. A Space with no name is "this
/// Space". `kept` names what is not deleted ("Your mail").
pub fn delete_words(name: &str, kept: &str) -> (String, String) {
    let named = if name.trim().is_empty() {
        "this Space".to_owned()
    } else {
        format!("\u{201c}{}\u{201d}", name.trim())
    };
    (
        format!("Delete {named}?"),
        format!(
            "{kept} is not affected. Only {named}\u{2019}s look and settings go. This cannot be undone."
        ),
    )
}
