//! The Overlays page's screenshot thumbnail (sill Q181): in each scheme, over the Work Space's
//! tint, a `ShotThumbnail` at its default 240 px (with its Delete action, shown on hover), a
//! portrait picture pillarboxed at 16:10, and the `ShotGhost` a host draws as its drag icon.
//! Live, a button hides the thumbnail (it slides out to the right) and shows it again (it rises
//! in). The picture is drawn here, a made-up desktop, never a capture of anybody's screen.

use super::Section;
use super::level_tile::work;
use crate::axes::{Axes, Showcase};
use dioxus::prelude::*;
use ds::{
    Appearance, Button, ButtonVariant, Ds, Icon, ImageSize, ImageSource, Inject, Material,
    ShotGhost, ShotThumbnail, Shown, Swipe, Theme, ThumbAction,
};
use image::{ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;

/// The made-up desktop's size: 16:10.
const DESKTOP: ImageSize = ImageSize {
    width: 160,
    height: 100,
};

/// A portrait picture's size, taller than 16:10, so the card pillarboxes it.
const PORTRAIT: ImageSize = ImageSize {
    width: 60,
    height: 100,
};

/// A made-up screen: a two-colour wallpaper ramp with a light window and its title bar on it.
fn desktop(size: ImageSize) -> Option<ImageSource> {
    let (w, h) = (size.width, size.height);
    let picture = RgbaImage::from_fn(w, h, |x, y| {
        let window = x > w / 5 && x < w * 4 / 5 && y > h / 5 && y < h * 4 / 5;
        let title = window && y < h / 5 + h / 12;
        let t = (x + y) as f32 / (w + h) as f32;
        match (window, title) {
            (true, true) => Rgba([222, 226, 220, 255]),
            (true, false) => Rgba([250, 250, 247, 255]),
            _ => Rgba([
                (70.0 + 90.0 * t) as u8,
                (110.0 + 40.0 * t) as u8,
                (150.0 - 30.0 * t) as u8,
                255,
            ]),
        }
    });
    let mut bytes = Cursor::new(Vec::new());
    picture.write_to(&mut bytes, ImageFormat::Png).ok()?;
    Some(ImageSource::png(&bytes.into_inner()))
}

/// The screenshot thumbnail section.
#[component]
pub fn ShotThumbnails() -> Element {
    rsx! {
        Section { title: "Screenshot thumbnail", note: "ShotThumbnail over the Work Space's tint, light and dark: the picture letterboxed in a Toast-material card 240 wide (the box follows the picture between 2:1 and 16:10; the portrait one is pillarboxed), rising in with rise at --t-big --e-spring and sliding out to the right with shot-out at --t-move --e-exit (Live: the button). Hover shows its actions (Delete) popping in; a swipe right dismisses it as a notification's does; a click opens, a drag past 8 px hands the host a DragStart. Right: ShotGhost, the drag icon, 120 wide and .8 opaque.",
            div { class: "g-shot-row",
                for theme in [Theme::Light, Theme::Dark] {
                    ShotScene { theme }
                }
            }
        }
    }
}

/// One scheme's scene.
#[component]
fn ShotScene(theme: Theme) -> Element {
    let axes = use_context::<Signal<Axes>>();
    let (accent, motion, showcase) = {
        let axes = axes.read();
        (axes.accent, axes.motion, axes.showcase)
    };
    let appearance = Appearance {
        theme,
        accent,
        motion,
    };
    let mut shown = use_signal(|| Shown::Visible);
    let (Some(wide), Some(tall)) = (desktop(DESKTOP), desktop(PORTRAIT)) else {
        return rsx! {};
    };
    let actions = vec![ThumbAction {
        icon: Icon::Trash,
        label: "Delete".into(),
        onpress: EventHandler::new(move |()| shown.set(Shown::Hidden)),
    }];
    rsx! {
        div { class: "g-shot",
            Ds { appearance, look: work(theme), material: Material::Window, stylesheet: Inject::Host,
                div { class: "g-shot-ground",
                    ShotThumbnail {
                        image: wide.clone(),
                        size: DESKTOP,
                        shown: shown(),
                        actions,
                        id: "thumb",
                        swipe: Swipe::Dismiss(EventHandler::new(move |()| shown.set(Shown::Hidden))),
                    }
                    ShotThumbnail { image: tall, size: PORTRAIT, shown: Shown::Visible }
                    ShotGhost { image: wide, size: DESKTOP }
                }
                if showcase == Showcase::Live {
                    div { class: "g-row",
                        Button {
                            variant: ButtonVariant::Secondary,
                            label: "Toggle the thumbnail",
                            onclick: move |_| shown.set(match shown() { Shown::Visible => Shown::Hidden, Shown::Hidden => Shown::Visible }),
                        }
                    }
                }
            }
        }
    }
}
