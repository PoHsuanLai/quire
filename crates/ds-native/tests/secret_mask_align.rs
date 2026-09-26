//! Q360: a secret typed into the polkit prompt draws its dots from the field's leading edge,
//! where Blitz puts the caret. The prompt's card centres its text, and the mask (a span across
//! the field) used to inherit that, so the dots began mid-field with the caret before them while
//! Blitz laid the hidden text and its caret out from the start. Read from the painted pixels:
//! every ink pixel in the field (the dots and the caret) lies in its leading third.

use dioxus::prelude::*;
use ds::{
    Appearance, AvatarFace, AvatarShape, AvatarSize, AvatarTone, Ds, Key, LockUser, Material,
    PolkitPrompt, Rect, person_hue,
};
use ds_native::{Harness, Viewport};
use image::RgbaImage;
use std::time::Duration;

const VIEW: Viewport = Viewport {
    width: 900,
    height: 420,
    scale_percent: 100,
};

/// What is typed: eight characters, wider hidden than their dots, as in sill's capture.
const TYPED: &str = "hunter2x";

#[allow(non_snake_case)]
fn Polkit() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Sheet,
            // The root is as tall as its content; the sheet is drawn over this.
            div { style: "height:400px" }
            PolkitPrompt {
                action: "Authentication is required to run the sill test program",
                user: LockUser::new(
                    "pohsuanlai",
                    AvatarFace {
                        initial: 'P',
                        size: AvatarSize::Size34,
                        tone: AvatarTone::Person(person_hue("pohsuanlai")),
                        shape: AvatarShape::Round,
                    },
                ),
                oninput: move |_: String| {},
                onsubmit: move |_: String| {},
                oncancel: move |()| {},
            }
        }
    }
}

/// Whether a pixel is the field's ink (the dots, the caret): dark and grey, so neither the
/// field's ground nor its accent focus ring.
fn inked(pixel: image::Rgba<u8>) -> bool {
    let [r, g, b, _] = pixel.0;
    let (hi, lo) = (r.max(g).max(b), r.min(g).min(b));
    hi < 110 && hi - lo < 30
}

/// The columns, from the field's left edge, holding ink along its middle, inside its border
/// and focus ring.
fn ink_columns(shot: &RgbaImage, field: Rect) -> Vec<u32> {
    let inset = 4.0;
    let left = field.origin.x.0 + inset;
    let right = field.origin.x.0 + field.size.width.0 - inset;
    let top = field.origin.y.0 + inset;
    let bottom = field.origin.y.0 + field.size.height.0 - inset;
    let to_px = |v: f32| v.round() as u32;
    (to_px(left)..to_px(right))
        .filter(|&x| (to_px(top)..to_px(bottom)).any(|y| inked(*shot.get_pixel(x, y))))
        .map(|x| x - to_px(field.origin.x.0))
        .collect()
}

#[test]
fn a_typed_secret_draws_its_dots_from_the_leading_edge_in_a_centred_card() {
    let mut harness = Harness::new(Polkit, VIEW);
    harness.advance(Duration::from_millis(600));
    for c in TYPED.chars() {
        harness.key(Key::Char(c));
        harness.advance(Duration::from_millis(20));
    }
    assert_eq!(
        harness
            .text_of(".ds-polkit .ds-input-mask")
            .map(|m| m.chars().count()),
        Some(TYPED.chars().count()),
        "one dot per character typed"
    );
    harness.advance(Duration::from_millis(300));
    let field = harness
        .rect(".ds-polkit input")
        .expect("the password field");
    let shot = harness.render().expect("a frame");
    let ink = ink_columns(&shot, field);
    let (first, last) = (
        ink.first().copied().expect("ink in the field"),
        ink.last().copied().expect("ink in the field"),
    );
    // The mask's text starts at the border and padding (1 + 11); a dot's side bearing adds a
    // pixel or two.
    assert!(
        first <= 16,
        "the first ink is {first} px into the field, not at its leading padding: {ink:?}"
    );
    let third = (field.size.width.0 / 3.0) as u32;
    assert!(
        last < third,
        "the last ink is {last} px into a {} px field, past its leading third: {ink:?}",
        field.size.width.0
    );
}
