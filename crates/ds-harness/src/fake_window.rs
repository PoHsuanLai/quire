//! The window the harness's `WindowSizer` asks to resize. It records every request and answers
//! as the test chose ([`SizerAck`]), so a component that sizes its window runs in a test the way
//! it runs in a window: the answer is a resize that arrives later, never inside the request.

use crate::recorded_window::WindowHosting;
use crate::snapshot::Viewport;
use dioxus::core::spawn_forever;
use ds::prelude::Scale;
use ds_blitz::seam::SizedWindow;
use ds_blitz::{Extent, ScreenArea, ScreenOf, WindowSizer, WorkBasis};
use ds_core::time::clock::sleep;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// How the harness's window answers a request to resize it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SizerAck {
    /// With the requested size, on the next turn with no time passing.
    #[default]
    Now,
    /// With the requested size, this long after the request (on the harness's clock).
    After(Duration),
    /// Never: the compositor ignores the request.
    Never,
    /// At once, as Wayland does: the window has the size when the request returns, and nothing
    /// is sent but the return value. The host delivers the resize
    /// (`Headless::deliver_applied`), as the event loop does for winit's `Some(size)`.
    Applied,
}

/// The output the harness's window reports through `WindowSizer::screen`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowScreen {
    /// A 1920 x 1080 logical output at the viewport's scale.
    #[default]
    Standard,
    /// No output: the platform lists none.
    Absent,
    /// This one.
    Area(ScreenArea),
}

impl WindowScreen {
    /// The area this reports for a window drawn at `viewport`.
    fn area(self, viewport: Viewport) -> Option<ScreenArea> {
        let output = Extent::new(1920, 1080);
        match self {
            WindowScreen::Standard => Some(ScreenArea {
                output,
                work: output,
                scale: Scale::from_percent(viewport.scale_percent),
                of: ScreenOf::Window,
                basis: WorkBasis::WholeOutput,
            }),
            WindowScreen::Absent => None,
            WindowScreen::Area(area) => Some(area),
        }
    }
}

/// What the window is built from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) struct WindowSpec {
    pub ack: SizerAck,
    pub screen: WindowScreen,
    pub hosting: WindowHosting,
}

/// A size the window took at once (`SizerAck::Applied`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct Applied {
    pub logical: Extent,
    /// What the sizer hears as the resize.
    pub physical: Extent,
}

struct State {
    /// The window's content size, physical px.
    size: Extent,
    scale: Scale,
    ack: SizerAck,
    requests: Vec<Extent>,
    /// Sizes applied at once, for the host to deliver as resizes.
    applied: Vec<Applied>,
    screen: Option<ScreenArea>,
}

/// A window with no screen: the viewport's size, scale and the spec's answers.
#[derive(Clone)]
pub(crate) struct FakeWindow(Rc<RefCell<State>>);

impl FakeWindow {
    pub(crate) fn new(viewport: Viewport, spec: WindowSpec) -> FakeWindow {
        let scale = Scale::from_percent(viewport.scale_percent);
        FakeWindow(Rc::new(RefCell::new(State {
            size: physical(Extent::new(viewport.width, viewport.height), scale),
            scale,
            ack: spec.ack,
            requests: Vec::new(),
            applied: Vec::new(),
            screen: spec.screen.area(viewport),
        })))
    }

    /// The sizes asked for so far, logical px, oldest first.
    pub(crate) fn requests(&self) -> Vec<Extent> {
        self.0.borrow().requests.clone()
    }

    /// The sizes applied at once since the last take, oldest first.
    pub(crate) fn take_applied(&self) -> Vec<Applied> {
        std::mem::take(&mut self.0.borrow_mut().applied)
    }

    /// The window is now `logical` px, as a person's drag leaves it: the size changes and the
    /// sizer hears the resize.
    pub(crate) fn resize(&self, logical: Extent, sizer: &WindowSizer) {
        let size = physical(logical, self.0.borrow().scale);
        self.0.borrow_mut().size = size;
        sizer.resized(size);
    }
}

/// `logical` px as physical ones at `scale`, rounded to the nearest.
fn physical(logical: Extent, scale: Scale) -> Extent {
    let per = |px: u32| (f64::from(px) * scale.as_f64()).round() as u32;
    Extent::new(per(logical.width), per(logical.height))
}

impl SizedWindow for FakeWindow {
    fn surface_size(&self) -> Extent {
        self.0.borrow().size
    }

    fn scale_factor(&self) -> f64 {
        self.0.borrow().scale.as_f64()
    }

    fn request_surface_size(&self, logical: Extent, answer_to: &WindowSizer) {
        let (ack, answer) = {
            let mut state = self.0.borrow_mut();
            state.requests.push(logical);
            (state.ack, physical(logical, state.scale))
        };
        let delay = match ack {
            SizerAck::Never => return,
            SizerAck::Applied => {
                let mut state = self.0.borrow_mut();
                state.size = answer;
                state.applied.push(Applied {
                    logical,
                    physical: answer,
                });
                return;
            }
            SizerAck::Now => Duration::ZERO,
            SizerAck::After(delay) => delay,
        };
        let (window, sizer) = (self.clone(), answer_to.clone());
        spawn_forever(async move {
            if !delay.is_zero() {
                sleep(delay).await;
            }
            window.0.borrow_mut().size = answer;
            sizer.resized(answer);
        });
    }

    fn screen(&self) -> Option<ScreenArea> {
        self.0.borrow().screen
    }
}
