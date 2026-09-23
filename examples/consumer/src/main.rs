//! Runs the example on Blitz, through `ds_native::launch` (`../CONSUMING.md` section 1).
//! `cargo run` opens a window; `cargo test` (see `tests/coherence.rs`) never opens one.
//! `consumer::App` reads its own settings at first render (`src/lib.rs::appearance`) — nothing
//! to load here.

fn main() {
    ds_native::launch(
        consumer::App,
        ds_native::AppConfig {
            title: "quire consumer example".to_owned(),
            width: 480,
            height: 360,
        },
    );
}
