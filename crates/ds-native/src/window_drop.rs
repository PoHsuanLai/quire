//! The window's side of a file drag: winit 0.31's data-transfer events become
//! `ds::FileDragInput`s, and `ds::HostFileDrop`'s answer goes back to the platform so the cursor
//! shows a copy over a target and a refusal elsewhere.
//!
//! winit reports a drag in four events plus the data (`DragEntered`, `DragPosition`,
//! `DragDropped`, `DragLeft`, `DataTransferReceived`), and a drag is refused until the app says
//! otherwise (`set_valid_dnd_actions`). The paths are not in the events: they are fetched as
//! `text/uri-list` when the drag enters (after the release Wayland no longer answers a fetch),
//! and arrive later. Only `file:` URIs count: a URL or text dragged out of a browser is
//! `Offer::Other`, refused, and never lights a target. blitz-shell ignores all of these events,
//! so the document never sees the drag; this hook, which hears every event first, is the only
//! listener.

use dioxus_native::winit::data_transfer::{DataTransferId, TypeHint};
use dioxus_native::winit::event::WindowEvent;
use dioxus_native::winit::event_loop::{ActiveEventLoop, DndAction};
use ds::{DropAcceptance, FileDragInput, Offer, Point, Px};
use std::io::ErrorKind;

/// The drag over this window, and what the platform was last told about it.
#[derive(Debug, Default)]
pub(crate) struct WindowDrop {
    transfer: Option<DataTransferId>,
    told: Option<DropAcceptance>,
}

impl WindowDrop {
    /// The inputs `event` makes, at `scale` device pixels per logical pixel. Entering asks the
    /// platform for the paths.
    pub(crate) fn inputs(
        &mut self,
        event: &WindowEvent,
        event_loop: &dyn ActiveEventLoop,
        scale: f64,
    ) -> Vec<FileDragInput> {
        let logical = |x: f64, y: f64| Point {
            x: Px((x / scale) as f32),
            y: Px((y / scale) as f32),
        };
        match event {
            WindowEvent::DragEntered { id, position } => {
                self.transfer = Some(*id);
                self.told = None;
                let entered = FileDragInput::Entered {
                    point: position.map(|p| logical(p.x, p.y)),
                };
                match fetch_paths(event_loop, *id) {
                    Fetch::Asked => vec![entered],
                    Fetch::NoFiles => vec![entered, FileDragInput::Offered(Offer::Other)],
                }
            }
            WindowEvent::DataTransferReceived { id, value, .. } if self.transfer == Some(*id) => {
                match value.try_as_file_paths() {
                    Ok(paths) => vec![FileDragInput::Offered(Offer::Files(paths))],
                    Err(error) if error.kind() == ErrorKind::WouldBlock => Vec::new(),
                    Err(_) => vec![FileDragInput::Offered(Offer::Other)],
                }
            }
            WindowEvent::DragPosition { id, position, .. } if self.transfer == Some(*id) => {
                vec![FileDragInput::Moved {
                    point: logical(position.x, position.y),
                }]
            }
            WindowEvent::DragDropped { id, .. } if self.transfer == Some(*id) => {
                vec![FileDragInput::Dropped]
            }
            WindowEvent::DragLeft { id } if self.transfer == Some(*id) => {
                self.transfer = None;
                vec![FileDragInput::Left]
            }
            _ => Vec::new(),
        }
    }

    /// Tell the platform whether the drag is taken where the pointer is, when that changed.
    pub(crate) fn answer(&mut self, event_loop: &dyn ActiveEventLoop, answer: DropAcceptance) {
        let Some(id) = self.transfer else {
            return;
        };
        if self.told == Some(answer) {
            return;
        }
        self.told = Some(answer);
        let actions: &[DndAction] = match answer {
            DropAcceptance::Copy => &[DndAction::Copy],
            DropAcceptance::Refuse => &[],
        };
        // A drag the platform has already finished (a release, a leave) cannot be answered,
        // and has nothing left to show.
        let _ = event_loop.set_valid_dnd_actions(id, actions);
    }
}

/// Whether the drag may carry files.
enum Fetch {
    /// It offers a URI list; its paths are on their way.
    Asked,
    /// It offers none, or the platform would not fetch it.
    NoFiles,
}

fn fetch_paths(event_loop: &dyn ActiveEventLoop, id: DataTransferId) -> Fetch {
    let offers_uris = event_loop
        .data_transfer(id)
        .is_ok_and(|transfer| transfer.has_type(&TypeHint::UriList));
    if offers_uris
        && event_loop
            .fetch_data_transfer(id, &TypeHint::UriList)
            .is_ok()
    {
        Fetch::Asked
    } else {
        Fetch::NoFiles
    }
}
