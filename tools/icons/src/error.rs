use thiserror::Error;

/// What can go wrong turning a render into an icon; each variant names the input at fault.
#[derive(Debug, Error)]
pub enum IconsError {
    /// The render keyed out to nothing: the background key ate the whole object.
    #[error("no object pixels left after keying the background")]
    EmptyObject,
    /// A family name on the command line that 08-ICONS 2.3 does not define.
    #[error("unknown plate family {0:?} (red, amber, green, blue, violet, paper)")]
    UnknownFamily(String),
    /// A colour in a spec that is neither a family name nor `#RRGGBB`.
    #[error("unknown colour {0:?} (a family name or #RRGGBB)")]
    UnknownColour(String),
    /// An abstract icon spec is not valid TOML or names something the vocabulary lacks.
    #[error("spec: {0}")]
    Spec(#[from] toml::de::Error),
    /// Reading or writing an image failed.
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    /// A file system operation failed.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
