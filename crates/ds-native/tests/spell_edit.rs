//! Spelling in an `EditSurface`, end to end in the harness (design/04-COMPONENTS.md section 50):
//! a misspelling typed is marked after the debounce, the word being typed stays unmarked until
//! the caret leaves it, a right-click opens the menu, a picked suggestion replaces the word as
//! one undoable edit (the app's undo restores it), and Ignore, Learn and the context-menu key.
//! The dictionary is the tiny test one (`support/spell_dict.rs`), never the system's.

#[path = "support/spell_dict.rs"]
mod spell_dict;

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, EditInput, EditSurface, Key, KeyInput, Lang, Material, Point, PointerButton,
    Px, RootExtent, Spell, SpellReplace, TextPosition,
};
use ds_native::harness::settle_until;
use ds_native::spell::{SpellConfig, provide_with};
use ds_native::{Harness, Viewport};
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::{Duration, Instant};

const VIEW: Viewport = Viewport {
    width: 420,
    height: 300,
    scale_percent: 100,
};

/// `DelayToken::SpellDebounce`.
const DEBOUNCE: Duration = Duration::from_millis(300);

thread_local! {
    static DICT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// The app's own model: the paragraph's text, the caret, and an undo stack of whole states.
#[derive(Debug, Clone, PartialEq, Default)]
struct Model {
    text: String,
    caret: usize,
    undo: Vec<(String, usize)>,
}

impl Model {
    fn remember(&mut self) {
        self.undo.push((self.text.clone(), self.caret));
    }

    fn input(&mut self, input: EditInput) {
        match input {
            EditInput::Text(typed) => {
                self.remember();
                self.text.insert_str(self.caret, &typed);
                self.caret += typed.len();
            }
            EditInput::Key(KeyInput { key, modifiers }) => match key {
                dioxus::prelude::Key::Character(z) if z == "z" && modifiers.ctrl() => {
                    if let Some((text, caret)) = self.undo.pop() {
                        self.text = text;
                        self.caret = caret;
                    }
                }
                dioxus::prelude::Key::ArrowLeft => self.caret = self.caret.saturating_sub(1),
                dioxus::prelude::Key::ArrowRight => {
                    self.caret = (self.caret + 1).min(self.text.len())
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// A suggestion: one edit, one undo step.
    fn replace(&mut self, replace: SpellReplace) {
        self.remember();
        let (start, end) = (replace.range.anchor.offset.0, replace.range.focus.offset.0);
        self.text.replace_range(start..end, &replace.text);
        self.caret = start + replace.text.len();
    }
}

#[allow(non_snake_case)]
fn Editor() -> Element {
    let dir = DICT
        .with(|dict| dict.borrow().clone())
        .expect("a dictionary");
    provide_with(
        SpellConfig {
            dictionaries: vec![dir.clone()],
            user: dir.join("user"),
        },
        Lang::parse("en_US").into_iter().collect(),
    );
    let mut model = use_signal(Model::default);
    let (text, caret) = {
        let read = model.read();
        (read.text.clone(), read.caret)
    };
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet, extent: RootExtent::Viewport,
            div { style: "padding:20px; width:360px; font-size:16px; line-height:20px",
                EditSurface {
                    id: "editor",
                    spell: Spell::On { lang: None },
                    caret: Some(TextPosition::new("p0", caret)),
                    on_input: move |input| model.write().input(input),
                    on_replace: move |replace| model.write().replace(replace),
                    p { id: "p0", "data-edit-node": "p0", style: "margin:0; min-height:20px", "{text}" }
                }
            }
        }
    }
}

fn fresh(name: &str) -> Harness {
    let dir = spell_dict::dictionary(name);
    DICT.with(|dict| *dict.borrow_mut() = Some(dir));
    let mut harness = Harness::new(Editor, VIEW);
    harness.advance(Duration::from_millis(50));
    let editor = harness.centre("#editor").expect("the editor");
    harness.click(editor);
    harness.advance(Duration::from_millis(50));
    harness
}

fn type_text(harness: &mut Harness, text: &str) {
    for c in text.chars() {
        match c {
            ' ' => harness.key(Key::Space),
            c => harness.key(Key::Char(c)),
        }
    }
}

fn marks(harness: &Harness) -> usize {
    harness.count(".ds-spell-mark")
}

fn paragraph(harness: &Harness) -> String {
    harness.text_of("#p0").unwrap_or_default()
}

fn dict_dir() -> PathBuf {
    DICT.with(|dict| dict.borrow().clone())
        .expect("a dictionary")
}

/// The centre of the first mark: over the misspelt word's baseline.
fn on_mark(harness: &Harness) -> Point {
    let mark = harness.rect(".ds-spell-mark").expect("a mark");
    let line = harness.rect("#p0").expect("the paragraph");
    Point {
        x: Px(mark.origin.x.0 + mark.size.width.0 / 2.0),
        y: Px(line.origin.y.0 + 10.0),
    }
}

#[test]
fn a_misspelling_is_marked_after_the_debounce_under_its_word() {
    let mut harness = fresh("marked");
    type_text(&mut harness, "the teh ");
    let typed = Instant::now();
    type_text(&mut harness, "cat");
    let landed = settle_until(&mut harness, |h| marks(h) == 1);
    assert!(
        landed.duration_since(typed) >= DEBOUNCE,
        "marked after {:?}, before the debounce",
        landed.duration_since(typed)
    );
    assert_eq!(paragraph(&harness), "the teh cat");
    let mark = harness.rect(".ds-spell-mark").expect("the mark");
    let line = harness.rect("#p0").expect("the paragraph");
    let left = mark.origin.x.0 - line.origin.x.0;
    assert!(
        (20.0..45.0).contains(&left),
        "the mark starts at the second word: {left}"
    );
    assert!(
        (15.0..40.0).contains(&mark.size.width.0),
        "one short word wide: {mark:?}"
    );
    let bottom = mark.origin.y.0 + mark.size.height.0 - line.origin.y.0;
    assert!(
        (14.0..=21.0).contains(&bottom),
        "the dots sit under the glyphs: {bottom}"
    );
    assert_eq!(
        harness
            .attr("#editor > .ds-spell-layer", "aria-hidden")
            .as_deref(),
        Some("true"),
        "the layer is the surface's own and hidden from assistive tech"
    );
}

#[test]
fn the_word_being_typed_is_not_marked_until_the_caret_leaves_it() {
    let mut harness = fresh("typing");
    type_text(&mut harness, "teh");
    harness.advance(DEBOUNCE * 2);
    assert_eq!(marks(&harness), 0, "the caret is still on the word");
    type_text(&mut harness, " ");
    settle_until(&mut harness, |h| marks(h) == 1);
}

#[test]
fn a_picked_suggestion_replaces_the_word_and_undo_restores_it() {
    let mut harness = fresh("replace");
    type_text(&mut harness, "teh cat");
    settle_until(&mut harness, |h| marks(h) == 1);
    let at = on_mark(&harness);
    harness.press(at, PointerButton::Secondary);
    menu_open(&mut harness);
    assert_eq!(
        harness.text_of(".ds-menu-item").as_deref().map(str::trim),
        Some("the"),
        "the best suggestion first"
    );
    let first = harness.centre(".ds-menu-item").expect("a suggestion");
    harness.click(first);
    settle_until(&mut harness, |h| paragraph(h) == "the cat");
    settle_until(&mut harness, |h| marks(h) == 0 && h.is_focused("#editor"));
    harness.chord(&[Key::Ctrl], Key::Char('z'));
    settle_until(&mut harness, |h| paragraph(h) == "teh cat");
    settle_until(&mut harness, |h| marks(h) == 1);
}

#[test]
fn ignore_spelling_unmarks_the_word() {
    let mut harness = fresh("ignore");
    type_text(&mut harness, "teh cat");
    settle_until(&mut harness, |h| marks(h) == 1);
    harness.press(on_mark(&harness), PointerButton::Secondary);
    menu_open(&mut harness);
    let ignore = menu_row(&harness, "Ignore Spelling");
    harness.click(ignore);
    settle_until(&mut harness, |h| {
        marks(h) == 0 && h.count(".ds-menu-item") == 0
    });
    type_text(&mut harness, " teh ");
    harness.advance(DEBOUNCE * 2);
    assert_eq!(marks(&harness), 0, "ignored for the session");
    assert!(!dict_dir().join("user").join("en_US.dic").exists());
}

#[test]
fn the_context_menu_key_on_a_marked_word_learns_it() {
    let mut harness = fresh("learn");
    type_text(&mut harness, "teh cat");
    settle_until(&mut harness, |h| marks(h) == 1);
    for _ in 0..5 {
        harness.key(Key::Left);
    }
    harness.key(Key::ContextMenu);
    menu_open(&mut harness);
    let learn = menu_row(&harness, "Learn Spelling");
    harness.click(learn);
    settle_until(&mut harness, |h| {
        marks(h) == 0 && h.count(".ds-menu-item") == 0
    });
    let file = dict_dir().join("user").join("en_US.dic");
    settle_until(&mut harness, |_| file.exists());
    assert_eq!(
        std::fs::read_to_string(&file).ok().as_deref(),
        Some("teh\n")
    );
}

#[test]
fn a_right_click_off_a_marked_word_opens_nothing() {
    let mut harness = fresh("unmarked");
    type_text(&mut harness, "teh cat");
    settle_until(&mut harness, |h| marks(h) == 1);
    let line = harness.rect("#p0").expect("the paragraph");
    let on_cat = Point {
        x: Px(line.origin.x.0 + 44.0),
        y: Px(line.origin.y.0 + 10.0),
    };
    harness.press(on_cat, PointerButton::Secondary);
    harness.advance(Duration::from_millis(200));
    assert_eq!(harness.count(".ds-menu-item"), 0);
}

/// Wait for the menu to open and finish its entrance (a row under a scaling menu is not where
/// its rect says until the entrance ends).
fn menu_open(harness: &mut Harness) {
    settle_until(harness, |h| h.count(".ds-menu-item") > 0);

    settle_until(harness, |h| {
        h.centre(".ds-menu-item:last-child")
            .is_some_and(|at| h.hits(at, ".ds-menu-item:last-child"))
    });
}

/// The centre of the menu row reading `title`: Ignore is the last row but one, Learn the last.
fn menu_row(harness: &Harness, title: &str) -> Point {
    let selector = match title {
        "Ignore Spelling" => ".ds-menu-item:nth-last-child(2)",
        _ => ".ds-menu-item:last-child",
    };
    assert_eq!(
        harness.text_of(selector).as_deref().map(str::trim),
        Some(title),
        "{}",
        harness.html()
    );
    harness.centre(selector).expect("the row")
}
