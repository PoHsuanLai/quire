//! Every tip quire draws is `Name  ⌘K` (owner decision 2026-10-09, design/30 section 2.5): a
//! short noun phrase in title case, two spaces, then the key in Kbd glyphs. No parentheses, no
//! sentence, no full stop. Each tip-bearing control in the catalogue is hovered, its tip read
//! off the page and held to the style; the accessible names keep their full wording.

use dioxus::prelude::*;
use ds::components::app::pin_tile::{PinFace, PinTile};
use ds::components::app::row_more::RowMore;
use ds::components::app::thread_row::ThreadRow;
use ds::components::content::avatar::AvatarSize;
use ds::components::content::provider_mark::MarkProvider;
use ds::components::controls::button_model::{Bezel, ImagePosition};
use ds::components::controls::chip::{Chip, ChipVariant};
use ds::components::controls::segmented::Tracking;
use ds::components::lists::row::row::{Outline, Row};
use ds::prelude::*;
use ds::style::space::frame_vars::FrameVars;
use ds_harness::{Clock, Driver, Harness, HarnessConfig, Input, Query, Viewport};
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 480,
    height: 320,
    scale_percent: 100,
};

fn cmd(c: char) -> Shortcut {
    Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char(c)])
}

#[allow(non_snake_case)]
fn Page() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            ProviderMark { provider: MarkProvider::Imap }
            PinTile {
                face: PinFace::Add { label: "Add account".to_owned(), hint: Some("Add Account".to_owned()) },
                onclick: |_| {},
            }
            SpaceDot {
                name: "Space 1",
                frame: FrameVars::of(&SpaceLook::default(), Scheme::Light),
                selection: Selection::Unselected,
                shortcut: cmd('1'),
                onclick: |_| {},
            }
            SegmentedControl::<u8> {
                label: "View".to_owned(),
                choices: vec![
                    Choice { icon: Some(Icon::Music.into()), name: Some("Music".to_owned()), ..Choice::new(1, "") },
                    Choice::new(2, "Words"),
                ],
                tracking: Tracking::SelectOne(1),
                onchange: |_| {},
            }
            RowMore { shown: Some(Shown::Visible), shortcut: Some(cmd('m')), onclick: |_| {} }
            Chip {
                variant: ChipVariant::Person(MarkProvider::Imap.avatar(AvatarSize::Size28)),
                text: "Ada".to_owned(),
                onremove: |_| {},
            }
            Row { title: "Folder", outline: Outline::Branch(Shown::Hidden), on_toggle: |_| {} }
            ThreadRow {
                name: "Ada".to_owned(),
                subject: "Hello",
                time: "09:41".to_owned(),
                tags: rsx! {},
                star: Some((Check::Off, EventHandler::new(|_| {}))),
                star_shortcut: Some(Shortcut(vec![ShortcutKey::Char('s')])),
                onclick: |_| {},
            }
            Button {
                bezel: Bezel::Toolbar,
                image: ImagePosition::Only,
                icon: Some(IconSource::from(Icon::Plus)),
                label: "New Space",
                title: Some("New Space".to_owned()),
                title_shortcut: Some(cmd('n')),
                onclick: |_| {},
                common: Common { id: Some("newspace".to_owned()), ..Common::default() },
            }
        }
    }
}

/// Each tip-bearing control and the tip it shows.
const TIPS: &[(&str, &str)] = &[
    (".ds-pin-tile", "Add Account"),
    (".ds-space-dot", "Space 1  ⌘1"),
    (".ds-segmented-segment:first-child", "Music"),
    (".ds-row-more", "More  ⌘M"),
    (".ds-chip-remove", "Remove"),
    (".ds-row-disclosure", "Expand"),
    (".ds-star", "Star  S"),
    ("#newspace", "New Space  ⌘N"),
];

/// Whether `key` is the key part of a tip: modifier glyphs, then one key glyph or key name.
fn is_key(key: &str) -> bool {
    const MODIFIERS: &str = "⌃⌥⇧⌘";
    const GLYPHS: &str = "↵⇥⌫↑↓←→↖↘⌦⇞⇟";
    const NAMES: &[&str] = &["Space", "Esc", "Ins", "Menu"];
    let rest = key.trim_start_matches(|c| MODIFIERS.contains(c));
    let mut chars = rest.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => GLYPHS.contains(c) || c.is_ascii_uppercase() || c.is_ascii_digit(),
        _ => NAMES.contains(&rest),
    }
}

/// Why `tip` breaks the house style, or `None` when it keeps it.
fn breach(tip: &str) -> Option<String> {
    if tip.contains('(') || tip.contains(')') {
        return Some("parentheses".to_owned());
    }
    if tip.ends_with('.') {
        return Some("a trailing full stop".to_owned());
    }
    let parts: Vec<&str> = tip.split("  ").collect();
    let (name, key) = match parts.as_slice() {
        [name] => (*name, None),
        [name, key] => (*name, Some(*key)),
        _ => return Some("more than one double-space separator".to_owned()),
    };
    if name.is_empty() || name != name.trim() || name.contains("  ") {
        return Some("a name with stray spaces".to_owned());
    }
    if name.chars().next().is_some_and(char::is_lowercase) {
        return Some("a name that is not capitalised".to_owned());
    }
    key.filter(|key| !is_key(key))
        .map(|key| format!("the key part {key:?} is not Kbd glyphs"))
}

#[test]
fn the_checker_itself_tells_the_style_from_its_breaches() {
    for good in [
        "Star",
        "Star  S",
        "Space 1  ⌘1",
        "Redo  ⇧⌘Z",
        "Reveal  ⌘Space",
    ] {
        assert_eq!(breach(good), None, "{good}");
    }
    for bad in [
        "Space 1 (⌘1)",
        "Star this thread.",
        "Star   S",
        "Star  S  T",
        "star",
        "Star  Cmd K",
        "Star  ",
    ] {
        assert!(breach(bad).is_some(), "{bad}");
    }
}

#[test]
fn every_tip_a_catalogue_control_draws_keeps_the_house_style() {
    for &(selector, expected) in TIPS {
        let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
        harness.advance(Duration::from_millis(100));
        let at = harness
            .centre(selector)
            .unwrap_or_else(|| panic!("{selector} is not drawn:\n{}", harness.html()));
        harness.send(Input::pointer_move(at));
        for _ in 0..30 {
            harness.advance(Duration::from_millis(100));
        }
        let tip = harness
            .text_of(".ds-tooltip")
            .unwrap_or_else(|| panic!("{selector} drew no tip:\n{}", harness.html()));
        assert_eq!(tip, expected, "{selector}");
        assert_eq!(breach(&tip), None, "{selector}: {tip:?}");
    }
}

#[test]
fn the_accessible_names_keep_their_full_wording() {
    let mut harness = Harness::new(Page, HarnessConfig::new(VIEW).with_clock(Clock::Virtual));
    harness.advance(Duration::from_millis(100));
    let html = harness.html();
    for name in [
        "aria-label=\"Star this thread\"",
        "aria-label=\"More actions\"",
        "aria-label=\"Remove Ada\"",
        "aria-label=\"Space 1 Space\"",
    ] {
        assert!(html.contains(name), "{name} missing:\n{html}");
    }
}
