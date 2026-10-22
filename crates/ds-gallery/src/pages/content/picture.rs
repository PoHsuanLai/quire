//! Picture: a landscape and a square frame cropping the same wide image to fill (`object-fit:
//! cover`), at the media radius.

use crate::pages::Section;
use dioxus::prelude::*;
use ds::components::content::picture::{Picture, PictureShape};
use ds::prelude::*;

/// A wide image (a sunset's bands), as a `data:` URI.
pub(crate) fn wide_image() -> ImageSource {
    let svg = "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 320 120' preserveAspectRatio='none'><rect width='320' height='60' fill='#ff8d28'/><rect y='60' width='320' height='60' fill='#6155f5'/><circle cx='160' cy='60' r='30' fill='#ffcc00'/></svg>";
    ImageSource(format!(
        "data:image/svg+xml;base64,{}",
        crate::data_uri::base64(svg.as_bytes())
    ))
}

/// The two frames side by side.
#[component]
pub fn PictureGallery() -> Element {
    rsx! {
        Section {
            title: "Picture",
            note: "An image cropped to fill a rounded frame, never stretched. Landscape is 16:10, a screen's; Square is 1:1. The frame is as wide as its parent and the alt text is required.",
            div { class: "g-row g-row-top",
                div { style: "width:240px",
                    Picture { source: wide_image(), alt: "A sunset over bands of orange and violet" }
                }
                div { style: "width:120px",
                    Picture {
                        source: wide_image(),
                        alt: "The same sunset, cropped square",
                        shape: PictureShape::Square,
                    }
                }
            }
        }
    }
}
