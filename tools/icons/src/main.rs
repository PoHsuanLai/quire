//! `icons`: turn raw renders into plated icons and contact sheets (design/08-ICONS.md 3.7).

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use icons::{
    Bevel, Cell, EXPORT_SIZES, Family, IconsError, Plane, Shadow, Sheet, SheetStyle, Spec,
    Template, alpha_bbox, build_sheet, compose, drop_shadow, emblem, export, finish, fit_object,
    grain_tile, grid_for, key_background, parse_spec, size_strip, strip, strip_sheet,
};
use image::imageops::{FilterType, resize};
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
    /// Compose abstract icons from TOML specs (08 2.8): per spec, <name>.png (1024 master with
    /// shadow), <name>.flat.png and the hicolor size set, each size rendered natively.
    Abstract {
        #[arg(long, num_args = 1.., required = true)]
        spec: Vec<PathBuf>,
        #[arg(long)]
        out_dir: PathBuf,
        /// Also write a sheet: each icon at 512, 48, 32 and 16 px 1:1 on light and dark grounds.
        #[arg(long)]
        sheet: Option<PathBuf>,
    },
    /// Plate a render that already contains the plate's face (08 2.9, round three):
    /// `ground` takes the whole full-bleed render as the face and adds our bevel; `tile` keys a
    /// model-drawn tile off its plain ground, stretches it to the plate and re-masks it with our
    /// squircle, keeping the tile's own bevel. Writes <name>.png and <name>.flat.png.
    Face {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        mode: FaceMode,
        #[arg(long)]
        out_dir: PathBuf,
        #[arg(long)]
        name: String,
    },
    /// A strip sheet: each `<dir>/<name>.flat.png` at 512, 48, 32 and 16 px 1:1 on light and dark.
    Strips {
        #[arg(long)]
        dir: PathBuf,
        #[arg(long, value_delimiter = ',')]
        names: Vec<String>,
        #[arg(long)]
        title: String,
        #[arg(long)]
        out: PathBuf,
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
enum FaceMode {
    Ground,
    Tile,
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

/// The centre share of a full-bleed `ground` render kept as the plate face.
const GROUND_CROP: f32 = 0.78;

/// Places `img` stretched over the plate square of a 1024 canvas.
fn onto_plate(img: &Rgba32FImage, t: &Template) -> Rgba32FImage {
    let grid = grid_for(1024, t);
    let small = resize(img, grid.side, grid.side, FilterType::Lanczos3);
    Rgba32FImage::from_fn(1024, 1024, |x, y| {
        let inside = grid.origin..grid.origin + grid.side;
        match inside.contains(&x) && inside.contains(&y) {
            true => *small.get_pixel(x - grid.origin, y - grid.origin),
            false => image::Rgba([0.0; 4]),
        }
    })
}

fn face(input: &Path, mode: FaceMode, out_dir: &Path, name: &str) -> Result<(), IconsError> {
    let t = Template::default();
    let raw = load(input)?;
    let (face, bevel) = match mode {
        FaceMode::Ground => {
            // The brief asks for a symbol about half the width; the centre crop brings it to
            // about 60 % of the plate (08 2.4 asks 56-80 %) and keeps the gradient's two ends.
            let side = (raw.width().min(raw.height()) as f32 * GROUND_CROP) as u32;
            let (x0, y0) = ((raw.width() - side) / 2, (raw.height() - side) / 2);
            let crop = image::imageops::crop_imm(&raw, x0, y0, side, side).to_image();
            (onto_plate(&crop, &t), Bevel::Ours)
        }
        FaceMode::Tile => {
            let key = key_background(&raw, &t);
            let b = alpha_bbox(&key.alpha, 0.5).ok_or(IconsError::EmptyObject)?;
            let crop = image::imageops::crop_imm(
                &key.layer,
                b.x0 as u32,
                b.y0 as u32,
                b.width() as u32,
                b.height() as u32,
            )
            .to_image();
            (onto_plate(&crop, &t), Bevel::Theirs)
        }
    };
    let grid = grid_for(1024, &t);
    let flat = finish(grid, &face, &t, bevel);
    save(&flat, &out_dir.join(format!("{name}.flat.png")))?;
    save(
        &drop_shadow(&flat, grid, &t),
        &out_dir.join(format!("{name}.png")),
    )
}

fn strips(dir: &Path, names: &[String], title: &str, out: &Path) -> Result<(), IconsError> {
    let t = Template::default();
    let rows = names
        .iter()
        .map(|n| {
            let flat = load(&dir.join(format!("{n}.flat.png")))?;
            Ok((
                n.to_uppercase(),
                strip(&[512, 48, 32, 16], |size| {
                    export(&flat, &t, size, Shadow::Baked).image
                }),
            ))
        })
        .collect::<Result<Vec<_>, IconsError>>()?;
    save(&strip_sheet(title, &rows, &SheetStyle::default()), out)
}

/// One size of an abstract icon: the flat render, plus the baked shadow from 48 px (08 2.5).
fn abstract_size(spec: &Spec, size: u32, t: &Template, tile: &Plane) -> Rgba32FImage {
    let flat = emblem(spec, size, t, tile);
    match size >= 48 {
        true => drop_shadow(&flat, grid_for(size, t), t),
        false => flat,
    }
}

fn abstract_icons(
    specs: &[PathBuf],
    out_dir: &Path,
    sheet: Option<&Path>,
) -> Result<(), IconsError> {
    let t = Template::default();
    let tile = grain_tile();
    let specs = specs
        .iter()
        .map(|p| parse_spec(&std::fs::read_to_string(p)?))
        .collect::<Result<Vec<_>, IconsError>>()?;
    for spec in &specs {
        let name = &spec.name;
        let flat = emblem(spec, 1024, &t, &tile);
        save(&flat, &out_dir.join(format!("{name}.flat.png")))?;
        save(
            &drop_shadow(&flat, grid_for(1024, &t), &t),
            &out_dir.join(format!("{name}.png")),
        )?;
        for size in EXPORT_SIZES {
            save(
                &abstract_size(spec, size, &t, &tile),
                &out_dir.join(format!("hicolor/{size}x{size}/apps/{name}.png")),
            )?;
            if size <= 256 {
                save(
                    &abstract_size(spec, size * 2, &t, &tile),
                    &out_dir.join(format!("hicolor/{size}x{size}@2/apps/{name}.png")),
                )?;
            }
        }
    }
    if let Some(out) = sheet {
        let rows: Vec<(String, Rgba32FImage)> = specs
            .iter()
            .map(|s| {
                (
                    s.name.to_uppercase(),
                    strip(&[512, 48, 32, 16], |size| abstract_size(s, size, &t, &tile)),
                )
            })
            .collect();
        save(
            &strip_sheet(
                "PROCEDURAL - FROM TOKENS - 512 48 32 16 AT 1:1 ON LIGHT AND DARK",
                &rows,
                &SheetStyle::default(),
            ),
            out,
        )?;
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
        Command::Abstract {
            spec,
            out_dir,
            sheet,
        } => abstract_icons(&spec, &out_dir, sheet.as_deref()),
        Command::Face {
            input,
            mode,
            out_dir,
            name,
        } => face(&input, mode, &out_dir, &name),
        Command::Strips {
            dir,
            names,
            title,
            out,
        } => strips(&dir, &names, &title, &out),
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
