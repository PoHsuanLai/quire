//! ShotGhost: the screenshot thumbnail drawn small and translucent, for the host's drag icon
//! (design/04-COMPONENTS.md section 39; sill Q181). The host draws it on its own drag-icon
//! surface while it carries the file; quire only paints it. Smaller than the card so the drop
//! target under it stays readable, and translucent so what it is dropped on shows through. No
//! motion: a drag icon follows the pointer, it does not arrive.

use crate::components::image_source::{ImageSize, ImageSource};
use crate::components::shot_frame::{picture_style, shot_frame};
use crate::geometry::Px;
use crate::material::Material;
use crate::root::chrome::RootChrome;
use crate::root::surface::Surface;
use dioxus::prelude::*;

/// The ghost's width: half the default card's.
const GHOST_WIDTH: Px = Px(120.0);

/// The drag ghost of a thumbnail showing `image` of `size`.
#[component]
pub fn ShotGhost(image: ImageSource, size: ImageSize) -> Element {
    let frame = shot_frame(GHOST_WIDTH, size);
    rsx! {
        Surface { material: Material::Toast, chrome: RootChrome::Transparent,
            div {
                class: "ds-shot-ghost",
                "aria-hidden": "true",
                style: "width:{frame.card.width.0}px",
                div { class: "ds-shot-plate", style: "height:{frame.card.height.0}px",
                    img { class: "ds-shot-image", alt: "", src: image.0, style: picture_style(frame.picture) }
                }
            }
        }
    }
}
