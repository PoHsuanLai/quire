//! Files dropped into a window from a file manager (mailo item 21: attachments dropped onto the
//! composer).
//!
//! `cargo run -p ds-native --example file_drop`
//!
//! Drag one or more files from Dolphin, Nautilus or the desktop over the window. The composer
//! box shows `data-drop="accepts"` (a dashed outline) while the files are anywhere over the
//! window and `target` (lit) while they are over it; the cursor shows a copy only over it. Let
//! go over the box: the paths are listed in it and printed. A link or text dragged out of a
//! browser lights nothing and is refused. Escape or closing the window quits.

use dioxus::prelude::*;
use ds::{Appearance, Ds, FileDrag, FileDrop, Material, use_file_drop};
use ds_native::{AppConfig, AppId, launch};

fn main() {
    launch(
        App,
        AppConfig::new("quire: drop files here", 520, 360)
            .with_app_id(AppId("dev.quire.FileDrop".to_owned())),
    );
}

#[allow(non_snake_case)]
fn App() -> Element {
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window, Composer {} }
    }
}

#[allow(non_snake_case)]
fn Composer() -> Element {
    let mut dropped = use_signal(Vec::<String>::new);
    let drop = use_file_drop(move |files: FileDrop| {
        let paths: Vec<String> = files
            .paths
            .iter()
            .map(|p| p.display().to_string())
            .collect();
        println!("dropped at {:?}: {paths:?}", files.point);
        dropped.set(paths);
    });
    let status = match drop.drag() {
        FileDrag::Idle => "Drag files here".to_owned(),
        FileDrag::Over { paths, .. } => format!("Let go to attach {} file(s)", paths.len()),
        FileDrag::Dropped { paths, .. } => format!("Attached {} file(s)", paths.len()),
    };
    let border = match drop.drop_attr() {
        Some("target") => "2px solid var(--accent)",
        Some(_) => "2px dashed var(--accent)",
        None => "2px dashed var(--line)",
    };
    rsx! {
        div { style: "padding:24px",
            div { class: "composer",
                style: "min-height:220px; padding:16px; border-radius:12px; border:{border}",
                "data-drop": drop.drop_attr(),
                onmounted: move |event| drop.mounted(event),
                p { "{status}" }
                for path in dropped() {
                    p { style: "font-size:12px", "{path}" }
                }
            }
        }
    }
}
