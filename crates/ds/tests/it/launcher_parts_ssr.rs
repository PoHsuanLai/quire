//! The launcher v2 parts as markup: each row shape, a palette group with
//! a "Show More" action at rest and under the cursor, an emoji grid in the palette and on its
//! own, the preview pane with each kind of content, and the palette widened by a pane. Each
//! matches its golden under `tests/snapshots/launcher/`, lints clean, and uses only `ds-`
//! classes the stylesheet styles.
//!
//! `DS_BLESS=1 cargo test -p ds --features lint --test it launcher_parts_ssr` rewrites the goldens.

use crate::support::golden;

use dioxus::prelude::*;
use ds::assembly::ds::Inject;
use ds::components::content::image_source::ImageSize;
use ds::components::content::pdf_thumb::PdfPage;
use ds::components::lists::emoji_grid::grid::{EMOJI_CELL, EMOJI_COLUMNS, EmojiCell, EmojiCells};
use ds::components::lists::preview::content::{PaneContent, PaneMono};
use ds::components::lists::preview::pane::PaneAction;
use ds::components::lists::row::shape::{ClipBody, RowShape};
use ds::components::menus::palette::palette_group::{PaletteGroup, PaletteGroups, PaletteRow};
use ds::prelude::*;
use ds_lint::{LintConfig, markup};

fn root(body: Element) -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet, stylesheet: Inject::Host,
            {body}
        }
    }
}

fn palette(groups: Vec<PaletteGroup<u8>>, selected: usize) -> Element {
    palette_with(groups, selected, None)
}

fn palette_with(groups: Vec<PaletteGroup<u8>>, selected: usize, aside: Option<Element>) -> Element {
    root(rsx! {
        CommandPalette::<u8> {
            label: "Search",
            placeholder: "Search",
            query: String::new(),
            tokens: Vec::new(),
            groups: PaletteGroups(groups),
            empty: "Nothing",
            oninput: |_| {},
            onpick: |_| {},
            onclose: |_| {},
            host: CommandPaletteHost::Surface,
            selected,
            aside,
        }
    })
}

fn row(value: u8, title: &str, shape: RowShape) -> PaletteRow<u8> {
    PaletteRow {
        leading: RowLeading::Icon(Icon::File),
        shape,
        ..PaletteRow::new(value, title)
    }
}

fn file(thumb: Option<ImageSource>) -> RowShape {
    RowShape::File {
        thumb,
        location: "~/Documents".to_owned(),
        modified: "Yesterday".to_owned(),
    }
}

fn picture() -> ImageSource {
    ImageSource("data:image/png;base64,AAAA".to_owned())
}

fn files() -> Element {
    palette(
        vec![PaletteGroup::list(
            "Documents",
            vec![
                row(1, "Invoice.pdf", file(None)),
                row(2, "Beach.png", file(Some(picture()))),
            ],
        )],
        0,
    )
}

/// A file row with a shortcut after its time: the two in boxes of their own.
fn file_with_keys() -> Element {
    let row = PaletteRow {
        leading: RowLeading::Icon(Icon::File),
        shape: file(None),
        accessory: Accessory::Text(Shortcut(vec![ShortcutKey::Enter]).glyphs()),
        ..PaletteRow::new(1, "Invoice.pdf")
    };
    palette(vec![PaletteGroup::list("Documents", vec![row])], 0)
}

/// Two file rows whose action chord waits for the selection (Spotlight's hint) and a plain row
/// whose chord shows always: on `selected`, only that file row ends in `⌘R`.
fn chord_rows(selected: usize) -> Element {
    let reveal = Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('r')]);
    let chorded = |value: u8, title: &str| PaletteRow {
        leading: RowLeading::Icon(Icon::File),
        shape: RowShape::File {
            thumb: None,
            location: "~/Documents".to_owned(),
            modified: "13:00".to_owned(),
        },
        chord: RowChord::on_selected(reveal.clone()),
        ..PaletteRow::new(value, title)
    };
    let always = PaletteRow {
        leading: RowLeading::Icon(Icon::Settings),
        accessory: Accessory::Text("Setting".to_owned()),
        chord: RowChord::always(Shortcut(vec![
            ShortcutKey::Super,
            ShortcutKey::Shift,
            ShortcutKey::Char('d'),
        ])),
        ..PaletteRow::new(3, "Displays")
    };
    palette(
        vec![PaletteGroup::list(
            "Documents",
            vec![chorded(1, "Invoice.pdf"), chorded(2, "Receipt.pdf"), always],
        )],
        selected,
    )
}

fn chord_first() -> Element {
    chord_rows(0)
}

fn chord_second() -> Element {
    chord_rows(1)
}

fn clips() -> Element {
    let text = RowShape::Clip {
        body: ClipBody::Text {
            excerpt: "fn main() {\n    println!(\"hi\");\n}".to_owned(),
            lines: 2,
        },
        age: "2 min".to_owned(),
    };
    let image = RowShape::Clip {
        body: ClipBody::Image {
            src: picture(),
            size: ImageSize {
                width: 1600,
                height: 900,
            },
        },
        age: "1 h".to_owned(),
    };
    palette(
        vec![PaletteGroup::list(
            "Clipboard",
            vec![row(1, "fn main", text), row(2, "Image", image)],
        )],
        0,
    )
}

fn more_group(selected: usize) -> Element {
    palette(
        vec![
            PaletteGroup::list(
                "Applications",
                vec![
                    row(1, "Files", RowShape::Plain),
                    row(2, "Firefox", RowShape::Plain),
                ],
            )
            .with_action("Show More", EventHandler::new(|()| {})),
            PaletteGroup::list("Settings", vec![row(3, "Displays", RowShape::Plain)]),
        ],
        selected,
    )
}

fn show_more() -> Element {
    more_group(0)
}

fn show_more_selected() -> Element {
    more_group(2)
}

fn cells() -> Vec<EmojiCell<u8>> {
    ["😀", "🎉", "👍", "❤️", "🔥", "🙂", "😂", "🤔", "🚀", "✨"]
        .iter()
        .zip([
            "grinning face",
            "party popper",
            "thumbs up",
            "red heart",
            "fire",
            "slightly smiling face",
            "face with tears of joy",
            "thinking face",
            "rocket",
            "sparkles",
        ])
        .enumerate()
        .map(|(value, (glyph, name))| EmojiCell {
            value: value as u8,
            glyph: (*glyph).to_owned(),
            name: name.to_owned(),
        })
        .collect()
}

fn grid_in_palette() -> Element {
    palette(
        vec![
            PaletteGroup::list("Top hit", vec![row(40, "Emoji", RowShape::Plain)]),
            PaletteGroup::grid(
                "Emoji",
                EmojiCells {
                    cells: cells(),
                    columns: EMOJI_COLUMNS,
                    cell: EMOJI_CELL,
                },
            ),
        ],
        3,
    )
}

fn grid_alone() -> Element {
    root(rsx! {
        EmojiGrid::<u8> { cells: cells(), columns: 4, cell: Px(40.0), selected: 5, onpick: |_| {} }
    })
}

fn actions() -> Vec<PaneAction> {
    vec![
        PaneAction {
            label: "Open".to_owned(),
            shortcut: Shortcut(vec![ShortcutKey::Enter]),
        },
        PaneAction {
            label: "Reveal in Files".to_owned(),
            shortcut: Shortcut(vec![ShortcutKey::Super, ShortcutKey::Char('r')]),
        },
    ]
}

fn pane(content: PaneContent, focused: Option<usize>) -> Element {
    root(rsx! {
        PreviewPane { content, actions: actions(), focused }
    })
}

fn pane_image() -> Element {
    pane(
        PaneContent::Image {
            src: picture(),
            size: ImageSize {
                width: 1600,
                height: 900,
            },
        },
        None,
    )
}

fn pane_text_mono() -> Element {
    pane(
        PaneContent::Text {
            excerpt: "let x = 1;".to_owned(),
            mono: PaneMono::Monospace,
        },
        Some(1),
    )
}

fn pane_text_prose() -> Element {
    pane(
        PaneContent::Text {
            excerpt: "Pick up the keys.".to_owned(),
            mono: PaneMono::Proportional,
        },
        None,
    )
}

fn pane_pdf() -> Element {
    pane(
        PaneContent::Pdf {
            page: PdfPage::Ready {
                image: picture(),
                sheet: ImageSize {
                    width: 612,
                    height: 792,
                },
            },
            name: "Invoice.pdf".to_owned(),
        },
        Some(0),
    )
}

fn pane_app() -> Element {
    pane(
        PaneContent::App {
            icon: IconSource::Glyph(Icon::Folder),
            name: "Files".to_owned(),
            detail: Some("Browse your files".to_owned()),
        },
        None,
    )
}

fn pane_facts() -> Element {
    pane(
        PaneContent::Facts {
            icon: IconSource::Glyph(Icon::Settings),
            title: "Displays".to_owned(),
            rows: vec![
                ("Kind".to_owned(), "Setting".to_owned()),
                ("Where".to_owned(), "Settings › Displays".to_owned()),
            ],
        },
        None,
    )
}

fn pane_emoji() -> Element {
    pane(
        PaneContent::Emoji {
            glyph: "🎉".to_owned(),
            name: "party popper".to_owned(),
        },
        None,
    )
}

fn pane_web() -> Element {
    pane(
        PaneContent::Web {
            host: "duckduckgo.com".to_owned(),
            url: "https://duckduckgo.com/?q=quire".to_owned(),
        },
        None,
    )
}

fn pane_slot() -> Element {
    pane(
        PaneContent::Slot {
            label: "Invoice.pdf".to_owned(),
            element: rsx! {
                div { class: "ds-truncate", "The caller's own picture, name and facts" }
            },
        },
        Some(0),
    )
}

fn with_aside() -> Element {
    let aside = rsx! {
        PreviewPane {
            content: PaneContent::Emoji { glyph: "🎉".to_owned(), name: "party popper".to_owned() },
            actions: actions(),
        }
    };
    palette_with(
        vec![PaletteGroup::list(
            "Applications",
            vec![row(1, "Files", RowShape::Plain)],
        )],
        0,
        Some(aside),
    )
}

/// A specimen: its golden name and how it is made.
type Specimen = (&'static str, fn() -> Element);

const SPECIMENS: &[Specimen] = &[
    ("row-file", files),
    ("row-file-keys", file_with_keys),
    ("row-clip", clips),
    ("row-chord-first", chord_first),
    ("row-chord-second", chord_second),
    ("show-more", show_more),
    ("show-more-selected", show_more_selected),
    ("grid-in-palette", grid_in_palette),
    ("grid-alone", grid_alone),
    ("pane-image", pane_image),
    ("pane-text-mono", pane_text_mono),
    ("pane-text-prose", pane_text_prose),
    ("pane-pdf", pane_pdf),
    ("pane-app", pane_app),
    ("pane-facts", pane_facts),
    ("pane-emoji", pane_emoji),
    ("pane-web", pane_web),
    ("pane-slot", pane_slot),
    ("palette-aside", with_aside),
];

fn render(make: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(make);
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

#[test]
fn every_specimen_matches_its_golden() {
    let failures: Vec<String> = SPECIMENS
        .iter()
        .filter_map(|(name, make)| {
            golden::check(&format!("launcher/{name}.html"), &render(*make)).err()
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn every_specimen_lints_clean_and_every_class_is_styled() {
    let sheet = ds::stylesheet();
    let mut failures = Vec::new();
    for (name, make) in SPECIMENS {
        let html = render(*make);
        for offence in markup(&html, sheet, &LintConfig::new(&ds::kits())) {
            failures.push(format!("{name}: {:?} {}", offence.rule, offence.text));
        }
        for class in html
            .split("class=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .flat_map(str::split_whitespace)
            .filter(|class| class.starts_with("ds-"))
        {
            let needle = format!(".{class}");
            let styled = sheet.match_indices(&needle).any(|(at, _)| {
                !sheet[at + needle.len()..]
                    .starts_with(|c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            });
            if !styled {
                failures.push(format!("{name}: .{class} is not styled"));
            }
        }
    }
    failures.dedup();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// What each part stamps, read from the markup: the shapes, the action under the cursor, the
/// grid's colour face and selection, the pane's content and focused action, the widened card.
#[test]
fn the_markup_carries_the_states() {
    let files = render(files);
    assert!(files.contains("data-shape=\"file\""), "{files}");
    assert!(files.contains("data-leading=\"thumb\""), "{files}");
    assert!(
        files.contains("~/Documents") && files.contains("ds-row-when"),
        "{files}"
    );
    let keyed = render(file_with_keys);
    assert!(
        keyed.contains(
            "<span class=\"ds-row-when\">Yesterday</span>\
             <span class=\"ds-row-trailing\" data-mark=\"text\">↵</span>"
        ),
        "the time and the shortcut are separate boxes (Q343): {keyed}"
    );
    let clips = render(clips);
    assert!(clips.contains("data-shape=\"clip-text\""), "{clips}");
    assert!(
        clips.contains("--lines:2") && clips.contains("width:78.22px;height:44.00px"),
        "{clips}"
    );
    let rest = render(show_more);
    assert!(
        rest.contains("Show More") && !rest.contains("data-selected"),
        "{rest}"
    );
    let on = render(show_more_selected);
    assert!(on.contains("data-selected=\"true\""), "{on}");
    let grid = render(grid_in_palette);
    assert!(grid.contains("ds-emoji-grid ds-emoji-text"), "{grid}");
    assert_eq!(grid.matches("role=\"gridcell\"").count(), 10, "{grid}");
    assert_eq!(grid.matches("aria-selected=\"true\"").count(), 1, "{grid}");
    let alone = render(grid_alone);
    assert!(
        alone.contains("repeat(4,40px)") && alone.contains("tabindex=\"0\""),
        "{alone}"
    );
    let mono = render(pane_text_mono);
    assert!(
        mono.contains("data-face=\"mono\"") && mono.contains("data-focused=\"true\""),
        "{mono}"
    );
    let pdf = render(pane_pdf);
    assert!(
        pdf.contains("ds-pdf-thumb") && pdf.contains("data-content=\"pdf\""),
        "{pdf}"
    );
    let slot = render(pane_slot);
    assert!(
        slot.contains("data-content=\"slot\"")
            && slot.contains("aria-label=\"Invoice.pdf preview\"")
            && slot.contains("The caller&#39;s own picture")
            && !slot.contains("ds-preview-caption")
            && slot.contains("ds-preview-actions"),
        "the slot keeps the frame and adds no caption: {slot}"
    );
    let aside = render(with_aside);
    assert!(
        aside.contains("data-aside=\"shown\"") && aside.contains("--aside:360px"),
        "{aside}"
    );
    assert!(
        aside.contains("ds-palette-aside") && aside.contains("width:360px"),
        "{aside}"
    );
}

/// Each row's markup, in order: the text from one `ds-row` to the next.
fn rows_of(html: &str) -> Vec<&str> {
    html.split("class=\"ds-row\"").skip(1).collect()
}

/// A chord case: its name, the palette, and the chord each of the three rows draws.
type ChordCase = (&'static str, fn() -> Element, [&'static str; 3]);

/// A row's chord waits for the selection: the selected file row ends in `⌘R` after its time,
/// the other file row shows its time alone, and the always-row shows its chord either way.
#[test]
fn a_rows_chord_follows_the_selection() {
    const CASES: &[ChordCase] = &[
        ("first selected", chord_first, ["⌘R", "", "⇧⌘D"]),
        ("second selected", chord_second, ["", "⌘R", "⇧⌘D"]),
    ];
    for (name, make, chords) in CASES {
        let html = render(*make);
        let rows = rows_of(&html);
        assert_eq!(rows.len(), 3, "{name}: {html}");
        for (at, (row, chord)) in rows.iter().zip(chords).enumerate() {
            let drawn = row
                .split("data-style=\"text\" data-size=\"regular\">")
                .nth(1)
                .and_then(|rest| rest.split('<').next())
                .unwrap_or("");
            assert_eq!(drawn, *chord, "{name}: row {at}: {row}");
        }
        assert_eq!(html.matches("ds-row-when\">13:00").count(), 2, "{name}");
    }
}

/// A pane action is its label then its keys as one plain key equivalent: no key caps.
#[test]
fn a_pane_action_ends_in_a_plain_key_equivalent() {
    let html = render(pane_pdf);
    assert!(!html.contains("ds-key-equivalent-key"), "{html}");
    assert!(
        html.contains("Reveal in Files</span><span class=\"ds-key-equivalent\" data-style=\"text\" data-size=\"regular\">⌘R</span>"),
        "{html}"
    );
    assert!(
        html.contains(
            "<span class=\"ds-key-equivalent\" data-style=\"text\" data-size=\"regular\">↵</span>"
        ),
        "{html}"
    );
}
