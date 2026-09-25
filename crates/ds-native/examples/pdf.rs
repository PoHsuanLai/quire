//! Print the PDF fixture and report its size and time: `cargo run --release -p ds-native
//! --example pdf -- out.pdf`. The first call builds the shared font context (system fonts
//! scanned once per process), so it is timed apart from a second, warm call.

#[path = "../tests/support/print_fixture.rs"]
mod print_fixture;

use ds_native::{PageSpec, pdf};
use std::time::Instant;

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "print-fixture.pdf".to_owned());
    let html = print_fixture::html(&print_fixture::jpeg(), &print_fixture::png());
    let cold = Instant::now();
    let first = pdf(&html, PageSpec::default()).expect("the fixture prints");
    let cold = cold.elapsed();
    let warm = Instant::now();
    let bytes = pdf(&html, PageSpec::default()).expect("the fixture prints");
    let warm = warm.elapsed();
    assert_eq!(
        first.len(),
        bytes.len(),
        "printing is deterministic in size"
    );
    std::fs::write(&out, &bytes).expect("the PDF is written");
    println!(
        "wrote {out}: {} bytes; first call {cold:?} (fonts scanned), second {warm:?}",
        bytes.len()
    );
}
