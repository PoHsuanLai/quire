//! `icons ship` (design/08-ICONS.md 2.11): export the shipped set in its three styles and build
//! the round-six sheet from the exported files. File handling only.

use std::path::Path;

use ds::icon::{IconStyle, Tint as DsTint, retint as ds_retint};
use ds::space::PRESETS;
use icons::{
    Cell, IconsError, Manifest, Prepared, SHIP_PX, Sheet, SheetStyle, Source, Style, Template,
    build_sheet, grain_tile, klein_face, parse_spec, ship_icon, strip, style_look,
};
use image::{DynamicImage, Rgba32FImage};

use crate::{load, save};

fn style_dir(out: &Path, app: &str, style: Style) -> std::path::PathBuf {
    match style.dir() {
        Some(d) => out.join(app).join(d),
        None => out.join(app),
    }
}

/// Writes `<out>/<app>/[<style>/]<px>.png` for every app, style and size.
pub fn ship(manifest_path: &Path, renders: &Path, out: &Path) -> Result<(), IconsError> {
    let m: Manifest = toml::from_str(&std::fs::read_to_string(manifest_path)?)?;
    let base = manifest_path.parent().unwrap_or(Path::new("."));
    let t = Template::default();
    let tile = grain_tile();
    for app in &m.apps {
        let spec = parse_spec(&std::fs::read_to_string(base.join(&app.spec))?)?;
        let raw = match &app.source {
            Source::Procedural => None,
            Source::KleinRetint { render } => Some(load(&renders.join(render))?),
        };
        for style in Style::ALL {
            let look = style_look(app, style, &m);
            let prepared = match &raw {
                Some(r) => Prepared::Face(klein_face(r, look)),
                None => Prepared::Procedural(&spec, look),
            };
            let dir = style_dir(out, &app.name, style);
            for px in SHIP_PX {
                save(
                    &ship_icon(&prepared, px, &t, &tile),
                    &dir.join(format!("{px}.png")),
                )?;
            }
        }
    }
    Ok(())
}

fn tinted(path: &Path, tint: DsTint) -> Result<Rgba32FImage, IconsError> {
    let mut rgba = image::open(path)?.to_rgba8();
    ds_retint(&mut rgba, IconStyle::Monochrome, tint);
    Ok(DynamicImage::ImageRgba8(rgba).to_rgba32f())
}

/// The round-six sheet from the exported files: the shipped (Colour) set, the Muted set, and the
/// neutral Monochrome set tinted by `ds::icon::retint` for the Work and Home Spaces.
pub fn sheet(manifest_path: &Path, out_dir: &Path, sheet: &Path) -> Result<(), IconsError> {
    let m: Manifest = toml::from_str(&std::fs::read_to_string(manifest_path)?)?;
    let space = |preset: usize| DsTint::space(PRESETS[preset].dots);
    type Load<'a> = Box<dyn Fn(&Path) -> Result<Rgba32FImage, IconsError> + 'a>;
    let rows: [(&str, Style, Load); 4] = [
        ("SHIPPED", Style::Colour, Box::new(|p: &Path| load(p))),
        ("MUTED 0.04", Style::Muted, Box::new(|p: &Path| load(p))),
        (
            "MONO WORK 268",
            Style::Monochrome,
            Box::new(move |p: &Path| tinted(p, space(0))),
        ),
        (
            "MONO HOME 152",
            Style::Monochrome,
            Box::new(move |p: &Path| tinted(p, space(1))),
        ),
    ];
    let rows = rows
        .iter()
        .map(|(label, style, get)| {
            let cells = m
                .apps
                .iter()
                .map(|app| {
                    let dir = style_dir(out_dir, &app.name, *style);
                    let small = [16, 32, 48]
                        .iter()
                        .map(|px| get(&dir.join(format!("{px}.png"))))
                        .collect::<Result<Vec<_>, IconsError>>()?;
                    Ok(Cell {
                        image: get(&dir.join("512.png"))?,
                        below: Some(strip(&[16, 32, 48], |px| {
                            let i = [16, 32, 48].iter().position(|p| *p == px).unwrap_or(0);
                            small[i].clone()
                        })),
                        caption: app.name.to_uppercase(),
                    })
                })
                .collect::<Result<Vec<_>, IconsError>>()?;
            Ok(((*label).to_owned(), cells))
        })
        .collect::<Result<Vec<_>, IconsError>>()?;
    let s = Sheet {
        title: "ROUND SIX - THE SHIPPED SET FROM ASSETS/ICONS/APPS - 512, THEN 16 32 48 ON LIGHT AND DARK"
            .to_owned(),
        rows,
    };
    save(
        &build_sheet(
            &s,
            &SheetStyle {
                tile: 512,
                ..SheetStyle::default()
            },
        ),
        sheet,
    )
}
