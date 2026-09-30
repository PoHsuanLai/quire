//! `cargo run -p ds-settings --example write_schema -- --write-schema <dir>`: writes
//! `quire.settings.toml`, the schema of `quire/appearance.toml`, the way `sill --write-schema`
//! writes sill's (design/22-SETTINGS.md section 9.2). quire has no app binary, so this example
//! is its `--write-schema`.

use std::process::ExitCode;

use ds_settings::quire_schema;
use ds_settings::schema::maybe_write_schema;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    match maybe_write_schema(&quire_schema(), &args) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => {
            eprintln!("usage: write_schema --write-schema <dir>");
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!("write_schema: {error}");
            ExitCode::FAILURE
        }
    }
}
