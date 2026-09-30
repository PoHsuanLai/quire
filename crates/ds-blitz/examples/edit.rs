//! An edit surface in a real window, for trying the IME by hand: click in the text, type, compose
//! with an input method (fcitx5, IBus), paste. Every input the surface hands the app is printed on
//! stdout, and the caret the app would draw sits at the last position a press reported. With
//! `QUIRE_EDIT_SECONDS=n` it closes itself after n seconds.
//!
//! `cargo run -p ds-blitz --example edit`

use dioxus::prelude::*;
use ds::base::time::clock::sleep;
use ds::edit::handle::use_edit_handle;
use ds::edit::input::EditInput;
use ds::edit::pointer::EditPointer;
use ds::host::captured::PointerPhase;
use ds::host::probe::Probe;
use ds::prelude::*;
use ds::root::common::Common;
use ds_blitz::{AppConfig, AppId, launch};
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
            sleep(Duration::from_secs(seconds)).await;
            std::process::exit(0);
        }))
    });
    let handle = use_edit_handle();
    let mut caret = use_signal(|| None::<Rect>);
    rsx! {
        Ds { appearance: Appearance::default(), material: Material::Window,
            div { style: "padding:24px; position:relative",
                EditSurface {
                    common: Common { aria_label: Some("Message".to_string()), ..Common::default() },
                    handle,
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
