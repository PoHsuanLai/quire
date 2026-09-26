//! The drag itself, as a pure state machine: what it carries, where the pointer is, and when
//! the files land. The host feeds it through [`HostFileDrop`](crate::HostFileDrop); a target's
//! view and the window's acceptance follow from it and from the target under the pointer.

use crate::components::vocab::DropState;
use crate::file_drop::drag::{DropAcceptance, FileDrag, FileDragInput, FileDrop, Offer};
use crate::geometry::Point;
use std::path::PathBuf;

/// What the drag carries, as far as the host knows.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum Contents {
    /// The platform has not said yet.
    #[default]
    Waiting,
    /// These files, never an empty list.
    Files(Vec<PathBuf>),
    /// Not files.
    Other,
}

impl Contents {
    fn of(offer: Offer) -> Contents {
        match offer {
            Offer::Files(paths) if !paths.is_empty() => Contents::Files(paths),
            Offer::Files(_) | Offer::Other => Contents::Other,
        }
    }
}

/// A drag from outside the window.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum DragTrack {
    /// No drag is over the window.
    #[default]
    Idle,
    /// A drag is over the window, the pointer at `point` once the platform has said.
    Dragging {
        contents: Contents,
        point: Option<Point>,
    },
    /// Let go before the platform said what it carries: the files land when it does.
    Released { point: Option<Point> },
}

impl DragTrack {
    /// Where the pointer is while the drag is over the window.
    pub(crate) fn point(&self) -> Option<Point> {
        match self {
            DragTrack::Dragging { point, .. } => *point,
            DragTrack::Idle | DragTrack::Released { .. } => None,
        }
    }
}

/// The drag after one input, and the files that landed with it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Step {
    pub(crate) track: DragTrack,
    pub(crate) landed: Option<FileDrop>,
}

impl Step {
    fn to(track: DragTrack) -> Step {
        Step {
            track,
            landed: None,
        }
    }

    fn landing(paths: Vec<PathBuf>, point: Point) -> Step {
        Step {
            track: DragTrack::Idle,
            landed: Some(FileDrop { paths, point }),
        }
    }
}

/// The drag after `input`.
pub(crate) fn step(track: DragTrack, input: FileDragInput) -> Step {
    match (track, input) {
        (_, FileDragInput::Entered { point }) => Step::to(DragTrack::Dragging {
            contents: Contents::Waiting,
            point,
        }),
        (_, FileDragInput::Left) => Step::to(DragTrack::Idle),
        (DragTrack::Dragging { point, .. }, FileDragInput::Offered(offer)) => {
            Step::to(DragTrack::Dragging {
                contents: Contents::of(offer),
                point,
            })
        }
        (DragTrack::Dragging { contents, .. }, FileDragInput::Moved { point }) => {
            Step::to(DragTrack::Dragging {
                contents,
                point: Some(point),
            })
        }
        (DragTrack::Dragging { contents, point }, FileDragInput::Dropped) => {
            dropped(contents, point)
        }
        (DragTrack::Released { point }, FileDragInput::Offered(offer)) => {
            match (Contents::of(offer), point) {
                (Contents::Files(paths), Some(point)) => Step::landing(paths, point),
                _ => Step::to(DragTrack::Idle),
            }
        }
        (track, FileDragInput::Offered(_) | FileDragInput::Moved { .. })
        | (track, FileDragInput::Dropped) => Step::to(track),
    }
}

/// A release while dragging: the files land now, or once the platform says what they are.
fn dropped(contents: Contents, point: Option<Point>) -> Step {
    match (contents, point) {
        (Contents::Files(paths), Some(point)) => Step::landing(paths, point),
        (Contents::Waiting, point) => Step::to(DragTrack::Released { point }),
        (Contents::Files(_) | Contents::Other, _) => Step::to(DragTrack::Idle),
    }
}

/// What one target shows while the drag is `track`, the pointer `over` it or not: `Over` and
/// `DropState::Target` under the pointer, `DropState::Accepts` elsewhere while files are
/// dragged, nothing otherwise.
pub(crate) fn target_view(track: &DragTrack, over: Over) -> TargetView {
    match (track, over) {
        (
            DragTrack::Dragging {
                contents: Contents::Files(paths),
                point: Some(point),
            },
            Over::Yes,
        ) => TargetView {
            drag: FileDrag::Over {
                paths: paths.clone(),
                point: *point,
            },
            place: DropState::Target,
        },
        (
            DragTrack::Dragging {
                contents: Contents::Files(_),
                ..
            },
            _,
        ) => TargetView {
            drag: FileDrag::Idle,
            place: DropState::Accepts,
        },
        _ => TargetView::default(),
    }
}

/// Whether the window takes the drag with the pointer over a target or not.
pub(crate) fn acceptance(track: &DragTrack, over: Over) -> DropAcceptance {
    match (track, over) {
        (
            DragTrack::Dragging {
                contents: Contents::Waiting | Contents::Files(_),
                ..
            },
            Over::Yes,
        ) => DropAcceptance::Copy,
        _ => DropAcceptance::Refuse,
    }
}

/// Whether the pointer is over a given target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Over {
    Yes,
    No,
}

/// What one target shows.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct TargetView {
    pub(crate) drag: FileDrag,
    pub(crate) place: DropState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Px;

    fn at(x: f32, y: f32) -> Point {
        Point { x: Px(x), y: Px(y) }
    }

    fn files() -> Vec<PathBuf> {
        vec![PathBuf::from("/tmp/a.pdf"), PathBuf::from("/tmp/b.png")]
    }

    fn run(inputs: Vec<FileDragInput>) -> (DragTrack, Vec<FileDrop>) {
        inputs.into_iter().fold(
            (DragTrack::Idle, Vec::new()),
            |(track, mut landed), input| {
                let next = step(track, input);
                landed.extend(next.landed);
                (next.track, landed)
            },
        )
    }

    #[test]
    fn a_drag_ends_as_its_inputs_say() {
        use FileDragInput::{Dropped, Entered, Left, Moved, Offered};
        let cases: Vec<(&str, Vec<FileDragInput>, DragTrack, Vec<FileDrop>)> = vec![
            (
                "files dropped after moving",
                vec![
                    Entered { point: None },
                    Offered(Offer::Files(files())),
                    Moved {
                        point: at(5.0, 6.0),
                    },
                    Dropped,
                ],
                DragTrack::Idle,
                vec![FileDrop {
                    paths: files(),
                    point: at(5.0, 6.0),
                }],
            ),
            (
                "released before the paths came: they land when they do",
                vec![
                    Entered {
                        point: Some(at(1.0, 2.0)),
                    },
                    Dropped,
                    Offered(Offer::Files(files())),
                ],
                DragTrack::Idle,
                vec![FileDrop {
                    paths: files(),
                    point: at(1.0, 2.0),
                }],
            ),
            (
                "a URL is never dropped",
                vec![
                    Entered {
                        point: Some(at(1.0, 2.0)),
                    },
                    Offered(Offer::Other),
                    Dropped,
                ],
                DragTrack::Idle,
                vec![],
            ),
            (
                "an empty file list is not files",
                vec![
                    Entered {
                        point: Some(at(1.0, 2.0)),
                    },
                    Offered(Offer::Files(vec![])),
                    Dropped,
                ],
                DragTrack::Idle,
                vec![],
            ),
            (
                "left before the release",
                vec![
                    Entered { point: None },
                    Offered(Offer::Files(files())),
                    Moved {
                        point: at(5.0, 6.0),
                    },
                    Left,
                    Dropped,
                ],
                DragTrack::Idle,
                vec![],
            ),
            (
                "still over the window",
                vec![
                    Entered { point: None },
                    Moved {
                        point: at(5.0, 6.0),
                    },
                    Offered(Offer::Files(files())),
                ],
                DragTrack::Dragging {
                    contents: Contents::Files(files()),
                    point: Some(at(5.0, 6.0)),
                },
                vec![],
            ),
            (
                "released with no position ever given: nowhere to land",
                vec![
                    Entered { point: None },
                    Offered(Offer::Files(files())),
                    Dropped,
                ],
                DragTrack::Idle,
                vec![],
            ),
        ];
        for (name, inputs, track, landed) in cases {
            assert_eq!(run(inputs), (track, landed), "{name}");
        }
    }

    #[test]
    fn a_target_lights_only_for_files() {
        let over_files = DragTrack::Dragging {
            contents: Contents::Files(files()),
            point: Some(at(3.0, 4.0)),
        };
        let waiting = DragTrack::Dragging {
            contents: Contents::Waiting,
            point: Some(at(3.0, 4.0)),
        };
        let other = DragTrack::Dragging {
            contents: Contents::Other,
            point: Some(at(3.0, 4.0)),
        };
        let cases = [
            (
                &over_files,
                Over::Yes,
                FileDrag::Over {
                    paths: files(),
                    point: at(3.0, 4.0),
                },
                DropState::Target,
                DropAcceptance::Copy,
            ),
            (
                &over_files,
                Over::No,
                FileDrag::Idle,
                DropState::Accepts,
                DropAcceptance::Refuse,
            ),
            (
                &waiting,
                Over::Yes,
                FileDrag::Idle,
                DropState::Idle,
                DropAcceptance::Copy,
            ),
            (
                &other,
                Over::Yes,
                FileDrag::Idle,
                DropState::Idle,
                DropAcceptance::Refuse,
            ),
            (
                &DragTrack::Idle,
                Over::Yes,
                FileDrag::Idle,
                DropState::Idle,
                DropAcceptance::Refuse,
            ),
        ];
        for (track, over, drag, place, accept) in cases {
            assert_eq!(
                target_view(track, over),
                TargetView { drag, place },
                "{track:?} {over:?}"
            );
            assert_eq!(acceptance(track, over), accept, "{track:?} {over:?}");
        }
    }
}
