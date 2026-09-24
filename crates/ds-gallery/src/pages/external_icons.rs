//! The Controls page's external icons: a status item's symbolic SVG and coloured pixmap, as
//! sill's tray draws them (design/08-ICONS.md section 1.5, quire gap Q6).

use super::{Section, Specimen};
use dioxus::prelude::*;
use ds::{
    Button, ButtonVariant, ExternalIcon, Icon, IconButton, IconButtonVariant, IconSize, IconSource,
    IconUrl, IconView, Switch,
};
use image::{ImageFormat, Rgba, RgbaImage};
use std::io::Cursor;
use std::path::Path;

/// A freedesktop-style symbolic bell, filled in the theme's grey as such icons are: only its
/// alpha counts once it is drawn as a mask.
const BELL: &str = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'>\
<path fill='#bebebe' d='M8 1a1 1 0 0 1 1 1v.6A4.5 4.5 0 0 1 12.5 7v3l1.5 2H2l1.5-2V7A4.5 4.5 0 0 1 7 2.6V2a1 1 0 0 1 1-1zM6.5 13h3a1.5 1.5 0 0 1-3 0z'/></svg>";

/// The keyboard glyph a tray's input-method item names, read from a file as an icon theme
/// lookup returns it.
fn keyboard_file() -> Option<IconUrl> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/input-keyboard-symbolic.svg");
    IconUrl::file(&path).ok()
}

/// A coloured status pixmap, the kind an SNI item sends as `IconPixmap`: a 22 px disc whose
/// hue turns around it, PNG-encoded.
fn pixmap() -> Option<IconUrl> {
    let picture = RgbaImage::from_fn(22, 22, |x, y| {
        let (dx, dy) = (x as f32 - 10.5, y as f32 - 10.5);
        let inside = dx * dx + dy * dy <= 10.0 * 10.0;
        let turn = (dy.atan2(dx) / std::f32::consts::TAU + 0.5) * 3.0;
        let [r, g, b] = [0.0f32, 1.0, 2.0].map(|phase| {
            let near = 1.0 - ((turn - phase).rem_euclid(3.0) - 1.5).abs().min(1.0);
            (60.0 + 180.0 * near) as u8
        });
        Rgba([r, g, b, if inside { 255 } else { 0 }])
    });
    let mut bytes = Cursor::new(Vec::new());
    picture.write_to(&mut bytes, ImageFormat::Png).ok()?;
    Some(IconUrl::png(&bytes.into_inner()))
}

fn symbolic(url: IconUrl, size: IconSize) -> IconSource {
    IconSource::Symbolic(ExternalIcon { url, size })
}

/// External icons in every slot that takes one.
#[component]
pub fn ExternalIcons() -> Element {
    let bell = IconUrl::svg(BELL);
    let keyboard = keyboard_file().unwrap_or_else(|| bell.clone());
    let colour = pixmap().unwrap_or_else(|| bell.clone());
    let image = |size| {
        IconSource::Image(ExternalIcon {
            url: colour.clone(),
            size,
        })
    };
    rsx! {
        Section { title: "External icons", note: "A symbolic icon is a mask painted in the text colour, so it follows hover and pressed like a glyph; an image keeps its own colours. data: and file: URLs.",
            div { class: "g-row",
                Specimen { name: "Tool, symbolic data: SVG",
                    div { class: "g-row",
                        IconButton { variant: IconButtonVariant::Tool, icon: symbolic(bell.clone(), IconSize::Base), label: "Notifications", onclick: |_| {} }
                        IconButton { variant: IconButtonVariant::Tool, icon: symbolic(bell.clone(), IconSize::Base), label: "Notifications expanded", expanded: Some(Switch::On), onclick: |_| {} }
                    }
                }
                Specimen { name: "Tool, symbolic file: SVG",
                    IconButton { variant: IconButtonVariant::Tool, icon: symbolic(keyboard.clone(), IconSize::Base), label: "Input method", onclick: |_| {} }
                }
                Specimen { name: "Tool, image data: PNG",
                    IconButton { variant: IconButtonVariant::Tool, icon: image(IconSize::Base), label: "Status", onclick: |_| {} }
                }
                Specimen { name: "Mini, symbolic",
                    Button { variant: ButtonVariant::Mini, label: "Updates", icon: symbolic(bell.clone(), IconSize::Compact), onclick: |_| {} }
                }
                Specimen { name: "IconView at Bar: glyph, symbolic, image",
                    div { class: "g-row",
                        IconView { source: Icon::Bell.into(), size: IconSize::Bar }
                        IconView { source: symbolic(bell.clone(), IconSize::Bar) }
                        IconView { source: symbolic(keyboard, IconSize::Bar) }
                        IconView { source: image(IconSize::Bar) }
                    }
                }
            }
        }
    }
}
