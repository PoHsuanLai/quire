//! The spelling menu and the marks' layer of an [`EditSurface`](crate::EditSurface)
//! (design/04-COMPONENTS.md section 50). A right-click, or the context-menu key with the caret
//! on a marked word, opens quire's context [`Menu`] with up to five suggestions, then "Ignore
//! Spelling" (this session) and "Learn Spelling" (the user's dictionary). A suggestion reaches
//! the app as one [`SpellReplace`], which it applies as one undoable edit.

use crate::components::edit_surface_spell::{SpellCtx, draw, touch};
use crate::components::menu::{Menu, MenuKind};
use crate::components::menu_entry::{MenuEntry, MenuRow};
use crate::edit::host::Probe;
use crate::geometry::{Anchor, Point, Rect};
use crate::spell::host::{HostSpell, Learned};
use crate::spell::marks::{Misspelt, SpellReplace};
use crate::task::{spawn_in, try_set};
use dioxus::prelude::*;
use std::rc::Rc;

/// How many suggestions the menu lists, at most (the reference's count).
pub const SPELL_SUGGESTIONS: usize = 5;

/// What the menu's rows do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SpellPick {
    /// Replace the word with this.
    Replace(String),
    /// Accept the word until the process ends.
    Ignore,
    /// Accept the word from now on.
    Learn,
}

/// The menu while it is open: the word it is about, where, and its suggestions.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Opened {
    pub(crate) mark: Misspelt,
    pub(crate) at: Point,
    pub(crate) suggestions: Vec<String>,
}

/// Whether a right-click or the menu key opened the spelling menu (the surface keeps the event)
/// or was not on a marked word (it goes on to the app).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Asked {
    Opened,
    Passed,
}

/// A right-click on the surface: open the menu when it landed on a marked word.
pub(crate) fn at_pointer(ctx: &SpellCtx, menu: Signal<Option<Opened>>, at: Point) -> Asked {
    if ctx.live().is_none() {
        return Asked::Passed;
    }
    let (Some(edit), Some(surface)) = (ctx.edit, ctx.state.surface.borrow().clone()) else {
        return Asked::Passed;
    };
    match (edit.hit_test)(&surface, at) {
        Probe::Found(position) => open(ctx, menu, |mark| mark.holds(&position), |_| Some(at)),
        Probe::Busy | Probe::Unknown => Asked::Passed,
    }
}

/// The context-menu key: open the menu under the caret's word when it is marked.
pub(crate) fn at_caret(ctx: &SpellCtx, menu: Signal<Option<Opened>>) -> Asked {
    let Some(caret) = ctx.state.caret.borrow().clone() else {
        return Asked::Passed;
    };
    let (Some(edit), Some(surface)) = (ctx.edit, ctx.state.surface.borrow().clone()) else {
        return Asked::Passed;
    };
    let below = |_: &Misspelt| match (edit.caret_rect)(&surface, &caret) {
        Probe::Found(rect) => Some(bottom_left(rect)),
        Probe::Busy | Probe::Unknown => None,
    };
    open(ctx, menu, |mark| mark.holds(&caret), below)
}

fn bottom_left(rect: Rect) -> Point {
    Point {
        x: rect.origin.x,
        y: rect.origin.y + rect.size.height,
    }
}

/// Find the marked word `on` accepts; if there is one, ask for its suggestions and open the menu
/// at `place(mark)`.
fn open(
    ctx: &SpellCtx,
    menu: Signal<Option<Opened>>,
    on: impl Fn(&Misspelt) -> bool,
    place: impl FnOnce(&Misspelt) -> Option<Point>,
) -> Asked {
    let Some(HostSpell(service)) = ctx.live().cloned() else {
        return Asked::Passed;
    };
    let Some(mark) = ctx
        .state
        .marks
        .borrow()
        .iter()
        .find(|mark| on(mark))
        .cloned()
    else {
        return Asked::Passed;
    };
    let Some(at) = place(&mark) else {
        return Asked::Passed;
    };
    let langs = ctx.langs();
    spawn_in(ctx.scope, async move {
        let mut suggestions = service.suggest(langs, mark.word.clone()).await;
        suggestions.truncate(SPELL_SUGGESTIONS);
        let opened = Opened {
            mark,
            at,
            suggestions,
        };
        let _ = try_set(menu, Some(opened));
    });
    Asked::Opened
}

/// The rows: the suggestions (or "No Guesses Found"), a rule, Ignore and Learn.
fn entries(opened: &Opened, replaces: Replaces) -> Vec<MenuEntry<SpellPick>> {
    let guesses: Vec<MenuEntry<SpellPick>> = match (replaces, opened.suggestions.is_empty()) {
        (Replaces::Yes, false) => opened
            .suggestions
            .iter()
            .map(|word| MenuRow::new(SpellPick::Replace(word.clone()), word.clone()).into())
            .collect(),
        (Replaces::Yes, true) | (Replaces::No, _) => vec![MenuEntry::Info {
            title: "No Guesses Found".to_owned(),
            detail: None,
        }],
    };
    guesses
        .into_iter()
        .chain([
            MenuEntry::Separator,
            MenuRow::new(SpellPick::Ignore, "Ignore Spelling").into(),
            MenuRow::new(SpellPick::Learn, "Learn Spelling").into(),
        ])
        .collect()
}

/// Whether the app takes replacements (it passed `on_replace`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Replaces {
    Yes,
    No,
}

/// Do what a row says.
fn picked(
    ctx: &SpellCtx,
    mark: &Misspelt,
    pick: SpellPick,
    on_replace: Option<EventHandler<SpellReplace>>,
) {
    let Some(HostSpell(service)) = ctx.live().cloned() else {
        return;
    };
    match pick {
        // The mark goes when the next read finds the word changed, so the checker has seen the
        // replacement before an undo brings the word back.
        SpellPick::Replace(text) => {
            if let Some(on_replace) = on_replace {
                on_replace.call(SpellReplace {
                    range: mark.range(),
                    text,
                });
            }
            touch(ctx);
        }
        SpellPick::Ignore => {
            service.ignore(mark.word.clone());
            forget(ctx, &mark.word);
        }
        SpellPick::Learn => {
            if let Some(lang) = ctx.langs().into_iter().next() {
                let learning = service.learn(lang, mark.word.clone());
                spawn_in(ctx.scope, async move {
                    let _: Learned = learning.await;
                });
            }
            forget(ctx, &mark.word);
        }
    }
}

/// Unmark every copy of `word` and redraw.
fn forget(ctx: &SpellCtx, word: &str) {
    ctx.state
        .marks
        .borrow_mut()
        .retain(|mark| mark.word != word);
    let drawing = ctx.clone();
    spawn_in(ctx.scope, async move { draw(&drawing).await });
}

/// The marks' layer and the menu while it is open, last in a surface with spelling on.
#[component]
pub(crate) fn SpellLayer(
    boxes: Signal<Vec<Rect>>,
    menu: Signal<Option<Opened>>,
    link: SpellLink,
) -> Element {
    let opened = menu();
    let state = Rc::clone(&link.ctx.state);
    let mounted = EventHandler::new(move |event: MountedEvent| {
        state.layer.replace(Some(event.data()));
    });
    rsx! {
        SpellMarks { boxes: boxes(), onmounted: mounted }
        if let Some(opened) = opened {
            {spell_menu(&link, menu, opened)}
        }
    }
}

/// Misspelling marks: a layer out of the flow that takes no pointer, holding one dotted
/// underline per box, each box placed from the layer's own corner (a line of a misspelt word:
/// its left, top, width and height). `EditSurface` draws it as its last child when spelling is
/// on; a host that checks spelling itself can draw the same marks over its own text.
#[component]
pub fn SpellMarks(
    boxes: Vec<Rect>,
    #[props(default)] onmounted: Option<EventHandler<MountedEvent>>,
) -> Element {
    rsx! {
        div {
            class: "ds-spell-layer",
            "aria-hidden": "true",
            onmounted: move |event| {
                if let Some(onmounted) = onmounted {
                    onmounted.call(event);
                }
            },
            for rect in boxes {
                span { class: "ds-spell-mark", style: "{mark_style(rect)}" }
            }
        }
    }
}

fn spell_menu(link: &SpellLink, mut menu: Signal<Option<Opened>>, opened: Opened) -> Element {
    let replaces = match link.on_replace {
        Some(_) => Replaces::Yes,
        None => Replaces::No,
    };
    let entries = entries(&opened, replaces);
    let (picking, closing) = (link.clone(), link.clone());
    let mark = opened.mark.clone();
    rsx! {
        Menu::<SpellPick> {
            kind: MenuKind::Context,
            anchor: Anchor::Point(opened.at),
            entries,
            onpick: move |pick| picked(&picking.ctx(), &mark, pick, picking.on_replace),
            onclose: move |()| {
                menu.set(None);
                closing.refocus.call(());
            },
        }
    }
}

/// How far below the middle of a line the dots sit, in the text's em: the lower half of the
/// glyph box (ascent and descent together are about 1.2 em in Inter), a little past the
/// descenders, where the reference draws them.
const BELOW_MIDDLE: &str = ".62em";

/// A mark's box: its line's left, top and width, tall enough that its bottom border falls just
/// under the glyphs whatever the line's height.
fn mark_style(rect: Rect) -> String {
    format!(
        "left:{:.2}px;top:{:.2}px;width:{:.2}px;height:calc({:.2}px + {BELOW_MIDDLE})",
        rect.origin.x.0,
        rect.origin.y.0,
        rect.size.width.0,
        rect.size.height.0 / 2.0
    )
}

/// What the layer needs from its surface: the checker, the app's replacement handler, and the
/// surface's own focus (the menu took the keyboard; picking or closing gives it back).
#[derive(Clone)]
pub(crate) struct SpellLink {
    pub(crate) ctx: SpellCtx,
    pub(crate) on_replace: Option<EventHandler<SpellReplace>>,
    pub(crate) refocus: EventHandler<()>,
}

impl SpellLink {
    fn ctx(&self) -> SpellCtx {
        self.ctx.clone()
    }
}

/// The same surface's link is the same link.
impl PartialEq for SpellLink {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.ctx.state, &other.ctx.state) && self.on_replace == other.on_replace
    }
}

#[cfg(test)]
mod tests {
    use super::{Opened, Replaces, SpellPick, entries};
    use crate::EditNode;
    use crate::components::menu_entry::MenuEntry;
    use crate::geometry::{Point, Px};
    use crate::spell::marks::Misspelt;
    use crate::spell::words::Span;

    fn opened(suggestions: &[&str]) -> Opened {
        Opened {
            mark: Misspelt {
                node: EditNode("p".to_owned()),
                span: Span::new(0, 3),
                word: "teh".to_owned(),
            },
            at: Point {
                x: Px(0.0),
                y: Px(0.0),
            },
            suggestions: suggestions.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    fn values(rows: &[MenuEntry<SpellPick>]) -> Vec<String> {
        rows.iter()
            .map(|row| match row {
                MenuEntry::Row(row) => format!("{:?}", row.value),
                MenuEntry::Info { title, .. } => title.clone(),
                MenuEntry::Separator => "---".to_owned(),
                other => format!("{other:?}"),
            })
            .collect()
    }

    #[test]
    fn the_menu_lists_suggestions_then_ignore_and_learn() {
        let rows = entries(&opened(&["the", "ten"]), Replaces::Yes);
        assert_eq!(
            values(&rows),
            vec![
                "Replace(\"the\")",
                "Replace(\"ten\")",
                "---",
                "Ignore",
                "Learn"
            ]
        );
        let none = entries(&opened(&[]), Replaces::Yes);
        assert_eq!(
            values(&none),
            vec!["No Guesses Found", "---", "Ignore", "Learn"]
        );
        let unreplaceable = entries(&opened(&["the"]), Replaces::No);
        assert_eq!(values(&unreplaceable)[0], "No Guesses Found");
    }
}
