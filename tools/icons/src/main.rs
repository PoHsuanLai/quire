//! `icons`: export the shipped app-icon set and install it (design/08-ICONS.md 2.11).

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
mod install;
mod ship_set;

use icons::IconsError;
use image::{DynamicImage, Rgba32FImage};

#[derive(Debug, Parser)]
#[command(about = "Export and install the shipped app-icon set (design/08-ICONS.md 2.11)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Export the shipped set (08 2.11) from `ship.toml`: every size of every style into
    /// `<out>/<app>/[muted|monochrome/]<px>.png`; optionally a contact sheet from the files.
    Ship {
        #[arg(long)]
        manifest: PathBuf,
        /// Where the manifest's Klein renders are.
        #[arg(long)]
        renders: PathBuf,
        #[arg(long)]
        out: PathBuf,
        #[arg(long)]
        sheet: Option<PathBuf>,
    },
    /// Copy the shipped app-icon set where `ds_settings::apps_dir` finds it: by default this
    /// repository's `assets/icons/apps` to `$XDG_DATA_HOME/quire/icons/apps`.
    Install {
        /// The set to copy (default: the repository's `assets/icons/apps`).
        #[arg(long)]
        from: Option<PathBuf>,
        /// Where to put it (default: `$XDG_DATA_HOME/quire/icons/apps`).
        #[arg(long)]
        to: Option<PathBuf>,
    },
}

pub(crate) fn load(path: &Path) -> Result<Rgba32FImage, IconsError> {
    Ok(image::open(path)?.to_rgba32f())
}

pub(crate) fn save(img: &Rgba32FImage, path: &Path) -> Result<(), IconsError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    DynamicImage::ImageRgba32F(img.clone())
        .to_rgba8()
        .save(path)?;
    Ok(())
}

fn main() -> Result<(), IconsError> {
    match Cli::parse().command {
        Command::Install { from, to } => {
            let from = from.unwrap_or_else(install::repository_set);
            let to = to.map_or_else(install::default_target, Ok)?;
            let copied = install::install(&from, &to)?;
            println!("{copied} icons: {} -> {}", from.display(), to.display());
            Ok(())
        }
        Command::Ship {
            manifest,
            renders,
            out,
            sheet,
        } => {
            ship_set::ship(&manifest, &renders, &out)?;
            match sheet {
                Some(s) => ship_set::sheet(&manifest, &out, &s),
                None => Ok(()),
            }
        }
    }
}
