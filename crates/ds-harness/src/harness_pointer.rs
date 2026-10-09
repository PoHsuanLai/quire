//! Delivering pointer and key inputs: the Blitz events a window would hand its document for
//! them, in the order it hands them.

use crate::harness::Harness;
use crate::harness_frames::Delivery;
use crate::harness_input::{blitz_button, keyboard, pointer};
use crate::input::{KeyInput, PointerAction, PointerInput, RawKeyInput, RawKeyPhase};
use blitz_kit::scroll::driver::KeyRepeat;
use blitz_kit::scroll::keys::scroll_key;
use blitz_traits::events::{BlitzKeyEvent, KeyState, MouseEventButton, UiEvent};
use ds::prelude::*;
use ds_core::press::PointerButton;
use ds_core::time::clock::now;
use ds_core::vocab::InputModality;
use keyboard_types::{Location, Modifiers};

impl Harness {
    /// Deliver one pointer input.
    pub(crate) fn pointer(&mut self, input: PointerInput) {
        let PointerInput { at, action, mods } = input;
        match action {
            PointerAction::Move => self.move_to(at, mods),
            PointerAction::Down(button) => self.button_down(at, button, mods),
            PointerAction::Up(button) => self.button_up(at, button, mods),
            PointerAction::Click(button) => {
                self.move_to(at, mods);
                self.button_down(at, button, mods);
                self.button_up(at, button, mods);
            }
            PointerAction::Drag { to, steps } => self.drag(at, to, steps, mods),
        }
    }

    /// Press and release one key.
    pub(crate) fn key(&mut self, input: KeyInput) {
        self.press_key(input, Delivery::Settled);
    }

    /// Press and release one key as [`Harness::key`] does, delivering each event as `delivery`.
    pub(crate) fn press_key(&mut self, input: KeyInput, delivery: Delivery) {
        self.doc.set_modality(InputModality::Keyboard);
        let (key, code) = keyboard(input.key);
        // The window's scrolling acts on a scroll key before the document hears it, and the
        // document hears it too.
        if let Some(scroll) = scroll_key(&key, input.mods) {
            self.doc.scroll.key(scroll, KeyRepeat::First, now());
            self.doc.scroll.key_up(now());
        }
        for state in [KeyState::Pressed, KeyState::Released] {
            let event = BlitzKeyEvent {
                key: key.clone(),
                key_without_modifiers: key.clone(),
                code,
                modifiers: input.mods,
                location: Location::Standard,
                is_auto_repeating: false,
                is_composing: false,
                state,
                text: None,
            };
            self.deliver_as(
                match state {
                    KeyState::Pressed => UiEvent::KeyDown(event),
                    KeyState::Released => UiEvent::KeyUp(event),
                },
                delivery,
            );
        }
    }

    /// Deliver one key event as given: no scroll step, no paired release, just what the window
    /// would hand the document for it.
    pub(crate) fn raw_key(&mut self, input: RawKeyInput) {
        self.doc.set_modality(InputModality::Keyboard);
        let state = match input.phase {
            RawKeyPhase::Down | RawKeyPhase::Repeat => KeyState::Pressed,
            RawKeyPhase::Up => KeyState::Released,
        };
        let event = BlitzKeyEvent {
            key: input.key.clone(),
            key_without_modifiers: input.key,
            code: input.code,
            modifiers: input.mods,
            location: input.location,
            is_auto_repeating: input.phase == RawKeyPhase::Repeat,
            is_composing: false,
            state,
            text: input.text.map(Into::into),
        };
        self.deliver_as(
            match state {
                KeyState::Pressed => UiEvent::KeyDown(event),
                KeyState::Released => UiEvent::KeyUp(event),
            },
            Delivery::Settled,
        );
    }

    fn move_to(&mut self, at: Point, mods: Modifiers) {
        self.deliver(UiEvent::PointerMove(pointer(
            at,
            MouseEventButton::Main,
            self.held_buttons().blitz(),
            mods,
        )));
        self.doc.hover_at(at);
    }

    fn button_down(&mut self, at: Point, button: PointerButton, mods: Modifiers) {
        self.doc.set_modality(InputModality::Pointer);
        self.held = self.held.with(button);
        let (which, _) = blitz_button(button);
        self.deliver(UiEvent::PointerDown(pointer(
            at,
            which,
            self.held_buttons().blitz(),
            mods,
        )));
    }

    fn button_up(&mut self, at: Point, button: PointerButton, mods: Modifiers) {
        self.held = self.held.without(button);
        let (which, _) = blitz_button(button);
        // The app hears a menu over a frame's link before the page does, as a window does
        if button == PointerButton::Secondary {
            self.doc.menu_at(at);
        }
        self.deliver(UiEvent::PointerUp(pointer(
            at,
            which,
            self.held_buttons().blitz(),
            mods,
        )));
    }

    fn drag(&mut self, from: Point, to: Point, steps: u16, mods: Modifiers) {
        self.move_to(from, mods);
        self.button_down(from, PointerButton::Primary, mods);
        let steps = steps.max(1);
        for step in 1..=steps {
            let part = f32::from(step) / f32::from(steps);
            self.move_to(
                Point {
                    x: Px(from.x.0 + (to.x.0 - from.x.0) * part),
                    y: Px(from.y.0 + (to.y.0 - from.y.0) * part),
                },
                mods,
            );
        }
        self.button_up(to, PointerButton::Primary, mods);
    }
}
