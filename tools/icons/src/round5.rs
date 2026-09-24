//! The round-five sheet commands (design/08-ICONS.md 2.10, docs/icons-bakeoff.md "Round five"):
//! file handling only; cells come from the library.

use std::path::Path;

use icons::{
    Cell, ChromaCap, Dialect, IconsError, Look, PALETTE, Sheet, SheetStyle, Spec, Template, Tint,
    build_sheet, face_cell, grain_tile, look_cell, palette_board, strip_sheet,
};

use crate::{load, save};

/// Which dialect each app is shown in, and whether its colourways come from a retinted model
/// face in `klein_dir` (`<app>-<hue>-<cap>.png` and `.flat.png`) instead of the spec.
pub struct Plan<'a> {
    pub dialects: &'a [(String, Dialect)],
    pub klein_dir: Option<&'a Path>,
}

impl Plan<'_> {
    fn dialect(&self, spec: &Spec) -> Dialect {
        self.dialects
            .iter()
            .find(|(n, _)| *n == spec.name)
            .map(|(_, d)| *d)
            .unwrap_or(spec.dialect)
    }

    /// The cell for one app in one hue at one cap.
    fn cell(
        &self,
        spec: &Spec,
        (name, hue): (&str, f32),
        cap: ChromaCap,
        tile_px: u32,
        caption: &str,
    ) -> Result<Cell, IconsError> {
        let t = Template::default();
        // `with_extension` would eat the cap's decimal point, so the names are spelled out.
        let file = |ext: &str| format!("{}-{name}-{:.2}.{ext}", spec.name, cap.0);
        let klein = self.klein_dir.filter(|d| d.join(file("png")).exists());
        match klein {
            Some(dir) => {
                let flat = load(&dir.join(file("flat.png")))?;
                Ok(face_cell(load(&dir.join(file("png")))?, &flat, caption, &t))
            }
            None => {
                let look = Look {
                    dialect: self.dialect(spec),
                    tint: Tint { hue, strength: 1.0 },
                    cap,
                };
                Ok(look_cell(spec, look, tile_px, caption, &t, &grain_tile()))
            }
        }
    }

    fn row_name(&self, spec: &Spec) -> String {
        let klein = self
            .klein_dir
            .map(|d| d.join(format!("{}-clay-0.07.png", spec.name)).exists())
            .unwrap_or(false);
        let source = if klein { " KLEIN" } else { "" };
        format!(
            "{} {}{source}",
            spec.name.to_uppercase(),
            self.dialect(spec).name().to_uppercase()
        )
    }
}

/// (a) Every app in its dialect in all eight hues at the palette's cap.
pub fn colourways(specs: &[Spec], plan: &Plan, out: &Path) -> Result<(), IconsError> {
    let rows = specs
        .iter()
        .map(|spec| {
            let cells = PALETTE
                .iter()
                .map(|&(name, hue)| {
                    plan.cell(
                        spec,
                        (name, hue),
                        ChromaCap::MUTED,
                        512,
                        &name.to_uppercase(),
                    )
                })
                .collect::<Result<Vec<_>, IconsError>>()?;
            Ok((plan.row_name(spec), cells))
        })
        .collect::<Result<Vec<_>, IconsError>>()?;
    let sheet = Sheet {
        title: "ROUND FIVE - EIGHT COLOURWAYS AT C 0.07 - 512, THEN 16 32 48 ON LIGHT AND DARK"
            .to_owned(),
        rows,
    };
    let style = SheetStyle {
        tile: 512,
        ..SheetStyle::default()
    };
    save(&build_sheet(&sheet, &style), out)
}

/// (b) Every app in three hues at the three caps.
pub fn bolder(specs: &[Spec], plan: &Plan, hues: &[String], out: &Path) -> Result<(), IconsError> {
    let picked = hues
        .iter()
        .map(|h| {
            PALETTE
                .iter()
                .find(|(n, _)| n == h)
                .copied()
                .ok_or_else(|| IconsError::UnknownTint(h.clone()))
        })
        .collect::<Result<Vec<_>, IconsError>>()?;
    let rows = specs
        .iter()
        .map(|spec| {
            let cells = picked
                .iter()
                .flat_map(|&hue| ChromaCap::LEVELS.iter().map(move |cap| (hue, *cap)))
                .map(|((name, h), cap)| {
                    let caption = format!("{} {:.2}", name.to_uppercase(), cap.0);
                    plan.cell(spec, (name, h), cap, 256, &caption)
                })
                .collect::<Result<Vec<_>, IconsError>>()?;
            Ok((plan.row_name(spec), cells))
        })
        .collect::<Result<Vec<_>, IconsError>>()?;
    let sheet = Sheet {
        title: "ROUND FIVE - BOLDER STEPS - C 0.07 0.11 0.15 PER HUE - 16 32 48 ON LIGHT AND DARK"
            .to_owned(),
        rows,
    };
    save(&build_sheet(&sheet, &SheetStyle::default()), out)
}

/// (c) The swatch board.
pub fn palette(out: &Path) -> Result<(), IconsError> {
    let rows = palette_board(&Template::default());
    save(
        &strip_sheet(
            "ROUND FIVE - THE EIGHT HUES AT C 0.07 0.11 0.15 - SOLID PLATE COLOUR L 0.62",
            &rows,
            &SheetStyle {
                text_scale: 2,
                ..SheetStyle::default()
            },
        ),
        out,
    )
}
