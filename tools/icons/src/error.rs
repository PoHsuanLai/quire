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
    /// A tint that is not in the muted palette (design/08-ICONS.md 2.10).
    #[error("unknown tint {0:?} (slate, teal, sage, ochre, clay, plum)")]
    UnknownTint(String),
    /// A `--as` override that is not `app=dialect`.
    #[error("bad dialect override {0:?} (app=monochrome|graphite|paper|solid)")]
    BadOverride(String),
    /// An abstract icon spec is not valid TOML or names something the vocabulary lacks.
    #[error("spec: {0}")]
    Spec(#[from] toml::de::Error),
    /// Reading or writing an image failed.
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    /// `icons install` without `--to`, and neither `$XDG_DATA_HOME` nor `$HOME` is set.
    #[error("no data directory to install into: set $XDG_DATA_HOME or $HOME, or pass --to")]
    NoInstallDir,
    /// A file system operation failed.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}
