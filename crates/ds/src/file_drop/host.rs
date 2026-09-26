//! The host's side of a file drag: the targets the app registered, the drag the host feeds in,
//! and the host's own hit test through the document.

use crate::components::vocab::DropState;
use crate::file_drop::drag::{DropAcceptance, DropHit, FileDrag, FileDragInput, FileDrop};
use crate::file_drop::track::{DragTrack, Over, Step, TargetView, acceptance, step, target_view};
use crate::geometry::Point;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Finds the innermost of `targets` under `point`: the element there, or its nearest ancestor
/// that is one of them.
pub type DropHitTest = fn(targets: &[Rc<MountedData>], point: Point) -> DropHit;

/// The host's file-drag seam, provided as root context by ds-native (`launch`'s window and the
/// harness): every mounted [`use_file_drop`](crate::use_file_drop) target enters it, and the host
/// feeds it the drag as the platform reports it.
#[derive(Clone)]
pub struct HostFileDrop {
    board: Rc<RefCell<Board>>,
    hit: DropHitTest,
}

impl std::fmt::Debug for HostFileDrop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HostFileDrop").finish_non_exhaustive()
    }
}

/// The drag and the targets it may land on.
#[derive(Default)]
struct Board {
    track: DragTrack,
    /// The target the pointer was last found over (kept while the document is busy).
    under: Option<usize>,
    targets: Vec<DropTarget>,
}

/// One mounted target.
#[derive(Clone)]
pub(crate) struct DropTarget {
    /// The target's scope: the entry's key.
    pub(crate) owner: ScopeId,
    pub(crate) element: Rc<MountedData>,
    pub(crate) view: Signal<TargetView>,
    pub(crate) ondrop: Callback<FileDrop>,
}

impl HostFileDrop {
    /// A seam that finds targets with `hit`.
    pub fn new(hit: DropHitTest) -> Self {
        HostFileDrop {
            board: Rc::new(RefCell::new(Board::default())),
            hit,
        }
    }

    /// One step of the drag: every target's view follows, the target under a release hears its
    /// `ondrop`, and the answer is what the host tells the platform. Call it inside the app's
    /// runtime (the host's window hook, the harness's `within`).
    pub fn feed(&self, input: FileDragInput) -> DropAcceptance {
        let turn = match input {
            FileDragInput::Entered { .. } => Turn::NewDrag,
            _ => Turn::SameDrag,
        };
        let (landed, answer) = {
            let mut board = self.board.borrow_mut();
            let Step { track, landed } = step(std::mem::take(&mut board.track), input);
            let elements: Vec<_> = board
                .targets
                .iter()
                .map(|t| Rc::clone(&t.element))
                .collect();
            board.under = match track.point() {
                Some(point) => found(self.hit, &elements, point, board.under),
                None => None,
            };
            let landed = landed.and_then(|files| {
                let at = found(self.hit, &elements, files.point, None)?;
                Some((board.targets.get(at)?.clone(), files))
            });
            show(&board, &track, turn, landed.as_ref());
            let answer = acceptance(&track, board.under.map_or(Over::No, |_| Over::Yes));
            board.track = track;
            (landed, answer)
        };
        if let Some((target, files)) = landed {
            target.ondrop.call(files);
        }
        answer
    }

    /// Enter `target`, replacing any earlier entry of its scope.
    pub(crate) fn enter(&self, target: DropTarget) {
        let mut board = self.board.borrow_mut();
        board.targets.retain(|held| held.owner != target.owner);
        board.targets.push(target);
        board.under = None;
    }

    /// The target in `owner` has unmounted.
    pub(crate) fn leave(&self, owner: ScopeId) {
        let mut board = self.board.borrow_mut();
        board.targets.retain(|held| held.owner != owner);
        board.under = None;
    }
}

/// The target under `point`, or `before` while the document is busy.
fn found(
    hit: DropHitTest,
    elements: &[Rc<MountedData>],
    point: Point,
    before: Option<usize>,
) -> Option<usize> {
    if elements.is_empty() {
        return None;
    }
    match hit(elements, point) {
        DropHit::Target(at) => Some(at),
        DropHit::Nothing => None,
        DropHit::Busy => before,
    }
}

/// Whether a step starts a new drag: a target's `Dropped` lasts until one does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Turn {
    NewDrag,
    SameDrag,
}

/// Whether the target at `at` is the one `under` the pointer.
fn over_at(under: Option<usize>, at: usize) -> Over {
    match under {
        Some(under) if under == at => Over::Yes,
        _ => Over::No,
    }
}

/// Write every target's view. A target files landed on shows `Dropped` until the next drag
/// enters.
fn show(board: &Board, track: &DragTrack, turn: Turn, landed: Option<&(DropTarget, FileDrop)>) {
    for (at, target) in board.targets.iter().enumerate() {
        let next = match landed {
            Some((on, files)) if on.owner == target.owner => TargetView {
                drag: FileDrag::Dropped {
                    paths: files.paths.clone(),
                    point: files.point,
                },
                place: DropState::Idle,
            },
            _ => target_view(track, over_at(board.under, at)),
        };
        let mut view = target.view;
        let kept = turn == Turn::SameDrag
            && landed.is_none()
            && next == TargetView::default()
            && matches!(view.peek().drag, FileDrag::Dropped { .. });
        if !kept && *view.peek() != next {
            view.set(next);
        }
    }
}
