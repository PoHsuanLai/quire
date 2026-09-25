//! The encoded source of every image in a document that came from a `data:` URL: the only
//! kind a sealed printout loads. Blitz keeps only decoded pixels; the source is decoded again
//! from the element's own URL, so the PDF can embed a JPEG as it arrived instead of re-encoding
//! its pixels (several times larger). An image from anywhere else (an app's fetcher) is drawn
//! from its pixels.

use crate::data_url;
use anyrender_krilla::{EncodedImage, ImageSources};
use blitz_dom::node::ImageData;
use blitz_dom::{BaseDocument, LocalName};

/// The traced sources of `doc`'s `<img>` elements and background images.
pub(crate) fn collect(doc: &BaseDocument) -> ImageSources {
    let mut sources = ImageSources::default();
    for (_, node) in doc.tree().iter() {
        let Some(element) = node.element_data() else {
            continue;
        };
        if let (Some(raster), Some(src)) = (
            element.raster_image_data(),
            element.attr(LocalName::from("src")),
        ) && let Some(source) = decode(src)
        {
            sources.insert(raster.data.id(), source);
        }
        for background in element.background_images.iter().flatten() {
            if let ImageData::Raster(raster) = &background.image
                && let Some(source) = decode(background.url.as_str())
            {
                sources.insert(raster.data.id(), source);
            }
        }
    }
    sources
}

fn decode(url: &str) -> Option<EncodedImage> {
    data_url::decode(url).and_then(EncodedImage::sniffed)
}
