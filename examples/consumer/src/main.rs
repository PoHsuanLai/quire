//! Runs the example on Blitz, through `ds_blitz::launch` (`../CONSUMING.md` section 1).
//! `cargo run` opens a window; `cargo test` (see `tests/coherence.rs`) never opens one.
//! `consumer::App` reads its own live settings at first render through
//! `ds_settings::use_environment` (`src/lib.rs::App`) — nothing to load here; `launch` enters
//! the Tokio runtime its watches run on.

fn main() {
    ds_blitz::launch(
        consumer::App,
        ds_blitz::AppConfig::new("quire consumer example", 480, 360),
    );
}
