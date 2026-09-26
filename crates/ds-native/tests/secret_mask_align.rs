//! Q360: a secret typed into the polkit prompt draws its dots from the field's leading edge,
//! where Blitz puts the caret. The prompt's card centres its text, and the mask (a span across
//! the field) used to inherit that, so the dots began mid-field with the caret before them while
//! Blitz laid the hidden text and its caret out from the start. Read from the painted pixels:
//! every ink pixel in the field (the dots and the caret) lies in its leading third.
//!
//! Q360b: Blitz measures the hidden text in its editor's own face, untracked, so its caret fell
//! short of the Inter dots tracked .1em (two dots short at eleven). The field draws its own caret
//! among the dots: after N characters the painted caret sits at the end of the N-th dot's advance,
//! after Left x3 before the third dot from the end, and an unfocused field paints no caret.

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

/// The mask's leading padding (`--s-11`): where its first dot's advance starts.
const MASK_PAD: f32 = 11.0;

/// A column of ink at least this many pixels tall is the caret (a line high); a dot is a few.
const CARET_ROWS: usize = 10;

/// The field's middle band, inside its border and focus ring, in whole pixels.
struct Band {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
}

impl Band {
    fn of(field: Rect) -> Self {
        let inset = 4.0;
        let to_px = |v: f32| v.round() as u32;
        Band {
            left: to_px(field.origin.x.0 + inset),
            right: to_px(field.origin.x.0 + field.size.width.0 - inset),
            top: to_px(field.origin.y.0 + inset),
            bottom: to_px(field.origin.y.0 + field.size.height.0 - inset),
        }
    }

    /// How many ink pixels column `x` holds.
    fn ink_rows(&self, shot: &RgbaImage, x: u32) -> usize {
        (self.top..self.bottom)
            .filter(|&y| inked(*shot.get_pixel(x, y)))
            .count()
    }

    /// How dark column `x` is against the field's ground: its darkness summed down the band.
    fn weight(&self, shot: &RgbaImage, x: u32, ground: f32) -> f32 {
        (self.top..self.bottom)
            .map(|y| (ground - luma(*shot.get_pixel(x, y))).max(0.0))
            .sum()
    }

    /// The darkness-weighted centre of columns `from..=to`.
    fn centre(&self, shot: &RgbaImage, from: u32, to: u32, ground: f32) -> f32 {
        let (mut sum, mut total) = (0.0, 0.0);
        for x in from..=to {
            let w = self.weight(shot, x, ground);
            sum += w * (x as f32 + 0.5);
            total += w;
        }
        sum / total
    }
}

fn luma(pixel: image::Rgba<u8>) -> f32 {
    let [r, g, b, _] = pixel.0;
    (u32::from(r) + u32::from(g) + u32::from(b)) as f32 / 3.0
}

/// The painted caret's left edge in page pixels (its ink's centre less half its 1.5 px), or
/// `None` when no column of the field holds a line of ink.
fn caret_left(shot: &RgbaImage, field: Rect) -> Option<f32> {
    let band = Band::of(field);
    let tall: Vec<u32> = (band.left..band.right)
        .filter(|&x| band.ink_rows(shot, x) >= CARET_ROWS)
        .collect();
    let (first, last) = (*tall.first()?, *tall.last()?);
    assert!(last - first <= 2, "one caret, not {tall:?}");
    let ground = luma(*shot.get_pixel(band.right - 2, band.top + 2));
    // Only the caret's rows above and below the dots, so an adjacent dot adds no weight.
    let caret = Band {
        top: band.top,
        bottom: band.top + (band.bottom - band.top) / 4,
        ..band
    };
    let from = first.saturating_sub(1);
    Some(caret.centre(shot, from, last + 1, ground) - 0.75)
}

/// Each dot's ink, left to right: `(left column, centre)` in page pixels.
fn dots(shot: &RgbaImage, field: Rect) -> Vec<(u32, f32)> {
    let band = Band::of(field);
    let ground = luma(*shot.get_pixel(band.right - 2, band.top + 2));
    let columns: Vec<u32> = (band.left..band.right)
        .filter(|&x| band.ink_rows(shot, x) > 0)
        .collect();
    let mut runs: Vec<(u32, u32)> = Vec::new();
    for x in columns {
        match runs.last_mut() {
            Some((_, end)) if *end + 1 == x => *end = x,
            _ => runs.push((x, x)),
        }
    }
    runs.into_iter()
        .map(|(from, to)| {
            (
                from,
                band.centre(shot, from.saturating_sub(1), to + 1, ground),
            )
        })
        .collect()
}

/// A polkit prompt with `typed` in its field and the keys `after` pressed, settled.
fn typed_into(typed: &str, after: &[Key]) -> Harness {
    let mut harness = Harness::new(Polkit, VIEW);
    harness.advance(Duration::from_millis(600));
    for c in typed.chars() {
        harness.key(Key::Char(c));
        harness.advance(Duration::from_millis(20));
    }
    for &key in after {
        harness.key(key);
        harness.advance(Duration::from_millis(20));
    }
    harness.advance(Duration::from_millis(300));
    harness
}

/// Where the field's dots are once it has lost the keyboard (and whether a caret still paints):
/// the dots unfocused are where they were, with no caret among them.
fn blurred_dots(harness: &mut Harness, field: Rect) -> (Vec<(u32, f32)>, Option<f32>) {
    harness.key(Key::Tab);
    harness.advance(Duration::from_millis(300));
    assert!(
        !harness.is_focused(".ds-polkit input"),
        "Tab took the keyboard from the field"
    );
    let shot = harness.render().expect("a frame");
    (dots(&shot, field), caret_left(&shot, field))
}

/// The first ink row of column `x` in the field: where a dot's top is.
fn ink_top(shot: &RgbaImage, field: Rect, x: u32) -> Option<u32> {
    let band = Band::of(field);
    (band.top..band.bottom).find(|&y| inked(*shot.get_pixel(x, y)))
}

/// Where the n-th dot's advance ends, from the dots' pitch and the mask's leading padding.
fn advance_end(dots: &[(u32, f32)], n: usize, text_start: f32) -> f32 {
    let (first, last) = (dots[0].1, dots[dots.len() - 1].1);
    let pitch = (last - first) / (dots.len() - 1) as f32;
    text_start + n as f32 * pitch
}

#[test]
fn the_drawn_caret_sits_at_the_end_of_the_last_dot() {
    for n in [4, 11, 24] {
        let typed: String = "hunter2xWq".chars().cycle().take(n).collect();
        let mut harness = typed_into(&typed, &[]);
        let field = harness
            .rect(".ds-polkit input")
            .expect("the password field");
        let mask = harness.rect(".ds-polkit .ds-input-mask").expect("the mask");
        let shot = harness.render().expect("a frame");
        let caret = caret_left(&shot, field).expect("a caret in the focused field");
        let (dots, after_blur) = blurred_dots(&mut harness, field);
        assert_eq!(dots.len(), n, "one dot per character: {dots:?}");
        assert_eq!(after_blur, None, "an unfocused field paints no caret");
        let first_dot = dots[0].0 + 1;
        assert_eq!(
            ink_top(&shot, field, first_dot),
            ink_top(&harness.render().expect("a frame"), field, first_dot),
            "{n} typed: the caret takes no room in the line, so the dots do not move with it"
        );
        let end = advance_end(&dots, n, mask.origin.x.0 + MASK_PAD);
        assert!(
            (caret - end).abs() <= 1.0,
            "{n} typed: the caret is at {caret:.2}, the last dot's advance ends at {end:.2}"
        );
    }
}

#[test]
fn after_left_three_times_the_caret_is_before_the_third_dot_from_the_end() {
    let n = 11;
    let typed: String = "hunter2xWq".chars().cycle().take(n).collect();
    let mut harness = typed_into(&typed, &[Key::Left, Key::Left, Key::Left]);
    let field = harness
        .rect(".ds-polkit input")
        .expect("the password field");
    let mask = harness.rect(".ds-polkit .ds-input-mask").expect("the mask");
    let shot = harness.render().expect("a frame");
    let caret = caret_left(&shot, field).expect("a caret in the focused field");
    let (dots, _) = blurred_dots(&mut harness, field);
    assert_eq!(dots.len(), n);
    let third_from_end = dots[n - 3].0 as f32;
    assert!(
        caret < third_from_end,
        "the caret at {caret:.2} is not before the third dot from the end at {third_from_end}"
    );
    let end = advance_end(&dots, n - 3, mask.origin.x.0 + MASK_PAD);
    assert!(
        (caret - end).abs() <= 1.0,
        "the caret is at {caret:.2}, the {}th dot's advance ends at {end:.2}",
        n - 3
    );
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
