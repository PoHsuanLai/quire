//! The quire gallery: every component across theme, accent, motion, material, blur and Space
//! preset; a matrix page; a motion lab; and `--snapshot DIR` for a contact sheet.

mod app;
mod args;
mod axes;
mod data_uri;
mod error;
mod legibility;
mod level_motion;
mod level_sheet;
mod page;
mod pages;
mod portrait;
mod registry;
mod sheet;
mod snapshot;
mod style;
mod toolbar;
mod wallpaper;
mod wallpaper_vivid;

#[cfg(test)]
mod tests;

use axes::{Axes, start_with};
use ds_native::{AppConfig, launch};

fn main() {
    let args = args::parse(std::env::args().skip(1));
    let args = match args {
        Ok(args) => args,
        Err(error) => {
            eprintln!("ds-gallery: {error}");
            std::process::exit(2);
        }
    };
    if let Some(dir) = args.level_sheet {
        if let Err(error) = level_sheet::run(&dir) {
            eprintln!("ds-gallery: {error}");
            std::process::exit(1);
        }
        return;
    }
    match args.snapshot {
        Some(dir) => {
            let typeface = args.typeface.unwrap_or_default();
            if let Err(error) = snapshot::run(&dir, args.page, args.scale.unwrap_or(100), typeface)
            {
                eprintln!("ds-gallery: {error}");
                std::process::exit(1);
            }
        }
        None => {
            start_with(Axes {
                page: args.page.unwrap_or(page::Page::Tokens),
                typeface: args.typeface.unwrap_or_default(),
                ..Axes::default()
            });
            launch(app::App, AppConfig::new("quire gallery", 1280, 900));
        }
    }
}
