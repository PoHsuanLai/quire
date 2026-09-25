//! Print the PDF fixture through the system dialog, by hand: `cargo run -p ds-native --features
//! print --example print`. On Linux the desktop portal's dialog opens (unparented); without a
//! portal the PDF opens in the system's viewer.

#[path = "../tests/support/print_fixture.rs"]
mod print_fixture;

use ds_native::{PageSpec, pdf, print_dialog};

fn main() {
    let html = print_fixture::html(&print_fixture::jpeg(), &print_fixture::png());
    let bytes = pdf(&html, PageSpec::default()).expect("the fixture prints");
    match print_dialog(&bytes, "quire print fixture") {
        Ok(outcome) => println!("{outcome:?}"),
        Err(error) => eprintln!("{error}"),
    }
}
