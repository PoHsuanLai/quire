//! `icons`: turn raw renders into plated icons and contact sheets (design/08-ICONS.md 3.7).

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
mod install;
mod round4;
mod round5;
mod round6;

use icons::{
    Bevel, Cell, Dialect, EXPORT_SIZES, Family, IconsError, Look, Shadow, Sheet, SheetStyle, Spec,
    Template, Tint, alpha_bbox, build_sheet, compose, drop_shadow, emblem, export, finish,
    fit_object, grain_tile, grid_for, key_background, parse_spec, render_icon, retint, roles,
    size_strip, strip, strip_sheet,
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
        /// Draw every spec in this dialect instead of its own (08 2.10).
        #[arg(long)]
        dialect: Option<Dialect>,
    },
    /// Round four's dialect sheet (08 2.10): every spec in all four dialects, plus the
    /// retinted model face `<klein-dir>/<name>.png` / `.flat.png` where one exists.
    Dialects {
        #[arg(long, num_args = 1.., required = true)]
        spec: Vec<PathBuf>,
        #[arg(long)]
        klein_dir: Option<PathBuf>,
        #[arg(long)]
        out: PathBuf,
    },
    /// Export the shipped set (08 2.11) from `ship.toml`: every size of every style into
    /// `<out>/<app>/[muted|monochrome/]<px>.png`; optionally the round-six sheet from the files.
    Ship {
        #[arg(long)]
        manifest: PathBuf,
        /// Where the manifest's Klein renders are (round three's face-mode renders).
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
    /// Round five (a): every app in eight colourways at the palette's chroma cap.
    Colourways {
        #[arg(long, num_args = 1.., required = true)]
        spec: Vec<PathBuf>,
        /// Show an app in another dialect than its spec's: `terminal=monochrome`.
        #[arg(long = "as", value_delimiter = ',')]
        as_dialect: Vec<String>,
        /// Retinted model faces, `<app>-<hue>-<cap>.png` / `.flat.png`, used where present.
        #[arg(long)]
        klein_dir: Option<PathBuf>,
        #[arg(long)]
        out: PathBuf,
    },
    /// Round five (b): every app in the given hues at chroma 0.07, 0.11 and 0.15.
    Bolder {
        #[arg(long, num_args = 1.., required = true)]
        spec: Vec<PathBuf>,
        #[arg(long = "as", value_delimiter = ',')]
        as_dialect: Vec<String>,
        #[arg(long)]
        klein_dir: Option<PathBuf>,
        #[arg(long, value_delimiter = ',')]
        hues: Vec<String>,
        #[arg(long)]
        out: PathBuf,
    },
    /// Round five (c): the palette's swatch board with OKLCh values.
    Palette {
        #[arg(long)]
        out: PathBuf,
    },
    /// The whole set in Monochrome tinted by the Work and Home presets' Space colours.
    Space {
        #[arg(long, num_args = 1.., required = true)]
        spec: Vec<PathBuf>,
        #[arg(long)]
        out: PathBuf,
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
        /// Re-colour the face into this dialect (round four), keeping its relief.
        #[arg(long)]
        retint: Option<Dialect>,
        /// The tint for `--retint`, from the muted palette.
        #[arg(long, default_value = "slate")]
        tint: String,
        /// The chroma cap for `--retint` (round five's bolder steps: 0.11, 0.15).
        #[arg(long, default_value_t = icons::CHROMA_CAP)]
        chroma: f32,
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

fn face(
    input: &Path,
    mode: FaceMode,
    out_dir: &Path,
    name: &str,
    recolour: Option<(Dialect, Tint, icons::ChromaCap)>,
) -> Result<(), IconsError> {
    let t = Template::default();
    let raw = load(input)?;
    let (face, bevel) = match mode {
        FaceMode::Ground => {
            // The brief asks for a symbol about half the width; the centre crop brings it to
            // about 60 % of the plate (08 2.4 asks 56-80 %) and keeps the gradient's two ends.
            let side = (raw.width().min(raw.height()) as f32 * GROUND_CROP) as u32;
            let (x0, y0) = ((raw.width() - side) / 2, (raw.height() - side) / 2);
            let crop = image::imageops::crop_imm(&raw, x0, y0, side, side).to_image();
            let crop = match recolour {
                Some((d, tint, cap)) => retint(&crop, &roles(d, tint, cap)),
                None => crop,
            };
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

/// `name=dialect` pairs from `--as`.
fn parse_as(pairs: &[String]) -> Result<Vec<(String, Dialect)>, IconsError> {
    pairs
        .iter()
        .map(|p| {
            let (name, d) = p
                .split_once('=')
                .ok_or_else(|| IconsError::BadOverride(p.clone()))?;
            let dialect = <Dialect as clap::ValueEnum>::from_str(d, true)
                .map_err(|_| IconsError::BadOverride(p.clone()))?;
            Ok((name.to_owned(), dialect))
        })
        .collect()
}

fn load_specs(paths: &[PathBuf]) -> Result<Vec<Spec>, IconsError> {
    paths
        .iter()
        .map(|p| parse_spec(&std::fs::read_to_string(p)?))
        .collect()
}

fn abstract_icons(
    specs: &[PathBuf],
    out_dir: &Path,
    sheet: Option<&Path>,
    dialect: Option<Dialect>,
) -> Result<(), IconsError> {
    let t = Template::default();
    let tile = grain_tile();
    let specs = load_specs(specs)?;
    let look = |s: &Spec| Look {
        dialect: dialect.unwrap_or(s.dialect),
        tint: s.tint,
        cap: icons::ChromaCap::default(),
    };
    for spec in &specs {
        let name = &spec.name;
        let flat = emblem(spec, look(spec), 1024, &t, &tile);
        save(&flat, &out_dir.join(format!("{name}.flat.png")))?;
        save(
            &drop_shadow(&flat, grid_for(1024, &t), &t),
            &out_dir.join(format!("{name}.png")),
        )?;
        for size in EXPORT_SIZES {
            save(
                &render_icon(spec, look(spec), size, &t, &tile),
                &out_dir.join(format!("hicolor/{size}x{size}/apps/{name}.png")),
            )?;
            if size <= 256 {
                save(
                    &render_icon(spec, look(spec), size * 2, &t, &tile),
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
                    strip(&[512, 48, 32, 16], |size| {
                        render_icon(s, look(s), size, &t, &tile)
                    }),
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
            dialect,
        } => abstract_icons(&spec, &out_dir, sheet.as_deref(), dialect),
        Command::Dialects {
            spec,
            klein_dir,
            out,
        } => round4::dialects(&load_specs(&spec)?, klein_dir.as_deref(), &out),
        Command::Space { spec, out } => round4::space(&load_specs(&spec)?, &out),
        Command::Colourways {
            spec,
            as_dialect,
            klein_dir,
            out,
        } => {
            let dialects = parse_as(&as_dialect)?;
            let plan = round5::Plan {
                dialects: &dialects,
                klein_dir: klein_dir.as_deref(),
            };
            round5::colourways(&load_specs(&spec)?, &plan, &out)
        }
        Command::Bolder {
            spec,
            as_dialect,
            klein_dir,
            hues,
            out,
        } => {
            let dialects = parse_as(&as_dialect)?;
            let plan = round5::Plan {
                dialects: &dialects,
                klein_dir: klein_dir.as_deref(),
            };
            round5::bolder(&load_specs(&spec)?, &plan, &hues, &out)
        }
        Command::Palette { out } => round5::palette(&out),
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
            round6::ship(&manifest, &renders, &out)?;
            match sheet {
                Some(s) => round6::sheet(&manifest, &out, &s),
                None => Ok(()),
            }
        }
        Command::Face {
            input,
            mode,
            out_dir,
            name,
            retint,
            tint,
            chroma,
        } => {
            let recolour = retint
                .map(|d| tint.parse().map(|t| (d, t, icons::ChromaCap(chroma))))
                .transpose()?;
            face(&input, mode, &out_dir, &name, recolour)
        }
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
