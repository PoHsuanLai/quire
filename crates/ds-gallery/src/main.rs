//! The quire gallery: every component across theme, accent, motion, material, blur and Space
//! preset; a matrix page; a motion lab; and `--snapshot DIR` for a contact sheet.

mod app;
mod args;
mod axes;
mod data_uri;
mod error;
mod legibility;
mod page;
mod pages;
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
    match args.snapshot {
        Some(dir) => {
            if let Err(error) = snapshot::run(&dir) {
                eprintln!("ds-gallery: {error}");
                std::process::exit(1);
            }
        }
        None => {
            start_with(Axes {
                page: args.page.unwrap_or(page::Page::Tokens),
                ..Axes::default()
            });
            launch(
                app::App,
                AppConfig {
                    title: "quire gallery".into(),
                    width: 1280,
                    height: 900,
                },
            );
        }
    }
}
