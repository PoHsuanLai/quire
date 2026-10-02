//! What a test hands a [`Driver`](crate::Driver): one [`Input`] value per thing a person or the
//! platform does, so a test reads as a list of inputs and the driver is the only place that
//! knows how each reaches the document.

use crate::harness_input::modifier;
use ds::file_drop::drag::FileDragInput;
use ds::host::gesture::Gesture;
use ds::prelude::*;
use ds_core::press::PointerButton;
use keyboard_types::Modifiers;

/// One thing done to the document under test.
#[derive(Debug, Clone, PartialEq)]
pub enum Input {
    /// The mouse or touchpad.
    Pointer(PointerInput),
    /// A key press and release.
    Key(KeyInput),
    /// A scroll delta at `at`; a positive `dx` is content moving right, a positive `dy` content
    /// moving down (winit's sign, which Blitz forwards unchanged). The pointer is moved to `at`
    /// first, since Blitz scrolls the element under the pointer.
    Wheel {
        /// Where the pointer is.
        at: Point,
        /// Horizontal delta.
        dx: Px,
        /// Vertical delta.
        dy: Px,
    },
    /// A touchpad gesture (a pinch, or a scroll with its phase), as the window publishes it to the
    /// components listening with `use_gestures`. A [`Input::Wheel`] publishes the scroll
    /// itself; this is for what Blitz has no event for.
    Gesture(Gesture),
    /// One step of a file drag from outside the window.
    FileDrag(FileDragInput),
    /// The input method.
    Ime(ImeInput),
    /// `html` and its plain `text` put on the clipboard as a browser's copy would, then Ctrl+V
    /// where the focus is.
    Paste {
        /// The rich flavour.
        html: String,
        /// The plain-text flavour.
        text: String,
    },
}

/// A pointer action, where it happens and the modifiers held while it does.
#[derive(Debug, Clone, PartialEq)]
pub struct PointerInput {
    /// Where the pointer is, or where the action starts.
    pub at: Point,
    /// What the pointer does.
    pub action: PointerAction,
    /// The modifiers held.
    pub mods: Modifiers,
}

/// What the pointer does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PointerAction {
    /// Move to the point, with whatever buttons are down (a drag while one is). A link in a
    /// frame that the pointer comes onto or leaves is reported as the window reports it.
    Move,
    /// Press the button; it stays down until [`PointerAction::Up`]. A pointer press makes the
    /// modality `pointer`.
    Down(PointerButton),
    /// Release the button.
    Up(PointerButton),
    /// Move to the point, press the button and release it: the order a host synthesises, so
    /// hover intent arms before the press. A right click is a `Secondary` click, which Blitz
    /// delivers as `contextmenu`.
    Click(PointerButton),
    /// Press the primary button at the point, move to `to` in `steps` even steps with it held,
    /// and release it there: a drag selection.
    Drag {
        /// Where the drag ends.
        to: Point,
        /// How many moves it takes to get there (at least one).
        steps: u16,
    },
}

/// A key pressed and released with the focus where it is; a key makes the modality `keyboard`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInput {
    /// The key.
    pub key: ShortcutKey,
    /// The modifiers held while it is pressed.
    pub mods: Modifiers,
}

/// The input method attaching, composing and detaching, as winit reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeInput {
    /// The IME attaches to the focused surface (`Ime::Enabled`); a composition starts with the
    /// first [`ImeInput::Update`].
    Start,
    /// The IME shows `text` as its preedit, with its cursor at byte `cursor` of it.
    Update {
        /// The preedit text.
        text: String,
        /// The cursor's byte offset in it.
        cursor: usize,
    },
    /// The IME commits `text`, clearing its preedit first as winit does.
    Commit(String),
    /// The IME detaches (`Ime::Disabled`).
    End,
}

impl PointerInput {
    /// `action` at `at` with no modifiers held.
    pub fn new(at: Point, action: PointerAction) -> Self {
        PointerInput {
            at,
            action,
            mods: Modifiers::empty(),
        }
    }

    /// This input with `mods` held (Shift+click is `Modifiers::SHIFT`).
    pub fn with_mods(self, mods: Modifiers) -> Self {
        PointerInput { mods, ..self }
    }
}

impl Input {
    /// Move the pointer to `at`.
    pub fn pointer_move(at: Point) -> Self {
        Input::pointer(at, PointerAction::Move)
    }

    /// Press the primary button at `at`.
    pub fn pointer_down(at: Point) -> Self {
        Input::button_down(at, PointerButton::Primary)
    }

    /// Release the primary button at `at`.
    pub fn pointer_up(at: Point) -> Self {
        Input::button_up(at, PointerButton::Primary)
    }

    /// Press `button` at `at`.
    pub fn button_down(at: Point, button: PointerButton) -> Self {
        Input::pointer(at, PointerAction::Down(button))
    }

    /// Release `button` at `at`.
    pub fn button_up(at: Point, button: PointerButton) -> Self {
        Input::pointer(at, PointerAction::Up(button))
    }

    /// Click the primary button at `at`.
    pub fn click(at: Point) -> Self {
        Input::press(at, PointerButton::Primary)
    }

    /// Click `button` at `at`.
    pub fn press(at: Point, button: PointerButton) -> Self {
        Input::pointer(at, PointerAction::Click(button))
    }

    /// Drag the primary button from `from` to `to` in `steps` moves.
    pub fn drag(from: Point, to: Point, steps: u16) -> Self {
        Input::pointer(from, PointerAction::Drag { to, steps })
    }

    /// Press and release `key`.
    pub fn key(key: ShortcutKey) -> Self {
        Input::chord(&[], key)
    }

    /// Press and release `key` while `held` modifiers (`ShortcutKey::Ctrl`, `Shift`, `Alt`,
    /// `Super`) are down: `chord(&[ShortcutKey::Ctrl], ShortcutKey::Char('k'))` is Ctrl+K. A key
    /// in `held` that is not a modifier adds nothing.
    pub fn chord(held: &[ShortcutKey], key: ShortcutKey) -> Self {
        let mods = held
            .iter()
            .copied()
            .map(modifier)
            .fold(Modifiers::empty(), |all, one| all | one);
        Input::Key(KeyInput { key, mods })
    }

    /// Scroll by `dx`, `dy` with the pointer at `at`.
    pub fn wheel(at: Point, dx: Px, dy: Px) -> Self {
        Input::Wheel { at, dx, dy }
    }

    /// `gesture` published to the window's listeners.
    pub fn gesture(gesture: Gesture) -> Self {
        Input::Gesture(gesture)
    }

    /// The IME attaches.
    pub fn ime_start() -> Self {
        Input::Ime(ImeInput::Start)
    }

    /// The IME shows `text` as its preedit, cursor at byte `cursor`.
    pub fn ime_update(text: &str, cursor: usize) -> Self {
        Input::Ime(ImeInput::Update {
            text: text.to_owned(),
            cursor,
        })
    }

    /// The IME commits `text`.
    pub fn ime_commit(text: &str) -> Self {
        Input::Ime(ImeInput::Commit(text.to_owned()))
    }

    /// The IME detaches.
    pub fn ime_end() -> Self {
        Input::Ime(ImeInput::End)
    }

    /// Paste `html` with its plain `text`.
    pub fn paste(html: &str, text: &str) -> Self {
        Input::Paste {
            html: html.to_owned(),
            text: text.to_owned(),
        }
    }

    fn pointer(at: Point, action: PointerAction) -> Self {
        Input::Pointer(PointerInput::new(at, action))
    }
}
