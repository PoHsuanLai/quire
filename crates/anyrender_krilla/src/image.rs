//! Images as krilla images: from their encoded source where the caller traced one (a JPEG is
//! then embedded byte for byte), else from the decoded pixels the brush carries.

use crate::resources::Resources;
use crate::sources::{EncodedImage, ImageCodec, ImageSources};
use krilla::image::Image;
use peniko::{ImageAlphaType, ImageData, ImageFormat};

/// The krilla image for `decoded`, made once per document.
pub(crate) fn prepare(
    resources: &mut Resources,
    sources: &ImageSources,
    decoded: &ImageData,
) -> Option<Image> {
    let blob = decoded.data.id();
    if let Some(image) = resources.images.get(&blob) {
        return Some(image.clone());
    }
    let image = sources
        .get(blob)
        .and_then(|source| from_source(source, decoded))
        .or_else(|| from_pixels(decoded))?;
    resources.images.insert(blob, image.clone());
    Some(image)
}

/// `source` as krilla reads it, if it reads and has the decoded image's size (a source a
/// renderer rotated or scaled while decoding is not the image painted, so it is not used).
fn from_source(source: &EncodedImage, decoded: &ImageData) -> Option<Image> {
    let data = source.bytes().clone().into();
    let image = match source.codec() {
        ImageCodec::Jpeg => Image::from_jpeg(data, true),
        ImageCodec::Png => Image::from_png(data, true),
    }
    .ok()?;
    (image.size() == (decoded.width, decoded.height)).then_some(image)
}

/// The decoded pixels as straight-alpha RGBA, which is what krilla takes.
fn from_pixels(decoded: &ImageData) -> Option<Image> {
    let pixels = decoded.data.data();
    let expected = usize::try_from(decoded.width)
        .ok()?
        .checked_mul(usize::try_from(decoded.height).ok()?)?
        .checked_mul(4)?;
    if pixels.len() < expected {
        return None;
    }
    let (quads, _) = pixels[..expected].as_chunks::<4>();
    let rgba = quads
        .iter()
        .flat_map(|pixel| straight_rgba(pixel, decoded.format, decoded.alpha_type))
        .collect();
    Some(Image::from_rgba8(rgba, decoded.width, decoded.height))
}

/// One pixel as straight RGBA.
fn straight_rgba(pixel: &[u8], format: ImageFormat, alpha: ImageAlphaType) -> [u8; 4] {
    let [r, g, b, a] = match format {
        ImageFormat::Bgra8 => [pixel[2], pixel[1], pixel[0], pixel[3]],
        _ => [pixel[0], pixel[1], pixel[2], pixel[3]],
    };
    match (alpha, a) {
        (ImageAlphaType::AlphaPremultiplied, 1..=254) => {
            let unmultiply = |channel: u8| (u16::from(channel) * 255 / u16::from(a)).min(255) as u8;
            [unmultiply(r), unmultiply(g), unmultiply(b), a]
        }
        _ => [r, g, b, a],
    }
}

#[cfg(test)]
mod tests {
    use super::straight_rgba;
    use peniko::{ImageAlphaType, ImageFormat};

    type Case = (([u8; 4], ImageFormat, ImageAlphaType), [u8; 4]);

    const CASES: &[Case] = &[
        (
            ([1, 2, 3, 4], ImageFormat::Rgba8, ImageAlphaType::Alpha),
            [1, 2, 3, 4],
        ),
        (
            ([1, 2, 3, 4], ImageFormat::Bgra8, ImageAlphaType::Alpha),
            [3, 2, 1, 4],
        ),
        (
            (
                [50, 0, 100, 127],
                ImageFormat::Rgba8,
                ImageAlphaType::AlphaPremultiplied,
            ),
            [100, 0, 200, 127],
        ),
        (
            (
                [9, 9, 9, 0],
                ImageFormat::Rgba8,
                ImageAlphaType::AlphaPremultiplied,
            ),
            [9, 9, 9, 0],
        ),
    ];

    #[test]
    fn pixels_become_straight_rgba() {
        for &((pixel, format, alpha), want) in CASES {
            assert_eq!(
                straight_rgba(&pixel, format, alpha),
                want,
                "{pixel:?} {format:?}"
            );
        }
    }
}
