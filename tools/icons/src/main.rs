//! `icons`: turn raw renders into plated icons and contact sheets (design/08-ICONS.md 3.7).

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use icons::{
    Cell, EXPORT_SIZES, Family, IconsError, Sheet, SheetStyle, Template, build_sheet, compose,
    drop_shadow, export, fit_object, grid_for, key_background, size_strip,
};
use image::{DynamicImage, Rgba32FImage};

#[derive(Debug, Parser)]
#[command(about = "Post-process generated app-icon objects (design/08-ICONS.md 3.7)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Key, fit and plate one raw render; writes <name>.object.png, <name>.flat.png, <name>.png.
    Plate {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        family: String,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        name: String,
        /// Also write the hicolor size set (08 2.6) under this directory.
        #[arg(long)]
        export_dir: Option<PathBuf>,
    },
    /// A contact sheet: rows x cols of `<dir>/<row>-<col><suffix>`.
    Sheet {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long, value_delimiter = ',')]
        rows: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        cols: Vec<String>,
        #[arg(long)]
        title: String,
        #[arg(long)]
        out: PathBuf,
        /// `raw` shows the render; `plated` shows `<name>.png` with the 16/32/48 strip under it.
        #[arg(long, default_value = "raw")]
        kind: SheetKind,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum SheetKind {
    Raw,
    Plated,
}

fn load(path: &Path) -> Result<Rgba32FImage, IconsError> {
    Ok(image::open(path)?.to_rgba32f())
}

fn save(img: &Rgba32FImage, path: &Path) -> Result<(), IconsError> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    DynamicImage::ImageRgba32F(img.clone())
        .to_rgba8()
        .save(path)?;
    Ok(())
}

fn plate(
    input: &Path,
    family: Family,
    out_dir: &Path,
    name: &str,
    export_dir: Option<&Path>,
) -> Result<(), IconsError> {
    let t = Template::default();
    let raw = load(input)?;
    let key = key_background(&raw, &t);
    let object = fit_object(&key.layer, &key.alpha, 1024, &t)?;
    let grid = grid_for(1024, &t);
    let flat = compose(grid, family.stops(), &object, &t);
    save(&object, &out_dir.join(format!("{name}.object.png")))?;
    save(&flat, &out_dir.join(format!("{name}.flat.png")))?;
    save(
        &drop_shadow(&flat, grid, &t),
        &out_dir.join(format!("{name}.png")),
    )?;
    if let Some(dir) = export_dir {
        for size in EXPORT_SIZES {
            let one = export(&flat, &t, size, icons::Shadow::Baked);
            save(
                &one.image,
                &dir.join(format!("hicolor/{size}x{size}/apps/{name}.png")),
            )?;
            if size <= 256 {
                let two = export(&flat, &t, size * 2, icons::Shadow::Baked);
                save(
                    &two.image,
                    &dir.join(format!("hicolor/{size}x{size}@2/apps/{name}.png")),
                )?;
            }
        }
    }
    Ok(())
}

fn cell(
    dir: &Path,
    name: &str,
    caption: &str,
    kind: SheetKind,
    t: &Template,
) -> Result<Cell, IconsError> {
    let (main, flat) = match kind {
        SheetKind::Raw => (dir.join(format!("{name}.png")), None),
        SheetKind::Plated => (
            dir.join(format!("{name}.png")),
            Some(dir.join(format!("{name}.flat.png"))),
        ),
    };
    if !main.exists() {
        return Ok(Cell {
            image: Rgba32FImage::new(1, 1),
            below: None,
            caption: format!("{caption} (none)"),
        });
    }
    let below = flat
        .map(|p| load(&p).map(|f| size_strip(&f, t)))
        .transpose()?;
    Ok(Cell {
        image: load(&main)?,
        below,
        caption: caption.to_owned(),
    })
}

fn sheet(
    dir: &Path,
    rows: &[String],
    cols: &[String],
    title: &str,
    out: &Path,
    kind: SheetKind,
) -> Result<(), IconsError> {
    let t = Template::default();
    let rows = rows
        .iter()
        .map(|r| {
            Ok((
                r.clone(),
                cols.iter()
                    .map(|c| cell(dir, &format!("{r}-{c}"), c, kind, &t))
                    .collect::<Result<Vec<_>, IconsError>>()?,
            ))
        })
        .collect::<Result<Vec<_>, IconsError>>()?;
    save(
        &build_sheet(
            &Sheet {
                title: title.to_owned(),
                rows,
            },
            &SheetStyle::default(),
        ),
        out,
    )
}

fn main() -> Result<(), IconsError> {
    match Cli::parse().command {
        Command::Plate {
            input,
            family,
            out_dir,
            name,
            export_dir,
        } => plate(
            &input,
            family.parse()?,
            &out_dir,
            &name,
            export_dir.as_deref(),
        ),
        Command::Sheet {
            dir,
            rows,
            cols,
            title,
            out,
            kind,
        } => sheet(&dir, &rows, &cols, &title, &out, kind),
    }
}
