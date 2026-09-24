//! A Space dot paints its gradient from `--dot-c*` in the stylesheet (mailo gaps 3, item 5), and
//! the picture is the one its old inline `background:linear-gradient(…)` drew: on Blitz, each
//! dot is compared pixel by pixel with a reference box painted with `space::gradient`'s text,
//! for a two-stop and a three-stop preset.

use dioxus::prelude::*;
use ds::{
    Appearance, CardAccent, Ds, FrameVars, Grain, Here, Key, Material, PRESETS, Scheme, Shortcut,
    SpaceDot, SpaceLook, Theme,
};
use ds_native::{Harness, Viewport};

const VIEW: Viewport = Viewport {
    width: 200,
    height: 120,
    scale_percent: 100,
};

fn frame(index: usize) -> FrameVars {
    let look = SpaceLook {
        dots: PRESETS[index].dots.to_vec(),
        grain: Grain(0),
        theme: Theme::Light,
        card_accent: CardAccent::Postmark,
    };
    FrameVars::of(&look, Scheme::Light)
}

/// Preset 0 (two stops) and the first three-stop preset.
fn cases() -> [usize; 2] {
    let three = PRESETS
        .iter()
        .position(|preset| preset.dots.len() == 3)
        .expect("a three-dot preset");
    [0, three]
}

/// Each dot at (40 + 60n, 30), its reference box at (40 + 60n, 70).
#[allow(non_snake_case)]
fn Dots() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Popover,
            for (n , index) in cases().into_iter().enumerate() {
                div { key: "{n}", style: "position:absolute;left:{40 + 60 * n}px;top:30px",
                    SpaceDot {
                        name: "Dot",
                        frame: frame(index),
                        here: Here::Elsewhere,
                        shortcut: Shortcut(vec![Key::Ctrl, Key::Char('1')]),
                        onclick: |_| {},
                    }
                }
                div {
                    style: "position:absolute;left:{40 + 60 * n}px;top:70px;width:22px;height:22px;box-sizing:border-box;border-radius:999px;border:2px solid transparent;background:{frame(index).gradient}",
                }
            }
        }
    }
}

#[test]
fn a_space_dot_paints_what_its_inline_gradient_painted() {
    let mut harness = Harness::new(Dots, VIEW);
    let picture = harness.render().expect("renders");
    for n in 0..cases().len() as u32 {
        let x0 = 40 + 60 * n;
        let mut worst = 0u8;
        let mut coloured = 0;
        for dy in 0..22 {
            for dx in 0..22 {
                let dot = picture.get_pixel(x0 + dx, 30 + dy).0;
                let reference = picture.get_pixel(x0 + dx, 70 + dy).0;
                let diff = (0..3)
                    .map(|i| dot[i].abs_diff(reference[i]))
                    .max()
                    .unwrap_or(0);
                worst = worst.max(diff);
                coloured += usize::from(dot != picture.get_pixel(x0 + dx, 5).0);
            }
        }
        assert!(
            coloured > 200,
            "case {n}: the dot painted {coloured} pixels"
        );
        assert!(worst <= 2, "case {n}: a pixel differs by {worst}");
    }
}
