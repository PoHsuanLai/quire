//! An edit surface in a real window, for trying the IME by hand: click in the text, type, compose
//! with an input method (fcitx5, IBus), paste. Every input the surface hands the app is printed on
//! stdout, and the caret the app would draw sits at the last position a press reported. With
//! `QUIRE_EDIT_SECONDS=n` it closes itself after n seconds.
//!
//! `cargo run -p ds-native --example edit`

use dioxus::prelude::*;
use ds::{
    Appearance, Ds, EditInput, EditPointer, EditSurface, Material, PointerPhase, Probe, Rect,
    use_edit_handle,
};
use ds_native::{AppConfig, AppId, launch};
use std::time::Duration;

fn main() {
    launch(
        App,
        AppConfig::new("quire: edit surface", 520, 300)
            .with_app_id(AppId("dev.quire.Edit".to_owned())),
    );
}

#[allow(non_snake_case)]
fn App() -> Element {
    use_hook(|| {
        let seconds = std::env::var("QUIRE_EDIT_SECONDS").ok()?.parse().ok()?;
        Some(spawn(async move {
            ds::sleep(Duration::from_secs(seconds)).await;
            std::process::exit(0);
        }))
    });
    let handle = use_edit_handle();
    let mut caret = use_signal(|| None::<Rect>);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:24px; position:relative",
                EditSurface {
                    handle,
                    label: "Message",
                    ime_area: caret(),
                    on_input: |input: EditInput| println!("{input:?}"),
                    on_pointer: move |pointer: EditPointer| {
                        if pointer.phase == PointerPhase::Press
                            && let Some(position) = pointer.position
                        {
                            println!("press at {position:?}");
                            caret.set(rect_at(handle.caret_rect(&position)));
                        }
                    },
                    p { "data-edit-node": "0", "Click here, then type or compose." }
                    p { "data-edit-node": "1", "注音, 拼音 and かな go through the IME." }
                }
            }
        }
    }
}

fn rect_at(probe: Probe<Rect>) -> Option<Rect> {
    let rect = probe.found();
    if let Some(rect) = rect {
        println!("caret rect {rect:?}");
    }
    rect
}
