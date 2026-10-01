//! What a texture layer is told: how its texture fills the box, which part of it shows, how it
//! is sampled, when it repaints, and the CPU pixels an app uploads. Data only; the arithmetic is
//! `fit`, the conversion `convert`, the GPU work `gpu`.

use ds::prelude::Word;
use peniko::ImageQuality;
use std::fmt;

/// A count of texture pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Texels(pub u32);

/// A rectangle in a texture's own pixels, origin at the top-left texel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TexelRect {
    /// The left edge.
    pub x: Texels,
    /// The top edge.
    pub y: Texels,
    /// The width.
    pub width: Texels,
    /// The height.
    pub height: Texels,
}

impl TexelRect {
    /// The rectangle whose top-left texel is (`x`, `y`) and whose size is `width` by `height`.
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        TexelRect {
            x: Texels(x),
            y: Texels(y),
            width: Texels(width),
            height: Texels(height),
        }
    }

    /// The whole of a `width` by `height` texture.
    pub fn whole(width: Texels, height: Texels) -> Self {
        TexelRect {
            x: Texels(0),
            y: Texels(0),
            width,
            height,
        }
    }
}

/// How the shown part of a texture fills the layer's box: `data-fit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum TextureFit {
    /// Scaled to fit whole inside the box, keeping its shape, centred; the box shows the ground
    /// beside or above it.
    #[default]
    Contain,
    /// Scaled to cover the box, keeping its shape, centred; what overhangs is cut.
    Cover,
    /// Stretched to the box, whatever its shape.
    Fill,
    /// One texture pixel to one device pixel, centred; what overhangs is cut.
    Actual,
    /// One texture pixel to one device pixel, repeated from the top-left corner to fill the box.
    Tile,
}

/// How the texture is resampled when it is drawn at another size: `data-sampling`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Sampling {
    /// The nearest texel: hard pixels, for pixel art and for `Actual`.
    Nearest,
    /// Bilinear: smooth, and cheap.
    #[default]
    Bilinear,
    /// Bicubic: sharper than bilinear when a picture is scaled down.
    Bicubic,
}

impl Sampling {
    /// The renderer's name for this quality.
    pub(crate) fn quality(self) -> ImageQuality {
        match self {
            Sampling::Nearest => ImageQuality::Low,
            Sampling::Bilinear => ImageQuality::Medium,
            Sampling::Bicubic => ImageQuality::High,
        }
    }
}

/// When a layer repaints: `data-pace`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Word)]
pub enum Pace {
    /// When the app says the texture changed (`TextureHandle::redraw`, or any call that replaces
    /// or updates it), and when the layer's own props change. An idle window repaints nothing.
    #[default]
    OnDemand,
    /// Every frame the window paints, as long as the layer is mounted: a texture something else
    /// writes without telling the layer, or a window that already paints continuously.
    EveryFrame,
}

/// How the bytes of a [`Pixels`] are laid out and what their alpha means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Word)]
pub enum PixelFormat {
    /// Three bytes a pixel, red green blue, fully opaque.
    Rgb8,
    /// Four bytes a pixel whose colour channels are already multiplied by the alpha: what
    /// pdfrum's `Pixmap` and every vello target hold. Uploaded as they are.
    Rgba8Premultiplied,
    /// Four bytes a pixel whose colour channels are not multiplied by the alpha: what a decoded
    /// PNG or a video frame with an alpha plane holds. Multiplied on the CPU at upload.
    Rgba8Straight,
}

impl PixelFormat {
    /// Bytes in one pixel.
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            PixelFormat::Rgb8 => 3,
            PixelFormat::Rgba8Premultiplied | PixelFormat::Rgba8Straight => 4,
        }
    }
}

/// Why bytes are not a picture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelsError {
    /// The width or the height is zero.
    Empty,
    /// The bytes are not `width * height * bytes_per_pixel` long.
    Length {
        /// The length the size and format need.
        want: usize,
        /// The length given.
        got: usize,
    },
}

impl fmt::Display for PixelsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PixelsError::Empty => f.write_str("a picture needs a width and a height"),
            PixelsError::Length { want, got } => {
                write!(f, "the picture needs {want} bytes, {got} were given")
            }
        }
    }
}

impl std::error::Error for PixelsError {}

/// CPU pixels to upload: a size, a format and the bytes, row after row from the top-left with no
/// padding. Borrowed, so a decoder's buffer is read where it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pixels<'a> {
    width: u32,
    height: u32,
    format: PixelFormat,
    bytes: &'a [u8],
}

impl<'a> Pixels<'a> {
    /// `bytes` as a `width` by `height` picture in `format`.
    pub fn new(
        format: PixelFormat,
        width: u32,
        height: u32,
        bytes: &'a [u8],
    ) -> Result<Self, PixelsError> {
        if width == 0 || height == 0 {
            return Err(PixelsError::Empty);
        }
        let want = width as usize * height as usize * format.bytes_per_pixel();
        if bytes.len() != want {
            return Err(PixelsError::Length {
                want,
                got: bytes.len(),
            });
        }
        Ok(Pixels {
            width,
            height,
            format,
            bytes,
        })
    }

    /// The width in pixels.
    pub fn width(&self) -> Texels {
        Texels(self.width)
    }

    /// The height in pixels.
    pub fn height(&self) -> Texels {
        Texels(self.height)
    }

    /// How the bytes are laid out.
    pub fn format(&self) -> PixelFormat {
        self.format
    }

    /// The bytes as given.
    pub fn bytes(&self) -> &'a [u8] {
        self.bytes
    }
}
