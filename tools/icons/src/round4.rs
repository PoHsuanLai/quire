//! The round-four sheet commands (design/08-ICONS.md 2.10): file handling only; the cells come
//! from the library's `look_cell` and `face_cell`.

use std::path::Path;

use ds::space::PRESETS;
use icons::{
    Dialect, IconsError, Look, Sheet, SheetStyle, Spec, Template, build_sheet, face_cell,
    grain_tile, look_cell, space_tint,
};

use crate::{load, save};

/// All five specs x the four dialects, plus the retinted Klein face where one exists.
pub fn dialects(specs: &[Spec], klein_dir: Option<&Path>, out: &Path) -> Result<(), IconsError> {
    let t = Template::default();
    let tile = grain_tile();
    let rows = specs
        .iter()
        .map(|spec| {
            let mut cells: Vec<_> = Dialect::ALL
                .iter()
                .map(|&dialect| {
                    let look = Look {
                        dialect,
                        tint: spec.tint,
                    };
                    let name = dialect.name().to_uppercase();
                    look_cell(spec, look, 512, &name, &t, &tile)
                })
                .collect();
            if let Some(dir) = klein_dir {
                let master = dir.join(format!("{}.png", spec.name));
                if master.exists() {
                    let flat = load(&dir.join(format!("{}.flat.png", spec.name)))?;
                    cells.push(face_cell(load(&master)?, &flat, "KLEIN MONOCHROME", &t));
                }
            }
            Ok((spec.name.to_uppercase(), cells))
        })
        .collect::<Result<Vec<_>, IconsError>>()?;
    let sheet = Sheet {
        title:
            "ROUND FOUR - FOUR DIALECTS, ONE MUTED PALETTE - 512, THEN 16 32 48 ON LIGHT AND DARK"
                .to_owned(),
        rows,
    };
    let style = SheetStyle {
        tile: 512,
        ..SheetStyle::default()
    };
    save(&build_sheet(&sheet, &style), out)
}

/// The set in Monochrome, one row per Space preset (Work, Home), tinted as `ds` derives it.
pub fn space(specs: &[Spec], out: &Path) -> Result<(), IconsError> {
    let t = Template::default();
    let tile = grain_tile();
    let rows = [("WORK 268", 0), ("HOME 152", 1)]
        .into_iter()
        .map(|(label, preset)| {
            let tint = space_tint(PRESETS[preset].dots);
            let look = Look {
                dialect: Dialect::Monochrome,
                tint,
            };
            let cells = specs
                .iter()
                .map(|s| look_cell(s, look, 256, &s.name.to_uppercase(), &t, &tile))
                .collect();
            (label.to_owned(), cells)
        })
        .collect();
    let sheet = Sheet {
        title: "MONOCHROME, TINTED BY THE SPACE - 16 32 48 ON LIGHT AND DARK UNDER EACH".to_owned(),
        rows,
    };
    save(&build_sheet(&sheet, &SheetStyle::default()), out)
}
