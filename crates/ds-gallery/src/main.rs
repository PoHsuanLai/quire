//! The quire gallery: every component across theme, accent, motion, material, blur and Space
//! preset; a matrix page; a motion lab; `--snapshot DIR` for a contact sheet; and
//! `--persona-frames DIR` for the persona's motion as stills.

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
mod persona_frames;
mod registry;
mod sheet;
mod snapshot;
mod style;
mod toolbar;
mod wallpaper;

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
    if let Some(dir) = args.persona_frames {
        if let Err(error) = persona_frames::run(&dir) {
            eprintln!("ds-gallery: {error}");
            std::process::exit(1);
        }
        return;
    }
    match args.snapshot {
        Some(dir) => {
            if let Err(error) = snapshot::run(&dir, args.page, args.scale.unwrap_or(100)) {
                eprintln!("ds-gallery: {error}");
                std::process::exit(1);
            }
        }
        None => {
            start_with(Axes {
                page: args.page.unwrap_or(page::Page::Tokens),
                ..Axes::default()
            });
            launch(app::App, AppConfig::new("quire gallery", 1280, 900));
        }
    }
}
