//! An [`EditSurface`](crate::EditSurface)'s spellchecking (design/04-COMPONENTS.md section 50):
//! after each input and each caret move, a frame later the paragraphs are read again and the
//! marks follow the edit ([`refresh`]); once typing has paused for `DelayToken::SpellDebounce`
//! the paragraphs whose text changed since their last check are checked on the host's worker
//! ([`check`]); then the marks' boxes are measured and drawn ([`draw`]). None of it re-renders
//! the app's content: the state is in cells, and only the layer reads the boxes.

use crate::appearance::MotionLevel;
use crate::edit::host::{HostEdit, Probe};
use crate::edit::position::{EditNode, TextPosition};
use crate::geometry::measure::{BUSY_ATTEMPTS, client_rect};
use crate::geometry::{Point, Rect};
use crate::root::Env;
use crate::spell::host::{HostSpell, Paragraph};
use crate::spell::lang::{Lang, Spell};
use crate::spell::marks::{Edit, Misspelt, Typing, marks_for, reconcile, shown, typing_after};
use crate::spell::words::words;
use crate::task::{spawn_in, try_set_if_changed};
use crate::time::{FRAME_SLACK, sleep};
use crate::tokens::DelayToken;
use dioxus::core::{ScopeId, Task, current_scope_id};
use dioxus::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// What the checker remembers between rounds, none of it drawn.
#[derive(Default)]
pub(crate) struct SpellState {
    pub(crate) spell: RefCell<Spell>,
    pub(crate) caret: RefCell<Option<TextPosition>>,
    /// Each paragraph's text at the last read, and at its last check.
    seen: RefCell<HashMap<EditNode, String>>,
    checked: RefCell<HashMap<EditNode, String>>,
    pub(crate) marks: RefCell<Vec<Misspelt>>,
    typing: RefCell<Option<Typing>>,
    pub(crate) surface: RefCell<Option<Rc<MountedData>>>,
    pub(crate) layer: RefCell<Option<Rc<MountedData>>>,
    pending: Cell<Option<Task>>,
}

/// A surface's checker: its memory, the hosts, and what the layer draws.
#[derive(Clone)]
pub(crate) struct SpellCtx {
    pub(crate) state: Rc<SpellState>,
    pub(crate) host: Option<HostSpell>,
    pub(crate) edit: Option<HostEdit>,
    /// The marks' boxes, from the layer's corner.
    pub(crate) boxes: Signal<Vec<Rect>>,
    pub(crate) level: MotionLevel,
    /// The surface's scope: every task the checker starts is the surface's, so a menu that
    /// closes (its handler started the work) cannot cancel it.
    pub(crate) scope: ScopeId,
}

impl SpellCtx {
    /// The checker for the calling surface.
    pub(crate) fn use_new(edit: Option<HostEdit>) -> SpellCtx {
        SpellCtx {
            state: use_hook(|| Rc::new(SpellState::default())),
            host: use_hook(try_consume_context::<HostSpell>),
            edit,
            boxes: use_signal(Vec::new),
            level: try_use_context::<Signal<Env>>()
                .map_or(MotionLevel::Standard, |env| env.peek().resolved.motion),
            scope: current_scope_id(),
        }
    }

    /// The host, when checking is on and there is one.
    pub(crate) fn live(&self) -> Option<&HostSpell> {
        match *self.state.spell.borrow() {
            Spell::On { .. } => self.host.as_ref(),
            Spell::Off => None,
        }
    }

    /// The languages this surface checks in: its own, else the host's.
    pub(crate) fn langs(&self) -> Vec<Lang> {
        match (&*self.state.spell.borrow(), &self.host) {
            (Spell::On { lang: Some(lang) }, _) => vec![lang.clone()],
            (Spell::On { lang: None }, Some(HostSpell(service))) => service.languages(),
            (Spell::On { lang: None }, None) | (Spell::Off, _) => Vec::new(),
        }
    }
}

/// The spell prop changed: forget every mark and check afresh (or stop).
pub(crate) fn respell(ctx: &SpellCtx, spell: Spell) {
    ctx.state.spell.replace(spell);
    ctx.state.marks.borrow_mut().clear();
    ctx.state.seen.borrow_mut().clear();
    ctx.state.checked.borrow_mut().clear();
    ctx.state.typing.replace(None);
    let _ = try_set_if_changed(ctx.boxes, Vec::new());
    touch(ctx);
}

/// The text or the caret may have changed: read, check and draw again, after the debounce.
pub(crate) fn touch(ctx: &SpellCtx) {
    if ctx.live().is_none() {
        return;
    }
    if let Some(pending) = ctx.state.pending.take() {
        pending.cancel();
    }
    let round = ctx.clone();
    ctx.state.pending.set(Some(spawn_in(ctx.scope, async move {
        sleep(FRAME_SLACK).await;
        if let Some(paragraphs) = read(&round).await {
            refresh(&round, &paragraphs);
            draw(&round).await;
        }
        sleep(
            DelayToken::SpellDebounce
                .delay(round.level)
                .saturating_sub(FRAME_SLACK),
        )
        .await;
        if let Some(paragraphs) = read(&round).await {
            refresh(&round, &paragraphs);
            check(&round, &paragraphs).await;
            draw(&round).await;
        }
    })));
}

/// The surface's paragraphs, a frame later while the document is busy.
async fn read(ctx: &SpellCtx) -> Option<Vec<Paragraph>> {
    let HostSpell(service) = ctx.live()?.clone();
    for _ in 0..BUSY_ATTEMPTS {
        let surface = ctx.state.surface.borrow().clone();
        match surface.map(|surface| service.paragraphs(&surface)) {
            Some(Probe::Found(paragraphs)) => return Some(paragraphs),
            Some(Probe::Busy) | None => sleep(FRAME_SLACK).await,
            Some(Probe::Unknown) => return None,
        }
    }
    None
}

/// Move the marks with the text, forget vanished paragraphs, and follow the word being typed.
fn refresh(ctx: &SpellCtx, paragraphs: &[Paragraph]) {
    let state = &ctx.state;
    let seen = state.seen.replace(
        paragraphs
            .iter()
            .map(|p| (p.node.clone(), p.text.clone()))
            .collect(),
    );
    let marks = state.marks.take();
    let moved = paragraphs.iter().flat_map(|paragraph| {
        let own: Vec<Misspelt> = marks
            .iter()
            .filter(|mark| mark.node == paragraph.node)
            .cloned()
            .collect();
        match seen.get(&paragraph.node) {
            Some(before) if *before != paragraph.text => reconcile(own, before, &paragraph.text),
            Some(_) | None => own,
        }
    });
    state.marks.replace(moved.collect());
    // A paragraph edited since the last read needs checking again, even if an undo brings it
    // back to the text its last check saw: that check's marks are gone.
    state.checked.borrow_mut().retain(|node, _| {
        paragraphs
            .iter()
            .any(|p| p.node == *node && seen.get(node) == Some(&p.text))
    });
    let caret = state.caret.borrow().clone();
    let at = caret
        .as_ref()
        .and_then(|caret| paragraphs.iter().find(|p| p.node == caret.node));
    let edit = match at.map(|p| seen.get(&p.node) == Some(&p.text)) {
        Some(true) | None => Edit::Same,
        Some(false) => Edit::Changed,
    };
    let typing = typing_after(
        state.typing.take(),
        caret.as_ref(),
        at.map(|p| p.text.as_str()),
        edit,
    );
    state.typing.replace(typing);
}

/// Check the paragraphs whose text changed since their last check, on the host's worker.
async fn check(ctx: &SpellCtx, paragraphs: &[Paragraph]) {
    let Some(HostSpell(service)) = ctx.live().cloned() else {
        return;
    };
    let changed: Vec<&Paragraph> = {
        let checked = ctx.state.checked.borrow();
        paragraphs
            .iter()
            .filter(|p| checked.get(&p.node) != Some(&p.text))
            .collect()
    };
    let cut: Vec<(&Paragraph, Vec<_>)> = changed
        .iter()
        .map(|p| (*p, words(&p.text, &p.skips)))
        .collect();
    let asked: HashSet<String> = cut
        .iter()
        .flat_map(|(p, spans)| spans.iter().filter_map(|s| p.text.get(s.start..s.end)))
        .map(str::to_owned)
        .collect();
    let wrong: HashSet<String> = match asked.is_empty() {
        true => HashSet::new(),
        false => service
            .check(ctx.langs(), asked.into_iter().collect())
            .await
            .into_iter()
            .collect(),
    };
    let mut marks = ctx.state.marks.borrow_mut();
    let mut checked = ctx.state.checked.borrow_mut();
    for (paragraph, spans) in cut {
        marks.retain(|mark| mark.node != paragraph.node);
        marks.extend(marks_for(&paragraph.node, &paragraph.text, &spans, &wrong));
        checked.insert(paragraph.node.clone(), paragraph.text.clone());
    }
}

/// Measure the shown marks from the layer's corner and hand the boxes to the layer.
pub(crate) async fn draw(ctx: &SpellCtx) {
    let (Some(edit), Some(surface), Some(layer)) = (
        ctx.edit,
        ctx.state.surface.borrow().clone(),
        ctx.state.layer.borrow().clone(),
    ) else {
        return;
    };
    let Some(corner) = client_rect(&layer).await else {
        return;
    };
    let shown: Vec<Misspelt> = {
        let marks = ctx.state.marks.borrow();
        let typing = ctx.state.typing.borrow();
        shown(&marks, typing.as_ref()).cloned().collect()
    };
    let mut boxes = Vec::new();
    for mark in shown {
        boxes.extend(
            rects(edit, &surface, &mark)
                .await
                .into_iter()
                .map(|rect| from_corner(rect, corner.origin)),
        );
    }
    let _ = try_set_if_changed(ctx.boxes, boxes);
}

/// A mark's boxes, one per line it spans.
async fn rects(edit: HostEdit, surface: &MountedData, mark: &Misspelt) -> Vec<Rect> {
    for _ in 0..BUSY_ATTEMPTS {
        match (edit.selection_rects)(surface, &mark.range()) {
            Probe::Found(rects) => return rects,
            Probe::Busy => sleep(FRAME_SLACK).await,
            Probe::Unknown => return Vec::new(),
        }
    }
    Vec::new()
}

fn from_corner(rect: Rect, corner: Point) -> Rect {
    Rect {
        origin: Point {
            x: rect.origin.x - corner.x,
            y: rect.origin.y - corner.y,
        },
        size: rect.size,
    }
}
