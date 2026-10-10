//! `ds::prelude::DocumentHost` on Blitz, and [`provide_host`], the one call that gives a root all of it.
//!
//! The host is built from what a document's owner knows: whether the keyboard falls back to a
//! focusable ancestor (and the keeper that watches removals for it), how to find an element by
//! selector, which clipboard it has, and the listeners its window feeds the IME and pointer
//! capture to. A window and the harness own all of it; a shell's surface root has none of it,
//! and its host answers what a document alone can.

use crate::click_focus::{fallback, restore, take};
use crate::clipboard::{Clipboard, System};
use crate::drop_hit::drop_hit;
use crate::edit::set_ime_cursor_area;
use crate::edit::{capture, caret_rect, forget, hit_test, listen, selection_rects, set_ime};
use crate::edit_ime::EditListeners;
use crate::focus::{
    blur, caret, caret_owed, focus, focus_placing, place_caret, select_all, selection,
};
use crate::focus_keep::{FocusKeeper, hand_back_seam};
use crate::measure::measure;
use crate::node_ref::same;
use crate::phase::Phase;
use crate::reveal::reveal;
use dioxus::prelude::*;
use ds::host::captured::CapturedPointer;
use ds::host::caret::{Caret, CaretOwed, FieldSelection, InitialCaret};
use ds::host::drop_hit::DropHit;
use ds::host::fallback::Fallback;
use ds::host::found::{Found, SameNode};
use ds::host::hand_back::HandBack;
use ds::host::ime::{ImeEvent, ImeListener, ImeSwitch};
use ds::host::parts::{
    CaretHost, ClickFocusHost, EditHost, FileDropHost, FocusHost, GeometryHost, ImeHost,
};
use ds::host::pasted::Pasted;
use ds::host::phase::{Observe, Observed, PhaseWrite, Queued};
use ds::host::position::{TextPosition, TextRange};
use ds::host::probe::Probe;
use ds::host::reveal::Scrolled;
use ds::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// How a document finds an element by selector, once it has a document to search.
pub(crate) type FindDocument = Rc<dyn Fn(&str) -> Found>;

/// What a document's owner knows, and the host is made of.
pub struct Wiring {
    /// Under `FocusFallback::Ancestor`, the keeper that hands a removed element's keyboard on.
    pub keeper: Option<Rc<RefCell<FocusKeeper>>>,
    /// Selector lookup in the owner's document; absent where the owner cannot reach it.
    pub find: Option<FindDocument>,
    /// The clipboard the edit surfaces paste from.
    pub clipboard: Rc<dyn Clipboard>,
    /// The edit surfaces the owner's window feeds IME events and captured pointers to.
    pub listeners: EditListeners,
    /// The frame phase the owner runs after each layout; a phase nothing runs supports nothing,
    /// and components read through the host's other calls.
    pub phase: Phase,
}

impl std::fmt::Debug for Wiring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Wiring").finish_non_exhaustive()
    }
}

impl Wiring {
    /// What a document alone has: the renderer's focus fallback, no selector lookup, the
    /// desktop's clipboard once a shell reaches it, and listeners nothing feeds.
    fn standalone() -> Self {
        Wiring {
            keeper: None,
            find: None,
            clipboard: Rc::new(System::default()),
            listeners: EditListeners::default(),
            phase: Phase::default(),
        }
    }
}

/// The contexts a document's root is given, made together.
pub struct Provided {
    pub host: Rc<dyn DocumentHost>,
    pub clipboard: Rc<dyn Clipboard>,
}

impl std::fmt::Debug for Provided {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Provided").finish_non_exhaustive()
    }
}

impl Provided {
    /// The host and clipboard `wiring` describes.
    pub fn of(wiring: Wiring) -> Self {
        let clipboard = Rc::clone(&wiring.clipboard);
        Provided {
            host: Rc::new(BlitzHost::new(wiring)),
            clipboard,
        }
    }

    /// Provide both to the calling component's subtree and to the root scope, and answer the
    /// host. A task spawned on the root (an anchor's rect follow, which outlives the component
    /// that started it) reads its contexts from the root scope; with the host provided only
    /// below it, that task got no host, measured nothing, and a tooltip stayed hidden for want
    /// of its anchor in every real window (the harness wires its host on the root).
    pub fn provide(self) -> Rc<dyn DocumentHost> {
        dioxus::core::provide_root_context(Rc::clone(&self.clipboard));
        dioxus::core::provide_root_context(Rc::clone(&self.host));
        provide_context(self.clipboard);
        provide_context(self.host)
    }
}

/// The Blitz document host: every part, the optional ones present where the owner wired them.
struct BlitzHost {
    focus: BlitzFocus,
    geometry: BlitzGeometry,
    click_focus: Option<BlitzClickFocus>,
    edit: BlitzEdit,
    file_drop: BlitzFileDrop,
}

impl BlitzHost {
    fn new(wiring: Wiring) -> Self {
        let Wiring {
            keeper,
            find,
            clipboard,
            listeners,
            phase,
        } = wiring;
        BlitzHost {
            focus: BlitzFocus {
                hand_back: keeper
                    .clone()
                    .map_or_else(HandBack::default, hand_back_seam),
            },
            geometry: BlitzGeometry { find, phase },
            click_focus: keeper.map(|_| BlitzClickFocus),
            edit: BlitzEdit {
                clipboard,
                ime: BlitzIme(listeners.clone()),
                listeners,
            },
            file_drop: BlitzFileDrop,
        }
    }
}

/// Give the calling root the whole Blitz host: the focus, caret, geometry, edit and drop parts of
/// [`ds::prelude::DocumentHost`] and the clipboard, together, so no root holds a subset. Call it at the top
/// of a root `ds_blitz::launch` did not start (a shell's surface, a popup's document), before
/// any quire component reads the document. A root that already has a host (a window's, the
/// harness's) keeps it, so a component calling this under one takes what the window wired.
///
/// The host of a root like that follows the renderer's own focus fallback, has no lookup by
/// selector and no click-focus part, and its edit surfaces are fed no IME events: the window
/// loop that owns those is `launch`'s and the harness's. Nor does it reach a window shell, so
/// [`clipboard::write_text`](crate::clipboard::write_text) answers
/// [`ClipboardError::NoShell`](crate::clipboard::ClipboardError::NoShell) there, never a
/// success that wrote nothing.
pub fn provide_host() -> Rc<dyn DocumentHost> {
    use_hook(|| {
        try_consume_context::<Rc<dyn DocumentHost>>()
            .unwrap_or_else(|| Provided::of(Wiring::standalone()).provide())
    })
}

impl DocumentHost for BlitzHost {
    fn focus(&self) -> &dyn FocusHost {
        &self.focus
    }

    fn caret(&self) -> &dyn CaretHost {
        &BlitzCaret
    }

    fn geometry(&self) -> &dyn GeometryHost {
        &self.geometry
    }

    fn click_focus(&self) -> Option<&dyn ClickFocusHost> {
        self.click_focus
            .as_ref()
            .map(|part| part as &dyn ClickFocusHost)
    }

    fn edit(&self) -> Option<&dyn EditHost> {
        Some(&self.edit)
    }

    fn file_drop(&self) -> Option<&dyn FileDropHost> {
        Some(&self.file_drop)
    }
}

struct BlitzFocus {
    hand_back: HandBack,
}

impl FocusHost for BlitzFocus {
    fn focus(&self, el: &MountedData) -> Focused {
        focus(el)
    }

    fn blur(&self, el: &MountedData) -> Focused {
        blur(el)
    }

    fn select(&self, el: &MountedData) -> Focused {
        select_all(el)
    }

    fn focus_placing(&self, el: &MountedData, at: InitialCaret) -> Focused {
        focus_placing(el, at)
    }

    fn hand_back(&self) -> &HandBack {
        &self.hand_back
    }
}

struct BlitzCaret;

impl CaretHost for BlitzCaret {
    fn caret(&self, el: &MountedData) -> Caret {
        caret(el)
    }

    fn place_caret(&self, el: &MountedData, at: InitialCaret) -> Focused {
        place_caret(el, at)
    }

    fn selection(&self, el: &MountedData) -> FieldSelection {
        selection(el)
    }

    fn caret_owed(&self, el: &MountedData) -> CaretOwed {
        caret_owed(el)
    }
}

struct BlitzGeometry {
    find: Option<FindDocument>,
    phase: Phase,
}

impl GeometryHost for BlitzGeometry {
    fn measure(&self, el: &MountedData) -> Measured {
        measure(el)
    }

    fn reveal(&self, list: &MountedData, item: &MountedData) -> Scrolled {
        reveal(list, item)
    }

    fn find(&self, selector: &str) -> Found {
        self.find
            .as_ref()
            .map_or(Found::Unreachable, |find| find(selector))
    }

    fn same(&self, a: &MountedData, b: &MountedData) -> SameNode {
        same(a, b)
    }

    fn observe(&self, el: &MountedData, what: Observe) -> Observed {
        self.phase.observe(el, what)
    }

    fn write(&self, el: &MountedData, write: PhaseWrite) -> Queued {
        self.phase.write(el, write)
    }
}

struct BlitzClickFocus;

impl ClickFocusHost for BlitzClickFocus {
    fn fallback(&self, root: &MountedData) -> Fallback {
        fallback(root)
    }

    fn restore(&self, ancestor: &MountedData) -> Focused {
        restore(ancestor)
    }

    fn press(&self, root: &MountedData) -> Focused {
        take(root)
    }
}

struct BlitzEdit {
    clipboard: Rc<dyn Clipboard>,
    listeners: EditListeners,
    ime: BlitzIme,
}

impl EditHost for BlitzEdit {
    fn hit_test(&self, surface: &MountedData, at: Point) -> Probe<TextPosition> {
        hit_test(surface, at)
    }

    fn caret_rect(&self, surface: &MountedData, at: &TextPosition) -> Probe<Rect> {
        caret_rect(surface, at)
    }

    fn selection_rects(&self, surface: &MountedData, range: &TextRange) -> Probe<Vec<Rect>> {
        selection_rects(surface, range)
    }

    fn paste(&self) -> Option<Pasted> {
        self.clipboard.read_html()
    }

    fn capture(&self, surface: &MountedData, sink: EventHandler<CapturedPointer>) -> Probe<()> {
        capture(&self.listeners, surface, sink)
    }

    fn ime(&self) -> &dyn ImeHost {
        &self.ime
    }
}

struct BlitzIme(EditListeners);

impl ImeHost for BlitzIme {
    fn switch(&self, surface: &MountedData, to: ImeSwitch) -> Probe<()> {
        set_ime(surface, to)
    }

    fn cursor_area(&self, surface: &MountedData, area: Rect) -> Probe<()> {
        set_ime_cursor_area(surface, area)
    }

    fn listen(&self, surface: &MountedData, sink: EventHandler<ImeEvent>) -> Probe<ImeListener> {
        listen(&self.0, surface, sink)
    }

    fn forget(&self, listener: ImeListener) {
        forget(&self.0, listener);
    }
}

struct BlitzFileDrop;

impl FileDropHost for BlitzFileDrop {
    fn hit(&self, targets: &[Rc<MountedData>], at: Point) -> DropHit {
        drop_hit(targets, at)
    }
}
